# Rwatch - Agent Documentation

> **For AI Agents:** This document provides essential context for working with the rwatch project.

## Project Overview

Rwatch is a lightweight, real-time monitoring tool for Linux systems built in Rust. It consists of:
- **Agent**: Daemon that runs on each node collecting system metrics
- **Client Library**: For querying agents and aggregating cluster-wide data
- **Common Types**: Shared data structures between agent and client
- **TUI**: Terminal user interface for visualizing cluster metrics

### Architecture

```
rwatch/
├── agent/      # Daemon that collects system metrics from /proc/meminfo
├── client/     # Library for querying agents (handles HTTP + discovery)
├── common/     # Shared types: HealthResponse, Memory, memory_display
└── tui/        # Terminal UI using ratatui (future) + client library
```

## Building and Running Locally

### Prerequisites
- Rust toolchain (edition 2024)
- Linux system (requires `/proc/meminfo`)
- Optional: Kubernetes cluster for deployment testing

### Build Commands

```bash
# Build the entire workspace
cargo build --release

# Run the agent locally
cargo run -p rwatch-agent
# Agent starts on http://0.0.0.0:3000

# Run the TUI (in another terminal)
cargo run -p rwatch-tui
# Connects to agents at http://localhost:3000 by default

# Build specific crate
cargo build -p rwatch-agent
cargo build -p rwatch-client
cargo build -p rwatch-common
cargo build -p rwatch-tui
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p rwatch-client
cargo test -p rwatch-common

# Run with output visible
cargo test -- --nocapture
```

## Key Components

### Agent (`agent/`)
- **Entry**: `src/main.rs`
- **Handlers**: `src/health.rs`, `src/memory.rs`
- HTTP server using Axum framework
- Binds to port 3000 (hardcoded - known limitation)
- Reads from `/proc/meminfo` (Linux only)

### Client Library (`client/`)
- **Entry**: `src/lib.rs`
- **Discovery**: `src/discovery.rs`, `src/agent.rs`
- `Client` struct for querying agents
- Supports concurrent queries with `tokio::try_join!`
- Discovery via environment variables (`RWATCH_AGENT_*`) or static config

### Common Types (`common/`)
- **HealthResponse**: `{ status, uptime, version }`
- **Memory**: `{ total, used, free, available }` (values in KB from /proc/meminfo)
- **memory_display**: Formatting utilities for TUI

### TUI (`tui/`)
- **Entry**: `src/main.rs`
- **UI**: `src/ui.rs`
- Currently text-based output (ratatui migration planned)
- Discovers agents, queries concurrently, displays aggregated metrics

## API Endpoints

The agent exposes the following HTTP endpoints:

### GET /health
Returns agent health status.

```json
{
  "status": "up",
  "uptime": 123,
  "version": "0.1.3"
}
```

### GET /memory
Returns memory metrics from `/proc/meminfo`.

```json
{
  "total": 16000000,
  "used": 0,
  "free": 0,
  "available": 8000000
}
```

**Note**: Values are in kilobytes (KB) as read from `/proc/meminfo`.

## Deployment Process

### Workflow for Agents
**IMPORTANT**: When making changes to this repository:
1. Verify changes work correctly locally (run `cargo test` and `cargo build --release`)
2. Commit changes with clear messages
3. **Push directly to origin/main** - This triggers the CI/CD pipeline
4. Do not create PRs for routine changes unless requested

The GitHub Actions workflow will automatically:
- Bump the version in all Cargo.toml files
- Build and push the Docker image
- Update the app-of-apps deployment repo

### GitHub Actions Workflow

**Trigger**: Push to `main` branch with changes to:
- `agent/`, `client/`, `common/`, `tui/`
- `Cargo.toml`, `Cargo.lock`
- `Dockerfile.agent`

**Workflow** (`.github/workflows/main.yml`):
1. **Check for changes** - Skip build if only non-source files changed
2. **Bump version** - Auto-increments patch version in all Cargo.toml files
3. **Build and push** - Builds Docker image via reusable workflow
4. **Deploy** - Updates app-of-apps repo with new image tag

**Authentication**: Uses GitHub App token (not PAT)
- Requires `APP_ID` and `APP_PRIVATE_KEY` secrets
- App needs access to both `rwatch` and `app-of-apps` repos

### Docker Image

- **Name**: `ghcr.io/davidcode2/rwatch-agent`
- **Tags**: `latest`, `sha-<commit>`, `<version>` (e.g., `0.1.3`)
- **File**: `Dockerfile.agent`

### Kubernetes Deployment

Deployed via ArgoCD from `app-of-apps` repository:
- **Namespace**: `rwatch`
- **DaemonSet**: Runs one agent pod per node
- **Service**: Headless service for agent discovery (`rwatch-agent.rwatch.svc.cluster.local:3000`)
- **Security**: `hostPID: true`, `hostNetwork: true` (required for /proc/meminfo access)

See `DEPLOYMENT.md` for detailed setup instructions.

## Testing Approach

### Unit Tests
- Each crate has inline tests in `#[cfg(test)]` modules
- Key areas:
  - Serialization/deserialization of common types
  - Client query logic and result aggregation
  - Memory parsing from /proc/meminfo

### Integration Testing
- Run agent locally: `cargo run -p rwatch-agent`
- Query manually: `curl http://localhost:3000/health`
- Run TUI to verify end-to-end flow

### Kubernetes Testing
- Port-forward to cluster: `kubectl port-forward -n rwatch service/rwatch-agent 3000:3000`
- Run TUI locally against forwarded port

## Known Issues and TODOs

### Current Limitations
1. **Platform**: Linux only (requires `/proc/meminfo`)
2. **Port**: Hardcoded to 3000, not configurable
3. **Metrics**: Only memory (total/available), no CPU/network yet
4. **History**: No persistence, agents return current snapshot only
5. **Security**: No authentication on HTTP endpoints (internal cluster only)
6. **TUI**: Currently text-based, not using ratatui yet

### Planned Improvements
- **Configuration**: Config file support instead of env vars
- **Metrics**: Add CPU and network I/O monitoring
- **History**: Implement ring buffer for metric history
- **Port**: Make agent port configurable
- **Discovery**: Complete Kubernetes API-based discovery
- **UI**: Full ratatui implementation with real-time updates
- **Web**: Web interface alternative to TUI

### Technical Debt
- Agent error handling could be more robust
- Client timeout is hardcoded to 5 seconds
- TUI discovery fallback is static (should read from config file)

## Dependencies

### Workspace-level (consistent across crates)
- `tokio = "1.41"` - Async runtime
- `serde = "1.0"` - Serialization
- `serde_json = "1.0"` - JSON handling
- `anyhow = "1.0"` - Error handling

### Agent-specific
- `axum = "0.7"` - Web framework
- `tower = "0.5"` - Middleware (future use)

### Client-specific
- `reqwest` - HTTP client (in Cargo.toml)

## Quick Reference

| Task | Command |
|------|---------|
| Run agent | `cargo run -p rwatch-agent` |
| Run TUI | `cargo run -p rwatch-tui` |
| Test all | `cargo test` |
| Build release | `cargo build --release` |
| Check health | `curl http://localhost:3000/health` |
| Check memory | `curl http://localhost:3000/memory` |

## Resources

- **Repository**: `Davidcode2/rwatch`
- **Deployment Repo**: `Davidcode2/app-of-apps`
- **Container Registry**: `ghcr.io/davidcode2/rwatch-agent`
- **Main Documentation**: `README.md`
- **Deployment Guide**: `DEPLOYMENT.md`
- **Rust Best Practices**: `RUST_BEST_PRACTICES.md`

---

*Last updated: 2025-03-06*
*Version: 0.1.3*
