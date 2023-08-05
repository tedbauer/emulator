use image::ImageFormat;
use std::fmt;
use std::fs;

pub trait MemoryAccess {
    fn read_byte(&self, addr: u16) -> u8;
    fn read_word(&self, addr: u16) -> u16;
    fn write_byte(&mut self, addr: u16, value: u8);
    fn write_word(&mut self, addr: u16, value: u16);
    fn dump_tileset(&self);
}

pub struct Memory {
    pub bios: [u8; 256],
    rom: [u8; 16384],

    // TODO: split up regions
    the_rest: [u8; 49152],

    bios_enabled: bool,
}

impl fmt::Debug for dyn MemoryAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in 0..=65_535 {
            if byte == 0 {
                write!(f, "-- BIOS --\n");
            } else if byte == 256 {
                write!(f, "-- ROM -- \n");
            }
            write!(f, "{:#04x} : {:#02x}\n", byte, self.read_byte(byte));
        }
        write!(f, "Done")
    }
}

impl Memory {
    pub fn initialize() -> Self {
        let bios = fs::read("roms/bios.rom").unwrap().try_into().unwrap();
        let rom = fs::read("roms/kirby_dream_land_game.rom").unwrap()[0..16384]
            .try_into()
            .unwrap();

        Self {
            bios,
            rom,
            the_rest: [0; 49152],

            bios_enabled: true,
        }
    }
}

impl MemoryAccess for Memory {
    fn read_byte(&self, addr: u16) -> u8 {
        if usize::from(addr) < self.bios.len() && self.bios_enabled {
            self.bios[addr as usize]
        } else if usize::from(addr) < self.rom.len() {
            self.rom[addr as usize]
        } else {
            self.the_rest[addr as usize - self.rom.len()]
        }
    }

    fn read_word(&self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) + ((self.read_byte(addr + 1) as u16) << 8)
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        if usize::from(addr) < self.bios.len() {
            self.bios[addr as usize] = value;
        } else if usize::from(addr) < self.bios.len() + self.rom.len() {
            self.rom[addr as usize] = value;
        } else {
            self.the_rest[addr as usize - self.rom.len()] = value;
        }
    }

    fn write_word(&mut self, addr: u16, value: u16) {
        self.write_byte(addr, value as u8);
        self.write_byte(addr + 1, ((value & 0xFF00) >> 8) as u8)
    }

    fn dump_tileset(&self) {
        use image::{Rgb, RgbImage};

        let mut img = RgbImage::new(200, 200);

        for address in (0x8000..0x87FF).step_by(16) {
            for line in 0..8 {
                let line_byte = self.read_byte(address + line * 2);
                let tile_x = (address - 0x8000) / 16;
                for pixel in 0..8 {
                    if ((1 << (7 - pixel)) & line_byte) > 0 {
                        img.put_pixel((tile_x + pixel) as u32, (line as u32), Rgb([0, 0, 0]))
                    } else {
                        img.put_pixel((tile_x + pixel) as u32, (line as u32), Rgb([255, 255, 255]))
                    }
                }
            }
        }

        fs::create_dir("debug").unwrap();
        img.save("debug/tiles.png");
    }
}
