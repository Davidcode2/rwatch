# Rust Quick Reference for Rwatch

## 🚀 Quick Start Commands

```bash
# Build everything
cargo build

# Build in release mode (optimized)
cargo build --release

# Run the agent
cargo run -p rwatch-agent

# Run the TUI
cargo run -p rwatch-tui

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check code without building (fast)
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Update dependencies
cargo update

# Clean build artifacts
cargo clean
```

---

## 📖 Ownership & Borrowing Cheat Sheet

### Move vs Borrow vs Clone

```rust
let s1 = String::from("hello");

// MOVE (default for non-Copy types)
let s2 = s1;  
// ❌ Can't use s1 anymore - ownership moved to s2

// BORROW (read-only)
let s1 = String::from("hello");
let s2 = &s1;
// ✅ Both s1 and s2 are valid, s2 is a reference

// MUTABLE BORROW
let mut s1 = String::from("hello");
let s2 = &mut s1;
s2.push_str(" world");
// ✅ s2 can modify, but only ONE mutable borrow at a time

// CLONE (explicit copy)
let s1 = String::from("hello");
let s2 = s1.clone();
// ✅ Both valid, but s2 is a separate copy (expensive)
```

### Borrowing Rules

1. **Either** one mutable reference **OR** any number of immutable references
2. References must always be valid (no dangling pointers)

```rust
let mut x = 5;
let r1 = &x;      // ✅ immutable borrow
let r2 = &x;      // ✅ immutable borrow
let r3 = &mut x;  // ❌ ERROR: can't borrow as mutable while immutable borrows exist

println!("{} {}", r1, r2);
// After last use of r1, r2...
let r3 = &mut x;  // ✅ Now OK!
```

---

## 🔄 Async Patterns

### Basic Async Function

```rust
async fn fetch_data() -> Result<String> {
    let response = reqwest::get("https://api.example.com")
        .await?;
    let text = response.text().await?;
    Ok(text)
}
```

### Concurrent Tasks

```rust
use tokio::task;

// Run tasks concurrently
let task1 = task::spawn(async { fetch_data("url1").await });
let task2 = task::spawn(async { fetch_data("url2").await });

// Wait for both
let (result1, result2) = tokio::join!(task1, task2);
```

### Select (First to Complete)

```rust
tokio::select! {
    result = fetch_data() => println!("Got data: {:?}", result),
    _ = tokio::time::sleep(Duration::from_secs(5)) => println!("Timeout!"),
}
```

---

## 🎯 Error Handling Patterns

### Option<T> - Value might not exist

```rust
let maybe_value: Option<i32> = Some(5);

// Pattern matching
match maybe_value {
    Some(v) => println!("Got {}", v),
    None => println!("No value"),
}

// Using if let
if let Some(v) = maybe_value {
    println!("Got {}", v);
}

// Unwrap with default
let value = maybe_value.unwrap_or(0);

// Map
let doubled = maybe_value.map(|v| v * 2);

// Chaining
let result = maybe_value
    .map(|v| v * 2)
    .filter(|v| v > &5)
    .unwrap_or(100);
```

### Result<T, E> - Operation might fail

```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// Early return with ?
fn calculate() -> Result<i32, String> {
    let x = divide(10, 2)?;  // If Err, returns early
    let y = divide(x, 3)?;
    Ok(y)
}

// Pattern matching
match divide(10, 2) {
    Ok(result) => println!("Result: {}", result),
    Err(e) => println!("Error: {}", e),
}

// Convert to Option
let maybe = divide(10, 2).ok();  // Result -> Option
```

### anyhow for Easy Error Handling

```rust
use anyhow::{Context, Result};

fn process() -> Result<()> {
    let file = std::fs::read_to_string("config.yaml")
        .context("Failed to read config file")?;
    
    let config: Config = serde_yaml::from_str(&file)
        .context("Failed to parse config")?;
    
    Ok(())
}
```

---

## 🏗️ Common Type Patterns

### Creating New Types (Type Safety)

```rust
// Instead of using String everywhere
pub struct AgentUrl(String);

impl AgentUrl {
    pub fn new(url: String) -> Result<Self, String> {
        if url.starts_with("http") {
            Ok(Self(url))
        } else {
            Err("URL must start with http".to_string())
        }
    }
}

// Now you can't accidentally pass the wrong string!
fn query_agent(url: AgentUrl) { ... }
```

### Builder Pattern

```rust
pub struct Config {
    port: u16,
    host: String,
    timeout: Duration,
}

impl Config {
    pub fn new() -> Self {
        Self {
            port: 3000,
            host: "localhost".to_string(),
            timeout: Duration::from_secs(30),
        }
    }
    
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    pub fn host(mut self, host: String) -> Self {
        self.host = host;
        self
    }
}

// Usage
let config = Config::new()
    .port(8080)
    .host("0.0.0.0".to_string());
```

---

## 🔒 Shared State Patterns

### Arc<RwLock<T>> - Shared mutable state

```rust
use std::sync::{Arc, RwLock};

// Create shared state
let metrics = Arc::new(RwLock::new(Vec::new()));

// Clone Arc for other threads (cheap - just increments reference count)
let metrics_clone = Arc::clone(&metrics);

tokio::spawn(async move {
    // Write lock
    let mut data = metrics_clone.write().unwrap();
    data.push(Metric::new());
});

// Read lock (multiple readers allowed)
let data = metrics.read().unwrap();
println!("Count: {}", data.len());
```

### When to use what?

- `Arc<T>`: Multiple ownership, immutable (T: Send + Sync)
- `Arc<Mutex<T>>`: Multiple ownership, mutable, exclusive access
- `Arc<RwLock<T>>`: Multiple ownership, mutable, multiple readers OR one writer
- `Rc<T>`: Single-threaded multiple ownership (not Send)

---

## 📝 Useful Macros

```rust
// Print to stderr for debugging (removed in release builds with cfg)
dbg!(my_variable);

// Assert in tests
assert_eq!(actual, expected);
assert!(condition);

// Format strings
let s = format!("Hello {}", name);

// Print
println!("Value: {}", x);      // Display trait
println!("Debug: {:?}", x);    // Debug trait
println!("Pretty: {:#?}", x);  // Pretty Debug

// Include file contents at compile time
let html = include_str!("template.html");

// Get environment variable at compile time
let version = env!("CARGO_PKG_VERSION");

// Vector literal
let v = vec![1, 2, 3];

// HashMap literal
let mut map = HashMap::new();
// Or with maplit crate:
// let map = hashmap!{ "key" => "value" };
```

---

## 🧪 Testing Patterns

### Basic Test

```rust
#[test]
fn test_addition() {
    assert_eq!(2 + 2, 4);
}

#[test]
#[should_panic]
fn test_panic() {
    panic!("This test expects a panic");
}
```

### Async Test

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Test with Setup

```rust
#[test]
fn test_with_setup() {
    // Arrange
    let data = setup_test_data();
    
    // Act
    let result = process(data);
    
    // Assert
    assert_eq!(result, expected);
}
```

### Test Module

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn common_setup() -> TestData {
        // Shared setup
    }
    
    #[test]
    fn test_one() {
        let data = common_setup();
        // ...
    }
}
```

---

## 🎨 Common Trait Implementations

### Display (for user-facing strings)

```rust
use std::fmt;

impl fmt::Display for HealthResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Status: {} (uptime: {}s)", self.status, self.uptime)
    }
}

// Usage
println!("{}", health);  // Uses Display
```

### From/Into (conversions)

```rust
impl From<u32> for MyType {
    fn from(value: u32) -> Self {
        MyType { id: value }
    }
}

// Usage
let my_type: MyType = 42u32.into();
let my_type = MyType::from(42u32);
```

---

## 🔍 Debugging Tips

### Print debugging

```rust
dbg!(&my_variable);  // Prints file, line, and value

println!("{:#?}", complex_struct);  // Pretty print
```

### Conditional compilation

```rust
#[cfg(debug_assertions)]
println!("Debug mode only");

#[cfg(feature = "my-feature")]
fn special_function() { }
```

### Logging with tracing

```rust
use tracing::{info, warn, error, debug};

info!("Server started on port {}", port);
warn!("Connection timeout for {}", client);
error!("Failed to process request: {:?}", err);
debug!(counter = count, "Processing batch");
```

---

## 💡 Quick Tips

1. **Use `cargo fmt`** before every commit
2. **Run `cargo clippy`** to catch common mistakes
3. **Read compiler errors carefully** - Rust's errors are very helpful!
4. **Use `_` prefix for unused variables**: `_unused` (silences warnings)
5. **Use `todo!()` macro** for unimplemented code that should compile
6. **Use `unimplemented!()` macro** for methods you don't plan to implement
7. **Match exhaustively** - compiler ensures you handle all cases
8. **Use `if let` / `while let`** for single-pattern matches
9. **Iterator methods** (map, filter, fold) are often better than loops
10. **Don't fight the borrow checker** - if it's hard, redesign

---

## 🚨 Common Compiler Errors Decoded

### "cannot borrow as mutable because it is also borrowed as immutable"

```rust
// ❌ Error
let mut x = vec![1, 2, 3];
let first = &x[0];  // immutable borrow
x.push(4);          // ❌ mutable borrow
println!("{}", first);

// ✅ Fixed - use reference after mutable borrow
let mut x = vec![1, 2, 3];
let first = x[0];  // Copy the value
x.push(4);
println!("{}", first);
```

### "move occurs because ... does not implement Copy"

```rust
// ❌ Error
let s = String::from("hello");
let s2 = s;   // s moved
println!("{}", s);  // ❌ s is moved

// ✅ Fixed - clone or borrow
let s = String::from("hello");
let s2 = s.clone();  // Explicit clone
println!("{}", s);   // ✅ s still valid
```

### "cannot infer type"

```rust
// ❌ Error
let result = vec.iter().map(|x| x * 2).collect();  // ❌ What type?

// ✅ Fixed - type annotation
let result: Vec<i32> = vec.iter().map(|x| x * 2).collect();
// Or turbofish
let result = vec.iter().map(|x| x * 2).collect::<Vec<i32>>();
```

---

Happy Rust coding! 🦀
