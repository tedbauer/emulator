mod gpu;
mod memory;
mod processor;

use gpu::Gpu;
use memory::Memory;
use memory::MemoryAccess;
use processor::instructions;
use processor::Cpu;
use processor::Instruction;
use std::fs;
use std::fs::File;
use std::io::Write;

fn main() {
    let mut memory = Box::new(Memory::initialize()) as Box<dyn MemoryAccess>;

    let mut gpu = Gpu::initialize(&mut memory);
    let mut cpu = Cpu::initialize(&mut memory, &gpu);

    loop {
        let time_increment = cpu.step();
        gpu.step(time_increment);
    }
}
