# Rwatch

A lightweight, real-time monitoring tool for Linux systems. Rwatch consists of an agent that runs on each node collecting metrics, and a client library that queries agents to display cluster-wide information.

## Current State

Rwatch provides memory monitoring via agents deployed as a Kubernetes DaemonSet. Each agent reads from `/proc/meminfo` and exposes metrics via HTTP. The client library queries agents concurrently and aggregates the results.

### What Works

- **Agent**: HTTP server with `/health` and `/memory` endpoints
- **Client**: Library for querying multiple agents and aggregating metrics
- **TUI**: Terminal interface displaying cluster memory usage
- **Kubernetes Deployment**: Automated deployment via GitHub Actions and ArgoCD

## Architecture

```
rwatch/
├── agent/      # Daemon that collects system metrics
├── client/     # Library for querying agents
├── common/     # Shared types (HealthResponse, Memory)
└── tui/        # Terminal UI using the client library
```

## Quick Start

### Local Development

```bash
# Start the agent
cargo run -p rwatch-agent

# In another terminal, run the TUI
cargo run -p rwatch-tui
```

### Frontend Development (Dummy Mode)

For developing the web frontend without a Kubernetes cluster, use dummy mode:

```bash
# Terminal 1: Start agent in dummy mode
cargo run -p rwatch-agent -- --dummy

# Terminal 2: Start the web frontend (from rwatch-web directory)
cd ../rwatch-web
npm run server  # or: npm run dev
```

**Dummy mode provides:**
- 3 simulated nodes with realistic CPU/memory metrics
- 40-50 pods across multiple namespaces
- Smooth metric variations to simulate real cluster behavior
- All API endpoints work identically to production mode

### Kubernetes Deployment

Deployed via GitHub Actions → ArgoCD:

1. Push to main branch triggers workflow
2. GitHub Actions builds and pushes Docker image
3. Updates app-of-apps repository
4. ArgoCD auto-deploys to cluster

See [DEPLOYMENT.md](DEPLOYMENT.md) for detailed setup.

## Accessing Metrics

### From Local Machine

```bash
# Port-forward to cluster
kubectl port-forward -n rwatch service/rwatch-agent 3000:3000

# Run TUI locally
cargo run -p rwatch-tui
```

### From Within Cluster

Agents are accessible at:
```
rwatch-agent.rwatch.svc.cluster.local:3000
```

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

## Open Topics

### Known Limitations
- **Platform**: Linux only (requires `/proc/meminfo`)
- **Metrics**: Only memory (total/available), no CPU/network yet
- **History**: No persistence, agents return current snapshot only
- **Security**: No authentication on HTTP endpoints

### Completed Features
- ✅ **Port**: Configurable via `--port` flag or `PORT` env variable
- ✅ **Dummy Mode**: Test data generation for frontend development

### Future Work
- **Configuration**: Config file support instead of env vars
- **Metrics**: Add CPU and network I/O monitoring
- **History**: Implement ring buffer for metric history
- **Discovery**: Complete Kubernetes API-based discovery
- **UI**: Interactive TUI with real-time updates (ratatui)
- **Web**: Web interface alternative to TUI

## Testing

```bash
# Run all tests
cargo test

# Run specific crate
cargo test -p rwatch-client
```

## License

[Add your license here]
