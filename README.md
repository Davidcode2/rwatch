# Rwatch

**Rwatch** is a lightweight, real-time monitoring tool built in Rust. 

This project is designed as a deep-dive into the Rust ecosystem. The goal is to
build a functional tool while exploring intermediate-to-advanced concepts like
async networking, concurrency, workspace management, and terminal user
interfaces.

## Project Architecture

Rwatch consists of three main components organized in a Cargo Workspace:

1.  **`agent`**: A background daemon that runs on Linux servers. It collects
    system metrics and exposes them via a REST API.
2.  **`tui`**: A Terminal User Interface client. It reads a config file,
    connects to multiple agents, and visualizes the data.
3.  **`common`**: A shared library crate containing data structures, protocol
    definitions, and utility functions shared between the Agent and TUI to
    ensure type safety (DRY).

### Key Features

* **Real-time monitoring**: CPU, Memory, and Network I/O.
* **Agent-side History**: Agents maintain a rolling history of metrics in memory.
* **Stateless Client**: The TUI can be restarted without losing historical data (as long as agents are running).
* **HTTP Protocol**: Simple, firewall-friendly JSON communication.

## Tech Stack & Learning Goals

This project explores the following Rust concepts and libraries:

| Concept | Library / Tool | Purpose |
| --- | --- |  --- |
| **Workspace** | Cargo | Managing a monorepo with shared dependencies. |
| **Async Runtime** | `tokio` | Handling non-blocking I/O and task scheduling. |
| **Web Server** | `axum` | Serving metrics from the Agent. |
| **HTTP Client** | `reqwest` | Fetching metrics in the TUI. |
| **Serialization** | `serde` | JSON parsing/generating. |
| **System Stats** | `sysinfo` | Cross-platform system metric collection. |
| **TUI** | `ratatui` | Rendering the interface. |
| **Shared State** | `Arc<RwLock<T>>` | Managing safe access to data across threads. |


## Implementation Details

### 1. The Agent (`rwatch-agent`)

The agent has two primary asynchronous responsibilities running concurrently:

1. **Collector Task**: Wakes up at a configurable interval (e.g., 1s), uses
   `sysinfo` to snapshot system metrics, and pushes them into storage.
2. **Server Task**: An `axum` web server listening for HTTP requests to serve
   data to the TUI.

**Storage Strategy (Ring Buffer):**

Instead of infinite growth or complex memory thresholds, the agent uses a
**Fixed-Capacity Ring Buffer** (e.g., `VecDeque` with a limit).

* *Logic:* "Keep the last 600 seconds of data."
* *Benefit:* Deterministic memory usage and O(1) operations.

### 2. The TUI (`rwatch-tui`)

The TUI is an async application that:

1. Reads a `config.yaml` to find Agent IPs.
2. Polls agents via HTTP.
3. Renders the UI using `ratatui`.

### 3. Communication & Data (`rwatch-common`)

Communication happens via HTTP. Data is serialized to JSON.

**Metric Structure:** 

Metrics are defined as an internally tagged enum to
handle different data types cleanly.

```json
// Example Metric Snapshot
{
  "kind": "memory",
  "timestamp": 1704921780,
  "used": 4000,
  "total": 16000,
  "unit": "MiB"
}

```

## Repository Structure

```text
rwatch/
├── Cargo.toml          # Workspace root
├── common/             # Shared library (Structs, Enums, Error types)
│   ├── src/lib.rs
│   └── Cargo.toml
├── agent/              # The daemon
│   ├── src/
│   │   ├── main.rs     # Entry point & Runtime setup
│   │   ├── state.rs    # Shared State (Arc<RwLock>) & RingBuffer logic
│   │   ├── collector.rs# Sysinfo gathering logic
│   │   └── server.rs   # Axum handlers
│   └── Cargo.toml
└── tui/                # The client
    ├── src/
    │   ├── main.rs
    │   ├── app.rs      # TUI State
    │   └── ui.rs       # Ratatui widgets
    └── Cargo.toml

```

---

## Iteration 1: The "Hello World" of Monitoring

The goal of the first iteration is to establish the workspace, networking, and
basic types without getting bogged down in complex UI logic.

**Definition of Done:**

1. **Workspace Created**: Project compiles with `agent`, `tui`, and `common`.
2. **Common Library**: `HealthResponse` struct is defined in `common` and used by both binaries.
3. **Agent**:
    * Runs an HTTP server on a configurable port.
    * Exposes `GET /health` returning JSON: `{"status": "up", "uptime": 123, "version": "0.1.0"}`.
4. **TUI**:
    * Reads a list of agents from config.
    * Queries the `/health` endpoint of an agent.
    * Displays the result in a basic list (can be simple stdout or a very basic Ratatui list).
