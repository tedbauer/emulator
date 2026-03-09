#![allow(dead_code)] // some methods are WASM-only APIs
use crate::apu::Apu;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::VecDeque;
use std::fmt;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};

pub trait MemoryAccess {
    fn read_byte(&self, addr: u16) -> u8;
    fn read_word(&self, addr: u16) -> u16;
    fn write_byte(&mut self, addr: u16, value: u8);
    fn write_word(&mut self, addr: u16, value: u16);
    fn generate_tileset_rgba(&self, buffer: &mut [u8]);
    fn generate_memory_rgba(&self, buffer: &mut [u8]);
    /// Update joypad state. buttons/dpad: bit=0 means pressed (active-low).
    fn set_joypad(&mut self, buttons: u8, dpad: u8);
    /// Tick APU by `cycles` T-cycles; returns a stereo sample when one is ready.
    fn tick_apu_sample(&mut self, cycles: u32) -> Option<(i16, i16)>;
    /// Tick APU by `cycles` T-cycles, pushing any generated samples into the queue.
    #[cfg(not(target_arch = "wasm32"))]
    fn tick_apu_into_queue(&mut self, cycles: u32, queue: &Arc<Mutex<VecDeque<i16>>>);
}

pub struct Memory {
    pub bios: [u8; 256],
    rom: Vec<u8>,
    the_rest: [u8; 49152],
    bios_enabled: bool,
    // MBC1 bank switching
    mbc_type: u8,
    rom_bank: usize,
    // Joypad state: 0=pressed, 1=released (active low)
    pub joypad_buttons: u8,
    pub joypad_dpad: u8,
    joypad_select: u8,
    pub apu: Apu,
}

impl fmt::Debug for dyn MemoryAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in 0..=65_535u32 {
            if byte == 0 {
                write!(f, "-- BIOS --\n")?;
            } else if byte == 256 {
                write!(f, "-- ROM -- \n")?;
            }
            write!(f, "{:#04x} : {:#02x}\n", byte, self.read_byte(byte as u16))?;
        }
        write!(f, "Done")
    }
}

impl Memory {
    pub fn initialize(rom_path: &str) -> Self {
        let bios: [u8; 256] = include_bytes!("../bios/bios.rom").clone();
        let rom: Vec<u8> = std::fs::read(rom_path)
            .unwrap_or_else(|e| panic!("Failed to read ROM '{}': {}", rom_path, e));
        let mbc_type = if rom.len() > 0x148 { rom[0x147] } else { 0 };
        Self {
            bios,
            mbc_type,
            rom_bank: 1,
            rom,
            the_rest: [0; 49152],
            bios_enabled: true,
            joypad_buttons: 0xFF,
            joypad_dpad: 0xFF,
            joypad_select: 0x30,
            apu: Apu::new(),
        }
    }

    /// Initialize with ROM data provided at runtime (used by WASM frontend).
    pub fn initialize_with_rom(rom_data: Vec<u8>) -> Self {
        let bios: [u8; 256] = include_bytes!("../bios/bios.rom").clone();
        let mbc_type = if rom_data.len() > 0x148 {
            rom_data[0x147]
        } else {
            0
        };
        Self {
            bios,
            mbc_type,
            rom_bank: 1,
            rom: rom_data,
            the_rest: [0; 49152],
            bios_enabled: true,
            joypad_buttons: 0xFF,
            joypad_dpad: 0xFF,
            joypad_select: 0x30,
            apu: Apu::new(),
        }
    }

    /// Advance the APU by `cycles` T-cycles; returns a stereo sample when one is ready.
    pub fn tick_apu(&mut self, cycles: u32) -> Option<(i16, i16)> {
        self.apu.tick(cycles)
    }
}

impl MemoryAccess for Memory {
    fn read_byte(&self, addr: u16) -> u8 {
        if addr == 0xFF00 {
            // Joypad: P15(bit5)=0 selects buttons row, P14(bit4)=0 selects d-pad row
            let select = self.joypad_select;
            let result = if select & 0x20 == 0 {
                // P15=low: buttons row (A=0, B=1, Select=2, Start=3)
                0xC0 | (self.joypad_select & 0x30) | (self.joypad_buttons & 0x0F)
            } else if select & 0x10 == 0 {
                // P14=low: d-pad row (Right=0, Left=1, Up=2, Down=3)
                0xC0 | (self.joypad_select & 0x30) | (self.joypad_dpad & 0x0F)
            } else {
                0xFF
            };
            return result;
        }
        // APU register reads
        if addr >= 0xFF10 && addr <= 0xFF3F {
            return self.apu.read(addr);
        }
        let addr = addr as usize;
        if addr < self.bios.len() && self.bios_enabled {
            self.bios[addr]
        } else if addr < 0x4000 {
            // ROM bank 0 always at 0x0000-0x3FFF
            if addr < self.rom.len() {
                self.rom[addr]
            } else {
                0xFF
            }
        } else if addr < 0x8000 {
            // 0x4000-0x7FFF: switchable ROM bank (MBC1) or bank 1 (ROM only)
            let bank = if self.mbc_type >= 1 { self.rom_bank } else { 1 };
            let offset = (bank * 0x4000) + (addr - 0x4000);
            if offset < self.rom.len() {
                self.rom[offset]
            } else {
                0xFF
            }
        } else if addr < 0x10000 {
            // 0x8000+: the_rest (VRAM, WRAM, OAM, IO, HRAM)
            let rest_idx = addr - 0x8000;
            if rest_idx < self.the_rest.len() {
                self.the_rest[rest_idx]
            } else {
                0xFF
            }
        } else {
            0xFF
        }
    }

    fn read_word(&self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) | ((self.read_byte(addr.wrapping_add(1)) as u16) << 8)
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        if addr == 0xFF00 {
            self.joypad_select = value & 0x30;
            return;
        }
        // MBC1: ROM bank select (writes to 0x2000-0x3FFF)
        if self.mbc_type >= 1 && addr >= 0x2000 && addr < 0x4000 {
            let bank = (value & 0x1F) as usize;
            self.rom_bank = if bank == 0 { 1 } else { bank };
            return;
        }
        // MBC1: RAM enable (0x0000-0x1FFF) — ignore
        if self.mbc_type >= 1 && addr < 0x2000 {
            return;
        }
        let addr = addr as usize;
        if addr < self.bios.len() && self.bios_enabled {
            // writes to BIOS region ignored
        } else if addr < 0x8000 {
            // ROM is read-only
        } else if addr == 0xFF50 {
            // Writing to 0xFF50 disables the BIOS
            if value != 0 {
                self.bios_enabled = false;
            }
        } else if addr == 0xFF46 {
            // OAM DMA transfer: copy 160 bytes from (value << 8) to 0xFE00
            let src_base = (value as u16) << 8;
            for i in 0..160u16 {
                let byte = self.read_byte(src_base + i);
                let dst = 0xFE00u16 + i;
                let rest_idx = (dst as usize) - 0x8000;
                self.the_rest[rest_idx] = byte;
            }
        } else if addr >= 0xFF10 && addr <= 0xFF3F {
            // APU registers — notify APU and also write through to the_rest for readback
            self.apu.write(addr as u16, value);
            let rest_idx = addr - 0x8000;
            if rest_idx < self.the_rest.len() {
                self.the_rest[rest_idx] = value;
            }
        } else if addr < 0x10000 {
            // the_rest covers 0x8000-0xFFFF, same mapping as read_byte
            let rest_idx = addr - 0x8000;
            if rest_idx < self.the_rest.len() {
                self.the_rest[rest_idx] = value;
            }
        }
    }

    fn write_word(&mut self, addr: u16, value: u16) {
        self.write_byte(addr, value as u8);
        self.write_byte(addr.wrapping_add(1), (value >> 8) as u8);
    }

    fn generate_tileset_rgba(&self, buffer: &mut [u8]) {
        // A simple grayscale palette for the 4 colors of the Game Boy
        const PALETTE: [[u8; 4]; 4] = [
            [255, 255, 255, 255], // White (RGBA)
            [170, 170, 170, 255], // Light Gray
            [85, 85, 85, 255],    // Dark Gray
            [0, 0, 0, 255],       // Black
        ];

        let output_width_pixels: usize = 128; // 16 tiles * 8 pixels/tile

        for tile_index in 0..384 {
            let tile_grid_x: usize = tile_index % 16;
            let tile_grid_y: usize = tile_index / 16;
            let base_pixel_x: usize = tile_grid_x * 8;
            let base_pixel_y: usize = tile_grid_y * 8;

            let tile_addr_start = 0x8000_u16.wrapping_add(tile_index as u16 * 16);

            for row in 0..8usize {
                let current_pixel_y: usize = base_pixel_y + row;

                let row_addr = tile_addr_start.wrapping_add(row as u16 * 2);
                let byte1 = self.read_byte(row_addr);
                let byte2 = self.read_byte(row_addr.wrapping_add(1));

                for pixel in 0..8usize {
                    let color_bit_1 = (byte1 >> (7 - pixel)) & 1;
                    let color_bit_2 = (byte2 >> (7 - pixel)) & 1;
                    let color_index = (color_bit_2 << 1) | color_bit_1;

                    let color_rgba = PALETTE[color_index as usize];

                    let current_pixel_x: usize = base_pixel_x + pixel;

                    let buffer_index: usize =
                        (current_pixel_y * output_width_pixels + current_pixel_x) * 4;

                    if buffer_index + 3 < buffer.len() {
                        buffer[buffer_index] = color_rgba[0]; // R
                        buffer[buffer_index + 1] = color_rgba[1]; // G
                        buffer[buffer_index + 2] = color_rgba[2]; // B
                        buffer[buffer_index + 3] = color_rgba[3]; // A
                    }
                }
            }
        }
    }

    fn generate_memory_rgba(&self, buffer: &mut [u8]) {
        for address in 0..=65535u16 {
            let value = self.read_byte(address);

            let color: [u8; 4] = match value {
                0x00 => [255, 255, 255, 255],
                0xFF => [255, 0, 0, 255],
                _ => {
                    let r = value.wrapping_mul(7);
                    let g = value.wrapping_mul(13);
                    let b = value.wrapping_mul(23);
                    [r, g, b, 255]
                }
            };

            let buffer_index = address as usize * 4;

            if buffer_index + 3 < buffer.len() {
                let buffer_slice = &mut buffer[buffer_index..buffer_index + 4];
                buffer_slice.copy_from_slice(&color);
            }
        }
    }
    fn set_joypad(&mut self, buttons: u8, dpad: u8) {
        self.joypad_buttons = buttons;
        self.joypad_dpad = dpad;
    }

    fn tick_apu_sample(&mut self, cycles: u32) -> Option<(i16, i16)> {
        self.apu.tick(cycles)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn tick_apu_into_queue(&mut self, cycles: u32, queue: &Arc<Mutex<VecDeque<i16>>>) {
        if let Some((l, r)) = self.apu.tick(cycles) {
            if let Ok(mut q) = queue.lock() {
                // Cap queue at ~2 frames of audio to avoid unbounded growth
                if q.len() < 44_100 / 30 * 2 {
                    q.push_back(l);
                    q.push_back(r);
                }
            }
        }
    }
}
