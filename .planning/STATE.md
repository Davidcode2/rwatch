# Rwatch Project State

**Last Updated:** 2026-03-06  
**Current Phase:** Phase 2 - Kubernetes Integration ✅ COMPLETE  
**Next Phase:** Phase 3 - Enhanced Metrics

---

## Current Status

### ✅ What's Working

1. **Agent (rwatch-agent)**
   - HTTP server on port 3000
   - Health endpoint with uptime/version
   - Memory endpoint reading from `/proc/meminfo`
   - Full Kubernetes metrics-server integration
   - Node, pod, and summary metrics endpoints
   - Proper error handling for missing metrics-server

2. **Client Library (rwatch-client)**
   - HTTP client with configurable timeout
   - Concurrent agent querying (`join_all`)
   - Agent discovery (Static, Env, Kubernetes)
   - Metrics aggregation across cluster
   - Error handling per-agent

3. **Common Types (rwatch-common)**
   - HealthResponse with serialization
   - Memory struct with `/proc/meminfo` parsing
   - K8s metrics types (NodeMetrics, PodMetrics, SummaryResponse)
   - Memory display formatting (as_gb)

4. **TUI (rwatch-tui)**
   - ASCII table display for health/memory
   - Aggregated cluster summary
   - Discovery from env vars or static config

5. **Deployment**
   - Kubernetes DaemonSet manifests
   - GitHub Actions for CI/CD
   - ArgoCD integration
   - RBAC for K8s API access

---

## Recent Changes (v0.1.5)

### Added
- Kubernetes metrics-server integration
  - `/api/metrics/nodes` - Node CPU/memory usage
  - `/api/metrics/pods` - Pod CPU/memory usage
  - `/api/metrics/summary` - Cluster-wide summary
- New `metrics` module in `agent/src/metrics/`
- `K8sState` for shared Kubernetes client
- Resource parsing utilities (cpu, memory)
- Error types for metrics operations

### Modified
- `agent/src/main.rs` - Added K8s routes and state
- `Cargo.lock` - Added kube, k8s-openapi, chrono dependencies
- `common/src/lib.rs` - Added metrics module

---

## Blockers & Concerns

### 🔴 Blockers
None currently.

### 🟡 Concerns
1. **Metrics-server dependency** - K8s endpoints fail gracefully but require metrics-server
2. **No authentication** - HTTP endpoints are open
3. **Hardcoded port** - Agent always binds to 3000
4. **Memory-only** - No CPU/disk/network metrics from host
5. **No persistence** - Only current snapshot, no history

### 🟢 Under Control
- Tests are in place for core functionality
- Error handling is comprehensive
- Deployment pipeline is working
- Code documentation is good

---

## Next Steps

### Immediate (Next Session)
1. Consider implementing CPU monitoring from `/proc/stat`
2. Add configuration file support
3. Make port configurable via env var

### Short Term (This Week)
1. Add network I/O metrics
2. Implement ring buffer for historical data
3. Add authentication middleware

### Medium Term (This Month)
1. Prometheus metrics export
2. Performance optimization
3. Multi-cluster support design

---

## Technical Debt

1. **KubernetesDiscovery** - Currently returns empty list (placeholder)
2. **Error handling** - Some unwrap() calls should be proper error handling
3. **Testing** - Need more integration tests for K8s endpoints
4. **Documentation** - API documentation could be more comprehensive

---

## Metrics

**Lines of Code:**
- agent: ~600 lines
- client: ~400 lines
- common: ~300 lines
- tui: ~250 lines
- Total: ~1550 lines

**Test Coverage:**
- Unit tests in most modules
- Integration tests for HTTP handlers
- Tests skip if no K8s config available

**Dependencies:**
- tokio (async runtime)
- axum (HTTP framework)
- kube, k8s-openapi (Kubernetes)
- serde, serde_json (serialization)
- anyhow (error handling)
- chrono (timestamps)

---

## API Usage

### From rwatch-web (React Frontend)
```
GET /api/metrics/nodes    → NodeMetricsResponse
GET /api/metrics/pods     → PodMetricsResponse
GET /api/metrics/summary  → SummaryResponse
```

### From TUI
```
GET /health  → HealthResponse
GET /memory  → Memory
```

---

## Deployment Status

| Environment | Status | Notes |
|-------------|--------|-------|
| Local dev | ✅ Working | `cargo run -p rwatch-agent` |
| Kubernetes | ✅ Deployed | DaemonSet running |
| CI/CD | ✅ Working | GitHub Actions → ArgoCD |
