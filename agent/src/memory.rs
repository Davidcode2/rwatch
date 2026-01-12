use axum::Json;
use rwatch_common::memory::Memory;

pub struct MemoryHandler {}

impl MemoryHandler {
    pub async fn memory_handler() -> Json<Memory> {
        let memory = Memory::memory();
        Json(memory)
    }
}
