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

        // DEBUG: Log raw response count
        println!("[DEBUG] PodMetricsHandler: Received {} pod metrics from metrics-server", pod_metrics.items.len());

        // Build response
        let mut pods = Vec::new();
        for (idx, metric) in pod_metrics.items.iter().enumerate() {
            // DEBUG: Log raw metadata fields
            println!("[DEBUG] Pod[{}]: name_any()='{}'", idx, metric.name_any());
            println!("[DEBUG] Pod[{}]: metadata.name='{:?}'", idx, metric.metadata.name);
            println!("[DEBUG] Pod[{}]: metadata.namespace='{:?}'", idx, metric.metadata.namespace);
            println!("[DEBUG] Pod[{}]: metadata.uid='{:?}'", idx, metric.metadata.uid);
            println!("[DEBUG] Pod[{}]: metadata.labels='{:?}'", idx, metric.metadata.labels);
            println!("[DEBUG] Pod[{}]: containers.len()={}", idx, metric.containers.len());

            // Sum containers to get total pod usage
            let total_cpu: u64 = metric
                .containers
                .iter()
                .filter_map(|c| {
                    // DEBUG: Log container parsing
                    let cpu_str = &c.usage.cpu.0;
                    let parse_result = parse_cpu_to_millicores(cpu_str);
                    println!("[DEBUG] Pod[{}] Container '{}': cpu='{}' parse_result='{:?}'", idx, c.name, cpu_str, parse_result);
                    parse_result.ok()
                })
                .sum();

            let total_mem: u64 = metric
                .containers
                .iter()
                .filter_map(|c| {
                    // DEBUG: Log container parsing
                    let mem_str = &c.usage.memory.0;
                    let parse_result = parse_memory_to_mib(mem_str);
                    println!("[DEBUG] Pod[{}] Container '{}': memory='{}' parse_result='{:?}'", idx, c.name, mem_str, parse_result);
                    parse_result.ok()
                })
                .sum();

            println!("[DEBUG] Pod[{}]: total_cpu={} total_mem={}", idx, total_cpu, total_mem);

            // Get node from metadata labels if available
            let node = metric
                .metadata
                .labels
                .as_ref()
                .and_then(|l: &std::collections::BTreeMap<String, String>| l.get("kubernetes.io/hostname"))
                .cloned()
                .unwrap_or_default();

            println!("[DEBUG] Pod[{}]: node from labels='{}'", idx, node);

            let namespace = metric.namespace().unwrap_or_default();
            println!("[DEBUG] Pod[{}]: final namespace='{}' node='{}'", idx, namespace, node);

            pods.push(PodMetrics {
                name: metric.name_any(),
                namespace,
                node,
                cpu: format_cpu(total_cpu),
                memory: format_memory(total_mem),
            });
        }

        println!("[DEBUG] PodMetricsHandler: Returning {} pods in response", pods.len());

        Ok(Json(PodMetricsResponse {
            pods,
            timestamp: Utc::now(),
        }))
    }
}
