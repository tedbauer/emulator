use std::fs;

pub trait MemoryAccess {
    fn read_byte(&self, addr: u8) -> u8;
    fn read_word(&self, addr: u8) -> u16;
    fn write_byte(&mut self, addr: u16, value: u8);
    fn write_word(&mut self, addr: u16, value: u16);
}

#[derive(Debug)]
pub struct Memory {
    pub bios: [u8; 256],
    rom: [u8; 16384],

    // TODO: split up regions
    the_rest: [u8; 48896],
}

impl Memory {
    pub fn initialize() -> Self {
        let bios = fs::read("roms/bios.rom").unwrap().try_into().unwrap();
        println!("{:?}", bios);
        Self {
            bios,
            rom: [0; 16384],
            the_rest: [0; 48896],
        }
    }
}

impl MemoryAccess for Memory {
    fn read_byte(&self, addr: u8) -> u8 {
        if usize::from(addr) < self.bios.len() {
            self.bios[addr as usize]
        } else if usize::from(addr) < self.bios.len() + self.rom.len() {
            self.rom[addr as usize]
        } else {
            self.the_rest[addr as usize - self.rom.len() - self.bios.len()]
        }
    }

    fn read_word(&self, addr: u8) -> u16 {
        (self.read_byte(addr) as u16) + ((self.read_byte(addr + 1) as u16) << 8)
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        if usize::from(addr) < self.bios.len() {
            self.bios[addr as usize] = value;
        } else if usize::from(addr) < self.bios.len() + self.rom.len() {
            self.rom[addr as usize] = value;
        } else {
            self.the_rest[addr as usize - self.rom.len() - self.bios.len()] = value;
        }
    }

    fn write_word(&mut self, addr: u16, value: u16) {
        self.write_byte(addr, value as u8);
        self.write_byte(addr + 1, (value & 0x0F) as u8)
    }
}
