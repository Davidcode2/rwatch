use super::{error::MetricsResult, resource::*, K8sState};
use axum::{extract::State, Json};
use k8s_openapi::api::core::v1::Node;
use kube::{Api, ResourceExt};
use rwatch_common::metrics::{NodeSummary, PodSummary, SummaryResponse};

pub struct SummaryHandler;

impl SummaryHandler {
    pub async fn handler(State(state): State<K8sState>) -> MetricsResult<Json<SummaryResponse>> {
        let client = state.client();
        
        // Get node metrics
        let node_metrics_api: Api<k8s_metrics::NodeMetrics> = Api::all(client.clone());
        let node_metrics = node_metrics_api
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
        
        // Get node capacities
        let nodes_api: Api<Node> = Api::all(client.clone());
        let nodes = nodes_api
            .list(&Default::default())
            .await
            .map_err(|e| super::error::MetricsError::K8sClient(e.to_string()))?;
        
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
        
        // Calculate node averages
        let mut total_cpu_pct = 0.0;
        let mut total_mem_pct = 0.0;
        let node_count = node_metrics.items.len();
        
        for metric in &node_metrics.items {
            let name = metric.name_any();
            if let Some((cap_cpu, cap_mem)) = capacities.get(&name) {
                let usage_cpu = &metric.usage.cpu.0;
                let usage_mem = &metric.usage.memory.0;
                
                if let (Ok(usage_milli), Ok(cap_milli)) = (
                    parse_cpu_to_millicores(usage_cpu),
                    parse_cpu_to_millicores(cap_cpu),
                ) {
                    if cap_milli > 0 {
                        total_cpu_pct += (usage_milli as f64 / cap_milli as f64) * 100.0;
                    }
                }
                
                if let (Ok(usage_mib), Ok(cap_mib)) = (
                    parse_memory_to_mib(usage_mem),
                    parse_memory_to_mib(cap_mem),
                ) {
                    if cap_mib > 0 {
                        total_mem_pct += (usage_mib as f64 / cap_mib as f64) * 100.0;
                    }
                }
            }
        }
        
        let avg_cpu_pct = if node_count > 0 {
            total_cpu_pct / node_count as f64
        } else {
            0.0
        };
        
        let avg_mem_pct = if node_count > 0 {
            total_mem_pct / node_count as f64
        } else {
            0.0
        };
        
        // Get pod metrics for totals
        let pod_metrics_api: Api<k8s_metrics::PodMetrics> = Api::all(client.clone());
        let pod_metrics = pod_metrics_api
            .list(&Default::default())
            .await
            .map_err(|e: kube::Error| super::error::MetricsError::K8sClient(e.to_string()))?;
        
        let total_pod_cpu: u64 = pod_metrics
            .items
            .iter()
            .flat_map(|m| &m.containers)
            .filter_map(|c| parse_cpu_to_millicores(&c.usage.cpu.0).ok())
            .sum();
        
        let total_pod_mem: u64 = pod_metrics
            .items
            .iter()
            .flat_map(|m| &m.containers)
            .filter_map(|c| parse_memory_to_mib(&c.usage.memory.0).ok())
            .sum();
        
        Ok(Json(SummaryResponse {
            nodes: NodeSummary {
                count: node_count,
                cpu_usage: avg_cpu_pct,
                memory_usage: avg_mem_pct,
            },
            pods: PodSummary {
                count: pod_metrics.items.len(),
                cpu_usage: format_cpu(total_pod_cpu),
                memory_usage: format_memory(total_pod_mem),
            },
        }))
    }
}
