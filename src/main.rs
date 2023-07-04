mod instructions;

use instructions::instructions;
use instructions::Instruction;
use instructions::Memory;
use instructions::Registers;

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

    let program = [1, 4];

    for i in program {
        (instrs[i].execute)(&mut regs, &mem);
        println!("Executing {}", instrs[i].pneumonic);
    }

    println!("{:?}", regs);
}
