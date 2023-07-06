mod instructions;

use instructions::instructions;
use instructions::Instruction;
use instructions::Memory;
use instructions::Registers;
use std::fs;

struct AMemory {}

impl Memory for AMemory {
    fn write_byte(&mut self, addr: u8) {}

    fn write_word(&mut self, addr: u8) {}

    fn read_byte(&self, addr: u8) -> u8 {
        0
    }

    fn read_word(&self, addr: u8) -> u16 {
        3242
    }
}

fn main() {
    let mem = Box::new(AMemory {}) as Box<dyn Memory>;
    let mut regs = Registers::default();

    let instrs = instructions();

    let bios: Vec<u8> = fs::read("roms/bios.rom").unwrap();
    println!("{:?}", bios);

    // for i in program {
    loop {
        println!(
            "Opcode: {} | {} | {:?}",
            bios[regs.program_counter as usize],
            instrs[bios[regs.program_counter as usize] as usize].mnemonic,
            regs
        );
        (instrs[bios[regs.program_counter as usize] as usize].execute)(&mut regs, &mem);
    }
}
