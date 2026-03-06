use super::{error::MetricsResult, resource::*, K8sState};
use axum::{extract::State, Json};
use chrono::Utc;
use kube::{Api, ResourceExt};
use rwatch_common::metrics::{PodMetrics, PodMetricsResponse};

pub struct PodMetricsHandler;

impl PodMetricsHandler {
    pub async fn handler(State(state): State<K8sState>) -> MetricsResult<Json<PodMetricsResponse>> {
        let client = state.client();
        
        // Query metrics-server for pod metrics (all namespaces)
        let metrics_api: Api<k8s_metrics::PodMetrics> = Api::all(client.clone());
        let pod_metrics = metrics_api
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
        
        // Build response
        let mut pods = Vec::new();
        for metric in pod_metrics.items {
            // Sum containers to get total pod usage
            let total_cpu: u64 = metric
                .containers
                .iter()
                .filter_map(|c| {
                    parse_cpu_to_millicores(&c.usage.cpu.0).ok()
                })
                .sum();
            
            let total_mem: u64 = metric
                .containers
                .iter()
                .filter_map(|c| {
                    parse_memory_to_mib(&c.usage.memory.0).ok()
                })
                .sum();
            
            // Get node from metadata labels if available
            let node = metric
                .metadata
                .labels
                .as_ref()
                .and_then(|l: &std::collections::BTreeMap<String, String>| l.get("kubernetes.io/hostname"))
                .cloned()
                .unwrap_or_default();
            
            pods.push(PodMetrics {
                name: metric.name_any(),
                namespace: metric.namespace().unwrap_or_default(),
                node,
                cpu: format_cpu(total_cpu),
                memory: format_memory(total_mem),
            });
        }
        
        Ok(Json(PodMetricsResponse {
            pods,
            timestamp: Utc::now(),
        }))
    }
}
