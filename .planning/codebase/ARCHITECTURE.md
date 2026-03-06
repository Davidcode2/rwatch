# Architecture

**Analysis Date:** 2026-03-06

## Pattern Overview

**Overall:** Client-Server with Monorepo Workspace

**Key Characteristics:**
- Rust workspace with 4 crates: `agent`, `client`, `common`, `tui`
- Asynchronous HTTP server (agent) + HTTP client library (client) + terminal UI (tui)
- Shared types in `common` crate for type safety across boundaries
- Axum-based REST API for agents, reqwest for client

## Layers

**Agent Layer (`agent/`):**
- Purpose: Daemon that exposes system metrics via REST API
- Location: `agent/src/`
- Contains: HTTP server, handlers for /health and /memory endpoints
- Depends on: `rwatch-common` (types), `axum` (web framework), `tokio` (runtime)
- Used by: Direct HTTP clients, load balancers

**Client Layer (`client/`):**
- Purpose: Business logic library for querying and aggregating agent data
- Location: `client/src/`
- Contains: `Client`, `AgentData`, `AgentResult`, `AggregatedMetrics`, discovery mechanisms
- Depends on: `rwatch-common` (types), `reqwest` (HTTP client), `futures` (async)
- Used by: `tui` crate, future UI consumers

**Common Layer (`common/`):**
- Purpose: Shared types and protocols used by all crates
- Location: `common/src/`
- Contains: `HealthResponse`, `Memory`, `MemoryWithUnit` display helpers
- Depends on: `serde`, `serde_json`
- Used by: `agent`, `client`, `tui`

**TUI Layer (`tui/`):**
- Purpose: Terminal UI application that queries agents and displays metrics
- Location: `tui/src/`
- Contains: Application entry point, display functions
- Depends on: `rwatch-client`, `rwatch-common`
- Used by: End users running the CLI

## Data Flow

**Agent Query Flow:**

1. TUI creates `Client` instance
2. TUI uses `Discovery` (env vars, static, or K8s) to find agent URLs
3. Client calls `query_agents()` which spawns concurrent async tasks
4. Each task makes HTTP GET to agent's `/health` and `/memory` endpoints
5. Results are collected as `AgentResult` enum (Success/Failure)
6. TUI calls `aggregate_results()` to compute cluster-wide metrics
7. TUI displays via `ui.rs` functions

**Agent Metrics Flow:**

1. External request hits axum router at `/health` or `/memory`
2. Handler instantiates (HealthHandler, MemoryHandler)
3. HealthHandler reads `START_TIME` static for uptime calculation
4. MemoryHandler calls `Memory::memory()` which reads `/proc/meminfo`
5. Response serialized to JSON via axum's `Json<T>` wrapper

## Key Abstractions

**Discovery (trait-like pattern via enum):**
- Purpose: Abstract agent discovery mechanisms
- Examples: `client/src/discovery.rs`
- Pattern: Enum with variants for different discovery strategies (Static, Kubernetes, Env)

**AgentResult:**
- Purpose: Type-safe result handling for agent queries
- Examples: `client/src/lib.rs` - `AgentResult::Success(AgentData)`, `AgentResult::Failure{url, error}`
- Pattern: Result enum pattern with methods (`is_success()`, `url()`, `data()`)

**Handler Pattern:**
- Purpose: HTTP request handlers in agent
- Examples: `agent/src/health.rs`, `agent/src/memory.rs`
- Pattern: Static struct with async methods returning `Json<T>`

## Entry Points

**Agent Server:**
- Location: `agent/src/main.rs`
- Triggers: Running `cargo run -p rwatch-agent`
- Responsibilities: Initialize axum router, bind to port 3000, serve HTTP

**TUI Application:**
- Location: `tui/src/main.rs`
- Triggers: Running `cargo run -p rwatch-tui`
- Responsibilities: Create client, discover agents, query agents, display output

**Library Crates:**
- `common`: Exported via `lib.rs` - `pub mod health`, `pub mod memory`, `pub mod memory_display`
- `client`: Exported via `lib.rs` - `Client`, `AgentData`, `AgentResult`, `AggregatedMetrics`, discovery

## Error Handling

**Strategy:** Result types with anyhow for context propagation

**Patterns:**
- Agent handlers return `Json<T>` - errors become HTTP 500 (panic) or use `Result` type
- Client methods return `Result<T>` with anyhow context: `.with_context(|| format!("Failed to..."))?`
- Discovery returns `Result<AgentList>` - sync methods use `?`, async use try_join
- TUI uses `anyhow::Result<()>` with `.context()` for user-friendly errors

## Cross-Cutting Concerns

**Logging:** Print statements with emojis and context (`println!("🚀 Rwatch Agent starting on {}", addr)`)

**Validation:** Not extensively implemented; type system handles much validation

**Authentication:** Not implemented - no auth on agent endpoints

---

*Architecture analysis: 2026-03-06*
