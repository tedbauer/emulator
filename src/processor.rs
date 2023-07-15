use crate::MemoryAccess;

#[derive(Default, Debug)]
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
    pub program_counter: u8,
    pub stack_pointer: u16,
}

#[derive(Debug)]
enum FlagBit {
    Z, /* Zero Flag */
    N, /* Subtract Flag */
    H, /* Half Carry Flag */
    C, /* Carry Flag */
}

impl Registers {
    fn write_flag(&mut self, bit: FlagBit) {}

    fn read_flag(&self, bit: FlagBit) {}
}

#[derive(Default)]
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
                registers.b = registers.a & 0xFF;
                //registers.c = registers.a >> 8;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC BC",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.b += 1;
            }),
        },
        Instruction {
            mnemonic: "INC B",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                registers.b += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC B",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD B,d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "RLCA",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD (a16),SP",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD HL,BC",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,(BC)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC BC,",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC C",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD C,d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "RRCA",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "STOP 0",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD DE,d16",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "LD (DE),A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC DE",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "DEC D",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD D,d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "RLA",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "JR r8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "ADD HL,DE",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,(DE)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
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
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD E, d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "RRA",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "JR NZ,r8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                // TODO: check if z flag is reset
                registers.program_counter += 1;
                registers.program_counter = ((registers.program_counter as i8)
                    + (memory.read_byte(registers.program_counter as u16) as i8))
                    as u8;
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD HL,d16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                registers.h = memory.read_byte(registers.program_counter as u16 + 1) as u8;
                registers.l = memory.read_byte(registers.program_counter as u16 + 2) as u8;
                registers.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "LD (HL+), A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC HL",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "INC H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "DEC H",
            time_increment: TimeIncrement { m: 0, t: 0 },
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
            mnemonic: "JR Z,r8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 2;
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
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 2;
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
                let address = (registers.h as u16) + ((registers.l as u16) << 8);
                memory.write_word(address, registers.a as u16);

                let decremented_value = address - 1;
                registers.h = decremented_value as u8;
                registers.l = (decremented_value & 0x0F) as u8;
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
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
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
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
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
            execute: Box::new(|registers, memory| -> () {}),
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
            execute: Box::new(|registers, memory| -> () {}),
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
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
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
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD A,H",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
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
            execute: Box::new(|registers, memory| -> () {}),
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

                registers.f = 0;
                if (registers.a == 0) {
                    registers.f += (1 << 7);
                }

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
            mnemonic: "POP BC",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "JP NZ,a16",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
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
            mnemonic: "PUSH BC",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "ADD A,d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
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
            mnemonic: "RET",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "JP Z,a16",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CALL Z,a16",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "PREFIX CB",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;

                ((cb_instructions()[memory.read_byte(registers.program_counter as u16) as usize]).execute)(
                    registers, memory,
                )
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
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 3;
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
            mnemonic: "LDH (a8),A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
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
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
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
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
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
            mnemonic: "LD A,(C)",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
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
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
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
                println!("executing BIT 7,H.");
                let is_set = (registers.h & 0b01000000) > 0;
                if is_set {
                    registers.f = registers.f | 0b10000000;
                }
                // TODO: set other flags
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
