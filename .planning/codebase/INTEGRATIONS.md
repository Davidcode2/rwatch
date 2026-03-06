# External Integrations

**Analysis Date:** 2026-03-06

## APIs & External Services

**Agent Communication:**
- REST API over HTTP
  - Agent: `rwatch-agent` crate
  - Endpoints:
    - `GET /health` - Returns `HealthResponse` with status, uptime, version (`agent/src/health.rs`)
    - `GET /memory` - Returns `Memory` metrics from `/proc/meminfo` (`agent/src/memory.rs`)
  - Default port: 3000 (hardcoded in `agent/src/main.rs`)
  - No authentication
  - No TLS

**Client to Agent Communication:**
- Reqwest 0.12 HTTP client (`client/src/lib.rs`)
- Configurable timeout (default 5 seconds)
- Concurrent querying via `futures::future::join_all`

## Data Storage

**Databases:**
- None - No persistent storage

**File Storage:**
- Linux `/proc/meminfo` - Memory metrics source (`common/src/memory.rs`)
  - Reads `MemTotal` and `MemAvailable` fields
  - Linux-only, not portable

**Caching:**
- None

## Authentication & Identity

**Auth Provider:**
- None - No authentication on agent endpoints
  - Risk: Anyone with network access can query metrics

## Monitoring & Observability

**Error Tracking:**
- None - No external error tracking service

**Logs:**
- Standard stdout/println for agent startup
- No structured logging framework
- No log aggregation

## CI/CD & Deployment

**Hosting:**
- Kubernetes (K8s)
  - Manifests in `deploy/k8s/`
  - Uses Kustomize for configuration management

**CI Pipeline:**
- None detected in repository

**Deployment Topology:**
- DaemonSet - One agent pod per node (`deploy/k8s/02-daemonset.yaml`)
- Headless Service - For agent discovery (`deploy/k8s/03-service.yaml`)
- Deployment - For TUI (`deploy/k8s/06-tui-deployment.yaml`)
- ConfigMap - TUI configuration (`deploy/k8s/04-configmap.yaml`)
- RBAC - Permissions for TUI to access node metrics (`deploy/k8s/05-rbac.yaml`)

## Environment Configuration

**Required env vars:**
- `RWATCH_AGENT_*` - Environment-based agent discovery (`client/src/discovery.rs`)
  - Example: `RWATCH_AGENT_0=http://localhost:3000`
- `RUST_LOG` - Logging level (K8s deployment sets to "info")
- `PORT` - Not implemented (hardcoded to 3000)

**Secrets location:**
- None - No secrets management

## Webhooks & Callbacks

**Incoming:**
- None - Agent does not accept write operations

**Outgoing:**
- None - No webhook or callback integrations

## Agent Discovery Mechanisms

**StaticDiscovery:**
- Predefined list of agent URLs (`client/src/discovery.rs`)
- Used for testing and simple deployments

**EnvDiscovery:**
- Environment variable-based discovery
- Looks for `RWATCH_AGENT_0`, `RWATCH_AGENT_1`, etc.
- Default prefix: `RWATCH_AGENT`

**KubernetesDiscovery:**
- Placeholder implementation (`client/src/discovery.rs`)
- Would query Kubernetes API for pod discovery
- Currently returns empty list
- Service DNS format: `{service}.{namespace}` (e.g., `rwatch-agent.rwatch`)

## Kubernetes Integration

**Current K8s Features Used:**
- DaemonSet - Runs agent on every node
- hostNetwork - Access node network namespace
- hostPID - Access node process namespace
- Liveness/Readiness probes - Health checking on `/health` endpoint
- Resource limits - CPU/memory constraints
- SecurityContext - Non-privileged container

**Missing K8s Integrations:**
- Custom Metrics API - For HPA (Horizontal Pod Autoscaler)
- ServiceMonitor - For Prometheus scraping
- PodDisruptionBudget - For graceful disruptions

---

*Integration audit: 2026-03-06*
