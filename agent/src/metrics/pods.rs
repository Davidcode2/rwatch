use super::{error::MetricsResult, resource::*, K8sState};
use axum::{extract::State, Json};
use chrono::Utc;
use k8s_openapi::api::core::v1::Pod;
use kube::Api;
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

        // Query K8s API for pods to get node mapping
        // metrics-server does NOT include node info in PodMetrics
        let pods_api: Api<Pod> = Api::all(client.clone());
        let pods = pods_api
            .list(&Default::default())
            .await
            .map_err(|e| super::error::MetricsError::K8sClient(e.to_string()))?;

        // Build a map of pod_uid -> node_name for quick lookup
        let mut node_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for pod in &pods {
            if let (Some(uid), Some(spec)) = (pod.metadata.uid.as_ref(), pod.spec.as_ref()) {
                if let Some(node_name) = spec.node_name.as_ref() {
                    node_map.insert(uid.clone(), node_name.clone());
                }
            }
        }

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

            // Get metadata fields directly from ObjectMeta
            let name = metric.metadata.name.clone().unwrap_or_default();
            let namespace = metric.metadata.namespace.clone().unwrap_or_default();

            // Get node name from our lookup map using the pod's UID
            let node = metric
                .metadata
                .uid
                .as_ref()
                .and_then(|uid| node_map.get(uid))
                .cloned()
                .unwrap_or_default();

            pods.push(PodMetrics {
                name,
                namespace,
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
