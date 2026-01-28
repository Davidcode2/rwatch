use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Memory {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub available: u64,
}

impl Memory {
    pub fn new(total: u64, used: u64, free: u64, available: u64) -> Self {
        Memory {
            total,
            used,
            free,
            available,
        }
    }

    pub fn memory() -> Memory {
        let memory = Self::get_memory();
        memory
    }

    fn get_memory() -> Memory {
        Self::read_meminfo()
    }

    fn read_meminfo() -> Memory {
        let path = Path::new("/proc/meminfo");
        let file = File::open(&path);
        let reader = io::BufReader::new(file.unwrap());

        let mut total = 0;
        let mut available = 0;

        for line in reader.lines() {
            let line = line.expect("Could not read line");
            if line.starts_with("MemTotal:") {
                total = Self::parse_kb_value(&line);
            }
            if line.starts_with("MemAvailable:") {
                available = Self::parse_kb_value(&line);
            }
        }

        return Memory::new(total, 0, 0, available);
    }

    fn parse_kb_value(line: &String) -> u64 {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(value) = parts[1].parse::<u64>() {
                return value;
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serialization() {
        let response = Memory::memory();
        let json = serde_json::to_string(&response).unwrap();

        // Verify it contains expected fields
        assert!(json.contains("\"available\""));
        assert!(json.contains("\"total\""));
        assert!(json.contains("\"used\""));
        assert!(json.contains("\"free\""));
    }

    #[test]
    fn test_health_response_deserialization() {
        let json = r#"{"total":1,"available":456,"free":1, "used": 1}"#;
        let response: Memory = serde_json::from_str(json).unwrap();

        assert_eq!(response.total, 1);
        assert_eq!(response.available, 456);
        assert_eq!(response.free, 1);
        assert_eq!(response.used, 1);
    }
}
