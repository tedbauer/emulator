#[derive(Default, Debug)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub program_counter: u8,
    pub stack_pointer: u16,
}

pub trait Memory {
    fn read_byte(&self, addr: u8) -> u8;
    fn read_word(&self, addr: u8) -> u16;
    fn write_byte(&mut self, addr: u8);
    fn write_word(&mut self, addr: u8);
}

#[derive(Default)]
pub struct TimeIncrement {
    pub m: u8,
    pub t: u8,
}

pub struct Instruction {
    pub mnemonic: &'static str,
    pub time_increment: TimeIncrement,
    pub execute: Box<dyn Fn(&mut Registers, &Box<dyn Memory>) -> ()>,
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction {
            mnemonic: "Unimplemented",
            time_increment: TimeIncrement::default(),
            execute: Box::new(|_registers, _memory| -> () { unimplemented!() }),
        }
    }
}

pub fn instructions() -> [Instruction; 50] {
    [
        Instruction::default(),
        Instruction {
            mnemonic: "LD BC,d16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                registers.b = (memory.read_word(registers.program_counter) & 0xf0) as u8;
                registers.c = (memory.read_word(registers.program_counter) & 0x0f) as u8;
            }),
        },
        Instruction::default(),
        Instruction::default(),
        Instruction {
            mnemonic: "INC BC",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.b += 1;
            }),
        },
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction::default(),
        Instruction {
            mnemonic: "LD SP,d16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                registers.stack_pointer = memory.read_word(registers.program_counter);
                registers.program_counter += 3;
            }),
        },
    ]
}
