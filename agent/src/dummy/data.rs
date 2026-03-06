//! Dummy data generators for all API endpoints
//!
//! Generates realistic-looking data for testing the frontend without
//! requiring a real Kubernetes cluster.

use chrono::Utc;
use rand::Rng;
use rwatch_common::health::HealthResponse;
use rwatch_common::memory::Memory;
use rwatch_common::metrics::{CpuMetrics, MemoryMetrics, NodeMetrics, NodeMetricsResponse};
use rwatch_common::metrics::{NodeSummary, PodSummary, SummaryResponse};
use rwatch_common::metrics::{PodMetrics, PodMetricsResponse};

use super::state::DummyState;

/// Generate health check response
pub fn generate_health_response(state: &DummyState) -> HealthResponse {
    let uptime = state.start_time.elapsed().as_secs();

    HealthResponse {
        status: "up".to_string(),
        uptime,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Generate memory metrics response
pub fn generate_memory_response() -> Memory {
    let mut rng = rand::thread_rng();

    // Generate a realistic total memory (8-32 GB)
    let total_gb: u64 = rng.gen_range(8..=32);
    let total_kb = total_gb * 1024 * 1024;

    // Calculate used memory (30-70% of total)
    let usage_pct: f64 = rng.gen_range(0.30..=0.70);
    let used_kb = (total_kb as f64 * usage_pct) as u64;
    let available_kb = total_kb - used_kb;

    // Free memory is roughly 10-20% of total (unused + buffers/cache)
    let free_pct: f64 = rng.gen_range(0.10..=0.20);
    let free_kb = (total_kb as f64 * free_pct) as u64;

    Memory {
        total: total_kb,
        used: used_kb,
        free: free_kb,
        available: available_kb,
    }
}

/// Generate node metrics response
pub fn generate_node_metrics_response(state: &DummyState) -> NodeMetricsResponse {
    let nodes = if let Ok(nodes) = state.node_metrics.lock() {
        nodes
            .iter()
            .map(|node| {
                let cpu_usage_milli =
                    ((node.cpu_usage_current / 100.0) * node.cpu_capacity as f64) as i32;
                let mem_usage_mi =
                    ((node.memory_usage_current / 100.0) * node.memory_capacity as f64) as i64;

                NodeMetrics {
                    name: node.name.clone(),
                    cpu: CpuMetrics {
                        usage: format!("{}m", cpu_usage_milli),
                        usage_percentage: node.cpu_usage_current,
                        capacity: format!("{}m", node.cpu_capacity),
                    },
                    memory: MemoryMetrics {
                        usage: format!("{}Mi", mem_usage_mi),
                        usage_percentage: node.memory_usage_current,
                        capacity: format!("{}Mi", node.memory_capacity),
                    },
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    NodeMetricsResponse {
        nodes,
        timestamp: Utc::now(),
    }
}

/// Generate pod metrics response
pub fn generate_pod_metrics_response(state: &DummyState) -> PodMetricsResponse {
    let pods = if let Ok(pods) = state.pod_metrics.lock() {
        pods.iter()
            .map(|pod| PodMetrics {
                name: pod.name.clone(),
                namespace: pod.namespace.clone(),
                node: pod.node.clone(),
                cpu: format!("{}m", pod.cpu_current),
                memory: format!("{}Mi", pod.memory_current),
            })
            .collect()
    } else {
        Vec::new()
    };

    PodMetricsResponse {
        pods,
        timestamp: Utc::now(),
    }
}

/// Generate summary response based on current node and pod metrics
pub fn generate_summary_response(state: &DummyState) -> SummaryResponse {
    // Calculate node summary
    let (node_count, avg_cpu, avg_memory) = if let Ok(nodes) = state.node_metrics.lock() {
        let count = nodes.len();
        if count > 0 {
            let avg_cpu: f64 =
                nodes.iter().map(|n| n.cpu_usage_current).sum::<f64>() / count as f64;
            let avg_memory: f64 =
                nodes.iter().map(|n| n.memory_usage_current).sum::<f64>() / count as f64;
            (count, avg_cpu, avg_memory)
        } else {
            (0, 0.0, 0.0)
        }
    } else {
        (0, 0.0, 0.0)
    };

    // Calculate pod summary
    let (pod_count, total_cpu, total_memory) = if let Ok(pods) = state.pod_metrics.lock() {
        let count = pods.len();
        let total_cpu: i32 = pods.iter().map(|p| p.cpu_current).sum();
        let total_memory: i64 = pods.iter().map(|p| p.memory_current).sum();
        (count, total_cpu, total_memory)
    } else {
        (0, 0, 0)
    };

    SummaryResponse {
        nodes: NodeSummary {
            count: node_count,
            cpu_usage: avg_cpu,
            memory_usage: avg_memory,
        },
        pods: PodSummary {
            count: pod_count,
            cpu_usage: format!("{}m", total_cpu),
            memory_usage: format!("{}Mi", total_memory),
        },
    }
}
