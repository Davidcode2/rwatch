//! # Rwatch TUI
//!
//! A client that connects to agents and displays their health status.
//! This iteration implements a simple stdout-based display.

use anyhow::{Context, Result};
use crate::ui::{display_health, display_memory};

mod ui;

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
    
    let memory = query_agent(agent_url, "memory") // returns Result<Memory>
            .await
            .context(format!("Failed to query memory from agent at {}", agent_url))?;
                                                  
    // Query the agent's health endpoint
    // **Common Pitfall**: Not handling network errors properly
    // Always use `.context()` or similar to provide helpful error messages
    let health = query_agent(agent_url, "health") // returns Result<HealthResponse>
        .await
        .context(format!("Failed to query agent at {}", agent_url))?;

    display_health(agent_url, &health);
    display_memory(&memory);

    Ok(())
}

/// Queries an agent's endpoint and deserializes the response
///
/// **Best Practice**: 
/// - Keep network logic in separate, testable functions
/// - Use `Result<T>` for operations that can fail
/// - Use `&str` for borrowed strings when you don't need ownership
/// 
/// **Trait Bounds**: The generic type `T` must implement `serde::de::DeserializeOwned`
/// This means it can be deserialized from owned data (required for async operations)
async fn query_agent<T>(base_url: &str, path: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    // Build the full URL
    let url = format!("{}/{}", base_url, path);

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
    let res = response
        .json::<T>()
        .await
        .context("Failed to parse JSON response")?;

    Ok(res)
}

