mod cpu;
mod gpu;
mod memory;

use cpu::Cpu;
use gpu::Gpu;
use memory::Memory;
use memory::MemoryAccess;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, Texture};
use sdl2::surface::Surface;
use sdl2::video::Window;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rom_path = if args.len() >= 2 {
        args[1].clone()
    } else {
        eprintln!("Usage: {} <rom_path>", args[0]);
        eprintln!("  e.g. cargo run -- roms/snake.rom");
        std::process::exit(1);
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let scalar = 3u32;
    let window = video_subsystem
        .window("emulator", 160 * scalar, 144 * scalar)
        .build()
        .unwrap();

    let mut canvas: Canvas<Window> = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut memory = Box::new(Memory::initialize(&rom_path)) as Box<dyn MemoryAccess>;
    let mut gpu = Gpu::initialize();
    let mut cpu = Cpu::initialize();

    let mut joypad_buttons: u8 = 0xFF;
    let mut joypad_dpad: u8 = 0xFF;

    let mut event_pump = sdl_context.event_pump().unwrap();

    // Game Boy native: 59.7275 fps  (~16.742 ms per frame)
    let frame_duration = std::time::Duration::from_nanos(16_742_706);

    'running: loop {
        let frame_start = std::time::Instant::now();

        // Poll SDL events once per frame (not once per CPU step)
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    match key {
                        Keycode::Right => joypad_dpad &= !0x01,
                        Keycode::Left => joypad_dpad &= !0x02,
                        Keycode::Up => joypad_dpad &= !0x04,
                        Keycode::Down => joypad_dpad &= !0x08,
                        Keycode::Z => joypad_buttons &= !0x01, // A
                        Keycode::X => joypad_buttons &= !0x02, // B
                        Keycode::Return => joypad_buttons &= !0x08, // Start
                        Keycode::Backspace => joypad_buttons &= !0x04, // Select
                        _ => {}
                    }
                    memory.set_joypad(joypad_buttons, joypad_dpad);
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    match key {
                        Keycode::Right => joypad_dpad |= 0x01,
                        Keycode::Left => joypad_dpad |= 0x02,
                        Keycode::Up => joypad_dpad |= 0x04,
                        Keycode::Down => joypad_dpad |= 0x08,
                        Keycode::Z => joypad_buttons |= 0x01,
                        Keycode::X => joypad_buttons |= 0x02,
                        Keycode::Return => joypad_buttons |= 0x08,
                        Keycode::Backspace => joypad_buttons |= 0x04,
                        _ => {}
                    }
                    memory.set_joypad(joypad_buttons, joypad_dpad);
                }
                _ => {}
            }
        }

        // Run CPU + GPU until a full frame (VBlank) is ready
        let framebuffer = loop {
            let (time_increment, _) = cpu.step(&mut memory);
            cpu.handle_interrupts(&mut memory);
            if let Some(fb) = gpu.step(time_increment, &mut memory) {
                break fb;
            }
        };

        // Render the frame
        let mut pixels: Vec<u8> = Vec::with_capacity(160 * 144 * 4);
        for pixel in framebuffer.0.iter() {
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

        // Sleep to cap at native Game Boy framerate (~59.7 fps)
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}
