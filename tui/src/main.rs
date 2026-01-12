//! # Rwatch TUI
//!
//! A client that connects to agents and displays their health status.
//! This iteration implements a simple stdout-based display.

use anyhow::{Context, Result};
use rwatch_common::health::HealthResponse;

#[tokio::main]
async fn main() -> Result<()> {
    // **Best Practice**: Use anyhow::Context to add context to errors
    // This makes debugging much easier
    run().await.context("Failed to run TUI application")?;
    Ok(())
}

/// Main application logic
///
/// **Best Practice**: Separate the main entry point from business logic
/// This makes testing and error handling cleaner
async fn run() -> Result<()> {
    println!("🔍 Rwatch TUI - Connecting to agent...\n");

    // **Note**: In a real app, this would come from a config file
    // For iteration 1, we hardcode the agent URL
    let agent_url = "http://localhost:3000";

    // Query the agent's health endpoint
    // **Common Pitfall**: Not handling network errors properly
    // Always use `.context()` or similar to provide helpful error messages
    let health = query_agent_health(agent_url)
        .await
        .context(format!("Failed to query agent at {}", agent_url))?;

    // Display the result
    display_health(agent_url, &health);

    Ok(())
}

/// Queries an agent's health endpoint
///
/// **Best Practice**: 
/// - Keep network logic in separate, testable functions
/// - Use `Result<T>` for operations that can fail
/// - Use `&str` for borrowed strings when you don't need ownership
async fn query_agent_health(base_url: &str) -> Result<HealthResponse> {
    // Build the full URL
    let url = format!("{}/health", base_url);

    // **Best Practice**: Create a client once and reuse it
    // For this simple example, we create it inline, but in a real app
    // you'd create it once and pass it around or store it in app state
    let client = reqwest::Client::new();

    // Make the request
    // **Common Pitfall**: Forgetting that both the request AND json parsing can fail
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to send HTTP request")?;

    // **Best Practice**: Check the status code before parsing
    if !response.status().is_success() {
        anyhow::bail!("Agent returned error status: {}", response.status());
    }

    // Parse the JSON response
    // **Common Pitfall**: Using unwrap() instead of proper error handling
    let health = response
        .json::<HealthResponse>()
        .await
        .context("Failed to parse JSON response")?;

    Ok(health)
}

/// Displays health information to stdout
///
/// **Best Practice**: Separate display logic from business logic
/// This makes it easy to swap stdout for a proper TUI later
fn display_health(agent_url: &str, health: &HealthResponse) {
    println!("╔════════════════════════════════════════╗");
    println!("║           Agent Health Status          ║");
    println!("╠════════════════════════════════════════╣");
    println!("║ Agent:   {:<30}║", agent_url);
    println!("║ Status:  {:<30}║", health.status);
    println!("║ Uptime:  {:<30}║", format!("{}s", health.uptime));
    println!("║ Version: {:<30}║", health.version);
    println!("╚════════════════════════════════════════╝");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Best Practice**: Test pure functions without external dependencies
    #[test]
    fn test_display_health() {
        let health = HealthResponse::healthy(42);
        // Just verify it doesn't panic - in a real app you'd capture stdout
        display_health("http://test", &health);
    }

    // **Note**: Testing `query_agent_health` would require mocking
    // For now, we skip it, but in production you'd use a trait-based
    // HTTP client or a mocking library like `mockito`
}
