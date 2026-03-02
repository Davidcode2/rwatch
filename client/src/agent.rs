//! Agent representation and configuration

use serde::{Deserialize, Serialize};

/// Configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub url: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub labels: Vec<(String, String)>,
}

impl AgentConfig {
    /// Create a new agent config with just a URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            name: None,
            labels: Vec::new(),
        }
    }

    /// Set a name for the agent
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add a label to the agent
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.push((key.into(), value.into()));
        self
    }

    /// Get the display name for this agent
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.url)
    }
}

/// A collection of agent configurations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentList {
    pub agents: Vec<AgentConfig>,
}

impl AgentList {
    /// Create an empty agent list
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an agent to the list
    pub fn add(&mut self, agent: AgentConfig) {
        self.agents.push(agent);
    }

    /// Get the number of agents
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Get all agent URLs as strings
    pub fn urls(&self) -> Vec<String> {
        self.agents.iter().map(|a| a.url.clone()).collect()
    }
}

impl From<Vec<AgentConfig>> for AgentList {
    fn from(agents: Vec<AgentConfig>) -> Self {
        Self { agents }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config() {
        let config = AgentConfig::new("http://localhost:3000")
            .with_name("local-agent")
            .with_label("env", "dev");

        assert_eq!(config.url, "http://localhost:3000");
        assert_eq!(config.name, Some("local-agent".to_string()));
        assert_eq!(config.display_name(), "local-agent");
        assert_eq!(config.labels.len(), 1);
    }

    #[test]
    fn test_agent_list() {
        let mut list = AgentList::new();
        list.add(AgentConfig::new("http://agent1:3000"));
        list.add(AgentConfig::new("http://agent2:3000"));

        assert_eq!(list.len(), 2);
        assert_eq!(
            list.urls(),
            vec!["http://agent1:3000", "http://agent2:3000"]
        );
    }
}
