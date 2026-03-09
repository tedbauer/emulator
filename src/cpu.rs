#![allow(unused_variables)]
use crate::MemoryAccess;
use std::fmt;

pub struct Cpu {
    registers: Registers,
    instruction_bank: [Instruction; 256],
    #[allow(dead_code)] // used by WASM frontend
    cb_instruction_bank: [Instruction; 256],
    pub ime: bool,
    pub halted: bool,
}

impl Cpu {
    pub fn initialize() -> Self {
        Self {
            registers: Registers::default(),
            instruction_bank: instructions(),
            cb_instruction_bank: cb_instructions(),
            ime: false,
            halted: false,
        }
    }

    pub fn step<'a>(&mut self, memory: &'a mut Box<dyn MemoryAccess>) -> (TimeIncrement, String) {
        // If halted, spin in place consuming minimal cycles until an interrupt fires
        if self.halted {
            return (TimeIncrement { m: 1, t: 4 }, "HALT (waiting)".to_string());
        }

        let opcode = memory.read_byte(self.registers.program_counter);
        let instruction = &self.instruction_bank[opcode as usize];

        (instruction.execute)(&mut self.registers, memory);

        // Post-execute: handle instructions that affect cpu-level state
        match opcode {
            0x76 => {
                self.halted = true;
            } // HALT: don't advance PC, set halted
            0xF3 => {
                self.ime = false;
            } // DI
            0xFB => {
                self.ime = true;
            } // EI
            0xD9 => {
                self.ime = true;
            } // RETI (already popped PC in opcode body)
            _ => {}
        }

        let log_entry = format!(
            "0x{:04X}: {:<12} (0x{:02X})",
            self.registers.program_counter, instruction.mnemonic, opcode
        );
        (instruction.time_increment.clone(), log_entry)
    }

    /// Check and dispatch pending interrupts.
    /// Returns true if an interrupt was serviced.
    pub fn handle_interrupts(&mut self, memory: &mut Box<dyn MemoryAccess>) -> bool {
        let if_reg = memory.read_byte(0xFF0F); // interrupt flags
        let ie_reg = memory.read_byte(0xFFFF); // interrupt enable
        let pending = if_reg & ie_reg & 0x1F;
        if pending == 0 {
            return false;
        }

        // Any pending interrupt wakes HALT regardless of IME
        let was_halted = self.halted;
        if self.halted {
            self.halted = false;
        }

        if !self.ime {
            return false;
        }

        // Find highest-priority interrupt (bit 0 = VBlank, bit 1 = LCD, bit 2 = Timer, ...)
        let bit = pending.trailing_zeros() as u8;
        let vector: u16 = 0x0040 + (bit as u16) * 8;

        // Acknowledge: clear the bit in IF
        memory.write_byte(0xFF0F, if_reg & !(1 << bit));

        // Disable IME, push PC, jump to vector
        // When waking from HALT, push PC+1 so RETI returns PAST the HALT instruction
        self.ime = false;
        let pc = if was_halted {
            self.registers.program_counter.wrapping_add(1)
        } else {
            self.registers.program_counter
        };
        self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(2);
        let sp = self.registers.stack_pointer;
        memory.write_byte(sp + 1, (pc >> 8) as u8);
        memory.write_byte(sp, pc as u8);
        self.registers.program_counter = vector;
        true
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
        write!(f, "[a: {:#02x}]", self.a)?;
        write!(f, "[b: {:#02x}]", self.b)?;
        write!(f, "[c: {:#02x}]", self.c)?;
        write!(f, "[d: {:#02x}]", self.d)?;
        write!(f, "[e: {:#02x}]", self.e)?;
        write!(f, "[f: {:#02x}]", self.f)?;
        write!(f, "[g: {:#02x}]", self.g)?;
        write!(f, "[h: {:#02x}]", self.h)?;
        write!(f, "[l: {:#02x}]", self.l)?;
        write!(f, "[pc: {:#04x}]", self.program_counter)?;
        write!(f, "[sp: {:#04x}]", self.stack_pointer)?;
        write!(f, " | [Z: {}", self.read_flag(FlagBit::Z))?;
        write!(f, " | N: {}", self.read_flag(FlagBit::N))?;
        write!(f, " | H: {}", self.read_flag(FlagBit::H))?;
        write!(f, " | C: {}", self.read_flag(FlagBit::C))?;
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
    #[allow(dead_code)] // used by WASM timing
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
    if bit > 7 {
        panic!("bit > 7");
    }
    n & (1 << bit) == (1 << bit)
}

pub fn instructions() -> [Instruction; 256] {
    [
        Instruction {
            mnemonic: "NOP",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, _memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD BC,d16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                registers.c = memory.read_byte(registers.program_counter + 1);
                registers.b = memory.read_byte(registers.program_counter + 2);
                registers.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "LD (BC),A",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let address = concatenate(registers.b, registers.c);
                memory.write_byte(address, registers.a);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC BC",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let value = concatenate(registers.b, registers.c).wrapping_add(1);
                registers.b = upper_eight_bits(value);
                registers.c = lower_eight_bits(value);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC B",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.write_flag(FlagBit::H, (registers.b & 0xF) == 0xF);
                registers.b = registers.b.wrapping_add(1);
                registers.write_flag(FlagBit::Z, registers.b == 0);
                registers.write_flag(FlagBit::N, false);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC B",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.write_flag(FlagBit::H, (registers.b & 0xF) == 0x0);
                registers.b = registers.b.wrapping_sub(1);
                registers.write_flag(FlagBit::Z, registers.b == 0);
                registers.write_flag(FlagBit::N, true);
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
            execute: Box::new(|r, _| {
                let hl = concatenate(r.h, r.l) as u32;
                let bc = concatenate(r.b, r.c) as u32;
                let result = hl + bc;
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, ((hl ^ bc ^ result) & 0x1000) != 0);
                r.write_flag(FlagBit::C, result > 0xFFFF);
                let result16 = result as u16;
                r.h = (result16 >> 8) as u8;
                r.l = result16 as u8;
                r.program_counter += 1;
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
            mnemonic: "DEC BC",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, _memory| -> () {
                let value = concatenate(registers.b, registers.c).wrapping_sub(1);
                registers.b = upper_eight_bits(value);
                registers.c = lower_eight_bits(value);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.write_flag(FlagBit::H, (registers.c & 0xF) == 0xF);
                registers.c = registers.c.wrapping_add(1);
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
                registers.write_flag(FlagBit::H, (registers.c & 0xF) == 0x0);
                registers.c = registers.c.wrapping_sub(1);
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
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, _memory| -> () {
                let bit0 = registers.a & 1;
                registers.write_flag(FlagBit::C, bit0 != 0);
                registers.a = (registers.a >> 1) | (bit0 << 7);
                registers.write_flag(FlagBit::Z, false);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, false);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "STOP 0",
            time_increment: TimeIncrement { m: 2, t: 4 },
            execute: Box::new(|registers, _memory| -> () {
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "LD DE,d16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                registers.e = memory.read_byte(registers.program_counter + 1);
                registers.d = memory.read_byte(registers.program_counter + 2);
                registers.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "LD (DE),A",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let address = concatenate(registers.d, registers.e);
                memory.write_byte(address, registers.a);
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
                registers.d = registers.d.wrapping_add(1);
                registers.write_flag(FlagBit::Z, registers.d == 0);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, registers.d & 0x0F == 0);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.write_flag(FlagBit::H, (registers.d & 0xF) == 0);
                registers.d = registers.d.wrapping_sub(1);
                registers.write_flag(FlagBit::Z, registers.d == 0);
                registers.write_flag(FlagBit::N, true);
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
            execute: Box::new(|r, _| {
                let hl = concatenate(r.h, r.l) as u32;
                let de = concatenate(r.d, r.e) as u32;
                let result = hl + de;
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, ((hl ^ de ^ result) & 0x1000) != 0);
                r.write_flag(FlagBit::C, result > 0xFFFF);
                let result16 = result as u16;
                r.h = (result16 >> 8) as u8;
                r.l = result16 as u8;
                r.program_counter += 1;
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
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let value = concatenate(registers.d, registers.e).wrapping_sub(1);
                registers.d = upper_eight_bits(value);
                registers.e = lower_eight_bits(value);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.e = registers.e.wrapping_add(1);
                registers.write_flag(FlagBit::Z, registers.e == 0);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, registers.e & 0x0F == 0);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.e = registers.e.wrapping_sub(1);
                registers.write_flag(FlagBit::Z, registers.e == 0);
                registers.write_flag(FlagBit::N, true);
                registers.write_flag(FlagBit::H, registers.e & 0x0F == 0x0F);
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
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, _memory| -> () {
                let old_c = registers.read_flag(FlagBit::C) as u8;
                let bit0 = registers.a & 1;
                registers.write_flag(FlagBit::C, bit0 != 0);
                registers.a = (registers.a >> 1) | (old_c << 7);
                registers.write_flag(FlagBit::Z, false);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, false);
                registers.program_counter += 1;
            }),
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
            execute: Box::new(|registers, memory| -> () {
                registers.h = registers.h.wrapping_sub(1);
                registers.write_flag(FlagBit::Z, registers.h == 0);
                registers.write_flag(FlagBit::N, true);
                registers.write_flag(FlagBit::H, registers.h & 0x0F == 0x0F);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD H,d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.h = memory.read_byte(registers.program_counter + 1);
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "DAA",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                // Basic DAA implementation
                let mut a = registers.a as u16;
                if !registers.read_flag(FlagBit::N) {
                    if registers.read_flag(FlagBit::H) || (a & 0x0F) > 9 {
                        a = a.wrapping_add(0x06);
                    }
                    if registers.read_flag(FlagBit::C) || a > 0x9F {
                        a = a.wrapping_add(0x60);
                    }
                } else {
                    if registers.read_flag(FlagBit::H) {
                        a = a.wrapping_sub(6);
                    }
                    if registers.read_flag(FlagBit::C) {
                        a = a.wrapping_sub(0x60);
                    }
                }
                registers.write_flag(FlagBit::H, false);
                if a > 0xFF {
                    registers.write_flag(FlagBit::C, true);
                }
                registers.a = a as u8;
                registers.write_flag(FlagBit::Z, registers.a == 0);
                registers.program_counter += 1;
            }),
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
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let hl = concatenate(registers.h, registers.l);
                let result = hl.wrapping_add(hl);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, (hl & 0x0FFF) + (hl & 0x0FFF) > 0x0FFF);
                registers.write_flag(FlagBit::C, (hl as u32) + (hl as u32) > 0xFFFF);
                registers.h = upper_eight_bits(result);
                registers.l = lower_eight_bits(result);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,(HL+)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let address = concatenate(registers.h, registers.l);
                registers.a = memory.read_byte(address);
                let incremented = address.wrapping_add(1);
                registers.h = upper_eight_bits(incremented);
                registers.l = lower_eight_bits(incremented);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC HL",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let value = concatenate(registers.h, registers.l).wrapping_sub(1);
                registers.h = upper_eight_bits(value);
                registers.l = lower_eight_bits(value);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.l = registers.l.wrapping_add(1);
                registers.write_flag(FlagBit::Z, registers.l == 0);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, registers.l & 0x0F == 0);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.l = registers.l.wrapping_sub(1);
                registers.write_flag(FlagBit::Z, registers.l == 0);
                registers.write_flag(FlagBit::N, true);
                registers.write_flag(FlagBit::H, registers.l & 0x0F == 0x0F);
                registers.program_counter += 1;
            }),
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
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.a = !registers.a;
                registers.write_flag(FlagBit::N, true);
                registers.write_flag(FlagBit::H, true);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "JR NC,r8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
                if !registers.read_flag(FlagBit::C) {
                    registers.program_counter = ((registers.program_counter as i16)
                        + (memory.read_byte(registers.program_counter) as i8 as i16))
                        as u16;
                }
                registers.program_counter += 1;
            }),
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

                let decremented_value = address.wrapping_sub(1);
                registers.h = (decremented_value >> 8) as u8;
                registers.l = decremented_value as u8;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC SP",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.stack_pointer = registers.stack_pointer.wrapping_add(1);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC (HL)",
            time_increment: TimeIncrement { m: 1, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                let addr = concatenate(registers.h, registers.l);
                let val = memory.read_byte(addr).wrapping_add(1);
                memory.write_byte(addr, val);
                registers.write_flag(FlagBit::Z, val == 0);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, val & 0x0F == 0);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC (HL)",
            time_increment: TimeIncrement { m: 1, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                let addr = concatenate(registers.h, registers.l);
                let val = memory.read_byte(addr).wrapping_sub(1);
                memory.write_byte(addr, val);
                registers.write_flag(FlagBit::Z, val == 0);
                registers.write_flag(FlagBit::N, true);
                registers.write_flag(FlagBit::H, val & 0x0F == 0x0F);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD (HL),d8",
            time_increment: TimeIncrement { m: 2, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                let val = memory.read_byte(registers.program_counter + 1);
                let addr = concatenate(registers.h, registers.l);
                memory.write_byte(addr, val);
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "SCF",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, false);
                registers.write_flag(FlagBit::C, true);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "JR C,r8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
                if registers.read_flag(FlagBit::C) {
                    registers.program_counter = ((registers.program_counter as i16)
                        + (memory.read_byte(registers.program_counter) as i8 as i16))
                        as u16;
                }
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD HL,SP",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, _memory| -> () {
                let hl = concatenate(registers.h, registers.l) as u32;
                let sp = registers.stack_pointer as u32;
                let result = hl.wrapping_add(sp);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, (hl ^ sp ^ result) & 0x1000 != 0);
                registers.write_flag(FlagBit::C, result > 0xFFFF);
                registers.h = upper_eight_bits(result as u16);
                registers.l = lower_eight_bits(result as u16);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,(HL-)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                let address = concatenate(registers.h, registers.l);
                registers.a = memory.read_byte(address);
                let decremented = address.wrapping_sub(1);
                registers.h = upper_eight_bits(decremented);
                registers.l = lower_eight_bits(decremented);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC SP",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.stack_pointer = registers.stack_pointer.wrapping_sub(1);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.a = registers.a.wrapping_add(1);
                registers.write_flag(FlagBit::Z, registers.a == 0);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, registers.a & 0x0F == 0);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.write_flag(FlagBit::H, (registers.a & 0xF) == 0x0);
                registers.a = registers.a.wrapping_sub(1);
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
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                let c = registers.read_flag(FlagBit::C);
                registers.write_flag(FlagBit::N, false);
                registers.write_flag(FlagBit::H, false);
                registers.write_flag(FlagBit::C, !c);
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD B,B",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD B,C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.b = r.c;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD B,D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.b = r.d;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD B,E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.b = r.e;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD B,H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.b = r.h;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD B,L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.b = r.l;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD B,(HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                r.b = m.read_byte(a);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD B,A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.b = r.a;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD C,B",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.c = r.b;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD C,C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD C,D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.c = r.d;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD C,E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.c = r.e;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD C,H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.c = r.h;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD C,L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.c = r.l;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD C,(HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                r.c = m.read_byte(a);
                r.program_counter += 1;
            }),
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
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.d = r.b;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD D,C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.d = r.c;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD D,D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD D,E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.d = r.e;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD D,H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.d = r.h;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD D,L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.d = r.l;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD D,(HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                r.d = m.read_byte(a);
                r.program_counter += 1;
            }),
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
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.e = r.b;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD E,C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.e = r.c;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD E,D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.e = r.d;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD E,E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD E,H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.e = r.h;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD E,L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.e = r.l;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD E,(HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                r.e = m.read_byte(a);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD E,A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, _memory| -> () {
                registers.e = registers.a;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD H,B",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.h = r.b;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD H,C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.h = r.c;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD H,D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.h = r.d;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD H,E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.h = r.e;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD H,H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD H,L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.h = r.l;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD H,(HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                r.h = m.read_byte(a);
                r.program_counter += 1;
            }),
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
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.l = r.b;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD L,C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.l = r.c;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD L,D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.l = r.d;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD L,E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.l = r.e;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD L,H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.l = r.h;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD L,L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD L,(HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                r.l = m.read_byte(a);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD L,A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.l = r.a;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD (HL),B",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                m.write_byte(a, r.b);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD (HL),C",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                m.write_byte(a, r.c);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD (HL),D",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                m.write_byte(a, r.d);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD (HL),E",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                m.write_byte(a, r.e);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD (HL),H",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                m.write_byte(a, r.h);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD (HL),L",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                m.write_byte(a, r.l);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "HALT",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| { /* halt: stay at current PC */ }),
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
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a = r.b;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a = r.c;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a = r.d;
                r.program_counter += 1;
            }),
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
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a = r.l;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,(HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                r.a = m.read_byte(a);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD A,B",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_add(r.b);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (r.b & 0xF) > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (r.b as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD A,C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_add(r.c);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (r.c & 0xF) > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (r.c as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD A,D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_add(r.d);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (r.d & 0xF) > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (r.d as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD A,E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_add(r.e);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (r.e & 0xF) > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (r.e as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD A,H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_add(r.h);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (r.h & 0xF) > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (r.h as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD A,L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_add(r.l);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (r.l & 0xF) > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (r.l as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD A,(HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(concatenate(r.h, r.l));
                let v = r.a.wrapping_add(n);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (n & 0xF) > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (n as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD A,A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_add(r.a);
                r.write_flag(FlagBit::H, (r.a & 0xF) * 2 > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) * 2 > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADC A,B",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_add(r.b).wrapping_add(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (r.b & 0xF) + c > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (r.b as u16) + (c as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADC A,C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_add(r.c).wrapping_add(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (r.c & 0xF) + c > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (r.c as u16) + (c as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADC A,D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_add(r.d).wrapping_add(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (r.d & 0xF) + c > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (r.d as u16) + (c as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADC A,E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_add(r.e).wrapping_add(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (r.e & 0xF) + c > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (r.e as u16) + (c as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADC A,H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_add(r.h).wrapping_add(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (r.h & 0xF) + c > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (r.h as u16) + (c as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADC A,L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_add(r.l).wrapping_add(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (r.l & 0xF) + c > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (r.l as u16) + (c as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADC A,(HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(concatenate(r.h, r.l));
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_add(n).wrapping_add(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (n & 0xF) + c > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (n as u16) + (c as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADC A,A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_add(r.a).wrapping_add(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) * 2 + c > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) * 2 + (c as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SUB B",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_sub(r.b);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.b & 0xF));
                r.write_flag(FlagBit::C, r.a < r.b);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SUB C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_sub(r.c);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.c & 0xF));
                r.write_flag(FlagBit::C, r.a < r.c);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SUB D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_sub(r.d);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.d & 0xF));
                r.write_flag(FlagBit::C, r.a < r.d);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SUB E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_sub(r.e);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.e & 0xF));
                r.write_flag(FlagBit::C, r.a < r.e);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SUB H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_sub(r.h);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.h & 0xF));
                r.write_flag(FlagBit::C, r.a < r.h);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SUB L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_sub(r.l);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.l & 0xF));
                r.write_flag(FlagBit::C, r.a < r.l);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SUB (HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(concatenate(r.h, r.l));
                let v = r.a.wrapping_sub(n);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (n & 0xF));
                r.write_flag(FlagBit::C, r.a < n);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SUB A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a = 0;
                r.write_flag(FlagBit::Z, true);
                r.write_flag(FlagBit::N, true);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SBC A,B",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_sub(r.b).wrapping_sub(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.b & 0xF) + c);
                r.write_flag(FlagBit::C, (r.a as u16) < (r.b as u16) + (c as u16));
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SBC A,C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_sub(r.c).wrapping_sub(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.c & 0xF) + c);
                r.write_flag(FlagBit::C, (r.a as u16) < (r.c as u16) + (c as u16));
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SBC A,D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_sub(r.d).wrapping_sub(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.d & 0xF) + c);
                r.write_flag(FlagBit::C, (r.a as u16) < (r.d as u16) + (c as u16));
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SBC A,E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_sub(r.e).wrapping_sub(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.e & 0xF) + c);
                r.write_flag(FlagBit::C, (r.a as u16) < (r.e as u16) + (c as u16));
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SBC A,H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_sub(r.h).wrapping_sub(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.h & 0xF) + c);
                r.write_flag(FlagBit::C, (r.a as u16) < (r.h as u16) + (c as u16));
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SBC A,L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_sub(r.l).wrapping_sub(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.l & 0xF) + c);
                r.write_flag(FlagBit::C, (r.a as u16) < (r.l as u16) + (c as u16));
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SBC A,(HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(concatenate(r.h, r.l));
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_sub(n).wrapping_sub(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (n & 0xF) + c);
                r.write_flag(FlagBit::C, (r.a as u16) < (n as u16) + (c as u16));
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SBC A,A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_sub(r.a).wrapping_sub(c);
                r.write_flag(FlagBit::H, c > 0);
                r.write_flag(FlagBit::C, c > 0);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "AND B",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a &= r.b;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "AND C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a &= r.c;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "AND D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a &= r.d;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "AND E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a &= r.e;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "AND H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a &= r.h;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "AND L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a &= r.l;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "AND (HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(concatenate(r.h, r.l));
                r.a &= n;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "AND A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "XOR B",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a ^= r.b;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "XOR C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a ^= r.c;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "XOR D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a ^= r.d;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "XOR E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a ^= r.e;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "XOR H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a ^= r.h;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "XOR L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a ^= r.l;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "XOR (HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(concatenate(r.h, r.l));
                r.a ^= n;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
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
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a |= r.b;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "OR C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a |= r.c;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "OR D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a |= r.d;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "OR E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a |= r.e;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "OR H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a |= r.h;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "OR L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.a |= r.l;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "OR (HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(concatenate(r.h, r.l));
                r.a |= n;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "OR A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "CP B",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_sub(r.b);
                r.write_flag(FlagBit::Z, v == 0);
                r.write_flag(FlagBit::N, true);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.b & 0xF));
                r.write_flag(FlagBit::C, r.a < r.b);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "CP C",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_sub(r.c);
                r.write_flag(FlagBit::Z, v == 0);
                r.write_flag(FlagBit::N, true);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.c & 0xF));
                r.write_flag(FlagBit::C, r.a < r.c);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "CP D",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_sub(r.d);
                r.write_flag(FlagBit::Z, v == 0);
                r.write_flag(FlagBit::N, true);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.d & 0xF));
                r.write_flag(FlagBit::C, r.a < r.d);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "CP E",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_sub(r.e);
                r.write_flag(FlagBit::Z, v == 0);
                r.write_flag(FlagBit::N, true);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.e & 0xF));
                r.write_flag(FlagBit::C, r.a < r.e);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "CP H",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_sub(r.h);
                r.write_flag(FlagBit::Z, v == 0);
                r.write_flag(FlagBit::N, true);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.h & 0xF));
                r.write_flag(FlagBit::C, r.a < r.h);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "CP L",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                let v = r.a.wrapping_sub(r.l);
                r.write_flag(FlagBit::Z, v == 0);
                r.write_flag(FlagBit::N, true);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (r.l & 0xF));
                r.write_flag(FlagBit::C, r.a < r.l);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "CP (HL)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(concatenate(r.h, r.l));
                let v = r.a.wrapping_sub(n);
                r.write_flag(FlagBit::Z, v == 0);
                r.write_flag(FlagBit::N, true);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (n & 0xF));
                r.write_flag(FlagBit::C, r.a < n);
                r.program_counter += 1;
            }),
        },
        // CP A
        Instruction {
            mnemonic: "CP A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, true);
                r.write_flag(FlagBit::N, true);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        // RET NZ
        Instruction {
            mnemonic: "RET NZ",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                if !r.read_flag(FlagBit::Z) {
                    r.program_counter = m.read_word(r.stack_pointer);
                    r.stack_pointer = r.stack_pointer.wrapping_add(2);
                } else {
                    r.program_counter += 1;
                }
            }),
        },
        // POP BC
        Instruction {
            mnemonic: "POP BC",
            time_increment: TimeIncrement { m: 1, t: 12 },
            execute: Box::new(|r, m| {
                let v = m.read_word(r.stack_pointer);
                r.stack_pointer = r.stack_pointer.wrapping_add(2);
                r.b = upper_eight_bits(v);
                r.c = lower_eight_bits(v);
                r.program_counter += 1;
            }),
        },
        // JP NZ,a16
        Instruction {
            mnemonic: "JP NZ,a16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|r, m| {
                if !r.read_flag(FlagBit::Z) {
                    r.program_counter = m.read_word(r.program_counter + 1);
                } else {
                    r.program_counter += 3;
                }
            }),
        },
        // JP a16
        Instruction {
            mnemonic: "JP a16",
            time_increment: TimeIncrement { m: 3, t: 16 },
            execute: Box::new(|r, m| {
                r.program_counter = m.read_word(r.program_counter + 1);
            }),
        },
        // CALL NZ,a16
        Instruction {
            mnemonic: "CALL NZ,a16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|r, m| {
                if !r.read_flag(FlagBit::Z) {
                    r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                    m.write_word(r.stack_pointer, r.program_counter + 3);
                    r.program_counter = m.read_word(r.program_counter + 1);
                } else {
                    r.program_counter += 3;
                }
            }),
        },
        // PUSH BC
        Instruction {
            mnemonic: "PUSH BC",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                let v = concatenate(r.b, r.c);
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, v);
                r.program_counter += 1;
            }),
        },
        // ADD A,d8
        Instruction {
            mnemonic: "ADD A,d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(r.program_counter + 1);
                let v = r.a.wrapping_add(n);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (n & 0xF) > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (n as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 2;
            }),
        },
        // RST 00H
        Instruction {
            mnemonic: "RST 00H",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, r.program_counter + 1);
                r.program_counter = 0x0000;
            }),
        },
        // 0xC8 = RET Z
        Instruction {
            mnemonic: "RET Z",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                if r.read_flag(FlagBit::Z) {
                    r.program_counter = m.read_word(r.stack_pointer);
                    r.stack_pointer = r.stack_pointer.wrapping_add(2);
                } else {
                    r.program_counter += 1;
                }
            }),
        },
        // 0xC9 = RET (unconditional)
        Instruction {
            mnemonic: "RET",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                r.program_counter = m.read_word(r.stack_pointer);
                r.stack_pointer = r.stack_pointer.wrapping_add(2);
            }),
        },
        // 0xCA = JP Z,a16
        Instruction {
            mnemonic: "JP Z,a16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|r, m| {
                if r.read_flag(FlagBit::Z) {
                    r.program_counter = m.read_word(r.program_counter + 1);
                } else {
                    r.program_counter += 3;
                }
            }),
        },
        // 0xCB = PREFIX CB
        Instruction {
            mnemonic: "PREFIX CB",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
                ((cb_instructions()[memory.read_byte(registers.program_counter as u16) as usize])
                    .execute)(registers, memory)
            }),
        },
        // 0xCC = CALL Z,a16
        Instruction {
            mnemonic: "CALL Z,a16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|r, m| {
                if r.read_flag(FlagBit::Z) {
                    r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                    m.write_word(r.stack_pointer, r.program_counter + 3);
                    r.program_counter = m.read_word(r.program_counter + 1);
                } else {
                    r.program_counter += 3;
                }
            }),
        },
        // 0xCD = CALL a16
        Instruction {
            mnemonic: "CALL a16",
            time_increment: TimeIncrement { m: 3, t: 24 },
            execute: Box::new(|r, m| {
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, r.program_counter + 3);
                r.program_counter = m.read_word(r.program_counter + 1);
            }),
        },
        // 0xCE = ADC A,d8
        Instruction {
            mnemonic: "ADC A,d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(r.program_counter + 1);
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_add(n).wrapping_add(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) + (n & 0xF) + c > 0xF);
                r.write_flag(FlagBit::C, (r.a as u16) + (n as u16) + (c as u16) > 0xFF);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 2;
            }),
        },
        // 0xCF = RST 08H
        Instruction {
            mnemonic: "RST 08H",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, r.program_counter + 1);
                r.program_counter = 0x0008;
            }),
        },
        Instruction {
            mnemonic: "RET NC",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                if !r.read_flag(FlagBit::C) {
                    r.program_counter = m.read_word(r.stack_pointer);
                    r.stack_pointer = r.stack_pointer.wrapping_add(2);
                } else {
                    r.program_counter += 1;
                }
            }),
        },
        Instruction {
            mnemonic: "POP DE",
            time_increment: TimeIncrement { m: 1, t: 12 },
            execute: Box::new(|r, m| {
                let v = m.read_word(r.stack_pointer);
                r.stack_pointer = r.stack_pointer.wrapping_add(2);
                r.d = upper_eight_bits(v);
                r.e = lower_eight_bits(v);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "JP NC,a16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|r, m| {
                if !r.read_flag(FlagBit::C) {
                    r.program_counter = m.read_word(r.program_counter + 1);
                } else {
                    r.program_counter += 3;
                }
            }),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "CALL NC,a16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|r, m| {
                if !r.read_flag(FlagBit::C) {
                    r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                    m.write_word(r.stack_pointer, r.program_counter + 3);
                    r.program_counter = m.read_word(r.program_counter + 1);
                } else {
                    r.program_counter += 3;
                }
            }),
        },
        Instruction {
            mnemonic: "PUSH DE",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                let v = concatenate(r.d, r.e);
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, v);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SUB d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(r.program_counter + 1);
                let v = r.a.wrapping_sub(n);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (n & 0xF));
                r.write_flag(FlagBit::C, r.a < n);
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "RST 10H",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, r.program_counter + 1);
                r.program_counter = 0x0010;
            }),
        },
        Instruction {
            mnemonic: "RET C",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                if r.read_flag(FlagBit::C) {
                    r.program_counter = m.read_word(r.stack_pointer);
                    r.stack_pointer = r.stack_pointer.wrapping_add(2);
                } else {
                    r.program_counter += 1;
                }
            }),
        },
        Instruction {
            mnemonic: "RETI",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                r.program_counter = m.read_word(r.stack_pointer);
                r.stack_pointer = r.stack_pointer.wrapping_add(2); /* TODO: enable interrupts */
            }),
        },
        Instruction {
            mnemonic: "JP C,a16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|r, m| {
                if r.read_flag(FlagBit::C) {
                    r.program_counter = m.read_word(r.program_counter + 1);
                } else {
                    r.program_counter += 3;
                }
            }),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "CALL C,a16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|r, m| {
                if r.read_flag(FlagBit::C) {
                    r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                    m.write_word(r.stack_pointer, r.program_counter + 3);
                    r.program_counter = m.read_word(r.program_counter + 1);
                } else {
                    r.program_counter += 3;
                }
            }),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SBC A,d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(r.program_counter + 1);
                let c = r.read_flag(FlagBit::C) as u8;
                let v = r.a.wrapping_sub(n).wrapping_sub(c);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (n & 0xF) + c);
                r.write_flag(FlagBit::C, (r.a as u16) < (n as u16) + (c as u16));
                r.a = v;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, true);
                r.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "RST 18H",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, r.program_counter + 1);
                r.program_counter = 0x0018;
            }),
        },
        // 0xE0-0xEF
        Instruction {
            mnemonic: "LDH (a8),A",
            time_increment: TimeIncrement { m: 2, t: 12 },
            execute: Box::new(|r, m| {
                let addr = 0xFF00u16 + m.read_byte(r.program_counter + 1) as u16;
                m.write_byte(addr, r.a);
                r.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "POP HL",
            time_increment: TimeIncrement { m: 1, t: 12 },
            execute: Box::new(|r, m| {
                let v = m.read_word(r.stack_pointer);
                r.stack_pointer = r.stack_pointer.wrapping_add(2);
                r.h = upper_eight_bits(v);
                r.l = lower_eight_bits(v);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD (C),A",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let addr = 0xFF00u16 + r.c as u16;
                m.write_byte(addr, r.a);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "PUSH HL",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                let v = concatenate(r.h, r.l);
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, v);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "AND d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(r.program_counter + 1);
                r.a &= n;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "RST 20H",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, r.program_counter + 1);
                r.program_counter = 0x0020;
            }),
        },
        Instruction {
            mnemonic: "ADD SP,r8",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(r.program_counter + 1) as i8 as i16;
                r.stack_pointer = ((r.stack_pointer as i16).wrapping_add(n)) as u16;
                r.write_flag(FlagBit::Z, false);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "JP (HL)",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter = concatenate(r.h, r.l);
            }),
        },
        Instruction {
            mnemonic: "LD (a16),A",
            time_increment: TimeIncrement { m: 3, t: 16 },
            execute: Box::new(|r, m| {
                let addr = m.read_word(r.program_counter + 1);
                m.write_byte(addr, r.a);
                r.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "XOR d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(r.program_counter + 1);
                r.a ^= n;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "RST 28H",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, r.program_counter + 1);
                r.program_counter = 0x0028;
            }),
        },
        // 0xF0-0xFF
        Instruction {
            mnemonic: "LDH A,(a8)",
            time_increment: TimeIncrement { m: 2, t: 12 },
            execute: Box::new(|r, m| {
                let addr = 0xFF00u16 + m.read_byte(r.program_counter + 1) as u16;
                r.a = m.read_byte(addr);
                r.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "POP AF",
            time_increment: TimeIncrement { m: 1, t: 12 },
            execute: Box::new(|r, m| {
                let v = m.read_word(r.stack_pointer);
                r.stack_pointer = r.stack_pointer.wrapping_add(2);
                r.a = upper_eight_bits(v);
                r.f = lower_eight_bits(v) & 0xF0;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,(C)",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, m| {
                let addr = 0xFF00u16 + r.c as u16;
                r.a = m.read_byte(addr);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DI",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                /* disable interrupts - stub */
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "PUSH AF",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                let v = concatenate(r.a, r.f);
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, v);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "OR d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(r.program_counter + 1);
                r.a |= n;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "RST 30H",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, r.program_counter + 1);
                r.program_counter = 0x0030;
            }),
        },
        Instruction {
            mnemonic: "LD HL,SP+r8",
            time_increment: TimeIncrement { m: 2, t: 12 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(r.program_counter + 1) as i8 as i16;
                let result = ((r.stack_pointer as i16).wrapping_add(n)) as u16;
                r.h = upper_eight_bits(result);
                r.l = lower_eight_bits(result);
                r.write_flag(FlagBit::Z, false);
                r.write_flag(FlagBit::N, false);
                r.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "LD SP,HL",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|r, _| {
                r.stack_pointer = concatenate(r.h, r.l);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,(a16)",
            time_increment: TimeIncrement { m: 3, t: 16 },
            execute: Box::new(|r, m| {
                let addr = m.read_word(r.program_counter + 1);
                r.a = m.read_byte(addr);
                r.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "EI",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                /* enable interrupts - stub */
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "nothing",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|r, _| {
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "CP d8",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, m| {
                let n = m.read_byte(r.program_counter + 1);
                r.write_flag(FlagBit::Z, r.a == n);
                r.write_flag(FlagBit::N, true);
                r.write_flag(FlagBit::H, (r.a & 0xF) < (n & 0xF));
                r.write_flag(FlagBit::C, r.a < n);
                r.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "RST 38H",
            time_increment: TimeIncrement { m: 1, t: 16 },
            execute: Box::new(|r, m| {
                r.stack_pointer = r.stack_pointer.wrapping_sub(2);
                m.write_word(r.stack_pointer, r.program_counter + 1);
                r.program_counter = 0x0038;
            }),
        },
    ]
}

pub fn cb_instructions() -> [Instruction; 256] {
    [
        // ── RLC r (rotate left circular, Cn ← b7) 0x00-0x07
        Instruction {
            mnemonic: "RLC B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.b >> 7;
                r.write_flag(FlagBit::C, c != 0);
                r.b = (r.b << 1) | c;
                r.write_flag(FlagBit::Z, r.b == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RLC C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.c >> 7;
                r.write_flag(FlagBit::C, c != 0);
                r.c = (r.c << 1) | c;
                r.write_flag(FlagBit::Z, r.c == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RLC D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.d >> 7;
                r.write_flag(FlagBit::C, c != 0);
                r.d = (r.d << 1) | c;
                r.write_flag(FlagBit::Z, r.d == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RLC E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.e >> 7;
                r.write_flag(FlagBit::C, c != 0);
                r.e = (r.e << 1) | c;
                r.write_flag(FlagBit::Z, r.e == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RLC H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.h >> 7;
                r.write_flag(FlagBit::C, c != 0);
                r.h = (r.h << 1) | c;
                r.write_flag(FlagBit::Z, r.h == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RLC L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.l >> 7;
                r.write_flag(FlagBit::C, c != 0);
                r.l = (r.l << 1) | c;
                r.write_flag(FlagBit::Z, r.l == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RLC (HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                let c = v >> 7;
                let nv = (v << 1) | c;
                m.write_byte(a, nv);
                r.write_flag(FlagBit::C, c != 0);
                r.write_flag(FlagBit::Z, nv == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RLC A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.a >> 7;
                r.write_flag(FlagBit::C, c != 0);
                r.a = (r.a << 1) | c;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        // ── RRC r (rotate right circular, C0 ← b0) 0x08-0x0F
        Instruction {
            mnemonic: "RRC B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.b & 1;
                r.write_flag(FlagBit::C, c != 0);
                r.b = (r.b >> 1) | (c << 7);
                r.write_flag(FlagBit::Z, r.b == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RRC C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.c & 1;
                r.write_flag(FlagBit::C, c != 0);
                r.c = (r.c >> 1) | (c << 7);
                r.write_flag(FlagBit::Z, r.c == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RRC D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.d & 1;
                r.write_flag(FlagBit::C, c != 0);
                r.d = (r.d >> 1) | (c << 7);
                r.write_flag(FlagBit::Z, r.d == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RRC E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.e & 1;
                r.write_flag(FlagBit::C, c != 0);
                r.e = (r.e >> 1) | (c << 7);
                r.write_flag(FlagBit::Z, r.e == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RRC H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.h & 1;
                r.write_flag(FlagBit::C, c != 0);
                r.h = (r.h >> 1) | (c << 7);
                r.write_flag(FlagBit::Z, r.h == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RRC L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.l & 1;
                r.write_flag(FlagBit::C, c != 0);
                r.l = (r.l >> 1) | (c << 7);
                r.write_flag(FlagBit::Z, r.l == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RRC (HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                let c = v & 1;
                let nv = (v >> 1) | (c << 7);
                m.write_byte(a, nv);
                r.write_flag(FlagBit::C, c != 0);
                r.write_flag(FlagBit::Z, nv == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RRC A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let c = r.a & 1;
                r.write_flag(FlagBit::C, c != 0);
                r.a = (r.a >> 1) | (c << 7);
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        // ── RL r (rotate left through carry) 0x10-0x17
        Instruction {
            mnemonic: "RL B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.b >> 7 != 0);
                r.b = (r.b << 1) | oc;
                r.write_flag(FlagBit::Z, r.b == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RL C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.c >> 7 != 0);
                r.c = (r.c << 1) | oc;
                r.write_flag(FlagBit::Z, r.c == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RL D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.d >> 7 != 0);
                r.d = (r.d << 1) | oc;
                r.write_flag(FlagBit::Z, r.d == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RL E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.e >> 7 != 0);
                r.e = (r.e << 1) | oc;
                r.write_flag(FlagBit::Z, r.e == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RL H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.h >> 7 != 0);
                r.h = (r.h << 1) | oc;
                r.write_flag(FlagBit::Z, r.h == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RL L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.l >> 7 != 0);
                r.l = (r.l << 1) | oc;
                r.write_flag(FlagBit::Z, r.l == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RL (HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                let oc = r.read_flag(FlagBit::C) as u8;
                let nv = (v << 1) | oc;
                m.write_byte(a, nv);
                r.write_flag(FlagBit::C, v >> 7 != 0);
                r.write_flag(FlagBit::Z, nv == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RL A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.a >> 7 != 0);
                r.a = (r.a << 1) | oc;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        // ── RR r (rotate right through carry) 0x18-0x1F
        Instruction {
            mnemonic: "RR B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.b & 1 != 0);
                r.b = (r.b >> 1) | (oc << 7);
                r.write_flag(FlagBit::Z, r.b == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RR C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.c & 1 != 0);
                r.c = (r.c >> 1) | (oc << 7);
                r.write_flag(FlagBit::Z, r.c == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RR D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.d & 1 != 0);
                r.d = (r.d >> 1) | (oc << 7);
                r.write_flag(FlagBit::Z, r.d == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RR E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.e & 1 != 0);
                r.e = (r.e >> 1) | (oc << 7);
                r.write_flag(FlagBit::Z, r.e == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RR H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.h & 1 != 0);
                r.h = (r.h >> 1) | (oc << 7);
                r.write_flag(FlagBit::Z, r.h == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RR L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.l & 1 != 0);
                r.l = (r.l >> 1) | (oc << 7);
                r.write_flag(FlagBit::Z, r.l == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RR (HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                let oc = r.read_flag(FlagBit::C) as u8;
                let nv = (v >> 1) | (oc << 7);
                m.write_byte(a, nv);
                r.write_flag(FlagBit::C, v & 1 != 0);
                r.write_flag(FlagBit::Z, nv == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RR A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                let oc = r.read_flag(FlagBit::C) as u8;
                r.write_flag(FlagBit::C, r.a & 1 != 0);
                r.a = (r.a >> 1) | (oc << 7);
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        // ── SLA r (shift left arithmetic) 0x20-0x27
        Instruction {
            mnemonic: "SLA B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.b >> 7 != 0);
                r.b <<= 1;
                r.write_flag(FlagBit::Z, r.b == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SLA C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.c >> 7 != 0);
                r.c <<= 1;
                r.write_flag(FlagBit::Z, r.c == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SLA D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.d >> 7 != 0);
                r.d <<= 1;
                r.write_flag(FlagBit::Z, r.d == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SLA E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.e >> 7 != 0);
                r.e <<= 1;
                r.write_flag(FlagBit::Z, r.e == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SLA H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.h >> 7 != 0);
                r.h <<= 1;
                r.write_flag(FlagBit::Z, r.h == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SLA L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.l >> 7 != 0);
                r.l <<= 1;
                r.write_flag(FlagBit::Z, r.l == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SLA (HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                let nv = v << 1;
                m.write_byte(a, nv);
                r.write_flag(FlagBit::C, v >> 7 != 0);
                r.write_flag(FlagBit::Z, nv == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SLA A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.a >> 7 != 0);
                r.a <<= 1;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        // ── SRA r (shift right arithmetic, b7 preserved) 0x28-0x2F
        Instruction {
            mnemonic: "SRA B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.b & 1 != 0);
                r.b = (r.b >> 1) | (r.b & 0x80);
                r.write_flag(FlagBit::Z, r.b == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRA C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.c & 1 != 0);
                r.c = (r.c >> 1) | (r.c & 0x80);
                r.write_flag(FlagBit::Z, r.c == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRA D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.d & 1 != 0);
                r.d = (r.d >> 1) | (r.d & 0x80);
                r.write_flag(FlagBit::Z, r.d == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRA E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.e & 1 != 0);
                r.e = (r.e >> 1) | (r.e & 0x80);
                r.write_flag(FlagBit::Z, r.e == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRA H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.h & 1 != 0);
                r.h = (r.h >> 1) | (r.h & 0x80);
                r.write_flag(FlagBit::Z, r.h == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRA L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.l & 1 != 0);
                r.l = (r.l >> 1) | (r.l & 0x80);
                r.write_flag(FlagBit::Z, r.l == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRA (HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                let nv = (v >> 1) | (v & 0x80);
                m.write_byte(a, nv);
                r.write_flag(FlagBit::C, v & 1 != 0);
                r.write_flag(FlagBit::Z, nv == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRA A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.a & 1 != 0);
                r.a = (r.a >> 1) | (r.a & 0x80);
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        // ── SWAP r (swap high/low nibbles) 0x30-0x37
        Instruction {
            mnemonic: "SWAP B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b = (r.b >> 4) | (r.b << 4);
                r.write_flag(FlagBit::Z, r.b == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SWAP C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c = (r.c >> 4) | (r.c << 4);
                r.write_flag(FlagBit::Z, r.c == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SWAP D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d = (r.d >> 4) | (r.d << 4);
                r.write_flag(FlagBit::Z, r.d == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SWAP E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e = (r.e >> 4) | (r.e << 4);
                r.write_flag(FlagBit::Z, r.e == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SWAP H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h = (r.h >> 4) | (r.h << 4);
                r.write_flag(FlagBit::Z, r.h == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SWAP L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l = (r.l >> 4) | (r.l << 4);
                r.write_flag(FlagBit::Z, r.l == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SWAP (HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                let nv = (v >> 4) | (v << 4);
                m.write_byte(a, nv);
                r.write_flag(FlagBit::Z, nv == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SWAP A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a = (r.a >> 4) | (r.a << 4);
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.write_flag(FlagBit::C, false);
                r.program_counter += 1;
            }),
        },
        // ── SRL r (shift right logical, b7=0) 0x38-0x3F
        Instruction {
            mnemonic: "SRL B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.b & 1 != 0);
                r.b >>= 1;
                r.write_flag(FlagBit::Z, r.b == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRL C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.c & 1 != 0);
                r.c >>= 1;
                r.write_flag(FlagBit::Z, r.c == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRL D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.d & 1 != 0);
                r.d >>= 1;
                r.write_flag(FlagBit::Z, r.d == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRL E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.e & 1 != 0);
                r.e >>= 1;
                r.write_flag(FlagBit::Z, r.e == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRL H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.h & 1 != 0);
                r.h >>= 1;
                r.write_flag(FlagBit::Z, r.h == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRL L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.l & 1 != 0);
                r.l >>= 1;
                r.write_flag(FlagBit::Z, r.l == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRL (HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                let nv = v >> 1;
                m.write_byte(a, nv);
                r.write_flag(FlagBit::C, v & 1 != 0);
                r.write_flag(FlagBit::Z, nv == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SRL A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::C, r.a & 1 != 0);
                r.a >>= 1;
                r.write_flag(FlagBit::Z, r.a == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, false);
                r.program_counter += 1;
            }),
        },
        // BIT 0-7 (0x40-0x7F)
        Instruction {
            mnemonic: "BIT 0,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.b & 0x01) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 0,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.c & 0x01) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 0,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.d & 0x01) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 0,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.e & 0x01) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 0,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.h & 0x01) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 0,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.l & 0x01) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 0,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                r.write_flag(FlagBit::Z, (v & 0x01) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 0,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.a & 0x01) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 1,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.b & 0x02) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 1,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.c & 0x02) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 1,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.d & 0x02) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 1,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.e & 0x02) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 1,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.h & 0x02) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 1,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.l & 0x02) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 1,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                r.write_flag(FlagBit::Z, (v & 0x02) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 1,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.a & 0x02) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 2,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.b & 0x04) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 2,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.c & 0x04) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 2,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.d & 0x04) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 2,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.e & 0x04) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 2,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.h & 0x04) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 2,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.l & 0x04) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 2,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                r.write_flag(FlagBit::Z, (v & 0x04) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 2,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.a & 0x04) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 3,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.b & 0x08) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 3,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.c & 0x08) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 3,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.d & 0x08) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 3,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.e & 0x08) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 3,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.h & 0x08) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 3,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.l & 0x08) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 3,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                r.write_flag(FlagBit::Z, (v & 0x08) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 3,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.a & 0x08) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 4,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.b & 0x10) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 4,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.c & 0x10) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 4,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.d & 0x10) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 4,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.e & 0x10) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 4,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.h & 0x10) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 4,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.l & 0x10) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 4,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                r.write_flag(FlagBit::Z, (v & 0x10) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 4,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.a & 0x10) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 5,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.b & 0x20) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 5,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.c & 0x20) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 5,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.d & 0x20) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 5,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.e & 0x20) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 5,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.h & 0x20) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 5,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.l & 0x20) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 5,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                r.write_flag(FlagBit::Z, (v & 0x20) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 5,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.a & 0x20) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 6,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.b & 0x40) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 6,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.c & 0x40) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 6,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.d & 0x40) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 6,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.e & 0x40) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 6,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.h & 0x40) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 6,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.l & 0x40) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 6,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                r.write_flag(FlagBit::Z, (v & 0x40) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 6,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.a & 0x40) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 7,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.b & 0x80) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 7,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.c & 0x80) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 7,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.d & 0x80) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 7,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.e & 0x80) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 7,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.h & 0x80) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 7,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.l & 0x80) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 7,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                r.write_flag(FlagBit::Z, (v & 0x80) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "BIT 7,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.write_flag(FlagBit::Z, (r.a & 0x80) == 0);
                r.write_flag(FlagBit::N, false);
                r.write_flag(FlagBit::H, true);
                r.program_counter += 1;
            }),
        },
        // RES 0-7 (0x80-0xBF)
        Instruction {
            mnemonic: "RES 0,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b &= 0xFE;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 0,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c &= 0xFE;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 0,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d &= 0xFE;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 0,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e &= 0xFE;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 0,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h &= 0xFE;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 0,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l &= 0xFE;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 0,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v & 0xFE);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 0,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a &= 0xFE;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 1,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b &= 0xFD;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 1,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c &= 0xFD;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 1,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d &= 0xFD;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 1,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e &= 0xFD;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 1,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h &= 0xFD;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 1,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l &= 0xFD;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 1,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v & 0xFD);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 1,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a &= 0xFD;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 2,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b &= 0xFB;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 2,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c &= 0xFB;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 2,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d &= 0xFB;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 2,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e &= 0xFB;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 2,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h &= 0xFB;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 2,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l &= 0xFB;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 2,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v & 0xFB);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 2,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a &= 0xFB;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 3,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b &= 0xF7;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 3,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c &= 0xF7;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 3,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d &= 0xF7;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 3,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e &= 0xF7;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 3,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h &= 0xF7;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 3,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l &= 0xF7;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 3,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v & 0xF7);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 3,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a &= 0xF7;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 4,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b &= 0xEF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 4,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c &= 0xEF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 4,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d &= 0xEF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 4,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e &= 0xEF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 4,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h &= 0xEF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 4,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l &= 0xEF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 4,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v & 0xEF);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 4,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a &= 0xEF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 5,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b &= 0xDF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 5,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c &= 0xDF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 5,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d &= 0xDF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 5,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e &= 0xDF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 5,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h &= 0xDF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 5,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l &= 0xDF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 5,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v & 0xDF);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 5,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a &= 0xDF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 6,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b &= 0xBF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 6,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c &= 0xBF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 6,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d &= 0xBF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 6,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e &= 0xBF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 6,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h &= 0xBF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 6,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l &= 0xBF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 6,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v & 0xBF);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 6,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a &= 0xBF;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 7,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b &= 0x7F;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 7,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c &= 0x7F;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 7,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d &= 0x7F;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 7,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e &= 0x7F;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 7,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h &= 0x7F;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 7,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l &= 0x7F;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 7,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v & 0x7F);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "RES 7,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a &= 0x7F;
                r.program_counter += 1;
            }),
        },
        // SET 0-7 (0xC0-0xFF)
        Instruction {
            mnemonic: "SET 0,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b |= 0x01;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 0,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c |= 0x01;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 0,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d |= 0x01;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 0,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e |= 0x01;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 0,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h |= 0x01;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 0,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l |= 0x01;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 0,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v | 0x01);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 0,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a |= 0x01;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 1,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b |= 0x02;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 1,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c |= 0x02;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 1,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d |= 0x02;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 1,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e |= 0x02;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 1,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h |= 0x02;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 1,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l |= 0x02;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 1,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v | 0x02);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 1,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a |= 0x02;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 2,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b |= 0x04;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 2,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c |= 0x04;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 2,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d |= 0x04;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 2,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e |= 0x04;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 2,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h |= 0x04;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 2,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l |= 0x04;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 2,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v | 0x04);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 2,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a |= 0x04;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 3,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b |= 0x08;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 3,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c |= 0x08;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 3,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d |= 0x08;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 3,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e |= 0x08;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 3,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h |= 0x08;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 3,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l |= 0x08;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 3,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v | 0x08);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 3,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a |= 0x08;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 4,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b |= 0x10;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 4,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c |= 0x10;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 4,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d |= 0x10;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 4,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e |= 0x10;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 4,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h |= 0x10;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 4,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l |= 0x10;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 4,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v | 0x10);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 4,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a |= 0x10;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 5,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b |= 0x20;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 5,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c |= 0x20;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 5,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d |= 0x20;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 5,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e |= 0x20;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 5,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h |= 0x20;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 5,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l |= 0x20;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 5,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v | 0x20);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 5,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a |= 0x20;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 6,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b |= 0x40;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 6,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c |= 0x40;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 6,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d |= 0x40;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 6,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e |= 0x40;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 6,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h |= 0x40;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 6,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l |= 0x40;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 6,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v | 0x40);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 6,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a |= 0x40;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 7,B",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.b |= 0x80;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 7,C",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.c |= 0x80;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 7,D",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.d |= 0x80;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 7,E",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.e |= 0x80;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 7,H",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.h |= 0x80;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 7,L",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.l |= 0x80;
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 7,(HL)",
            time_increment: TimeIncrement { m: 2, t: 16 },
            execute: Box::new(|r, m| {
                let a = concatenate(r.h, r.l);
                let v = m.read_byte(a);
                m.write_byte(a, v | 0x80);
                r.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "SET 7,A",
            time_increment: TimeIncrement { m: 2, t: 8 },
            execute: Box::new(|r, _| {
                r.a |= 0x80;
                r.program_counter += 1;
            }),
        },
    ]
}
