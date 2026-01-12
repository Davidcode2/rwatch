use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Memory {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

impl Memory {
    pub fn new(total: u64, used: u64, free: u64) -> Self {
        Memory { total, used, free }
    }

    pub fn memory() -> Memory {
        // Dummy values for illustration
        let memory = Self::new(8192, 4096, 4096);
        memory
    }
}
