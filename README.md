# Rwatch

**Rwatch** is a lightweight, real-time monitoring tool built in Rust. 

This project is designed as a deep-dive into the Rust ecosystem. The goal is to
build a functional tool while exploring intermediate-to-advanced concepts like
async networking, concurrency, workspace management, and terminal user
interfaces.

## Project Architecture

Rwatch consists of **four** main components organized in a Cargo Workspace:

1.  **`agent`**: A background daemon that runs on Linux servers. It collects
    system metrics and exposes them via a REST API.
2.  **`client`**: A library crate that handles all business logic for querying agents.
    Provides a clean API for UI consumers to discover, query, and aggregate data
    from multiple agents.
3.  **`tui`**: A Terminal User Interface client. Uses the client library to
    connect to multiple agents and visualize the data.
4.  **`common`**: A shared library crate containing data structures, protocol
    definitions, and utility functions shared between all components to
    ensure type safety (DRY).

### Key Features

* **Real-time monitoring**: Memory metrics from Linux hosts.
* **Multi-Agent Support**: Query and aggregate data from multiple agents concurrently.
* **Agent Discovery**: Multiple discovery methods (static, environment variables, Kubernetes - placeholder).
* **Stateless Client**: The TUI can be restarted without losing data (agents maintain current state).
* **HTTP Protocol**: Simple, firewall-friendly JSON communication.
* **Modular Architecture**: Client library enables multiple UI implementations (TUI, web, etc.).

### What's Working Now ✅

- **Agent**: HTTP server with `/health` and `/memory` endpoints
- **Client Library**: Concurrent multi-agent querying with aggregation
- **TUI**: Displays aggregated cluster metrics and per-agent status
- **Memory Metrics**: Reading from `/proc/meminfo` (Linux only)
- **Agent Discovery**: Static and environment variable based

### Planned Features 🚧

- **CPU Metrics**: CPU usage monitoring
- **Network I/O**: Network traffic monitoring  
- **Ring Buffer**: Historical metric storage in agents
- **Configuration Files**: YAML/JSON config file support
- **Interactive TUI**: Real-time updating interface with ratatui
- **Web Interface**: Browser-based UI using the client library
- **Kubernetes Discovery**: Full K8s API integration

## Tech Stack & Learning Goals

This project explores the following Rust concepts and libraries:

| Concept | Library / Tool | Purpose | Component |
| --- | --- | --- | --- |
| **Workspace** | Cargo | Managing a monorepo with shared dependencies. | All |
| **Async Runtime** | `tokio` | Handling non-blocking I/O and task scheduling. | Agent, Client, TUI |
| **Web Server** | `axum` | Serving metrics from the Agent. | Agent |
| **HTTP Client** | `reqwest` | Fetching metrics from agents. | Client |
| **Serialization** | `serde` | JSON parsing/generating. | Common |
| **Concurrent Futures** | `futures` | Querying multiple agents concurrently. | Client |
| **System Stats** | `/proc/meminfo` | Linux memory metric collection. | Agent |
| **TUI** | stdout (ratatui planned) | Rendering the interface. | TUI |
| **Shared State** | `Arc<RwLock<T>>` | Managing safe access to data across threads. | Future |
| **Error Handling** | `anyhow` | Simple error propagation. | All |


## Implementation Details

### 1. The Agent (`rwatch-agent`)

The agent runs an `axum` HTTP server that serves system metrics:

**Endpoints:**
- `GET /health` - Returns status, uptime, and version
- `GET /memory` - Returns memory metrics from `/proc/meminfo`

**Current Implementation:**
- Reads `MemTotal` and `MemAvailable` from `/proc/meminfo`
- No historical data storage yet (returns current snapshot only)
- Hardcoded to port 3000
- Single-threaded async with tokio

**Future:** Ring buffer for storing last N seconds of metrics.

### 2. The Client Library (`rwatch-client`)

The client crate provides a reusable API for querying agents:

**Key Components:**
- `Client` - HTTP client for querying agent endpoints
- `AgentData` - Combined health and memory data from an agent
- `AgentResult` - Success/failure result type with error handling
- `AggregatedMetrics` - Cluster-wide aggregated statistics
- Discovery implementations for finding agents

**Features:**
- Concurrent querying of multiple agents using `futures::join_all`
- Automatic data aggregation across agents
- Pluggable discovery mechanisms (static, env vars, Kubernetes)
- Clean API for UI consumers

**Example Usage:**
```rust
let client = Client::new();
let agents = vec!["http://agent1:3000", "http://agent2:3000"];
let results = client.query_agents(&agents).await;
let metrics = aggregate_results(&results);
```

### 3. The TUI (`rwatch-tui`)

The TUI is an async application that uses the client library:

1. Discovers agents via the client library's discovery mechanisms
2. Queries all agents concurrently
3. Displays aggregated cluster metrics and per-agent details

**Current Display:**
- Cluster summary (total nodes, healthy/failed counts)
- Total cluster memory usage
- Per-agent health status and memory details
- Error messages for failed agents

**Future:** Interactive ratatui-based interface with real-time updates.

### 4. Communication & Data (`rwatch-common`)

Shared types between all components via HTTP JSON:

**Health Response:**
```json
{
  "status": "up",
  "uptime": 123,
  "version": "0.1.0"
}
```

**Memory Response:**
```json
{
  "total": 16000000,
  "used": 0,
  "free": 0,
  "available": 8000000
}
```

## Repository Structure

```text
rwatch/
├── Cargo.toml              # Workspace root
├── common/                 # Shared library (Structs, Enums, Error types)
│   ├── src/
│   │   ├── lib.rs         # Module exports
│   │   ├── health.rs      # HealthResponse struct
│   │   ├── memory.rs      # Memory struct, /proc/meminfo parsing
│   │   └── memory_display.rs # Display formatting
│   └── Cargo.toml
├── client/                 # Client library (Multi-agent querying)
│   ├── src/
│   │   ├── lib.rs         # Client API, AgentResult, AggregatedMetrics
│   │   ├── agent.rs       # AgentConfig, AgentList
│   │   └── discovery.rs   # Discovery implementations
│   └── Cargo.toml
├── agent/                  # The daemon
│   ├── src/
│   │   ├── main.rs        # HTTP server setup
│   │   ├── health.rs      # Health endpoint handler
│   │   └── memory.rs      # Memory endpoint handler
│   └── Cargo.toml
├── tui/                    # The terminal UI
│   ├── src/
│   │   ├── main.rs        # Application logic
│   │   └── ui.rs          # Display functions
│   └── Cargo.toml
└── deploy/                 # Kubernetes deployment files
    └── k8s/               # K8s manifests
```

---

## Iterations

### Iteration 1: The "Hello World" of Monitoring ✅ COMPLETE

**Goal**: Establish the workspace, networking, and basic types.

**Definition of Done - COMPLETED:**

1. ✅ **Workspace Created**: Project compiles with `agent`, `tui`, and `common`.
2. ✅ **Common Library**: `HealthResponse` and `Memory` structs defined and shared.
3. ✅ **Agent**: HTTP server with `/health` and `/memory` endpoints.
4. ✅ **TUI**: Connects to agents and displays results.

---

### Iteration 2: Multi-Agent Support & Client Library ✅ COMPLETE

**Goal**: Add support for querying multiple agents concurrently and create a reusable client library.

**Definition of Done - COMPLETED:**

1. ✅ **Client Library**: New `client` crate that handles all agent communication
   - `Client` struct for querying agents
   - `AgentResult` enum for handling success/failure
   - `AgentData` struct combining health and memory
   - `AggregatedMetrics` for cluster-wide statistics
   - Concurrent querying using `futures::join_all`

2. ✅ **Agent Discovery**: Pluggable discovery mechanisms
   - `StaticDiscovery` - Predefined list of URLs
   - `EnvDiscovery` - Environment variables (RWATCH_AGENT_*)
   - `KubernetesDiscovery` - Placeholder for K8s integration

3. ✅ **TUI Refactoring**: TUI now uses the client library
   - Discovers agents via multiple methods
   - Queries all agents concurrently
   - Displays aggregated cluster metrics
   - Shows per-agent status with error handling

4. ✅ **Data Aggregation**: Compute cluster-wide metrics
   - Total memory across all nodes
   - Memory usage percentage
   - Healthy vs failed node counts

---

## Quick Start

### Running Locally

**1. Start the Agent:**

```bash
# Terminal 1 - Start the agent
cargo run -p rwatch-agent

# Agent will start on http://localhost:3000
```

**2. Run the TUI:**

```bash
# Terminal 2 - Run TUI to query the agent
cargo run -p rwatch-tui
```

### Running with Multiple Agents

```bash
# Terminal 1 - Agent on port 3000
cargo run -p rwatch-agent

# Terminal 2 - Start another agent (requires port config - see limitations)
# Currently requires modifying the agent source to change port

# Terminal 3 - TUI with multiple agents via env vars
RWATCH_AGENT_0=http://localhost:3000 \
RWATCH_AGENT_1=http://localhost:3001 \
cargo run -p rwatch-tui
```

### Running Tests

```bash
# Run all tests across the workspace
cargo test

# Run tests for a specific crate
cargo test -p rwatch-client
cargo test -p rwatch-agent
cargo test -p rwatch-common
cargo test -p rwatch-tui
```

---

## Kubernetes Deployment

Rwatch is designed to be deployed in Kubernetes clusters using a DaemonSet pattern for the agents. This ensures every node in the cluster has a monitoring agent running, while the TUI can aggregate metrics from all agents.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Kubernetes Cluster                        │
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │    Node 1    │    │    Node 2    │    │    Node N    │       │
│  │  ┌────────┐  │    │  ┌────────┐  │    │  ┌────────┐  │       │
│  │  │ Agent  │  │    │  │ Agent  │  │    │  │ Agent  │  │       │
│  │  │ Pod    │  │    │  │ Pod    │  │    │  │ Pod    │  │       │
│  │  │:3000   │  │    │  │:3000   │  │    │  │:3000   │  │       │
│  │  └────┬───┘  │    │  └────┬───┘  │    │  └────┬───┘  │       │
│  │       │      │    │       │      │    │       │      │       │
│  │  hostPID:true  │    │  hostPID:true  │    │  hostPID:true  │       │
│  │  hostNetwork   │    │  hostNetwork   │    │  hostNetwork   │       │
│  └───────┼──────┘    └───────┼──────┘    └───────┼──────┘       │
│          │                   │                   │              │
│          └───────────────────┼───────────────────┘              │
│                              │                                   │
│                    ┌─────────┴─────────┐                        │
│                    │   TUI Pod         │                        │
│                    │   (Deployment)    │                        │
│                    │                   │                        │
│                    │ Queries all agents│                        │
│                    │ via pod IPs/DNS   │                        │
│                    └───────────────────┘                        │
└─────────────────────────────────────────────────────────────────┘
```

### Networking Topology

**Agent (DaemonSet):**
- Runs with `hostPID: true` to access the host's `/proc` filesystem for memory metrics
- Uses `hostNetwork: true` to bind directly to the node's network interface
- Exposes port 3000 on each node
- One agent pod per node, scheduled automatically by Kubernetes

**TUI (Deployment):**
- Single replica Deployment for the TUI
- Discovers agents using the Kubernetes API or headless service DNS
- Connects to each agent individually to collect metrics
- Displays aggregated view of all nodes in the cluster

**Service Discovery:**
- Headless service (`clusterIP: None`) allows direct access to agent pods
- TUI queries Kubernetes API to get list of all agent pods
- Alternatively, uses DNS SRV records to discover agent endpoints

### Deployment Files

All Kubernetes manifests are located in `deploy/k8s/`:

```
deploy/k8s/
├── 01-namespace.yaml      # rwatch namespace
├── 02-daemonset.yaml      # Agent DaemonSet
├── 03-service.yaml        # Headless service for agent discovery
├── 04-configmap.yaml      # TUI configuration
├── 05-rbac.yaml          # RBAC for TUI to query Kubernetes API
└── 06-tui-deployment.yaml # TUI deployment
```

### Installation Steps

#### 1. Build the Docker Image

First, build the agent Docker image:

```bash
# Build the agent image
docker build -f Dockerfile.agent -t rwatch-agent:latest .

# For a real deployment, tag and push to your registry
docker tag rwatch-agent:latest your-registry/rwatch-agent:v0.1.0
docker push your-registry/rwatch-agent:v0.1.0
```

#### 2. Deploy to Kubernetes

Apply the manifests in order:

```bash
# Create namespace
kubectl apply -f deploy/k8s/01-namespace.yaml

# Deploy agents (DaemonSet)
kubectl apply -f deploy/k8s/02-daemonset.yaml

# Create service for agent discovery
kubectl apply -f deploy/k8s/03-service.yaml

# Create ConfigMap for TUI configuration
kubectl apply -f deploy/k8s/04-configmap.yaml

# Create RBAC for TUI
kubectl apply -f deploy/k8s/05-rbac.yaml

# Deploy TUI
kubectl apply -f deploy/k8s/06-tui-deployment.yaml
```

#### 3. Verify Deployment

Check that agents are running on all nodes:

```bash
# List all agent pods
kubectl get pods -n rwatch -l app=rwatch-agent -o wide

# Check agent health on a specific node
kubectl exec -n rwatch <agent-pod-name> -- wget -qO- http://localhost:3000/health

# View TUI logs
kubectl logs -n rwatch -l app=rwatch-tui
```

#### 4. Access the TUI

For interactive access to the TUI:

```bash
# Exec into the TUI pod
kubectl exec -it -n rwatch deployment/rwatch-tui -- /bin/sh

# Or port-forward to access TUI from your local machine (if TUI exposes HTTP)
kubectl port-forward -n rwatch deployment/rwatch-tui 8080:8080
```

### Security Considerations

**Agent Security:**
- Runs with minimal privileges (`privileged: false`)
- Requires `hostPID: true` to read `/proc/meminfo` from the host
- Read-only root filesystem for additional security
- Drops all Linux capabilities

**TUI Security:**
- Uses dedicated ServiceAccount with limited RBAC permissions
- Can only read pods and endpoints in the cluster
- No write permissions to cluster resources

**Network Security:**
- Agents bind to localhost by default (via hostNetwork)
- Internal cluster communication only
- No external exposure

### Scaling Considerations

**Horizontal Scaling:**
- Agents: Automatically scale with cluster nodes via DaemonSet
- TUI: Currently single replica; for high availability, could run multiple TUI instances

**Resource Limits:**
- Agent: 128Mi memory limit, 200m CPU limit per node
- TUI: 128Mi memory limit, 200m CPU limit

### Troubleshooting

**Agent cannot read memory:**
- Verify `hostPID: true` is set in the DaemonSet
- Check node has Linux with `/proc/meminfo` available

**TUI cannot connect to agents:**
- Verify RBAC permissions are correctly applied
- Check that headless service exists and agent pods are ready
- View TUI logs: `kubectl logs -n rwatch -l app=rwatch-tui`

**Agents not scheduling:**
- Check for taints/tolerations on nodes
- Verify node selector matches cluster nodes

