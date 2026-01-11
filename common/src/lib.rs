//! # Rwatch Common Library
//!
//! This crate contains shared types and protocols used by both the agent and TUI.
//! Keeping these in a shared library ensures type safety and DRY principles.

use serde::{Deserialize, Serialize};

/// Response from the agent's health endpoint
///
/// # Best Practice Notes:
/// - Using `#[derive(Debug)]` is essential for error messages and debugging
/// - `Clone` is useful for sharing data without ownership issues
/// - `Serialize/Deserialize` from serde enables JSON conversion
/// - Using `#[serde(rename_all = "snake_case")]` ensures consistent JSON formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HealthResponse {
    /// Current status of the agent ("up", "degraded", etc.)
    pub status: String,
    
    /// Uptime in seconds
    /// 
    /// **Common Pitfall**: Using `i64` vs `u64` for time values.
    /// Since uptime can't be negative, `u64` is more semantically correct.
    pub uptime: u64,
    
    /// Version of the agent
    pub version: String,
}

impl HealthResponse {
    /// Creates a new HealthResponse
    ///
    /// # Best Practice: Constructor pattern
    /// Providing a `new()` method is idiomatic Rust and makes intent clear.
    /// It also allows for validation or default values in the future.
    pub fn new(status: String, uptime: u64, version: String) -> Self {
        Self {
            status,
            uptime,
            version,
        }
    }

    /// Creates a healthy response with the given uptime
    ///
    /// # Best Practice: Factory methods
    /// Providing semantic constructors makes code more readable.
    pub fn healthy(uptime: u64) -> Self {
        Self::new("up".to_string(), uptime, env!("CARGO_PKG_VERSION").to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Best Practice**: Always include tests, even for simple types.
    /// Tests serve as documentation and prevent regression.
    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse::healthy(123);
        let json = serde_json::to_string(&response).unwrap();
        
        // Verify it contains expected fields
        assert!(json.contains("\"status\""));
        assert!(json.contains("\"uptime\""));
        assert!(json.contains("\"version\""));
    }

    #[test]
    fn test_health_response_deserialization() {
        let json = r#"{"status":"up","uptime":456,"version":"0.1.0"}"#;
        let response: HealthResponse = serde_json::from_str(json).unwrap();
        
        assert_eq!(response.status, "up");
        assert_eq!(response.uptime, 456);
        assert_eq!(response.version, "0.1.0");
    }
}
