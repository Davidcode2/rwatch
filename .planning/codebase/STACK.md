# Technology Stack

**Analysis Date:** 2026-03-06

## Languages

**Primary:**
- Rust (2024 edition) - All crates use Rust as the primary language

**Secondary:**
- YAML - Kubernetes deployment manifests in `deploy/k8s/`

## Runtime

**Environment:**
- Native Rust binary - Compiled to standalone executables
- No VM or interpreter required

**Package Manager:**
- Cargo (Rust's package manager)
- Lockfile: `Cargo.lock` (present in workspace)

## Frameworks

**Core:**
- Axum 0.7 - Web framework for the agent HTTP server (`agent/src/main.rs`)
- Tokio 1.41 (with "full" features) - Async runtime for all crates

**HTTP Client:**
- Reqwest 0.12 (with "json" feature) - HTTP client for the client library (`client/src/lib.rs`)

**Serialization:**
- Serde 1.0 (with derive) - Serialization/deserialization framework
- Serde JSON 1.0 - JSON parsing

**Utilities:**
- Futures 0.3 - Concurrent futures utilities
- Anyhow 1.0 - Error handling with context

**Middleware:**
- Tower 0.5 - Middleware abstraction (planned for future use)

## Key Dependencies

**Workspace-level dependencies** (defined in root `Cargo.toml`):
- `tokio` 1.41 - Async runtime
- `serde` 1.0 - Serialization
- `serde_json` 1.0 - JSON handling
- `anyhow` 1.0 - Error handling

**Agent-specific (`agent/Cargo.toml`):**
- `axum` 0.7 - HTTP server
- `tower` 0.5 - Middleware (reserved for future)
- `rwatch-common` - Internal shared types

**Client-specific (`client/Cargo.toml`):**
- `reqwest` 0.12 - HTTP client with JSON support
- `futures` 0.3 - Future combinators
- `rwatch-common` - Internal shared types

**TUI-specific (`tui/Cargo.toml`):**
- `rwatch-client` - Agent querying
- `rwatch-common` - Shared types

**Common (`common/Cargo.toml`):**
- `serde` 1.0 - Serialization only (no runtime dependencies)

## Configuration

**Build Configuration:**
- Workspace `Cargo.toml` - Centralized dependency management
- Individual crate `Cargo.toml` files - Package metadata and dependencies

**Runtime Configuration:**
- Environment variables for agent discovery:
  - `RWATCH_AGENT_0`, `RWATCH_AGENT_1`, etc. - Agent URLs (`client/src/discovery.rs`)
- Hardcoded port 3000 in agent (`agent/src/main.rs` line 28)
- No config file support currently

**Kubernetes Configuration:**
- Kustomize manifests in `deploy/k8s/`
- Resource limits defined in DaemonSet

## Platform Requirements

**Development:**
- Rust 2024 edition support
- Standard Rust toolchain (cargo, rustc)
- Linux (for `/proc/meminfo` parsing)

**Production:**
- Containerized deployment via Kubernetes
- DaemonSet for agent daemons
- Deployment for TUI
- hostNetwork and hostPID enabled for node metrics

---

*Stack analysis: 2026-03-06*
