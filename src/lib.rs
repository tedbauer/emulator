// WASM frontend bindings — exports the emulator to JavaScript via wasm-bindgen.
// All items here are used by the browser build target, not the native binary.
#![allow(dead_code)]

mod apu;
mod cpu;
mod gpu;
mod memory;

use std::panic;
use wasm_bindgen::prelude::*;

use cpu::Cpu;
use gpu::Gpu;
use memory::{Memory, MemoryAccess};

const LOG_CAPACITY: usize = 20;

// Game Boy screen dimensions
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

const TILESET_WIDTH: usize = 128; // 16 tiles across
const TILESET_HEIGHT: usize = 192; // 24 tiles down

const MEMORY_WIDTH: usize = 256;
const MEMORY_HEIGHT: usize = 256;

/// The main Emulator struct exposed to JavaScript.
#[wasm_bindgen]
pub struct Emulator {
    cpu: Cpu,
    gpu: Gpu,
    memory: Box<dyn MemoryAccess>,
    pixel_buffer: Vec<u8>,
    // Joypad state: bit=0 means pressed (active-low)
    joypad_buttons: u8,
    joypad_dpad: u8,
}

#[wasm_bindgen]
impl Emulator {
    /// Creates a new Emulator instance with the ROM bytes provided by JavaScript.
    #[wasm_bindgen(constructor)]
    pub fn new(rom_data: Vec<u8>) -> Self {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let memory = Box::new(Memory::initialize_with_rom(rom_data)) as Box<dyn MemoryAccess>;
        Emulator {
            cpu: Cpu::initialize(),
            gpu: Gpu::initialize(),
            memory,
            pixel_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 4],
            joypad_buttons: 0xFF,
            joypad_dpad: 0xFF,
        }
    }

    /// Executes enough CPU and GPU cycles to produce one frame.
    /// Call this from a requestAnimationFrame loop in JavaScript.
    pub fn tick(&mut self) {
        loop {
            let (time_increment, _) = self.cpu.step(&mut self.memory);
            self.cpu.handle_interrupts(&mut self.memory);
            if let Some(framebuffer) = self.gpu.step(time_increment, &mut self.memory) {
                for (i, pixel) in framebuffer.0.iter().enumerate() {
                    let offset = i * 4;
                    self.pixel_buffer[offset] = pixel.r;
                    self.pixel_buffer[offset + 1] = pixel.g;
                    self.pixel_buffer[offset + 2] = pixel.b;
                    self.pixel_buffer[offset + 3] = 255;
                }
                break;
            }
        }
    }

    /// Returns a pointer to the pixel buffer for JavaScript to read.
    pub fn framebuffer_ptr(&self) -> *const u8 {
        self.pixel_buffer.as_ptr()
    }

    /// Called by JavaScript on keydown. key_code is the browser KeyboardEvent.code value.
    pub fn key_down(&mut self, key_code: String) {
        match key_code.as_str() {
            "ArrowRight" => self.joypad_dpad &= !0x01,
            "ArrowLeft" => self.joypad_dpad &= !0x02,
            "ArrowUp" => self.joypad_dpad &= !0x04,
            "ArrowDown" => self.joypad_dpad &= !0x08,
            "KeyZ" => self.joypad_buttons &= !0x01,  // A
            "KeyX" => self.joypad_buttons &= !0x02,  // B
            "Enter" => self.joypad_buttons &= !0x08, // Start
            "Backspace" | "ShiftLeft" => self.joypad_buttons &= !0x04, // Select
            _ => return,
        }
        self.memory
            .set_joypad(self.joypad_buttons, self.joypad_dpad);
    }

    /// Called by JavaScript on keyup.
    pub fn key_up(&mut self, key_code: String) {
        match key_code.as_str() {
            "ArrowRight" => self.joypad_dpad |= 0x01,
            "ArrowLeft" => self.joypad_dpad |= 0x02,
            "ArrowUp" => self.joypad_dpad |= 0x04,
            "ArrowDown" => self.joypad_dpad |= 0x08,
            "KeyZ" => self.joypad_buttons |= 0x01,
            "KeyX" => self.joypad_buttons |= 0x02,
            "Enter" => self.joypad_buttons |= 0x08,
            "Backspace" | "ShiftLeft" => self.joypad_buttons |= 0x04,
            _ => return,
        }
        self.memory
            .set_joypad(self.joypad_buttons, self.joypad_dpad);
    }
}
