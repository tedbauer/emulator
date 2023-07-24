use crate::processor::TimeIncrement;
use crate::MemoryAccess;

pub struct Gpu {}

impl Gpu {
    pub fn initialize(memory: &mut Box<dyn MemoryAccess>) -> Self {
        Self {}
    }

    pub fn step(&self, time_increment: TimeIncrement) {}
}
