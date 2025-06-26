use std::fmt;
use std::fs;

pub trait MemoryAccess {
    fn read_byte(&self, addr: u16) -> u8;
    fn read_word(&self, addr: u16) -> u16;
    fn write_byte(&mut self, addr: u16, value: u8);
    fn write_word(&mut self, addr: u16, value: u16);
    fn generate_tileset_rgba(&self, buffer: &mut [u8]);
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
        let bios: [u8; 256] = include_bytes!("../roms/bios.rom").clone();
        let rom: [u8; 16384] = include_bytes!("../roms/kirby_dream_land_game.rom")[0..16384]
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

    fn generate_tileset_rgba(&self, buffer: &mut [u8]) {
        // A simple grayscale palette for the 4 colors of the Game Boy
        const PALETTE: [[u8; 4]; 4] = [
            [255, 255, 255, 255], // White (RGBA)
            [170, 170, 170, 255], // Light Gray
            [85, 85, 85, 255],    // Dark Gray
            [0, 0, 0, 255],       // Black
        ];

        // Use `usize` for all pixel-grid and buffer index calculations.
        // This is the native type for indexing in Rust.
        let output_width_pixels: usize = 128; // 16 tiles * 8 pixels/tile

        // The loop variable `tile_index` is already `usize` by default.
        for tile_index in 0..384 {
            // `usize` math for positioning tiles in the output image
            let tile_grid_x: usize = tile_index % 16;
            let tile_grid_y: usize = tile_index / 16;
            let base_pixel_x: usize = tile_grid_x * 8;
            let base_pixel_y: usize = tile_grid_y * 8;

            // VRAM tile data starts at 0x8000. We calculate the 16-bit Game Boy
            // address separately from our `usize` pixel grid math.
            let tile_addr_start = 0x8000_u16.wrapping_add(tile_index as u16 * 16);

            // The loop variable `row` is `usize`.
            for row in 0..8 {
                let current_pixel_y: usize = base_pixel_y + row;

                // Calculate the 16-bit address for the two bytes making up the pixel row.
                let row_addr = tile_addr_start.wrapping_add(row as u16 * 2);
                let byte1 = self.read_byte(row_addr);
                let byte2 = self.read_byte(row_addr.wrapping_add(1));

                // The loop variable `pixel` is `usize`.
                for pixel in 0..8 {
                    let color_bit_1 = (byte1 >> (7 - pixel)) & 1;
                    let color_bit_2 = (byte2 >> (7 - pixel)) & 1;
                    let color_index = (color_bit_2 << 1) | color_bit_1;

                    let color_rgba = PALETTE[color_index as usize];

                    let current_pixel_x: usize = base_pixel_x + pixel;

                    // **THIS IS THE CRITICAL PART**
                    // All variables in this calculation are now explicitly `usize`.
                    // The result, `buffer_index`, is guaranteed to be `usize`.
                    let buffer_index: usize =
                        (current_pixel_y * output_width_pixels + current_pixel_x) * 4;

                    // This will now compile correctly because `buffer_index` is a `usize`.
                    buffer[buffer_index] = color_rgba[0]; // R
                    buffer[buffer_index + 1] = color_rgba[1]; // G
                    buffer[buffer_index + 2] = color_rgba[2]; // B
                    buffer[buffer_index + 3] = color_rgba[3]; // A
                }
            }
        }
    }

    /*
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
    */
}
