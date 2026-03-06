use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Kubernetes client error: {0}")]
    K8sClient(String),
    
    #[error("Metrics server unavailable: {0}")]
    MetricsServerUnavailable(String),
    
    #[error("Failed to parse resource quantity: {0}")]
    ParseError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for MetricsError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            MetricsError::MetricsServerUnavailable(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        
        let body = Json(json!({
            "error": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));
        
        (status, body).into_response()
    }
}

pub type MetricsResult<T> = Result<T, MetricsError>;
