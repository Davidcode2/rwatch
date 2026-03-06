# Kubernetes Metrics-Server Support Implementation Plan

## Overview
Add three new endpoints to rwatch-agent for querying Kubernetes metrics-server data:
- `GET /api/metrics/nodes` - Node CPU/memory with capacity percentages
- `GET /api/metrics/pods` - Pod metrics by namespace
- `GET /api/metrics/summary` - Aggregated cluster stats

## Current State Analysis

### Project Structure
```
rwatch/
├── Cargo.toml           # Workspace root
├── agent/               # HTTP server (Axum)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs      # Router setup
│       ├── health.rs    # Health handler
│       └── memory.rs    # Memory handler
├── common/              # Shared types
│   └── src/
│       ├── lib.rs
│       ├── health.rs
│       └── memory.rs
├── client/              # HTTP client
└── tui/                 # Terminal UI
```

### Existing Patterns
- Handlers are implemented as unit structs with static async methods
- Common types are defined in `rwatch-common` crate
- Axum 0.7 with `Json<T>` responses
- Error handling uses `anyhow::Result`

---

## Dependencies to Add

### 1. Agent Crate (`agent/Cargo.toml`)

Add to `[dependencies]`:

```toml
# Kubernetes client libraries
kube = { version = "0.98", features = ["client", "runtime", "ws"] }
k8s-openapi = { version = "0.24", features = ["v1_32"] }

# For resource quantity parsing
thiserror = "1.0"

# Already present (workspace):
# - tokio
# - serde
# - serde_json
# - anyhow
# - axum
# - tower
```

### 2. Common Crate (`common/Cargo.toml`)

Add to `[dependencies]`:
```toml
chrono = { version = "0.4", features = ["serde"] }
```

---

## File Changes Required

### New Files to Create

1. `agent/src/metrics/` - New module directory
2. `agent/src/metrics/mod.rs` - Module exports and shared K8s client
3. `agent/src/metrics/nodes.rs` - Node metrics handler
4. `agent/src/metrics/pods.rs` - Pod metrics handler
5. `agent/src/metrics/summary.rs` - Summary handler
6. `agent/src/metrics/error.rs` - Error handling types
7. `common/src/metrics.rs` - Shared metrics response types

### Files to Modify

1. `agent/Cargo.toml` - Add dependencies
2. `agent/src/main.rs` - Add new routes to router
3. `common/Cargo.toml` - Add chrono dependency
4. `common/src/lib.rs` - Export metrics module

---

## Implementation Details

### 1. K8s Client Initialization

**File:** `agent/src/metrics/mod.rs`

```rust
use kube::{Client, Config};
use std::sync::Arc;

/// Shared Kubernetes client state
#[derive(Clone)]
pub struct K8sState {
    client: Arc<Client>,
}

impl K8sState {
    pub async fn new() -> anyhow::Result<Self> {
        // Try in-cluster config first, then kubeconfig
        let config = Config::infer()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load K8s config: {}", e))?;
        
        let client = Client::try_from(config)
            .map_err(|e| anyhow::anyhow!("Failed to create K8s client: {}", e))?;
        
        Ok(Self {
            client: Arc::new(client),
        })
    }
    
    pub fn client(&self) -> &Client {
        &self.client
    }
}
```

### 2. Error Handling Strategy

**File:** `agent/src/metrics/error.rs`

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Kubernetes client error: {0}")]
    K8sClient(String),
    
    #[error("Metrics server unavailable: {0}")]
    MetricsServerUnavailable(String),
    
    #[error("Failed to parse resource quantity: {0}")]
    ParseError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for MetricsError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            MetricsError::MetricsServerUnavailable(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        
        let body = Json(json!({
            "error": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));
        
        (status, body).into_response()
    }
}

pub type MetricsResult<T> = Result<T, MetricsError>;
```

### 3. Resource Quantity Parsing

**File:** `agent/src/metrics/resource.rs` (or in mod.rs)

Kubernetes resource quantities like "450m" (millicores) and "2048Mi" need parsing:

```rust
/// Parse CPU quantity to millicores
/// Examples: "100m" -> 100, "2" -> 2000, "0.5" -> 500
pub fn parse_cpu_to_millicores(quantity: &str) -> MetricsResult<u64> {
    if quantity.ends_with('m') {
        quantity[..quantity.len()-1]
            .parse::<u64>()
            .map_err(|e| MetricsError::ParseError(format!("CPU '{}': {}", quantity, e)))
    } else {
        // Plain number means cores, convert to millicores
        quantity
            .parse::<f64>()
            .map(|v| (v * 1000.0) as u64)
            .map_err(|e| MetricsError::ParseError(format!("CPU '{}': {}", quantity, e)))
    }
}

/// Parse memory quantity to MiB
/// Examples: "2048Mi" -> 2048, "2Gi" -> 2048, "1048576Ki" -> 1024
pub fn parse_memory_to_mib(quantity: &str) -> MetricsResult<u64> {
    // Handle binary suffixes (Ki, Mi, Gi, Ti)
    if quantity.ends_with("Ki") {
        parse_with_suffix(&quantity[..quantity.len()-2], 1.0 / 1024.0)
    } else if quantity.ends_with("Mi") {
        parse_with_suffix(&quantity[..quantity.len()-2], 1.0)
    } else if quantity.ends_with("Gi") {
        parse_with_suffix(&quantity[..quantity.len()-2], 1024.0)
    } else if quantity.ends_with("Ti") {
        parse_with_suffix(&quantity[..quantity.len()-2], 1024.0 * 1024.0)
    } else if quantity.ends_with('k') {
        // Decimal kilobytes
        parse_with_suffix(&quantity[..quantity.len()-1], 1.0 / 1024.0)
    } else if quantity.ends_with('M') {
        // Decimal megabytes
        parse_with_suffix(&quantity[..quantity.len()-1], 0.953674) // MB to MiB
    } else if quantity.ends_with('G') {
        // Decimal gigabytes  
        parse_with_suffix(&quantity[..quantity.len()-1], 953.674) // GB to MiB
    } else {
        // Assume bytes
        quantity
            .parse::<u64>()
            .map(|v| v / (1024 * 1024))
            .map_err(|e| MetricsError::ParseError(format!("Memory '{}': {}", quantity, e)))
    }
}

fn parse_with_suffix(num_str: &str, multiplier: f64) -> MetricsResult<u64> {
    num_str
        .parse::<f64>()
        .map(|v| (v * multiplier) as u64)
        .map_err(|e| MetricsError::ParseError(format!("'{}': {}", num_str, e)))
}

/// Format millicores back to K8s style
pub fn format_cpu(millicores: u64) -> String {
    if millicores >= 1000 && millicores % 1000 == 0 {
        format!("{}", millicores / 1000)
    } else {
        format!("{}m", millicores)
    }
}

/// Format MiB back to K8s style
pub fn format_memory(mib: u64) -> String {
    if mib >= 1024 && mib % 1024 == 0 {
        format!("{}Gi", mib / 1024)
    } else {
        format!("{}Mi", mib)
    }
}
```

### 4. Common Types

**File:** `common/src/metrics.rs`

```rust
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
    pub cpu_usage: f64,      // Average percentage
    pub memory_usage: f64,   // Average percentage
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodSummary {
    pub count: usize,
    pub cpu_usage: String,   // Total CPU in K8s format
    pub memory_usage: String, // Total memory in K8s format
}
```

### 5. Node Metrics Handler

**File:** `agent/src/metrics/nodes.rs`

```rust
use super::{error::MetricsResult, resource::*, K8sState};
use axum::{extract::State, Json};
use chrono::Utc;
use k8s_openapi::api::core::v1::Node;
use k8s_openapi::api::metrics::v1beta1::NodeMetrics as K8sNodeMetrics;
use kube::{Api, ResourceExt};
use rwatch_common::metrics::{CpuMetrics, MemoryMetrics, NodeMetrics, NodeMetricsResponse};

pub struct NodeMetricsHandler;

impl NodeMetricsHandler {
    pub async fn handler(State(state): State<K8sState>) -> MetricsResult<Json<NodeMetricsResponse>> {
        let client = state.client();
        
        // Query metrics-server for node usage
        let metrics_api: Api<K8sNodeMetrics> = Api::all(client.clone());
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
        let mut nodes = Vec::new();
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
            
            nodes.push(NodeMetrics {
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
            nodes,
            timestamp: Utc::now(),
        }))
    }
}
```

### 6. Pod Metrics Handler

**File:** `agent/src/metrics/pods.rs`

```rust
use super::{error::MetricsResult, K8sState};
use axum::{extract::State, Json};
use chrono::Utc;
use k8s_openapi::api::metrics::v1beta1::PodMetrics as K8sPodMetrics;
use kube::{Api, ResourceExt};
use rwatch_common::metrics::{PodMetrics, PodMetricsResponse};

pub struct PodMetricsHandler;

impl PodMetricsHandler {
    pub async fn handler(State(state): State<K8sState>) -> MetricsResult<Json<PodMetricsResponse>> {
        let client = state.client();
        
        // Query metrics-server for pod metrics (all namespaces)
        let metrics_api: Api<K8sPodMetrics> = Api::all(client.clone());
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
                    super::resource::parse_cpu_to_millicores(&c.usage.cpu.0).ok()
                })
                .sum();
            
            let total_mem: u64 = metric
                .containers
                .iter()
                .filter_map(|c| {
                    super::resource::parse_memory_to_mib(&c.usage.memory.0).ok()
                })
                .sum();
            
            // Get node from metadata labels if available
            let node = metric
                .metadata
                .labels
                .as_ref()
                .and_then(|l| l.get("kubernetes.io/hostname"))
                .cloned()
                .unwrap_or_default();
            
            pods.push(PodMetrics {
                name: metric.name_any(),
                namespace: metric.namespace().unwrap_or_default(),
                node,
                cpu: super::resource::format_cpu(total_cpu),
                memory: super::resource::format_memory(total_mem),
            });
        }
        
        Ok(Json(PodMetricsResponse {
            pods,
            timestamp: Utc::now(),
        }))
    }
}
```

### 7. Summary Handler

**File:** `agent/src/metrics/summary.rs`

```rust
use super::{error::MetricsResult, resource::*, K8sState};
use axum::{extract::State, Json};
use k8s_openapi::api::core::v1::{Node, Pod};
use k8s_openapi::api::metrics::v1beta1::{NodeMetrics as K8sNodeMetrics, PodMetrics as K8sPodMetrics};
use kube::Api;
use rwatch_common::metrics::{NodeSummary, PodSummary, SummaryResponse};

pub struct SummaryHandler;

impl SummaryHandler {
    pub async fn handler(State(state): State<K8sState>) -> MetricsResult<Json<SummaryResponse>> {
        let client = state.client();
        
        // Get node metrics
        let node_metrics_api: Api<K8sNodeMetrics> = Api::all(client.clone());
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
        let pod_metrics_api: Api<K8sPodMetrics> = Api::all(client.clone());
        let pod_metrics = pod_metrics_api
            .list(&Default::default())
            .await
            .map_err(|e| super::error::MetricsError::K8sClient(e.to_string()))?;
        
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
```

### 8. Main.rs Router Updates

**File:** `agent/src/main.rs`

```rust
//! # Rwatch Agent
//!
//! A daemon that exposes system metrics via a REST API.

use axum::{Router, routing::get};
use std::net::SocketAddr;
use std::time::Instant;

mod health;
mod memory;
mod metrics;

use health::HealthHandler;
use memory::MemoryHandler;
use metrics::{K8sState, NodeMetricsHandler, PodMetricsHandler, SummaryHandler};

static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    START_TIME
        .set(Instant::now())
        .expect("START_TIME already initialized");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    // Initialize K8s client
    let k8s_state = K8sState::new().await?;

    let app = create_router(k8s_state);

    println!("🚀 Rwatch Agent starting on {}", addr);
    println!("📊 Health endpoint: http://{}/health", addr);
    println!("📊 Memory endpoint: http://{}/memory", addr);
    println!("☸️  K8s Nodes endpoint: http://{}/api/metrics/nodes", addr);
    println!("☸️  K8s Pods endpoint: http://{}/api/metrics/pods", addr);
    println!("☸️  K8s Summary endpoint: http://{}/api/metrics/summary", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_router(k8s_state: K8sState) -> Router {
    Router::new()
        .route("/health", get(HealthHandler::health_handler))
        .route("/memory", get(MemoryHandler::memory_handler))
        .route("/api/metrics/nodes", get(NodeMetricsHandler::handler))
        .route("/api/metrics/pods", get(PodMetricsHandler::handler))
        .route("/api/metrics/summary", get(SummaryHandler::handler))
        .with_state(k8s_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_router_creation() {
        let _ = START_TIME.set(Instant::now());
        let k8s_state = K8sState::new().await.expect("Failed to create K8sState");
        let app = create_router(k8s_state);
        // Router created successfully
    }
}
```

---

## Module Exports

### `agent/src/metrics/mod.rs`

```rust
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
    
    pub fn client(&self) -> &Client {
        &self.client
    }
}
```

---

## Testing Strategy

### Unit Tests (in each handler file)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu() {
        assert_eq!(parse_cpu_to_millicores("100m").unwrap(), 100);
        assert_eq!(parse_cpu_to_millicores("2").unwrap(), 2000);
        assert_eq!(parse_cpu_to_millicores("0.5").unwrap(), 500);
    }

    #[test]
    fn test_parse_memory() {
        assert_eq!(parse_memory_to_mib("1024Mi").unwrap(), 1024);
        assert_eq!(parse_memory_to_mib("1Gi").unwrap(), 1024);
        assert_eq!(parse_memory_to_mib("1048576Ki").unwrap(), 1024);
    }

    #[test]
    fn test_format_cpu() {
        assert_eq!(format_cpu(100), "100m");
        assert_eq!(format_cpu(2000), "2");
        assert_eq!(format_cpu(1500), "1500m");
    }
}
```

### Integration Tests (requires running cluster)

Create `agent/tests/metrics_integration.rs`:

```rust
//! Integration tests for K8s metrics endpoints
//! Requires a running Kubernetes cluster with metrics-server

use axum::body::Body;
use axum::http::StatusCode;
use tower::ServiceExt;

// These tests would use the actual K8s client
// and require a test cluster to be running
```

---

## Error Handling Summary

| Scenario | Response Code | Response Body |
|----------|---------------|---------------|
| Metrics server unavailable | 503 | `{"error": "Metrics server unavailable: ...", "timestamp": "..."}` |
| K8s API error | 500 | `{"error": "Kubernetes client error: ...", "timestamp": "..."}` |
| Resource parse error | 500 | `{"error": "Failed to parse resource quantity: ...", "timestamp": "..."}` |
| Success | 200 | API contract response |

---

## Implementation Order

1. **Phase 1: Setup**
   - Add dependencies to `agent/Cargo.toml` and `common/Cargo.toml`
   - Create `common/src/metrics.rs` with response types
   - Update `common/src/lib.rs` to export metrics module

2. **Phase 2: Core Infrastructure**
   - Create `agent/src/metrics/mod.rs` with `K8sState`
   - Create `agent/src/metrics/error.rs` for error handling
   - Create `agent/src/metrics/resource.rs` for quantity parsing

3. **Phase 3: Handlers**
   - Create `agent/src/metrics/nodes.rs`
   - Create `agent/src/metrics/pods.rs`
   - Create `agent/src/metrics/summary.rs`

4. **Phase 4: Integration**
   - Update `agent/src/main.rs` to initialize K8s state and add routes
   - Add logging for new endpoints

5. **Phase 5: Testing**
   - Add unit tests for resource parsing
   - Verify existing endpoints still work
   - Test against live cluster (if available)

---

## Notes

- **In-cluster config**: The `kube::Config::infer()` automatically detects if running inside a pod and uses the in-cluster config, otherwise falls back to `~/.kube/config`
- **metrics-server path**: Uses `/apis/metrics.k8s.io/v1beta1/` which is the standard metrics-server API
- **Node capacity**: Required to calculate percentages as per API contract
- **Pod node assignment**: Uses label lookup since PodMetrics doesn't directly expose node name
- **Container aggregation**: PodMetrics shows per-container usage which must be summed for total pod usage
