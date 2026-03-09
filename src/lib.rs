// WASM frontend bindings — exports the emulator to JavaScript via wasm-bindgen.
// All items here are used by the browser build target, not the native binary.
#![allow(dead_code)]

mod cpu;
mod gpu;
mod memory;

use std::panic;
use wasm_bindgen::prelude::*;

use cpu::Cpu;
use gpu::Gpu;
use memory::{Memory, MemoryAccess};
use std::collections::VecDeque;

const LOG_CAPACITY: usize = 20;

// Game Boy screen dimensions
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

const TILESET_WIDTH: usize = 128; // 16 tiles across
const TILESET_HEIGHT: usize = 192; // 24 tiles down

const MEMORY_WIDTH: usize = 256;
const MEMORY_HEIGHT: usize = 256;

/// The main Emulator struct that will be exposed to JavaScript.
#[wasm_bindgen]
pub struct Emulator {
    cpu: Cpu,
    gpu: Gpu,
    memory: Box<dyn MemoryAccess>,
    // This buffer holds the pixel data (RGBA) for one frame.
    // JavaScript will read from this buffer to draw to the canvas.
    pixel_buffer: Vec<u8>,

    tileset_buffer: Vec<u8>,
    memory_buffer: Vec<u8>,
    instruction_log: VecDeque<String>,
}

#[wasm_bindgen]
impl Emulator {
    /// Creates a new Emulator instance.
    /// JavaScript will call this with the raw bytes of the ROM file.
    #[wasm_bindgen(constructor)]
    pub fn new(rom_data: Vec<u8>) -> Self {
        // Set a panic hook to get better error messages in the browser console.
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        // Initialize memory with the ROM data passed from JavaScript.
        let memory = Box::new(Memory::initialize_with_rom(rom_data)) as Box<dyn MemoryAccess>;

        Emulator {
            cpu: Cpu::initialize(),
            gpu: Gpu::initialize(),
            memory,
            // Initialize the pixel buffer. Its size will never change.
            pixel_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 4],
            tileset_buffer: vec![0; TILESET_WIDTH * TILESET_HEIGHT * 4],
            memory_buffer: vec![0; MEMORY_WIDTH * MEMORY_HEIGHT * 4],
            instruction_log: VecDeque::with_capacity(LOG_CAPACITY),
        }
    }

    /// Executes enough CPU and GPU cycles to generate a single frame.
    /// This function will be called by JavaScript in a loop.
    pub fn tick(&mut self) {
        // The original loop from main.rs is now inside tick().
        // We loop until the GPU has produced a new frame.
        loop {
            let (time_increment, instruction_string) = self.cpu.step(&mut self.memory);
            self.log_instruction(instruction_string);
            self.cpu.handle_interrupts(&mut self.memory);

            // The GPU step function tells us when a frame is ready
            if let Some(framebuffer) = self.gpu.step(time_increment, &mut self.memory) {
                // Copy the frame data from your GPU into our RGBA pixel_buffer
                for (i, pixel) in framebuffer.0.iter().enumerate() {
                    let offset = i * 4;
                    self.pixel_buffer[offset] = pixel.r;
                    self.pixel_buffer[offset + 1] = pixel.g;
                    self.pixel_buffer[offset + 2] = pixel.b;
                    self.pixel_buffer[offset + 3] = 255; // Alpha channel
                }
                // We have a full frame, so we can exit the loop for this tick.

                self.memory.generate_tileset_rgba(&mut self.tileset_buffer);
                self.memory.generate_memory_rgba(&mut self.memory_buffer);

                break;
            }
        }
    }

    fn log_instruction(&mut self, instruction: String) {
        if self.instruction_log.len() == LOG_CAPACITY {
            self.instruction_log.pop_back();
        }
        self.instruction_log.push_front(instruction);
    }

    /// Returns the entire instruction log as a single formatted string.
    #[wasm_bindgen]
    pub fn get_instruction_log(&self) -> String {
        self.instruction_log
            .iter()
            .cloned()
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Returns a pointer to the internal pixel buffer.
    /// This is the most efficient way to get the screen data to JavaScript.
    pub fn framebuffer_ptr(&self) -> *const u8 {
        self.pixel_buffer.as_ptr()
    }

    #[wasm_bindgen]
    pub fn tileset_ptr(&self) -> *const u8 {
        self.tileset_buffer.as_ptr()
    }

    #[wasm_bindgen]
    pub fn memory_ptr(&self) -> *const u8 {
        self.memory_buffer.as_ptr()
    }

    /// Handles key down events from the browser.
    pub fn key_down(&mut self, _key_code: String) {
        // Here you would map the JavaScript key_code to your
        // emulator's joypad registers.
        // For example:
        // match key_code.as_str() {
        //     "ArrowUp" => self.memory.key_down(JoypadKey::Up),
        //     "KeyZ" => self.memory.key_down(JoypadKey::A),
        //     _ => {}
        // }
    }

    /// Handles key up events from the browser.
    pub fn key_up(&mut self, _key_code: String) {
        // Similarly, handle key releases to update the joypad state.
    }
}
