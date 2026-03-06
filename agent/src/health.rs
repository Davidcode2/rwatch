use crate::START_TIME;
use axum::Json;
use rwatch_common::health::HealthResponse;

pub struct HealthHandler {}

impl HealthHandler {
    /// Handler for the /health endpoint
    ///
    /// **Best Practice Notes**:
    /// - Async handlers are the norm with axum + tokio
    /// - Returning `Json<T>` automatically serializes to JSON with correct headers
    /// - Handler functions should be focused and single-purpose
    pub async fn health_handler() -> Json<HealthResponse> {
        let start = START_TIME.get().expect("START_TIME not initialized");
        let uptime = start.elapsed().as_secs();

        // Use the factory method from common
        let response = HealthResponse::healthy(uptime);

        // **Common Pitfall**: Forgetting that Json<T> is a wrapper
        // The Json extractor/response handles serialization automatically
        Json(response)
    }
}

#[cfg(test)]
mod tests {
    use crate::Instant;
    use crate::create_router;
    use crate::metrics::K8sState;
    use rwatch_common::health::HealthResponse;
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use tower::ServiceExt; // For `oneshot` method

    /// **Best Practice**: Test your HTTP handlers without starting a real server
    /// This test requires a valid Kubernetes config. It will be skipped if not available.
    #[tokio::test]
    async fn test_health_endpoint() {
        // Initialize START_TIME for the test
        let _ = START_TIME.set(Instant::now());

        // Try to create K8s state, skip test if no config available
        let k8s_state = match K8sState::new().await {
            Ok(state) => state,
            Err(_) => {
                eprintln!("Skipping test: no Kubernetes config available");
                return;
            }
        };

        let app = create_router(k8s_state);

        // Create a test request
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Check status code
        assert_eq!(response.status(), StatusCode::OK);

        // **Best Practice**: Verify the response body structure
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health.status, "up");
    }
}
