//! # Rwatch Agent
//!
//! A daemon that exposes system metrics via a REST API.
//! This iteration implements a basic health check endpoint.

use axum::{ Router, routing::get};
use memory::MemoryHandler;
use health::HealthHandler;
use std::net::SocketAddr;
use std::time::Instant;

mod memory;
mod health;

/// **Best Practice**: Use a global or shared state for tracking server start time.
/// Here we use a static for simplicity, but in production you'd use Arc<T> for shared state.
static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the start time
    START_TIME
        .set(Instant::now())
        .expect("START_TIME already initialized");

    // **Best Practice**: Make the bind address configurable
    // For now it's hardcoded, but you'd typically use clap or config files
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    // Build the router with our routes
    let app = create_router();

    // **Best Practice**: Log important startup information
    println!("🚀 Rwatch Agent starting on {}", addr);
    println!("📊 Health endpoint: http://{}/health", addr);

    // Start the server
    // **Common Pitfall**: Not handling the Result from serve()
    // Always propagate errors with `?` or handle them explicitly
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Creates the application router
///
/// **Best Practice**: Separate router creation from main() for testability
fn create_router() -> Router {
    Router::new()
        .route("/health", get(HealthHandler::health_handler))
        .route("/memory", get(MemoryHandler::memory_handler))
}
