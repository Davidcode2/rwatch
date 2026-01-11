# Rust Best Practices & Common Pitfalls for Rwatch

This document highlights key Rust concepts, best practices, and common pitfalls based on the Rwatch codebase.

## 🏗️ Project Structure

### ✅ Best Practice: Cargo Workspaces

```toml
[workspace]
resolver = "2"  # Use resolver v2 for better dependency resolution
members = ["agent", "tui", "common"]
```

**Why?**
- Shares a single `Cargo.lock` across all crates
- Ensures dependency versions are consistent
- Allows `cargo build` at the root to build all crates
- Enables internal dependencies via path references

### ✅ Best Practice: Workspace Dependencies

```toml
[workspace.dependencies]
tokio = { version = "1.41", features = ["full"] }
```

**Why?**
- Single source of truth for versions
- Each crate uses `tokio = { workspace = true }`
- Makes updates easier and prevents version conflicts

---

## 📦 Type Design (common crate)

### ✅ Best Practice: Derive All The Things

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse { ... }
```

**Essential derives:**
- `Debug`: Required for error messages and `dbg!()`
- `Clone`: Needed when data is shared across threads/tasks
- `Serialize/Deserialize`: For JSON/YAML/etc conversion

**Common Pitfall ❌**: Forgetting `Debug`
```rust
struct MyType { data: String }  // ❌ No Debug

// Later...
println!("{:?}", my_type);  // Compilation error!
```

### ✅ Best Practice: Constructor Patterns

```rust
impl HealthResponse {
    pub fn new(status: String, uptime: u64, version: String) -> Self { ... }
    pub fn healthy(uptime: u64) -> Self { ... }
}
```

**Why?**
- `new()` is idiomatic (like constructors in other languages)
- Factory methods (`healthy()`) provide semantic intent
- Allows validation or defaults without exposing struct details

### ✅ Best Practice: Correct Integer Types

```rust
pub uptime: u64,  // ✅ Can't be negative
```

**Common Pitfall ❌**: Using signed integers for always-positive values
```rust
pub uptime: i64,  // ❌ Allows negative values, wastes a bit
```

---

## 🌐 Agent (Axum Server)

### ✅ Best Practice: Async Main with Error Handling

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ... code that can fail
    Ok(())
}
```

**Why?**
- `anyhow::Result` allows you to use `?` operator
- Automatically prints error chains on failure
- Much better than `.unwrap()` or `panic!`

**Common Pitfall ❌**: Not returning Result
```rust
#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();  // ❌ Panics on error
}
```

### ✅ Best Practice: Separate Router Creation

```rust
fn create_router() -> Router {
    Router::new().route("/health", get(health_handler))
}
```

**Why?**
- Testable without starting a server
- Can be reused or composed
- Clear separation of concerns

### ✅ Best Practice: Typed Responses

```rust
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse::healthy(uptime))
}
```

**Why?**
- `Json<T>` automatically serializes and sets `Content-Type: application/json`
- Type-safe: won't compile if T doesn't implement Serialize
- Clear function signature documents what's returned

**Common Pitfall ❌**: Manual JSON serialization
```rust
async fn health_handler() -> String {
    serde_json::to_string(&response).unwrap()  // ❌ Loses type safety, panics, wrong headers
}
```

### ✅ Best Practice: Shared State with OnceLock

```rust
static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn main() {
    START_TIME.set(Instant::now()).expect("...");
}
```

**Why?**
- Thread-safe initialization
- Only initialized once
- Better than lazy_static for simple cases

**Future**: For mutable shared state, use `Arc<RwLock<T>>`:
```rust
type AppState = Arc<RwLock<Metrics>>;
```

---

## 💻 TUI (Reqwest Client)

### ✅ Best Practice: Error Context with anyhow

```rust
let health = query_agent_health(agent_url)
    .await
    .context(format!("Failed to query agent at {}", agent_url))?;
```

**Why?**
- Adds context to error chains
- Makes debugging SO much easier
- Shows the full error path

**Output example:**
```
Error: Failed to query agent at http://localhost:3000
Caused by:
    Failed to send HTTP request
Caused by:
    tcp connect error: Connection refused
```

**Common Pitfall ❌**: Bare error propagation
```rust
let health = query_agent_health(agent_url).await?;  // ❌ No context
```

### ✅ Best Practice: Check HTTP Status Before Parsing

```rust
let response = client.get(&url).send().await?;

if !response.status().is_success() {
    anyhow::bail!("Agent returned error status: {}", response.status());
}

let health = response.json::<HealthResponse>().await?;
```

**Why?**
- `.json()` will fail mysteriously if the response is an error page
- Explicit status check gives better error messages

**Common Pitfall ❌**: Direct JSON parsing
```rust
let health = client.get(&url).send().await?.json::<HealthResponse>().await?;
// ❌ If server returns 500, you get confusing JSON parse error
```

### ✅ Best Practice: String Borrowing

```rust
async fn query_agent_health(base_url: &str) -> Result<HealthResponse> { ... }
```

**Why?**
- `&str` is a borrowed string slice (no allocation)
- More flexible: accepts `&String`, `&str`, string literals
- Use `String` only when you need ownership

**Common Pitfall ❌**: Unnecessary String allocation
```rust
async fn query_agent_health(base_url: String) -> Result<HealthResponse> { ... }
// ❌ Forces caller to give up ownership or clone
```

---

## 🧪 Testing

### ✅ Best Practice: Unit Tests in Same File

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serialization() { ... }
}
```

**Why?**
- Tests live next to the code they test
- `#[cfg(test)]` means code is only compiled during `cargo test`
- Private functions can still be tested

### ✅ Best Practice: Async Tests

```rust
#[tokio::test]
async fn test_health_endpoint() {
    let app = create_router();
    let response = app.oneshot(...).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

**Why?**
- `#[tokio::test]` is like `#[tokio::main]` but for tests
- Axum's `oneshot()` lets you test handlers without a real server

---

## 🚨 Common Pitfalls Summary

| ❌ Pitfall | ✅ Solution |
|-----------|-----------|
| Using `.unwrap()` everywhere | Return `Result<T>` and use `?` |
| Forgetting `Debug` derive | Always derive Debug on types |
| Using `i64` for always-positive values | Use `u64`, `u32`, `usize` |
| Manual JSON serialization | Use `Json<T>` with Axum |
| No error context | Use `.context("...")` from anyhow |
| Not checking HTTP status | Check `response.status()` before parsing |
| Taking `String` instead of `&str` | Use `&str` for borrowed strings |
| No tests | Write tests for all public APIs |
| Blocking code in async | Use `tokio::task::spawn_blocking` |
| Not handling async task failures | Store `JoinHandle` and `.await` them |

---

## 🔄 Async/Await Concepts

### Key Rules:
1. **`.await` only works in `async` functions**
   ```rust
   async fn fetch() -> String { ... }
   
   // ❌ Won't compile
   fn main() {
       let data = fetch().await;
   }
   
   // ✅ Correct
   #[tokio::main]
   async fn main() {
       let data = fetch().await;
   }
   ```

2. **Futures are lazy** - they do nothing until `.await`ed
   ```rust
   let future = fetch_data();  // Does nothing yet!
   let data = future.await;     // Now it runs
   ```

3. **`.await` is NOT blocking** - it yields to the runtime
   - Other tasks can run while waiting
   - This is why tokio can handle thousands of connections

---

## 🛠️ Next Steps for Iteration 2

When you're ready to expand:

1. **Shared State**: Use `Arc<RwLock<T>>` for the metrics ring buffer
   ```rust
   type AppState = Arc<RwLock<MetricsBuffer>>;
   ```

2. **Structured Logging**: Replace `println!` with `tracing`
   ```rust
   tracing::info!("Server started on {}", addr);
   ```

3. **Configuration**: Use `config` crate or `clap` for CLI args

4. **Proper TUI**: Integrate `ratatui` with `crossterm` for interactive UI

5. **Error Types**: Create custom error types with `thiserror`
   ```rust
   #[derive(Debug, thiserror::Error)]
   enum AgentError {
       #[error("Failed to collect metrics: {0}")]
       CollectionError(String),
   }
   ```

---

## 📚 Learning Resources

- **The Rust Book**: https://doc.rust-lang.org/book/
- **Async Book**: https://rust-lang.github.io/async-book/
- **Tokio Tutorial**: https://tokio.rs/tokio/tutorial
- **Axum Examples**: https://github.com/tokio-rs/axum/tree/main/examples

---

## ✅ Compilation Check

To verify everything works:

```bash
# Build all workspace members
cargo build

# Run tests
cargo test

# Run the agent (in terminal 1)
cargo run -p rwatch-agent

# Run the TUI (in terminal 2)
cargo run -p rwatch-tui
```

Expected output:
- Agent: Starts on port 3000
- TUI: Connects and displays health status
- All tests pass ✓
