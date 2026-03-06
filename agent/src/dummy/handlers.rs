//! HTTP handlers for dummy data endpoints
//!
//! These handlers mirror the real handlers but return generated data
//! instead of querying actual system/Kubernetes resources.

use axum::{Json, extract::State};
use rwatch_common::health::HealthResponse;
use rwatch_common::memory::Memory;
use rwatch_common::metrics::{NodeMetricsResponse, PodMetricsResponse, SummaryResponse};

use super::state::DummyState;
use super::data;

/// State wrapper that combines dummy state with a flag
#[derive(Clone)]
pub struct DummyAppState {
    pub dummy_state: std::sync::Arc<DummyState>,
}

/// Handler for GET /health
pub async fn health_handler(State(state): State<DummyAppState>) -> Json<HealthResponse> {
    // Update variations on each request for smooth changes
    state.dummy_state.update_variations();
    
    let response = data::generate_health_response(&state.dummy_state);
    Json(response)
}

/// Handler for GET /memory
pub async fn memory_handler(State(_state): State<DummyAppState>) -> Json<Memory> {
    // Memory is generated fresh each time with slight variations
    let response = data::generate_memory_response();
    Json(response)
}

/// Handler for GET /api/metrics/nodes
pub async fn nodes_handler(State(state): State<DummyAppState>) -> Json<NodeMetricsResponse> {
    let response = data::generate_node_metrics_response(&state.dummy_state);
    Json(response)
}

/// Handler for GET /api/metrics/pods
pub async fn pods_handler(State(state): State<DummyAppState>) -> Json<PodMetricsResponse> {
    let response = data::generate_pod_metrics_response(&state.dummy_state);
    Json(response)
}

/// Handler for GET /api/metrics/summary
pub async fn summary_handler(State(state): State<DummyAppState>) -> Json<SummaryResponse> {
    let response = data::generate_summary_response(&state.dummy_state);
    Json(response)
}
