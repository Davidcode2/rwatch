use rwatch_common::{health::HealthResponse, memory::Memory};

/// Displays health information to stdout
///
/// **Best Practice**: Separate display logic from business logic
/// This makes it easy to swap stdout for a proper TUI later
pub fn display_health(agent_url: &str, health: &HealthResponse) {
    println!("╔════════════════════════════════════════╗");
    println!("║           Agent Health Status          ║");
    println!("╠════════════════════════════════════════╣");
    println!("║ Agent:   {:<30}║", agent_url);
    println!("║ Status:  {:<30}║", health.status);
    println!("║ Uptime:  {:<30}║", format!("{}s", health.uptime));
    println!("║ Version: {:<30}║", health.version);
    println!("╚════════════════════════════════════════╝");
}

pub fn display_memory(memory: &Memory) {
    println!("╔════════════════════════════════════════╗");
    println!("║           Memory Status                ║");
    println!("╠════════════════════════════════════════╣");
    println!("║ Total:      {:<27}║", memory.total);
    println!("║ Available:  {:<27}║", memory.available);
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
