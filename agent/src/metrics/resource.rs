use super::error::{MetricsError, MetricsResult};
use serde::{Deserialize, Serialize};

/// Kubernetes metrics types (metrics-server API)
/// These are not part of k8s-openapi, so we define them here
pub mod k8s_metrics {
    use super::*;
    use k8s_openapi::apimachinery::pkg::api::resource::Quantity;

    /// NodeMetrics represents a node's resource usage metrics
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NodeMetrics {
        pub metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta,
        pub usage: NodeUsage,
    }

    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    pub struct NodeUsage {
        pub cpu: Quantity,
        pub memory: Quantity,
    }

    impl k8s_openapi::Resource for NodeMetrics {
        const GROUP: &'static str = "metrics.k8s.io";
        const VERSION: &'static str = "v1beta1";
        const API_VERSION: &'static str = "metrics.k8s.io/v1beta1";
        const KIND: &'static str = "NodeMetrics";
        const URL_PATH_SEGMENT: &'static str = "nodes";
        type Scope = k8s_openapi::ClusterResourceScope;
    }

    impl k8s_openapi::Metadata for NodeMetrics {
        type Ty = k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

        fn metadata(&self) -> &Self::Ty {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> &mut Self::Ty {
            &mut self.metadata
        }
    }

    impl k8s_openapi::ListableResource for NodeMetrics {
        const LIST_KIND: &'static str = "NodeMetricsList";
    }

    /// PodMetrics represents a pod's resource usage metrics
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PodMetrics {
        pub metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta,
        pub containers: Vec<ContainerMetrics>,
    }

    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContainerMetrics {
        pub name: String,
        pub usage: ContainerUsage,
    }

    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    pub struct ContainerUsage {
        pub cpu: Quantity,
        pub memory: Quantity,
    }

    impl k8s_openapi::Resource for PodMetrics {
        const GROUP: &'static str = "metrics.k8s.io";
        const VERSION: &'static str = "v1beta1";
        const API_VERSION: &'static str = "metrics.k8s.io/v1beta1";
        const KIND: &'static str = "PodMetrics";
        const URL_PATH_SEGMENT: &'static str = "pods";
        type Scope = k8s_openapi::NamespaceResourceScope;
    }

    impl k8s_openapi::Metadata for PodMetrics {
        type Ty = k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

        fn metadata(&self) -> &Self::Ty {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> &mut Self::Ty {
            &mut self.metadata
        }
    }

    impl k8s_openapi::ListableResource for PodMetrics {
        const LIST_KIND: &'static str = "PodMetricsList";
    }
}

/// Parse CPU quantity to millicores
/// Examples: "100m" -> 100, "2" -> 2000, "0.5" -> 500, "1000000000n" -> 1
pub fn parse_cpu_to_millicores(quantity: &str) -> MetricsResult<u64> {
    if quantity.ends_with('m') {
        // Millicores
        quantity[..quantity.len()-1]
            .parse::<u64>()
            .map_err(|e| MetricsError::ParseError(format!("CPU '{}': {}", quantity, e)))
    } else if quantity.ends_with('n') {
        // Nanocores - convert to millicores (1 millicore = 1,000,000 nanocores)
        quantity[..quantity.len()-1]
            .parse::<f64>()
            .map(|v| (v / 1_000_000.0) as u64)
            .map_err(|e| MetricsError::ParseError(format!("CPU '{}': {}", quantity, e)))
    } else if quantity.ends_with("u") {
        // Microcores - convert to millicores (1 millicore = 1,000 microcores)
        quantity[..quantity.len()-1]
            .parse::<f64>()
            .map(|v| (v / 1000.0) as u64)
            .map_err(|e| MetricsError::ParseError(format!("CPU '{}': {}", quantity, e)))
    } else {
        // Plain number means cores, convert to millicores
        quantity
            .parse::<f64>()
            .map(|v| (v * 1000.0) as u64)
            .map_err(|e| MetricsError::ParseError(format!("CPU '{}': {}", quantity, e)))
    }
}

/// Parse memory quantity to MiB
/// Examples: "2048Mi" -> 2048, "2Gi" -> 2048, "1048576Ki" -> 1024
pub fn parse_memory_to_mib(quantity: &str) -> MetricsResult<u64> {
    // Handle binary suffixes (Ki, Mi, Gi, Ti)
    if quantity.ends_with("Ki") {
        parse_with_suffix(&quantity[..quantity.len()-2], 1.0 / 1024.0)
    } else if quantity.ends_with("Mi") {
        parse_with_suffix(&quantity[..quantity.len()-2], 1.0)
    } else if quantity.ends_with("Gi") {
        parse_with_suffix(&quantity[..quantity.len()-2], 1024.0)
    } else if quantity.ends_with("Ti") {
        parse_with_suffix(&quantity[..quantity.len()-2], 1024.0 * 1024.0)
    } else if quantity.ends_with('k') {
        // Decimal kilobytes
        parse_with_suffix(&quantity[..quantity.len()-1], 1.0 / 1024.0)
    } else if quantity.ends_with('M') {
        // Decimal megabytes
        parse_with_suffix(&quantity[..quantity.len()-1], 0.953674) // MB to MiB
    } else if quantity.ends_with('G') {
        // Decimal gigabytes  
        parse_with_suffix(&quantity[..quantity.len()-1], 953.674) // GB to MiB
    } else {
        // Assume bytes
        quantity
            .parse::<u64>()
            .map(|v| v / (1024 * 1024))
            .map_err(|e| MetricsError::ParseError(format!("Memory '{}': {}", quantity, e)))
    }
}

fn parse_with_suffix(num_str: &str, multiplier: f64) -> MetricsResult<u64> {
    num_str
        .parse::<f64>()
        .map(|v| (v * multiplier) as u64)
        .map_err(|e| MetricsError::ParseError(format!("'{}': {}", num_str, e)))
}

/// Format millicores back to K8s style
pub fn format_cpu(millicores: u64) -> String {
    if millicores >= 1000 && millicores % 1000 == 0 {
        format!("{}", millicores / 1000)
    } else {
        format!("{}m", millicores)
    }
}

/// Format MiB back to K8s style
pub fn format_memory(mib: u64) -> String {
    if mib >= 1024 && mib % 1024 == 0 {
        format!("{}Gi", mib / 1024)
    } else {
        format!("{}Mi", mib)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu() {
        assert_eq!(parse_cpu_to_millicores("100m").unwrap(), 100);
        assert_eq!(parse_cpu_to_millicores("2").unwrap(), 2000);
        assert_eq!(parse_cpu_to_millicores("0.5").unwrap(), 500);
    }

    #[test]
    fn test_parse_memory() {
        assert_eq!(parse_memory_to_mib("1024Mi").unwrap(), 1024);
        assert_eq!(parse_memory_to_mib("1Gi").unwrap(), 1024);
        assert_eq!(parse_memory_to_mib("1048576Ki").unwrap(), 1024);
    }

    #[test]
    fn test_format_cpu() {
        assert_eq!(format_cpu(100), "100m");
        assert_eq!(format_cpu(2000), "2");
        assert_eq!(format_cpu(1500), "1500m");
    }

    #[test]
    fn test_format_memory() {
        assert_eq!(format_memory(512), "512Mi");
        assert_eq!(format_memory(1024), "1Gi");
        assert_eq!(format_memory(2048), "2Gi");
    }
}
