//! # Rwatch Agent
//!
//! A daemon that exposes system metrics via a REST API.
//! Supports dummy/test mode for frontend development.

use axum::{Router, routing::get};
use clap::Parser;
use std::net::SocketAddr;
use std::time::Instant;

mod health;
mod memory;
mod metrics;

// Dummy mode module (only compiled when dummy feature is enabled)
mod dummy;

use health::HealthHandler;
use memory::MemoryHandler;
use metrics::{K8sState, NodeMetricsHandler, PodMetricsHandler, SummaryHandler};

use dummy::{DummyAppState, DummyState};

/// Command-line arguments for the agent
#[derive(Parser, Debug)]
#[command(name = "rwatch-agent")]
#[command(about = "Rwatch monitoring agent")]
struct Args {
    /// Enable dummy/test mode with simulated data
    #[arg(long, short, env = "DUMMY_MODE")]
    dummy: bool,
    
    /// Port to bind to
    #[arg(long, short, env = "PORT", default_value = "3000")]
    port: u16,
}

/// **Best Practice**: Use a global or shared state for tracking server start time.
/// Here we use a static for simplicity, but in production you'd use Arc<T> for shared state.
static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize the start time
    START_TIME
        .set(Instant::now())
        .expect("START_TIME already initialized");

    // Build the bind address from command-line args
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));

    // **Best Practice**: Log important startup information
    if args.dummy {
        println!("🧪 DUMMY MODE ENABLED - Serving simulated data");
        println!("   Use this mode for frontend development without a K8s cluster");
    }
    println!("🚀 Rwatch Agent starting on {}", addr);

    // Build the appropriate router based on mode
    let app = if args.dummy {
        create_dummy_router()
    } else {
        // Initialize K8s client for real mode
        let k8s_state = K8sState::new().await?;
        create_real_router(k8s_state)
    };

    // Log endpoints
    println!("📊 Health endpoint: http://{}/health", addr);
    println!("📊 Memory endpoint: http://{}/memory", addr);
    println!("☸️  K8s Nodes endpoint: http://{}/api/metrics/nodes", addr);
    println!("☸️  K8s Pods endpoint: http://{}/api/metrics/pods", addr);
    println!("☸️  K8s Summary endpoint: http://{}/api/metrics/summary", addr);

    // Start the server
    // **Common Pitfall**: Not handling the Result from serve()
    // Always propagate errors with `?` or handle them explicitly
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Creates the application router for real mode (with K8s)
///
/// **Best Practice**: Separate router creation from main() for testability
fn create_real_router(k8s_state: K8sState) -> Router {
    Router::new()
        .route("/health", get(HealthHandler::health_handler))
        .route("/memory", get(MemoryHandler::memory_handler))
        .route("/api/metrics/nodes", get(NodeMetricsHandler::handler))
        .route("/api/metrics/pods", get(PodMetricsHandler::handler))
        .route("/api/metrics/summary", get(SummaryHandler::handler))
        .with_state(k8s_state)
}

/// Creates the application router for dummy/test mode
///
/// Returns simulated data without requiring a K8s cluster
fn create_dummy_router() -> Router {
    let dummy_state = DummyAppState {
        dummy_state: std::sync::Arc::new(DummyState::new()),
    };

    Router::new()
        .route("/health", get(dummy::handlers::health_handler))
        .route("/memory", get(dummy::handlers::memory_handler))
        .route("/api/metrics/nodes", get(dummy::handlers::nodes_handler))
        .route("/api/metrics/pods", get(dummy::handlers::pods_handler))
        .route("/api/metrics/summary", get(dummy::handlers::summary_handler))
        .with_state(dummy_state)
}
