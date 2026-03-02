use rwatch_client::{AgentResult, AggregatedMetrics};
use rwatch_common::{health::HealthResponse, memory::Memory, memory_display::as_gb};

/// Displays health information to stdout
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

/// Displays memory information to stdout
pub fn display_memory(memory: &Memory) {
    println!("╔════════════════════════════════════════╗");
    println!("║           Memory Status                ║");
    println!("╠════════════════════════════════════════╣");
    let total = as_gb(memory.total);
    let available = as_gb(memory.available);
    println!(
        "║ Total:      {:>3}.{:02} GB                ║",
        total.amount, total.decimal
    );
    println!(
        "║ Available:  {:>3}.{:02} GB                ║",
        available.amount, available.decimal
    );
    println!(
        "║ Used:       {:>3}.{:02} GB                ║",
        as_gb(memory.total - memory.available).amount,
        as_gb(memory.total - memory.available).decimal
    );
    println!("╚════════════════════════════════════════╝");
}

/// Displays aggregated metrics from all agents
pub fn display_aggregated_metrics(metrics: &AggregatedMetrics) {
    println!("╔════════════════════════════════════════╗");
    println!("║       Cluster Health Summary           ║");
    println!("╠════════════════════════════════════════╣");
    println!("║ Total Nodes:    {:<22}║", metrics.total_nodes);
    println!("║ Healthy Nodes:  {:<22}║", metrics.healthy_nodes);
    println!("║ Failed Nodes:   {:<22}║", metrics.failed_nodes);
    println!("╠════════════════════════════════════════╣");

    // Convert bytes to GB for display
    let total_gb = metrics.total_memory_bytes as f64 / 1_000_000_000.0;
    let available_gb = metrics.available_memory_bytes as f64 / 1_000_000_000.0;
    let used_gb = total_gb - available_gb;
    let usage_pct = metrics.memory_usage_percent();

    println!("║ Total Memory:   {:>6.2} GB            ║", total_gb);
    println!(
        "║ Used Memory:    {:>6.2} GB ({:>5.1}%)   ║",
        used_gb, usage_pct
    );
    println!(
        "║ Available:      {:>6.2} GB ({:>5.1}%)   ║",
        available_gb,
        metrics.memory_available_percent()
    );
    println!("╚════════════════════════════════════════╝");
}

/// Displays a summary line for an agent
pub fn display_agent_summary(result: &AgentResult) {
    match result {
        AgentResult::Success(data) => {
            println!(
                "✅ {} - {} (uptime: {}s)",
                data.url, data.health.status, data.health.uptime
            );
        }
        AgentResult::Failure { url, error } => {
            println!("❌ {} - Error: {}", url, error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rwatch_client::{AgentData, AgentResult};

    #[test]
    fn test_display_health() {
        let health = HealthResponse::healthy(42);
        // Just verify it doesn't panic
        display_health("http://test", &health);
    }

    #[test]
    fn test_display_memory() {
        let memory = Memory::new(16000000, 4000000, 12000000, 12000000);
        // Just verify it doesn't panic
        display_memory(&memory);
    }

    #[test]
    fn test_display_aggregated_metrics() {
        let metrics = AggregatedMetrics {
            total_nodes: 3,
            healthy_nodes: 2,
            failed_nodes: 1,
            total_memory_bytes: 48_000_000_000,     // 48 GB
            available_memory_bytes: 24_000_000_000, // 24 GB
        };
        // Just verify it doesn't panic
        display_aggregated_metrics(&metrics);
    }

    #[test]
    fn test_display_agent_summary_success() {
        let data = AgentData {
            url: "http://agent1:3000".to_string(),
            health: HealthResponse::new("up".to_string(), 100, "0.1.0".to_string()),
            memory: Memory::new(16000000, 0, 0, 8000000),
        };
        let result = AgentResult::Success(data);
        // Just verify it doesn't panic
        display_agent_summary(&result);
    }

    #[test]
    fn test_display_agent_summary_failure() {
        let result = AgentResult::Failure {
            url: "http://agent1:3000".to_string(),
            error: "Connection refused".to_string(),
        };
        // Just verify it doesn't panic
        display_agent_summary(&result);
    }
}
