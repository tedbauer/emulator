mod gpu;
mod memory;
mod processor;

use gpu::Gpu;
use memory::Memory;
use memory::MemoryAccess;
use processor::instructions;
use processor::Cpu;
use processor::Instruction;
use std::fs;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::fs::File;
use std::io::Write;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("emulator", 160, 144)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.clear();
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
    }

    /*
    let mut memory = Box::new(Memory::initialize()) as Box<dyn MemoryAccess>;

    let mut gpu = Gpu::initialize(&mut memory);
    let mut cpu = Cpu::initialize(&mut memory, &gpu);

    loop {
        let time_increment = cpu.step();
        gpu.step(time_increment);
    }
    */
}
