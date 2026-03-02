//! # Rwatch TUI
//!
//! A terminal user interface that connects to agents and displays their health status.
//! Uses the rwatch-client library for all agent communication.

use anyhow::{Context, Result};
use rwatch_client::{Client, aggregate_results};
use rwatch_client::discovery::{StaticDiscovery, Discovery};
use crate::ui::{display_health, display_memory, display_aggregated_metrics, display_agent_summary};

mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    run().await.context("Failed to run TUI application")?;
    Ok(())
}

/// Main application logic
async fn run() -> Result<()> {
    println!("🔍 Rwatch TUI - Monitoring Cluster\n");

    // Create the client for querying agents
    let client = Client::new();

    // Discover agents from configuration
    // For now, use static discovery with multiple agent URLs
    // In production, this would come from config or Kubernetes discovery
    let discovery = get_discovery().await?;
    
    let agent_list = discovery.discover().await
        .context("Failed to discover agents")?;

    if agent_list.is_empty() {
        println!("⚠️  No agents discovered. Please check your configuration.");
        return Ok(());
    }

    println!("📡 Discovered {} agent(s)\n", agent_list.len());

    // Query all agents concurrently
    let urls = agent_list.urls();
    let results = client.query_agents(&urls).await;

    // Aggregate metrics
    let aggregated = aggregate_results(&results);

    // Display results
    display_aggregated_metrics(&aggregated);
    println!();

    // Display individual agent details
    for result in &results {
        display_agent_summary(result);
        
        if let Some(data) = result.data() {
            display_health(&data.url, &data.health);
            display_memory(&data.memory);
        }
        println!();
    }

    Ok(())
}

/// Get the discovery mechanism based on environment/configuration
async fn get_discovery() -> Result<Discovery> {
    // Priority:
    // 1. Environment variables (RWATCH_AGENT_*)
    // 2. Static configuration from config file
    // 3. Kubernetes service discovery
    
    // For now, try environment variables first
    let env_discovery = rwatch_client::discovery::EnvDiscovery::default_prefix();
    
    // If env vars are set, use them
    if let Ok(agent_list) = env_discovery.discover().await {
        if !agent_list.is_empty() {
            return Ok(Discovery::from(env_discovery));
        }
    }

    // Fallback to static configuration
    // In a real implementation, this would read from a config file
    let default_agents = vec![
        "http://localhost:3000".to_string(),
        // Add more default agents here or read from config
    ];
    
    Ok(StaticDiscovery::new(default_agents).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        // Test that we can create a client
        let _client = Client::new();
        // If we get here without panicking, the client was created successfully
    }
}
