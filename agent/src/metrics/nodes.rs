use super::{error::MetricsResult, resource::*, K8sState};
use axum::{extract::State, Json};
use chrono::Utc;
use k8s_openapi::api::core::v1::Node;
use kube::{Api, ResourceExt};
use rwatch_common::metrics::{CpuMetrics, MemoryMetrics, NodeMetrics, NodeMetricsResponse};

pub struct NodeMetricsHandler;

impl NodeMetricsHandler {
    pub async fn handler(State(state): State<K8sState>) -> MetricsResult<Json<NodeMetricsResponse>> {
        let client = state.client();
        
        // Query metrics-server for node usage
        let metrics_api: Api<k8s_metrics::NodeMetrics> = Api::all(client.clone());
        let node_metrics = metrics_api
            .list(&Default::default())
            .await
            .map_err(|e| match e {
                kube::Error::Api(resp) if resp.code == 404 => {
                    super::error::MetricsError::MetricsServerUnavailable(
                        "metrics-server not found".to_string()
                    )
                }
                _ => super::error::MetricsError::K8sClient(e.to_string()),
            })?;
        
        // Query K8s API for node capacity
        let nodes_api: Api<Node> = Api::all(client.clone());
        let nodes = nodes_api
            .list(&Default::default())
            .await
            .map_err(|e| super::error::MetricsError::K8sClient(e.to_string()))?;
        
        // Build capacity map
        let mut capacities = std::collections::HashMap::new();
        for node in &nodes {
            let name = node.name_any();
            if let Some(status) = &node.status {
                if let Some(capacity) = &status.capacity {
                    let cpu = capacity.get("cpu").map(|q| q.0.clone()).unwrap_or_default();
                    let memory = capacity.get("memory").map(|q| q.0.clone()).unwrap_or_default();
                    capacities.insert(name, (cpu, memory));
                }
            }
        }
        
        // Build response
        let mut nodes_result = Vec::new();
        for metric in node_metrics.items {
            let name = metric.name_any();
            
            let usage_cpu = metric.usage.cpu.0.clone();
            let usage_mem = metric.usage.memory.0.clone();
            
            let (capacity_cpu, capacity_mem) = capacities
                .get(&name)
                .cloned()
                .unwrap_or_default();
            
            // Calculate percentages
            let cpu_pct = if !capacity_cpu.is_empty() && !usage_cpu.is_empty() {
                let usage_milli = parse_cpu_to_millicores(&usage_cpu)?;
                let cap_milli = parse_cpu_to_millicores(&capacity_cpu)?;
                if cap_milli > 0 {
                    (usage_milli as f64 / cap_milli as f64) * 100.0
                } else {
                    0.0
                }
            } else {
                0.0
            };
            
            let mem_pct = if !capacity_mem.is_empty() && !usage_mem.is_empty() {
                let usage_mib = parse_memory_to_mib(&usage_mem)?;
                let cap_mib = parse_memory_to_mib(&capacity_mem)?;
                if cap_mib > 0 {
                    (usage_mib as f64 / cap_mib as f64) * 100.0
                } else {
                    0.0
                }
            } else {
                0.0
            };
            
            nodes_result.push(NodeMetrics {
                name,
                cpu: CpuMetrics {
                    usage: usage_cpu,
                    usage_percentage: cpu_pct,
                    capacity: capacity_cpu,
                },
                memory: MemoryMetrics {
                    usage: usage_mem,
                    usage_percentage: mem_pct,
                    capacity: capacity_mem,
                },
            });
        }
        
        Ok(Json(NodeMetricsResponse {
            nodes: nodes_result,
            timestamp: Utc::now(),
        }))
    }
}
