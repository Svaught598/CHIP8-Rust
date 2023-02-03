use crate::memory::Memory;

#[derive(Clone, Debug)]
pub struct Processor {
    memory: Memory,
}

impl Processor {
    pub fn make_cpu() -> Self {
        Self {
            memory: Memory::make_memory(),
        }
    }
}