use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Node metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetricsResponse {
    pub nodes: Vec<NodeMetrics>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub name: String,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage: String,
    pub usage_percentage: f64,
    pub capacity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub usage: String,
    pub usage_percentage: f64,
    pub capacity: String,
}

// Pod metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodMetricsResponse {
    pub pods: Vec<PodMetrics>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodMetrics {
    pub name: String,
    pub namespace: String,
    pub node: String,
    pub cpu: String,
    pub memory: String,
}

// Summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryResponse {
    pub nodes: NodeSummary,
    pub pods: PodSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSummary {
    pub count: usize,
    #[serde(rename = "cpuUsage")]
    pub cpu_usage: f64,      // Average percentage
    #[serde(rename = "memoryUsage")]
    pub memory_usage: f64,   // Average percentage
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodSummary {
    pub count: usize,
    #[serde(rename = "cpuUsage")]
    pub cpu_usage: String,   // Total CPU in K8s format
    #[serde(rename = "memoryUsage")]
    pub memory_usage: String, // Total memory in K8s format
}
