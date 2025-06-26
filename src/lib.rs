// Place your existing module declarations here
mod cpu;
mod gpu;
mod memory;

// --- Imports ---
use std::panic;
use wasm_bindgen::prelude::*;

// Bring in your emulator components
use cpu::Cpu;
use gpu::Gpu;
use memory::{Memory, MemoryAccess};

// Game Boy screen dimensions
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

const TILESET_WIDTH: usize = 128; // 16 tiles across
const TILESET_HEIGHT: usize = 192; // 24 tiles down

/// The main Emulator struct that will be exposed to JavaScript.
#[wasm_bindgen]
pub struct Emulator {
    cpu: Cpu,
    gpu: Gpu,
    memory: Box<dyn MemoryAccess>,
    // This buffer holds the pixel data (RGBA) for one frame.
    // JavaScript will read from this buffer to draw to the canvas.
    pixel_buffer: Vec<u8>,

    // ADD: A buffer for our tileset visualization
    tileset_buffer: Vec<u8>,
}

#[wasm_bindgen]
impl Emulator {
    /// Creates a new Emulator instance.
    /// JavaScript will call this with the raw bytes of the ROM file.
    #[wasm_bindgen(constructor)]
    pub fn new(rom_data: Vec<u8>) -> Self {
        // Set a panic hook to get better error messages in the browser console.
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        // Note: We are assuming your `Memory` module can be initialized
        // with ROM data like this. You may need to adjust `Memory::initialize`.
        let mut memory = Box::new(Memory::initialize()) as Box<dyn MemoryAccess>;

        Emulator {
            cpu: Cpu::initialize(),
            gpu: Gpu::initialize(),
            memory,
            // Initialize the pixel buffer. Its size will never change.
            pixel_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 4],
            tileset_buffer: vec![0; TILESET_WIDTH * TILESET_HEIGHT * 4],
        }
    }

    /// Executes enough CPU and GPU cycles to generate a single frame.
    /// This function will be called by JavaScript in a loop.
    pub fn tick(&mut self) {
        // The original loop from main.rs is now inside tick().
        // We loop until the GPU has produced a new frame.
        loop {
            let time_increment = self.cpu.step(&mut self.memory);

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

                // ADD: Update the tileset buffer on every completed frame
                self.memory.generate_tileset_rgba(&mut self.tileset_buffer);
                break;
            }
        }
    }

    /// Returns a pointer to the internal pixel buffer.
    /// This is the most efficient way to get the screen data to JavaScript.
    pub fn framebuffer_ptr(&self) -> *const u8 {
        self.pixel_buffer.as_ptr()
    }

    // ADD: A new function to get a pointer to the tileset data
    #[wasm_bindgen]
    pub fn tileset_ptr(&self) -> *const u8 {
        self.tileset_buffer.as_ptr()
    }

    /// Handles key down events from the browser.
    pub fn key_down(&mut self, key_code: String) {
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
    pub fn key_up(&mut self, key_code: String) {
        // Similarly, handle key releases to update the joypad state.
    }
}
