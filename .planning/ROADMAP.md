# Rwatch Project Roadmap

## Project Overview

**Rwatch** is a lightweight, real-time monitoring tool for Linux systems and Kubernetes clusters. It consists of an agent that runs on each node collecting metrics, and a client library that queries agents to display cluster-wide information.

**Tech Stack:** Rust, Tokio, Axum, kube-rs, k8s-openapi
**Version:** 0.1.5
**Repository:** https://github.com/Davidcode2/rwatch

---

## Architecture

### Workspace Structure
```
rwatch/
├── agent/      # Daemon exposing system/K8s metrics via HTTP (port 3000)
├── client/     # Library for querying agents
├── common/     # Shared types between agent and client
└── tui/        # Terminal UI for visualizing cluster metrics
```

### Current Capabilities

| Feature | Status | Endpoint |
|---------|--------|----------|
| Health check | ✅ Complete | `GET /health` |
| Memory metrics | ✅ Complete | `GET /memory` |
| K8s node metrics | ✅ Complete | `GET /api/metrics/nodes` |
| K8s pod metrics | ✅ Complete | `GET /api/metrics/pods` |
| K8s summary | ✅ Complete | `GET /api/metrics/summary` |
| Agent discovery | ✅ Complete | Static, Env, Kubernetes |
| Concurrent querying | ✅ Complete | Via `Client::query_agents()` |
| TUI display | ✅ Complete | ASCII tables, aggregated metrics |

---

## Phases

### Phase 1: Foundation ✅ COMPLETE
**Goal:** Basic agent with health and memory endpoints

**Deliverables:**
- [x] HTTP server with Axum
- [x] `/health` endpoint (status, uptime, version)
- [x] `/memory` endpoint (reads from `/proc/meminfo`)
- [x] Client library for querying agents
- [x] TUI for displaying metrics
- [x] Kubernetes DaemonSet deployment

**Completion Date:** Prior to v0.1.0

---

### Phase 2: Kubernetes Integration ✅ COMPLETE
**Goal:** Full Kubernetes metrics-server integration

**Deliverables:**
- [x] K8s client initialization (`K8sState`)
- [x] Node metrics endpoint (`/api/metrics/nodes`)
- [x] Pod metrics endpoint (`/api/metrics/pods`)
- [x] Summary endpoint (`/api/metrics/summary`)
- [x] Error handling for missing metrics-server
- [x] CPU/Memory percentage calculations
- [x] Proper K8s resource formatting (millicores, Mi/Gi)

**Key Components:**
- `agent/src/metrics/mod.rs` - K8s state management
- `agent/src/metrics/nodes.rs` - Node metrics handler
- `agent/src/metrics/pods.rs` - Pod metrics handler
- `agent/src/metrics/summary.rs` - Aggregated summary handler
- `agent/src/metrics/resource.rs` - Resource parsing utilities
- `agent/src/metrics/error.rs` - Metrics error types
- `common/src/metrics.rs` - Shared metrics types

**Completion Date:** v0.1.5 (current)

---

### Phase 3: Enhanced Metrics ⏳ PLANNED
**Goal:** Additional system metrics and historical data

**Deliverables:**
- [ ] CPU monitoring (per-node usage)
- [ ] Network I/O metrics
- [ ] Disk usage metrics
- [ ] Historical data persistence (ring buffer)
- [ ] Time-series data storage

**Notes:** Currently agents return only current snapshot. Need to implement:
- In-memory ring buffer for recent history
- Optional persistence to disk
- API for querying historical data

---

### Phase 4: Security & Configuration ⏳ PLANNED
**Goal:** Production-ready security and configuration

**Deliverables:**
- [ ] Configurable agent port (currently hardcoded 3000)
- [ ] Configuration file support (YAML/TOML)
- [ ] Authentication on HTTP endpoints
- [ ] TLS/HTTPS support
- [ ] Service mesh integration (Istio/Linkerd compatibility)

---

### Phase 5: Advanced Features ⏳ FUTURE
**Goal:** Enterprise features and ecosystem integration

**Deliverables:**
- [ ] Prometheus metrics export
- [ ] Grafana dashboard templates
- [ ] Alerting integration (webhooks)
- [ ] Multi-cluster support
- [ ] RBAC for metrics access
- [ ] Performance optimization (caching, compression)

---

## Current Status

**Phase:** 2 of 5 (Kubernetes Integration) ✅ COMPLETE  
**Next Phase:** Phase 3 - Enhanced Metrics  
**Blockers:** None  

## Decisions

1. **Kubernetes-first approach** - Prioritized K8s metrics-server integration over additional system metrics
2. **Rust for agent** - Chose Rust for performance and safety in resource-constrained environments
3. **HTTP/JSON API** - Simple REST API for easy integration with frontend (rwatch-web)
4. **Workspace structure** - Separated concerns into crates for testability and reusability

## Known Limitations

- Linux only (requires `/proc/meminfo`)
- Port 3000 hardcoded
- No authentication on endpoints
- No persistence (agents return current snapshot only)
- Requires Kubernetes config for K8s metrics (fails gracefully without)
