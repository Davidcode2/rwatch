//! State management for dummy data generation
//!
//! Maintains smooth variations in metrics data to simulate real cluster behavior

use rand::rngs::StdRng;
use rand::SeedableRng;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Shared state for dummy data generation
pub struct DummyState {
    /// When the dummy server started
    pub start_time: Instant,
    /// Current node metrics that vary smoothly
    pub node_metrics: Arc<Mutex<Vec<NodeData>>>,
    /// Current pod metrics that vary smoothly
    pub pod_metrics: Arc<Mutex<Vec<PodData>>>,
    /// Random number generator for smooth variations
    pub rng: Mutex<StdRng>,
}

/// Internal representation of node data for smooth variation
#[derive(Clone)]
pub struct NodeData {
    pub name: String,
    pub cpu_capacity: i32,         // in millicores (e.g., 2000 = 2 cores)
    pub cpu_usage_base: f64,       // base usage percentage (0-100)
    pub cpu_usage_current: f64,    // current usage percentage
    pub memory_capacity: i64,      // in Mi
    pub memory_usage_base: f64,    // base usage percentage (0-100)
    pub memory_usage_current: f64, // current usage percentage
}

/// Internal representation of pod data
#[derive(Clone)]
pub struct PodData {
    pub name: String,
    pub namespace: String,
    pub node: String,
    pub cpu_base: i32, // in millicores
    pub cpu_current: i32,
    pub memory_base: i64, // in Mi
    pub memory_current: i64,
}

impl DummyState {
    /// Initialize dummy state with 3 nodes and 40-50 pods
    pub fn new() -> Self {
        let mut rng = StdRng::from_entropy();

        // Initialize 3 nodes with varying capacities
        let node_metrics = vec![
            NodeData {
                name: "node-1".to_string(),
                cpu_capacity: 2000,
                cpu_usage_base: 25.0,
                cpu_usage_current: 25.0,
                memory_capacity: 8192,
                memory_usage_base: 40.0,
                memory_usage_current: 40.0,
            },
            NodeData {
                name: "node-2".to_string(),
                cpu_capacity: 4000,
                cpu_usage_base: 35.0,
                cpu_usage_current: 35.0,
                memory_capacity: 16384,
                memory_usage_base: 55.0,
                memory_usage_current: 55.0,
            },
            NodeData {
                name: "node-3".to_string(),
                cpu_capacity: 2000,
                cpu_usage_base: 20.0,
                cpu_usage_current: 20.0,
                memory_capacity: 8192,
                memory_usage_base: 30.0,
                memory_usage_current: 30.0,
            },
        ];

        // Generate 40-50 pods across namespaces
        let namespaces = vec!["default", "kube-system", "app-namespace", "monitoring"];
        let pod_count = rand::Rng::gen_range(&mut rng, 40..=50);
        let mut pod_metrics = Vec::with_capacity(pod_count);

        for i in 0..pod_count {
            let node_idx = i % 3;
            let namespace = namespaces[i % namespaces.len()];

            pod_metrics.push(PodData {
                name: format!("pod-{:03}", i + 1),
                namespace: namespace.to_string(),
                node: format!("node-{}", node_idx + 1),
                cpu_base: rand::Rng::gen_range(&mut rng, 50..=500),
                cpu_current: 0,
                memory_base: rand::Rng::gen_range(&mut rng, 64..=1024),
                memory_current: 0,
            });
        }

        // Initialize current values
        let state = Self {
            start_time: Instant::now(),
            node_metrics: Arc::new(Mutex::new(node_metrics)),
            pod_metrics: Arc::new(Mutex::new(pod_metrics)),
            rng: Mutex::new(rng),
        };

        state.update_variations();
        state
    }

    /// Update metrics with smooth variations
    /// This should be called periodically to simulate changing cluster state
    pub fn update_variations(&self) {
        let mut rng = self.rng.lock().unwrap();

        // Update node metrics with small variations (±2%)
        if let Ok(mut nodes) = self.node_metrics.lock() {
            for node in nodes.iter_mut() {
                let cpu_variation: f64 = rand::Rng::gen_range(&mut *rng, -2.0..=2.0);
                node.cpu_usage_current = (node.cpu_usage_base + cpu_variation).clamp(5.0, 95.0);

                let mem_variation: f64 = rand::Rng::gen_range(&mut *rng, -2.0..=2.0);
                node.memory_usage_current =
                    (node.memory_usage_base + mem_variation).clamp(10.0, 90.0);
            }
        }

        // Update pod metrics with small variations (±10%)
        if let Ok(mut pods) = self.pod_metrics.lock() {
            for pod in pods.iter_mut() {
                let variation: f64 = rand::Rng::gen_range(&mut *rng, 0.9..=1.1);
                pod.cpu_current = ((pod.cpu_base as f64 * variation) as i32).clamp(10, 2000);

                let variation: f64 = rand::Rng::gen_range(&mut *rng, 0.9..=1.1);
                pod.memory_current = ((pod.memory_base as f64 * variation) as i64).clamp(32, 2048);
            }
        }
    }
}

impl Default for DummyState {
    fn default() -> Self {
        Self::new()
    }
}
