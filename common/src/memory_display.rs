use std::{fmt::{Display, Error, Formatter}};

impl Display for MemoryWithUnit  {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{},{} {}", self.amount, self.decimal, self.unit)
    }
}

pub struct MemoryWithUnit {
    pub amount: u64,
    pub decimal: u64,
    pub unit: &'static str, 
}

pub fn as_gb(value: u64) -> MemoryWithUnit {
    MemoryWithUnit {
        amount: value / 1_000_000,
        decimal: (value % 1_000_000) / 10_000, 
        unit: "GB", 
    }
}
