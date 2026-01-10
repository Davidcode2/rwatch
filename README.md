# Rwatch

This project is my first project using rust.
The goal is to learn the language and have some fun in the process.

I'll rely heavily on AI for the first iteration.

## Project goal

Rwatch is a lightweight, real-time monitoring tool built in Rust, consisting of:

- **TUI (Terminal User Interface)**: A client application for visualizing metrics
- **Agents**: One or more daemons collecting system metrics on Linux servers

### Key Features

- **Real-time monitoring**: CPU, memory, and network metrics
- **In-memory storage**: Fast access with automatic retention management
- **HTTP-based communication**: Simple, firewall-friendly protocol
- **Multiple agent support**: Monitor multiple servers from a single TUI

### Rust Learning Focus

This project explores:

- Workspace-based monorepo structure
- Async/await with Tokio for network I/O
- Type-safe data serialization with Serde
- Error handling with Result types
- Shared libraries between binaries
- TUI development with Ratatui

## Implementation details

### tui

The tui reads a configuration file in yaml format specifying the connection details for the agents.

### agent

The agent will collect metrics at configurable intervals. The snapshots will be
stored by the agent in memory until a configurable memory threshold is reached.
After the threshold is reached, the oldest snapshot will be deleted.

Metrics are stored as strongly-typed Rust structs and serialized to JSON:

```json
{
  "kind": "memory",
  "timestamp": "2026-01-10T21:23:00Z",
  "used": 4000,
  "total": 16000,
  "unit": "MiB"
}
```

**Rust Best Practices:**
- Use `serde` for JSON serialization/deserialization
- Define metrics as enums with associated data (sum types)
- Use `chrono` or `time` crate for timestamps (ISO 8601 format)
- Store numeric values as proper types (u64, f64), not strings

### communication

communication between tui and agent happens via http.

### Repository Setup

Cargo workspace structure:
```
rwatch/
├── Cargo.toml          # Workspace root
├── agent/              # Agent binary crate
├── tui/                # TUI binary crate
└── common/             # Shared library crate (data types, protocols)
```

**Benefits:**
- Shared types between agent and TUI (DRY principle)
- Single `target/` directory for build artifacts
- Easy to test integration between components
- Version dependencies centrally

# First iteration

- clean project setup according to rust best practices
- tui capable of querying /health endpoint of agents 
- basic tui showing
    - available agents
- agent with 
    - http GET endpoint for /health
    - /health endpoint providing health response in json format.
