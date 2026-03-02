//! # Rwatch Client Library
//!
//! This crate provides a client for querying rwatch agents.
//! It handles agent discovery, querying multiple agents, and provides
//! structured data for UI consumers.

pub mod agent;
pub mod discovery;

use anyhow::{Context, Result};
use rwatch_common::health::HealthResponse;
use rwatch_common::memory::Memory;
use serde::de::DeserializeOwned;
use std::time::Duration;

/// Client for interacting with rwatch agents
#[derive(Debug, Clone)]
pub struct Client {
    http_client: reqwest::Client,
    timeout: Duration,
}

impl Default for Client {
    fn default() -> Self {
        // For Default trait, we use a reasonable fallback
        // In production, use Client::new() which returns Result
        Self {
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("Failed to create HTTP client"),
            timeout: Duration::from_secs(5),
        }
    }
}

impl Client {
    /// Create a new client with default settings
    pub fn new() -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            http_client,
            timeout: Duration::from_secs(5),
        })
    }

    /// Create a new client with custom timeout
    pub fn with_timeout(timeout: Duration) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            http_client,
            timeout,
        })
    }

    /// Generic method to query any endpoint
    async fn query_endpoint<T: DeserializeOwned>(
        &self,
        agent_url: &str,
        endpoint: &str,
    ) -> Result<T> {
        let url = format!("{}/{}", agent_url, endpoint);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to connect to agent at {}", agent_url))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Agent at {} returned error status: {} for endpoint {}",
                agent_url,
                response.status(),
                endpoint
            );
        }

        let data = response
            .json::<T>()
            .await
            .with_context(|| {
                format!(
                    "Failed to parse {} response from {}",
                    endpoint, agent_url
                )
            })?;

        Ok(data)
    }

    /// Query the health endpoint of an agent
    pub async fn query_health(&self, agent_url: &str) -> Result<HealthResponse> {
        self.query_endpoint(agent_url, "health").await
    }

    /// Query the memory endpoint of an agent
    pub async fn query_memory(&self, agent_url: &str) -> Result<Memory> {
        self.query_endpoint(agent_url, "memory").await
    }

    /// Query both health and memory from an agent concurrently
    pub async fn query_agent(&self, agent_url: &str) -> Result<AgentData> {
        // Query both endpoints concurrently for better performance
        let (health, memory) = tokio::try_join!(
            self.query_health(agent_url),
            self.query_memory(agent_url)
        )?;

        Ok(AgentData {
            url: agent_url.to_string(),
            health,
            memory,
        })
    }

    /// Query multiple agents concurrently
    pub async fn query_agents(&self, agent_urls: &[String]) -> Vec<AgentResult> {
        let futures = agent_urls.iter().map(|url| async move {
            match self.query_agent(url).await {
                Ok(data) => AgentResult::Success(data),
                Err(e) => AgentResult::Failure {
                    url: url.clone(),
                    error: e.to_string(),
                },
            }
        });

        futures::future::join_all(futures).await
    }
}

/// Complete data from an agent
#[derive(Debug, Clone)]
pub struct AgentData {
    pub url: String,
    pub health: HealthResponse,
    pub memory: Memory,
}

/// Result of querying an agent
#[derive(Debug, Clone)]
pub enum AgentResult {
    Success(AgentData),
    Failure { url: String, error: String },
}

impl AgentResult {
    /// Check if the result is a success
    pub fn is_success(&self) -> bool {
        matches!(self, AgentResult::Success(_))
    }

    /// Get the agent URL
    pub fn url(&self) -> &str {
        match self {
            AgentResult::Success(data) => &data.url,
            AgentResult::Failure { url, .. } => url,
        }
    }

    /// Get the agent data if successful
    pub fn data(&self) -> Option<&AgentData> {
        match self {
            AgentResult::Success(data) => Some(data),
            AgentResult::Failure { .. } => None,
        }
    }
}

/// Aggregated metrics from multiple agents
#[derive(Debug, Clone, Default)]
pub struct AggregatedMetrics {
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub failed_nodes: usize,
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
}

impl AggregatedMetrics {
    /// Calculate memory usage percentage
    pub fn memory_usage_percent(&self) -> f64 {
        if self.total_memory_bytes == 0 {
            return 0.0;
        }
        let used = self.total_memory_bytes.saturating_sub(self.available_memory_bytes);
        (used as f64 / self.total_memory_bytes as f64) * 100.0
    }

    /// Calculate available memory percentage
    pub fn memory_available_percent(&self) -> f64 {
        if self.total_memory_bytes == 0 {
            return 0.0;
        }
        (self.available_memory_bytes as f64 / self.total_memory_bytes as f64) * 100.0
    }
}

/// Aggregate results from multiple agents
pub fn aggregate_results(results: &[AgentResult]) -> AggregatedMetrics {
    let mut metrics = AggregatedMetrics::default();

    for result in results {
        metrics.total_nodes += 1;

        match result {
            AgentResult::Success(data) => {
                metrics.healthy_nodes += 1;
                // Memory values are in KB from /proc/meminfo, convert to bytes
                metrics.total_memory_bytes += data.memory.total * 1024;
                metrics.available_memory_bytes += data.memory.available * 1024;
            }
            AgentResult::Failure { .. } => {
                metrics.failed_nodes += 1;
            }
        }
    }

    metrics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_result_success() {
        let data = AgentData {
            url: "http://localhost:3000".to_string(),
            health: HealthResponse::new("up".to_string(), 100, "0.1.0".to_string()),
            memory: Memory::new(16000000, 0, 0, 8000000),
        };
        let result = AgentResult::Success(data);
        
        assert!(result.is_success());
        assert_eq!(result.url(), "http://localhost:3000");
        assert!(result.data().is_some());
    }

    #[test]
    fn test_agent_result_failure() {
        let result = AgentResult::Failure {
            url: "http://localhost:3000".to_string(),
            error: "Connection refused".to_string(),
        };
        
        assert!(!result.is_success());
        assert_eq!(result.url(), "http://localhost:3000");
        assert!(result.data().is_none());
    }

    #[test]
    fn test_aggregate_metrics() {
        let results = vec![
            AgentResult::Success(AgentData {
                url: "http://agent1:3000".to_string(),
                health: HealthResponse::new("up".to_string(), 100, "0.1.0".to_string()),
                memory: Memory::new(16000000, 0, 0, 8000000),
            }),
            AgentResult::Success(AgentData {
                url: "http://agent2:3000".to_string(),
                health: HealthResponse::new("up".to_string(), 200, "0.1.0".to_string()),
                memory: Memory::new(16000000, 0, 0, 4000000),
            }),
            AgentResult::Failure {
                url: "http://agent3:3000".to_string(),
                error: "Timeout".to_string(),
            },
        ];

        let metrics = aggregate_results(&results);
        
        assert_eq!(metrics.total_nodes, 3);
        assert_eq!(metrics.healthy_nodes, 2);
        assert_eq!(metrics.failed_nodes, 1);
        assert_eq!(metrics.total_memory_bytes, 16000000 * 1024 * 2);
        assert_eq!(metrics.available_memory_bytes, (8000000 + 4000000) * 1024);
    }
}
