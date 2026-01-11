//! # Rwatch Agent
//!
//! A daemon that exposes system metrics via a REST API.
//! This iteration implements a basic health check endpoint.

use axum::{
    routing::get,
    Json, Router,
};
use rwatch_common::HealthResponse;
use std::net::SocketAddr;
use std::time::Instant;

/// **Best Practice**: Use a global or shared state for tracking server start time.
/// Here we use a static for simplicity, but in production you'd use Arc<T> for shared state.
static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the start time
    START_TIME.set(Instant::now()).expect("START_TIME already initialized");

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
        .route("/health", get(health_handler))
}

/// Handler for the /health endpoint
///
/// **Best Practice Notes**:
/// - Async handlers are the norm with axum + tokio
/// - Returning `Json<T>` automatically serializes to JSON with correct headers
/// - Handler functions should be focused and single-purpose
async fn health_handler() -> Json<HealthResponse> {
    let start = START_TIME.get().expect("START_TIME not initialized");
    let uptime = start.elapsed().as_secs();

    // Use the factory method from common
    let response = HealthResponse::healthy(uptime);

    // **Common Pitfall**: Forgetting that Json<T> is a wrapper
    // The Json extractor/response handles serialization automatically
    Json(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::body::Body;
    use tower::ServiceExt; // For `oneshot` method

    /// **Best Practice**: Test your HTTP handlers without starting a real server
    #[tokio::test]
    async fn test_health_endpoint() {
        // Initialize START_TIME for the test
        let _ = START_TIME.set(Instant::now());

        let app = create_router();

        // Create a test request
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        // Check status code
        assert_eq!(response.status(), StatusCode::OK);

        // **Best Practice**: Verify the response body structure
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(health.status, "up");
        // Uptime should be a small positive number in tests
        assert!(health.uptime >= 0);
    }
}
