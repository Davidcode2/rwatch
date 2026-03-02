//! Agent discovery mechanisms

use crate::agent::{AgentConfig, AgentList};
use anyhow::Result;

/// Static discovery from a predefined list
#[derive(Debug, Clone)]
pub struct StaticDiscovery {
    urls: Vec<String>,
}

impl StaticDiscovery {
    /// Create a new static discovery with a list of URLs
    pub fn new(urls: Vec<String>) -> Self {
        Self { urls }
    }

    /// Create from a list of agent URLs
    pub fn from_urls(urls: &[&str]) -> Self {
        Self {
            urls: urls.iter().map(|&s| s.to_string()).collect(),
        }
    }

    /// Discover available agents
    pub async fn discover(&self) -> Result<AgentList> {
        let agents = self
            .urls
            .iter()
            .map(|url| AgentConfig::new(url.clone()))
            .collect::<Vec<_>>();

        Ok(AgentList::from(agents))
    }
}

/// Kubernetes-based discovery for rwatch agents
#[derive(Debug, Clone)]
pub struct KubernetesDiscovery {
    namespace: String,
    service_name: String,
    port: u16,
}

impl KubernetesDiscovery {
    /// Create a new Kubernetes discovery
    pub fn new(namespace: impl Into<String>, service_name: impl Into<String>, port: u16) -> Self {
        Self {
            namespace: namespace.into(),
            service_name: service_name.into(),
            port,
        }
    }

    /// Get the DNS name for the headless service
    pub fn service_dns(&self) -> String {
        format!("{}.{}", self.service_name, self.namespace)
    }

    /// Discover available agents
    pub async fn discover(&self) -> Result<AgentList> {
        // In a real implementation, this would query the Kubernetes API
        // For now, we return an empty list with a note about implementation
        // TODO: Implement actual Kubernetes API queries
        
        // Placeholder: would query pods with label app=rwatch-agent
        // and build URLs from pod IPs
        Ok(AgentList::new())
    }
}

/// Environment-based discovery from env vars
#[derive(Debug, Clone)]
pub struct EnvDiscovery {
    prefix: String,
}

impl EnvDiscovery {
    /// Create a new environment discovery
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

    /// Create with default prefix "RWATCH_AGENT"
    pub fn default_prefix() -> Self {
        Self::new("RWATCH_AGENT")
    }

    /// Discover available agents
    pub async fn discover(&self) -> Result<AgentList> {
        use std::env;

        let mut agents = Vec::new();
        let mut index = 0;

        loop {
            let var_name = format!("{}_{}", self.prefix, index);
            match env::var(&var_name) {
                Ok(url) => {
                    agents.push(AgentConfig::new(url).with_name(format!("agent-{}", index)));
                    index += 1;
                }
                Err(_) => break,
            }
        }

        Ok(AgentList::from(agents))
    }
}

/// Unified discovery enum that wraps all discovery types
#[derive(Debug, Clone)]
pub enum Discovery {
    Static(StaticDiscovery),
    Kubernetes(KubernetesDiscovery),
    Env(EnvDiscovery),
}

impl Discovery {
    /// Create a static discovery
    pub fn static_discovery(urls: Vec<String>) -> Self {
        Self::Static(StaticDiscovery::new(urls))
    }

    /// Create a Kubernetes discovery
    pub fn kubernetes(namespace: impl Into<String>, service_name: impl Into<String>, port: u16) -> Self {
        Self::Kubernetes(KubernetesDiscovery::new(namespace, service_name, port))
    }

    /// Create an environment discovery
    pub fn env(prefix: impl Into<String>) -> Self {
        Self::Env(EnvDiscovery::new(prefix))
    }

    /// Discover available agents
    pub async fn discover(&self) -> Result<AgentList> {
        match self {
            Discovery::Static(d) => d.discover().await,
            Discovery::Kubernetes(d) => d.discover().await,
            Discovery::Env(d) => d.discover().await,
        }
    }
}

impl From<StaticDiscovery> for Discovery {
    fn from(d: StaticDiscovery) -> Self {
        Self::Static(d)
    }
}

impl From<KubernetesDiscovery> for Discovery {
    fn from(d: KubernetesDiscovery) -> Self {
        Self::Kubernetes(d)
    }
}

impl From<EnvDiscovery> for Discovery {
    fn from(d: EnvDiscovery) -> Self {
        Self::Env(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_static_discovery() {
        let discovery = StaticDiscovery::from_urls(&[
            "http://agent1:3000",
            "http://agent2:3000",
        ]);

        let agents = discovery.discover().await.unwrap();
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn test_discovery_enum() {
        let discovery = Discovery::static_discovery(vec![
            "http://agent1:3000".to_string(),
            "http://agent2:3000".to_string(),
        ]);

        let agents = discovery.discover().await.unwrap();
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn test_kubernetes_discovery() {
        let discovery = KubernetesDiscovery::new("rwatch", "rwatch-agent", 3000);
        assert_eq!(discovery.service_dns(), "rwatch-agent.rwatch");

        // Currently returns empty list (placeholder implementation)
        let agents = discovery.discover().await.unwrap();
        assert!(agents.is_empty());
    }
}
