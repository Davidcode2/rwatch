//! Kubernetes metrics-server integration

mod error;
mod nodes;
mod pods;
mod resource;
mod summary;

pub use error::{MetricsError, MetricsResult};
pub use nodes::NodeMetricsHandler;
pub use pods::PodMetricsHandler;
pub use summary::SummaryHandler;

use kube::Client;
use std::sync::Arc;

/// Shared Kubernetes client state
#[derive(Clone)]
pub struct K8sState {
    client: Arc<Client>,
}

impl K8sState {
    pub async fn new() -> anyhow::Result<Self> {
        let config = kube::Config::infer()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load K8s config: {}", e))?;
        
        let client = Client::try_from(config)
            .map_err(|e| anyhow::anyhow!("Failed to create K8s client: {}", e))?;
        
        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Create a K8sState from an existing client (useful for testing)
    pub fn from_client(client: Client) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
    
    pub fn client(&self) -> &Client {
        &self.client
    }
}
