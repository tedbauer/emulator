use crate::Gpu;
use crate::MemoryAccess;
use std::fmt;
use std::fs;

pub struct Cpu {
    registers: Registers,
    instruction_bank: [Instruction; 256],
    cb_instruction_bank: [Instruction; 256],
}

impl Cpu {
    pub fn initialize() -> Self {
        Self {
            registers: Registers::default(),
            instruction_bank: instructions(),
            cb_instruction_bank: cb_instructions(),
        }
    }

    pub fn step<'a>(&mut self, memory: &'a mut Box<dyn MemoryAccess>) -> TimeIncrement {
        // println!(
        //     "PC: {} | Opcode: {} | {} | {:?}",
        //     self.registers.program_counter,
        //     memory.read_byte(self.registers.program_counter as u16),
        //     self.instruction_bank[memory.read_byte(self.registers.program_counter as u16) as usize]
        //         .mnemonic,
        //     self.registers
        // );
        //println!("ff44: {}", memory.read_byte(0xFF44));

        let opcode = memory.read_byte(self.registers.program_counter);
        let instruction = &self.instruction_bank[opcode as usize];
        (instruction.execute)(&mut self.registers, memory);
        instruction.time_increment.clone()
    }
}

#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub g: u8,
    pub h: u8,
    pub l: u8,
    pub program_counter: u16,
    pub stack_pointer: u16,
}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[a: {:#02x}]", self.a);
        write!(f, "[b: {:#02x}]", self.b);
        write!(f, "[c: {:#02x}]", self.c);
        write!(f, "[d: {:#02x}]", self.d);
        write!(f, "[e: {:#02x}]", self.e);
        write!(f, "[f: {:#02x}]", self.f);
        write!(f, "[g: {:#02x}]", self.g);
        write!(f, "[h: {:#02x}]", self.h);
        write!(f, "[l: {:#02x}]", self.l);
        write!(f, "[pc: {:#04x}]", self.program_counter);
        write!(f, "[sp: {:#04x}]", self.stack_pointer);
        write!(f, " | [Z: {}", self.read_flag(FlagBit::Z));
        write!(f, " | N: {}", self.read_flag(FlagBit::N));
        write!(f, " | H: {}", self.read_flag(FlagBit::H));
        write!(f, " | C: {}", self.read_flag(FlagBit::C));
        write!(f, "]")
    }
}

#[derive(Debug)]
enum FlagBit {
    Z, // Zero Flag
    N, // Subtract Flag
    H, // Half Carry Flag
    C, // Carry Flag
}

fn write_bit(original_value: u8, bit: u8, value: bool) -> u8 {
    if value {
        original_value | (1 << bit)
    } else {
        original_value & !(1 << bit)
    }
}

fn stop_and_dump(regs: &mut Registers, mem: &mut Box<dyn MemoryAccess>) {
    let memdump = format!("{:?}", mem);
    fs::write("memdump.txt", memdump);
    println!("{:?}", regs);
    panic!("done");
}

impl Registers {
    fn write_flag(&mut self, bit: FlagBit, value: bool) {
        match bit {
            FlagBit::Z => self.f = write_bit(self.f, 7, value),
            FlagBit::N => self.f = write_bit(self.f, 6, value),
            FlagBit::H => self.f = write_bit(self.f, 5, value),
            FlagBit::C => self.f = write_bit(self.f, 4, value),
        }
    }

    fn read_flag(&self, bit: FlagBit) -> bool {
        match bit {
            FlagBit::Z => self.f & (1 << 7) == 0b10000000,
            FlagBit::N => self.f & (1 << 6) == 0b01000000,
            FlagBit::H => self.f & (1 << 5) == 0b00100000,
            FlagBit::C => self.f & (1 << 4) == 0b00010000,
        }
    }
}

#[derive(Default, Clone)]
pub struct TimeIncrement {
    pub m: u8,
    pub t: u8,
}

pub struct Instruction {
    pub mnemonic: &'static str,
    pub time_increment: TimeIncrement,
    pub execute: Box<dyn Fn(&mut Registers, &mut Box<dyn MemoryAccess>) -> ()>,
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

/// Concatenates together two eight-bit numbers into a sixteen bit number.
fn concatenate(a: u8, b: u8) -> u16 {
    ((a as u16) << 8) + (b as u16)
}

/// Grabs the eight most significant bits of `n`.
fn upper_eight_bits(n: u16) -> u8 {
    (n >> 8) as u8
}

/// Grabs the eight least significant bits of `n`.
fn lower_eight_bits(n: u16) -> u8 {
    n as u8
}

fn read_bit(n: u8, bit: u8) -> bool {
    if (bit > 7) {
        panic!("bit > 7");
    }
    n & (1 << bit) == (1 << bit)
}

pub fn instructions() -> [Instruction; 256] {
    [
        Instruction {
            mnemonic: "NOP",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|_registers, _memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD BC,d16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                registers.b = (memory.read_word(registers.program_counter as u16) & 0xf0) as u8;
                registers.c = (memory.read_word(registers.program_counter as u16) & 0x0f) as u8;
            }),
        },
        Instruction {
            mnemonic: "LD (BC),A",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC BC",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let value = concatenate(registers.b, registers.c) + 1;
                registers.b = upper_eight_bits(value);
                registers.c = lower_eight_bits(value);
                registers.b += 1;
            }),
        },
        Instruction {
            mnemonic: "INC B",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.b += 1;
                registers.write_flag(FlagBit::Z, registers.b == 0);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, true); // TODO: Only if half carry
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC B",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.b -= 1;
                registers.write_flag(FlagBit::Z, registers.b == 0);
                registers.write_flag(FlagBit::N, true);
                // TODO: no borrow from bit 4
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD B,d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.b = memory.read_byte(registers.program_counter + 1);
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "RLCA",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, false);
                registers.write_flag(FlagBit::C, read_bit(registers.a, 7));
                registers.a = registers.a.rotate_left(1);
                registers.write_flag(FlagBit::Z, registers.a == 0);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD (a16),SP",
            time_increment: TimeIncrement { m: 3, t: 20 },
            execute: Box::new(|registers, memory| -> () {
                let address = concatenate(
                    memory.read_byte(registers.program_counter + 1),
                    memory.read_byte(registers.program_counter + 2),
                );
                memory.write_word(address, registers.stack_pointer);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD HL,BC",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let result =
                    concatenate(registers.h, registers.l) + concatenate(registers.b, registers.c);
                registers.h = upper_eight_bits(result);
                registers.l = lower_eight_bits(result);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,(BC)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let address = concatenate(registers.b, registers.c);
                registers.a = memory.read_byte(address);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "INC C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.c += 1;
                registers.write_flag(FlagBit::Z, registers.c == 0);
                registers.write_flag(FlagBit::N, false);
                // TODO: write H if carry from bit 3? what

                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.c -= 1;
                registers.write_flag(FlagBit::Z, registers.c == 0);
                registers.write_flag(FlagBit::N, true);
                // TODO: no borrow from bit 4
                registers.program_counter += 1;
                //stop_and_dump(registers, memory);
            }),
        },
        Instruction {
            mnemonic: "LD C,d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.c = memory.read_byte((registers.program_counter as u16) + 1);
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "RRCA",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "STOP 0",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD DE,d16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                registers.d = memory.read_byte(registers.program_counter as u16 + 2);
                registers.e = memory.read_byte(registers.program_counter as u16 + 1);
                registers.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "LD (DE),A",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC DE",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let value = ((registers.d as u16) << 8) + (registers.e as u16) + 1;
                registers.d = (value >> 8) as u8;
                registers.e = value as u8;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.d = registers.d - 1;
                registers.write_flag(FlagBit::Z, registers.d == 0);
                registers.write_flag(FlagBit::N, true);
                registers.write_flag(FlagBit::H, true); // if no borrow from bit 4
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD D,d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.d = memory.read_byte(registers.program_counter + 1);
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "RLA",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                let carry = registers.read_flag(FlagBit::C);
                registers.write_flag(FlagBit::C, read_bit(registers.a, 7));
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, false);
                registers.a = write_bit(registers.a.rotate_left(1), 0, carry);
                registers.write_flag(FlagBit::Z, registers.a == 0);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "JR r8",
            time_increment: TimeIncrement { m: 2, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter = ((registers.program_counter as i16)
                    + ((memory.read_byte(registers.program_counter + 1) as i8) as i16))
                    as u16
                    + 2;
            }),
        },
        Instruction {
            mnemonic: "ADD HL,DE",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,(DE)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let address = ((registers.d as u16) << 8) + (registers.e as u16);
                registers.a = memory.read_byte(address);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC DE",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "INC E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "DEC E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD E, d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.e = memory.read_byte(registers.program_counter + 1);
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "RRA",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "JR NZ,r8",
            time_increment: TimeIncrement { m: 2, t: 8 }, // todo: 12 if taken
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
                // println!("adding {} to pc", (memory.read_byte(registers.program_counter) as i8));

                if !registers.read_flag(FlagBit::Z) {
                    registers.program_counter = ((registers.program_counter as i16)
                        + ((memory.read_byte(registers.program_counter) as i8) as i16))
                        as u16;
                }

                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD HL,d16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                registers.h = memory.read_byte(registers.program_counter as u16 + 2) as u8;
                registers.l = memory.read_byte(registers.program_counter as u16 + 1) as u8;
                registers.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "LD (HL+), A",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let address = concatenate(registers.h, registers.l);
                memory.write_byte(address, registers.a);
                let incremented_address = address + 1;
                registers.h = upper_eight_bits(incremented_address);
                registers.l = lower_eight_bits(incremented_address);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC HL",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let value = concatenate(registers.h, registers.l);
                let incremented_value = value + 1;
                registers.h = upper_eight_bits(incremented_value);
                registers.l = lower_eight_bits(incremented_value);
                registers.program_counter += 1;
                //stop_and_dump(registers, memory);
            }),
        },
        Instruction {
            mnemonic: "INC H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.h = registers.h + 1;
                registers.write_flag(FlagBit::Z, registers.h == 0);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, false); // if carry from 3
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD H,d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "DAA",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "JR Z,r8 hi",
            time_increment: TimeIncrement { m: 2, t: 8 }, // 12
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
                if registers.read_flag(FlagBit::Z) {
                    registers.program_counter = ((registers.program_counter as i16)
                        + ((memory.read_byte(registers.program_counter) as i8) as i16))
                        as u16;
                }
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD HL,HL",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD A,(HL+)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "DEC HL",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "INC L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "DEC L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD L,d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.l = memory.read_byte(registers.program_counter + 1);
                registers.program_counter += 2;
                //stop_and_dump(registers, memory);
            }),
        },
        Instruction {
            mnemonic: "CPL",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                // TODO!
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "JR NC,r8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD SP,d16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                registers.stack_pointer = memory.read_word(registers.program_counter as u16 + 1);
                registers.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "LD (HL-),A",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let address = ((registers.h as u16) << 8) + (registers.l as u16);
                memory.write_byte(address, registers.a);

                let decremented_value = address - 1;
                registers.h = (decremented_value >> 8) as u8;
                registers.l = decremented_value as u8;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC SP",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "INC (HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "DEC (HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD (HL),d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SCF",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "JR C,r8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADD HL,SP",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD A,(HL-)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "DEC SP",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "INC A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "DEC A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.a -= 1;
                registers.write_flag(FlagBit::Z, registers.a == 0);
                registers.write_flag(FlagBit::N, true);
                // TODO: no borrow from bit 4
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.a = memory.read_byte((registers.program_counter as u16) + 1);
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "CCF",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD B,B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD B,C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD B,D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD B,E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD B,H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD B,L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD B,(HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD B,A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD C,B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD C,C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD C,D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD C,E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD C,H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD C,L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD C,(HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD C,A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.c = registers.a;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD D,B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD D,C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD D,D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD D,E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD D,H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD D,L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD D,(HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD D,A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.d = registers.a;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD E,B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD E,C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD E,D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD E,E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD E,H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD E,L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD E,(HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD E,A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD H,B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD H,C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD H,D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD H,E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD H,H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD H,L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD H,(HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD H,A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.h = registers.a;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD L,B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD L,C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD L,D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD L,E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD L,H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD L,L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD L,(HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD L,A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD (HL),B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD (HL),C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD (HL),D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD (HL),E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD (HL),H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD (HL),L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "HALT",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD (HL),A",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let address = registers.l as u16 + (((registers.h as u16) << 8) as u16);
                memory.write_byte(address, registers.a);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD A,C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD A,D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD A,E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.a = registers.e;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.a = registers.h;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD A,(HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD A,A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADD A,B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADD A,C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADD A,D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADD A,E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADD A,H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADD A,L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADD A,(HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADD A,A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADC A,B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADC A,C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADC A,D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADC A,E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADC A,H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADC A,L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADC A,(HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADC A,A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SUB B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.a = registers.a - registers.b;
                registers.write_flag(FlagBit::Z, registers.b == 0);
                registers.write_flag(FlagBit::N, true);
                registers.write_flag(FlagBit::H, true); // if no carry from 4
                registers.write_flag(FlagBit::C, true); // if no borrow
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SUB C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SUB D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SUB E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SUB H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SUB L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SUB (HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SUB A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SBC A,B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SBC A,C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SBC A,D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SBC A,E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SBC A,H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SBC A,L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SBC A,(HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SBC A,A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "AND B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "AND C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "AND D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "AND E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "AND H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "AND L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "AND (HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "AND A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "XOR B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "XOR C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "XOR D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "XOR E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "XOR H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "XOR L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "XOR (HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "XOR A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.a = registers.a ^ registers.a;

                // TODO: it seems like we are supposed to write this flag?
                // registers.write_flag(FlagBit::Z, true);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, false);
                registers.write_flag(FlagBit::C, false);

                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "OR B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "OR C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "OR D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "OR E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "OR H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "OR L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "OR (HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "OR A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CP B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CP C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CP D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CP E",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CP H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CP L",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CP (HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RET NZ",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "fixme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "POP BC",
            time_increment: TimeIncrement { m: 1, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                let value = memory.read_word(registers.stack_pointer + 2);
                registers.b = upper_eight_bits(value);
                registers.c = lower_eight_bits(value);
                registers.stack_pointer += 2;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "JP a16",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CALL NZ,a16",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "fix this",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "PUSH BC",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|registers, memory| -> () {
                let value = ((registers.b as u16) << 8) + (registers.c as u16);
                memory.write_word(registers.stack_pointer, value);
                registers.stack_pointer -= 2;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RST 00H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RET Z",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "fixme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RET",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|registers, memory| -> () {
                registers.stack_pointer += 2;
                let address = memory.read_word(registers.stack_pointer);
                registers.program_counter = address;
            }),
        },
        Instruction {
            mnemonic: "CALL Z,a16",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "PREFIX CB",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;

                ((cb_instructions()[memory.read_byte(registers.program_counter as u16) as usize])
                    .execute)(registers, memory)
            }),
        },
        Instruction {
            mnemonic: "ADC A,d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "CALL a16",
            time_increment: TimeIncrement { m: 3, t: 24 },
            execute: Box::new(|registers, memory| -> () {
                memory.write_word(registers.stack_pointer, registers.program_counter + 3);
                registers.stack_pointer -= 2;
                registers.program_counter = memory.read_byte(registers.program_counter + 1) as u16;
            }),
        },
        Instruction {
            mnemonic: "RST 08H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RET NC",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "POP DE",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "JP NC,a16",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CALL NC,a16",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "PUSH DE",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SUB d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RST 10H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RET C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RETI",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "JP C,a16",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CALL C,a16",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "SBC A,d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RST 18H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ahhh idk",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            // i think this is the problem
            mnemonic: "LDH (a8),A",
            time_increment: TimeIncrement { m: 2, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                let address = (memory.read_byte((registers.program_counter as u16) + 1) as u16)
                    + (0xFF00 as u16);
                memory.write_byte(address, registers.a);
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "POP HL",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD (C),A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let address = (registers.c as u16) + (0xFF00 as u16);
                memory.write_byte(address as u16, registers.a);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "PUSH HL",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "AND d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RST 20H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADD SP,r8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "JP (HL)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "nothing??!?!",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "nothing!!!",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD (a16),A",
            time_increment: TimeIncrement { m: 3, t: 16 },
            execute: Box::new(|registers, memory| -> () {
                let address = concatenate(
                    memory.read_byte(registers.program_counter + 2),
                    memory.read_byte(registers.program_counter + 1),
                );
                memory.write_byte(address, registers.a);
                registers.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "XOR d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RST 28H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LDH A,(a8)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "POP AF",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD A,($FF00+n)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                let address = 0xFF00 + (memory.read_byte(registers.program_counter+1) as u16);
                registers.a = memory.read_byte(address);
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "DI",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "PUSH AF",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () { unimplemented!() }),
        },
        Instruction {
            mnemonic: "OR d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RST 30H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD HL,SP+r8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD SP,HL",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD A,(a16)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "EI",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "hmmmm",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RST 38H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CP d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let value = memory.read_byte(registers.program_counter + 1);
                registers.write_flag(FlagBit::Z, registers.a == value);
                // println!("value: {}, a: {}, equal? {}", value, registers.a, value == registers.a);
                registers.write_flag(FlagBit::N, true);
                // registers.write_flag(FlagBit::H, true); todo: figure this one out
                if registers.a < value {
                    registers.write_flag(FlagBit::C, true);
                }
                registers.program_counter += 2;
            }),
        },
        Instruction::default(),
    ]
}

pub fn cb_instructions() -> [Instruction; 256] {
    [
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RL C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.write_flag(FlagBit::C, read_bit(registers.c, 7));
                registers.c = registers.c.rotate_left(1);
                registers.write_flag(FlagBit::Z, registers.c == 0);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, false);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "193250215",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                println!("yoyoyoy");
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 7,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                if registers.h & 0b01000000 == 0b01000000 {
                    registers.write_flag(FlagBit::Z, true)
                }
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, true);

                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "12348123",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                println!("hihihihi");
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "replaceme",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
    ]
}
