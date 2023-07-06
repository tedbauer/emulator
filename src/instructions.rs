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

pub fn instructions() -> [Instruction; 227] {
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
                registers.b = (memory.read_word(registers.program_counter) & 0xf0) as u8;
                registers.c = (memory.read_word(registers.program_counter) & 0x0f) as u8;
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
                registers.b += 1;
            }),
        },
        Instruction {
            mnemonic: "LD B,d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
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
                registers.program_counter += 1;
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
                registers.program_counter += 1;
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
            execute: Box::new(|registers, memory| -> () {}),
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
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "LD HL,d16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                registers.h = (memory.read_word(registers.program_counter) & 0xFF) as u8;
                registers.l = (memory.read_word(registers.program_counter) >> 8) as u8;
                registers.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "LD (HL+), A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "INC HL",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
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
            mnemonic: "39",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "40",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "41",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "42",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "43",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "44",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "45",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "45 actually how",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "lies",
            time_increment: TimeIncrement { m: 1, t: 8 },
            execute: Box::new(|registers, memory| -> () {
                // TODO!
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "LD SP,d16",
            time_increment: TimeIncrement { m: 3, t: 12 },
            execute: Box::new(|registers, memory| -> () {
                registers.stack_pointer = memory.read_word(registers.program_counter);
                registers.program_counter += 3;
            }),
        },
        Instruction {
            mnemonic: "LD (HL-),A",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "47",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "48",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "49",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "50",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "51",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "52",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "53",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "54",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "55",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "56",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "57",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "LD A,d8",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "59",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "60",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "61",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "62",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "63",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "64",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "65",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "66",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "67",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "68",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "69",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "70",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "71",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "72",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "73",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "74",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "75",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "76",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "77",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "78",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "79",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "80",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "81",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "82",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "83",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "84",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "85",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "86",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "87",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "88",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "89",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "90",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "91",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "92",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "93",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "94",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "95",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "96",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "97",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "98",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "99",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "100",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "101",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "102",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "103",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "104",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "105",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "106",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "107",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "108",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "109",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "110",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "111",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "112",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "113",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "114",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "115",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "116",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "117",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "118",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "119",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "120",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "121",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "122",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "123",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "124",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "125",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "126",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "127",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "128",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "129",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "130",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "131",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "132",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "133",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "134",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "135",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "136",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "137",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "138",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "139",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "140",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "141",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "142",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "143",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "144",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "145",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "146",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "147",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "148",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "149",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "150",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "151",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "152",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "153",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "154",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "155",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "156",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "157",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "158",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "159",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "160",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "161",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "162",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "163",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "164",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "165",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "166",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "167",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "168",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "169",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "171",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "XOR A",
            time_increment: TimeIncrement { m: 1, t: 4 },
            execute: Box::new(|registers, memory| -> () {
                registers.a = registers.a ^ registers.a;

                registers.program_counter += 1;
            }),
        },
        Instruction {
            mnemonic: "172",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12521",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12522",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12523",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12524",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12525",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12526",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12527",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12528",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12529",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12530",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12531",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12532",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12533",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12534",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12535",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12536",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12537",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12538",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12539",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12540",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12541",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12542",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12543",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12544",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12545",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12546",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "CB",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {
                registers.program_counter += 2;
            }),
        },
        Instruction {
            mnemonic: "12548",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12549",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12550",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12551",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12552",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12553",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12554",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12555",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12556",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12557",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12558",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12559",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12560",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12561",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12562",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12563",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12564",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12565",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12566",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12567",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12568",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
        Instruction {
            mnemonic: "12569",
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
            mnemonic: "12571",
            time_increment: TimeIncrement { m: 0, t: 0 },
            execute: Box::new(|registers, memory| -> () {}),
        },
    ]
}
    



