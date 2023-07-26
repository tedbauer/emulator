mod gpu;
mod memory;
mod processor;

use gpu::Gpu;
use memory::Memory;
use memory::MemoryAccess;
use processor::instructions;
use processor::Cpu;
use processor::Instruction;
use rand::Rng;
use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::surface::Surface;
use sdl2::video::Window;
use std::fs;
use std::fs::File;
use std::io::Write;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("emulator", 160, 144)
        .build()
        .unwrap();

    let mut canvas: Canvas<Window> = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut memory = Box::new(Memory::initialize()) as Box<dyn MemoryAccess>;
    let mut gpu = Gpu::initialize(&mut memory);
    let mut cpu = Cpu::initialize(&mut memory, &gpu);

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }

        let mut pixels = Vec::new();
        let framebuffer = gpu.step(cpu.step()).unwrap();
        for (index, pixel) in framebuffer.0.iter().enumerate() {
            pixels.push(pixel.r);
            pixels.push(pixel.g);
            pixels.push(pixel.b);
            pixels.push(pixel.a);
        }

        let surface = Surface::from_data(
            pixels.as_mut_slice(),
            160,
            144,
            160 * 4,
            texture_creator.default_pixel_format(),
        )
        .unwrap();

        let texture = Texture::from_surface(&surface, &texture_creator).unwrap();
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    }
}
