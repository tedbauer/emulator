mod memory;
mod processor;

use memory::Memory;
use memory::MemoryAccess;
use processor::instructions;
use processor::Instruction;
use processor::Registers;
use std::fs;

fn main() {
    let mem = Box::new(Memory::initialize()) as Box<dyn MemoryAccess>;
    let mut regs = Registers::default();

    let instrs = instructions();

    // for i in program {
    loop {
        println!(
            "Opcode: {} | {} | {:?}",
            mem.read_byte(regs.program_counter),
            instrs[mem.read_byte(regs.program_counter) as usize].mnemonic,
            regs
        );
        (instrs[mem.read_byte(regs.program_counter) as usize].execute)(&mut regs, &mem);
    }
}
