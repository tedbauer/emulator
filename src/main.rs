mod memory;
mod processor;

use memory::Memory;
use memory::MemoryAccess;
use processor::instructions;
use processor::Instruction;
use processor::Registers;
use std::fs;
use std::fs::File;
use std::io::Write;

fn main() {
    let mut mem = Box::new(Memory::initialize()) as Box<dyn MemoryAccess>;
    let mut regs = Registers::default();

    let instrs = instructions();

    // for i in program {
    loop {
        println!(
            "PC: {} | Opcode: {} | {} | {:?}",
            regs.program_counter,
            mem.read_byte(regs.program_counter as u16),
            instrs[mem.read_byte(regs.program_counter as u16) as usize].mnemonic,
            regs
        );
        // println!("{}", regs.program_counter);
        (instrs[mem.read_byte(regs.program_counter as u16) as usize].execute)(&mut regs, &mut mem);
    }
}
