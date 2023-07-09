use std::fs;

pub trait MemoryAccess {
    fn read_byte(&self, addr: u8) -> u8;
    fn read_word(&self, addr: u8) -> u16;
    fn write_byte(&mut self, addr: u8);
    fn write_word(&mut self, addr: u8);
}

#[derive(Debug)]
pub struct Memory {
    pub bios: [u8; 256],
    rom: [u8; 16384],
}

impl Memory {
    pub fn initialize() -> Self {
        let bios = fs::read("roms/bios.rom").unwrap().try_into().unwrap();
        println!("{:?}", bios);
        Self {
            bios,
            rom: [0; 16384],
        }
    }
}

impl MemoryAccess for Memory {
    fn read_byte(&self, addr: u8) -> u8 {
        if usize::from(addr) >= self.rom.len() + self.bios.len() {
            unimplemented!()
        }

        if usize::from(addr) < self.bios.len() {
            return self.bios[addr as usize];
        }

        self.rom[addr as usize]
    }

    fn read_word(&self, addr: u8) -> u16 {
        (self.read_byte(addr) as u16) + ((self.read_byte(addr + 1) as u16) << 8)
    }

    fn write_byte(&mut self, addr: u8) {
        unimplemented!()
    }

    fn write_word(&mut self, addr: u8) {
        unimplemented!()
    }
}
