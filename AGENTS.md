# Rwatch Agent Documentation

This document describes the current implementation state of the rwatch monitoring tool for AI agents and developers.

## Current Implementation Status

**Version**: 0.1.0  
**Last Updated**: 2026-03-02

## Architecture Overview

The project now consists of **four crates** in a Cargo workspace:

```
rwatch/
├── Cargo.toml              # Workspace root
├── agent/                  # The monitoring daemon (rwatch-agent)
├── client/                 # NEW: Business logic for querying agents (rwatch-client)
├── common/                 # Shared types and utilities (rwatch-common)
└── tui/                    # Terminal UI (rwatch-tui)
```

## What Currently Works

### ✅ Agent (`rwatch-agent`)

The agent is a lightweight HTTP server that exposes two endpoints:

**Endpoints:**
- `GET /health` - Returns agent status, uptime, and version
- `GET /memory` - Returns memory metrics from `/proc/meminfo`

**Implementation Details:**
- Built with `axum` web framework
- Binds to `0.0.0.0:3000` (hardcoded)
- Reads memory from Linux `/proc/meminfo` (Linux only)
- Returns JSON responses
- Single-threaded async with tokio

**Memory Reading:**
- Only reads `MemTotal` and `MemAvailable` from `/proc/meminfo`
- Values are in KB
- Other fields (used, free) are hardcoded to 0

### ✅ Client Library (`rwatch-client`) - NEW

A new crate that centralizes all agent communication logic:

**Features:**
- `Client` struct for querying agents
- Concurrent querying of multiple agents via `query_agents()`
- Agent discovery mechanisms (static, environment-based, Kubernetes placeholder)
- Data aggregation across multiple agents
- Structured response types (`AgentResult`, `AgentData`)

**Discovery Methods:**
1. `StaticDiscovery` - Predefined list of URLs
2. `EnvDiscovery` - Environment variables (`RWATCH_AGENT_*`)
3. `KubernetesDiscovery` - Placeholder for K8s API integration

**Key Types:**
```rust
pub struct Client { ... }
pub struct AgentData { url, health, memory }
pub enum AgentResult { Success(AgentData), Failure { url, error } }
pub struct AggregatedMetrics { ... }
```

### ✅ Common Library (`rwatch-common`)

Shared types between all components:

**Types:**
- `HealthResponse` - Status, uptime, version
- `Memory` - total, used, free, available (in KB)
- `MemoryWithUnit` - Display formatting utilities

### ✅ TUI (`rwatch-tui`)

The terminal interface now uses the client library:

**Features:**
- Discovers agents via multiple methods
- Queries all agents concurrently
- Displays aggregated cluster metrics
- Shows per-agent status (success/failure)
- Pretty-printed output to stdout

**Display Output:**
- Cluster summary (total/healthy/failed nodes)
- Total cluster memory usage
- Per-agent health and memory details

## What Does NOT Work (Future Plans)

### ❌ Ring Buffer / History
Agents do not maintain historical data. Currently only return current snapshot.

### ❌ CPU Metrics
CPU monitoring is planned but not implemented.

### ❌ Network I/O Metrics
Network monitoring is planned but not implemented.

### ❌ Configuration Files
No config file support yet. URLs are hardcoded or from env vars.

### ❌ Interactive TUI
Current TUI is batch mode only (queries once and exits). No ratatui implementation yet.

### ❌ Kubernetes Discovery (Full)
KubernetesDiscovery is a placeholder. It needs actual K8s API integration.

### ❌ Web Interface
No web UI exists yet.

## API Endpoints

### GET /health

```json
{
  "status": "up",
  "uptime": 123,
  "version": "0.1.0"
}
```

### GET /memory

```json
{
  "total": 16000000,
  "used": 0,
  "free": 0,
  "available": 8000000
}
```

*Note: Values are in KB from /proc/meminfo*

## Running the Project

### Local Development

```bash
# Start the agent
cargo run -p rwatch-agent

# In another terminal, run the TUI
cargo run -p rwatch-tui
```

### With Multiple Agents

```bash
# Terminal 1 - Agent 1
PORT=3000 cargo run -p rwatch-agent

# Terminal 2 - Agent 2  
# (modify agent to accept port arg - currently hardcoded)

# Terminal 3 - TUI with env vars
RWATCH_AGENT_0=http://localhost:3000 \
RWATCH_AGENT_1=http://localhost:3001 \
cargo run -p rwatch-tui
```

## Code Organization

### Agent (`agent/src/`)
- `main.rs` - HTTP server setup, axum router
- `health.rs` - Health endpoint handler
- `memory.rs` - Memory endpoint handler

### Client (`client/src/`)
- `lib.rs` - Main Client API, AgentData, AgentResult, AggregatedMetrics
- `agent.rs` - AgentConfig, AgentList
- `discovery.rs` - Discovery implementations

### Common (`common/src/`)
- `lib.rs` - Module exports
- `health.rs` - HealthResponse struct
- `memory.rs` - Memory struct, /proc/meminfo parsing
- `memory_display.rs` - Display formatting

### TUI (`tui/src/`)
- `main.rs` - Application logic, agent discovery
- `ui.rs` - Display functions

## Testing

All crates have unit tests:

```bash
cargo test
```

Current test coverage:
- Health response serialization/deserialization
- Memory struct creation
- Agent discovery mechanisms
- Client result handling
- Aggregation logic
- UI display functions

## Kubernetes Deployment

Kubernetes manifests exist in `deploy/k8s/`:
- DaemonSet for agents
- Headless service for discovery
- ConfigMap for TUI config
- RBAC for TUI permissions
- Deployment for TUI

See README.md for deployment details.

## Dependencies

### Key Dependencies per Crate

**Agent:**
- `axum` - Web framework
- `tokio` - Async runtime
- `tower` - Middleware (future use)

**Client:**
- `reqwest` - HTTP client
- `tokio` - Async runtime
- `futures` - Concurrent futures

**Common:**
- `serde` - Serialization
- No external HTTP dependencies

**TUI:**
- `rwatch-client` - Business logic
- `rwatch-common` - Shared types

## Known Limitations

1. **Platform Specific**: Agent only works on Linux (requires /proc/meminfo)
2. **Hardcoded Port**: Agent binds to port 3000 (not configurable)
3. **Single Node**: Each agent only monitors its own node
4. **No Authentication**: No API keys or authentication on endpoints
5. **No TLS**: HTTP only, no HTTPS support
6. **Limited Metrics**: Only memory (total/available), no CPU/network
7. **No Persistence**: No database, no historical data retention
8. **No Alerting**: No notification system for failures

## For AI Agents

When working with this codebase:

1. **Adding New Metrics**: Extend the Memory struct or add new endpoints in agent/, add corresponding types in common/, and update client/ to query them.

2. **Adding UI Consumers**: The client crate is designed to support multiple UI types. Use the Client API:
   ```rust
   let client = Client::new();
   let discovery = StaticDiscovery::from_urls(&["http://agent:3000"]);
   let agents = discovery.discover().await?;
   let results = client.query_agents(&agents.urls()).await;
   ```

3. **Configuration**: Currently uses environment variables. Config file support would be added to client/src/discovery.rs or a new config module.

4. **Testing**: All business logic is tested. When adding features, add corresponding unit tests in the `#[cfg(test)]` modules.

## Next Steps

Based on current state, likely next features:
1. Implement actual Kubernetes API discovery
2. Add configuration file support
3. Implement ring buffer for metric history
4. Add CPU metrics endpoint
5. Make agent port configurable
6. Add proper ratatui-based interactive TUI
7. Implement web interface using the client crate
