mod apu;
mod cpu;
mod gpu;
mod memory;

use cpu::Cpu;
use gpu::Gpu;
use memory::Memory;
use memory::MemoryAccess;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, Texture};
use sdl2::surface::Surface;
use sdl2::video::Window;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// SDL2 audio callback — drains samples from the shared queue into the output.
// ---------------------------------------------------------------------------

struct SampleQueue {
    queue: Arc<Mutex<VecDeque<i16>>>,
}

impl AudioCallback for SampleQueue {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        let mut q = self.queue.lock().unwrap();
        for sample in out.iter_mut() {
            *sample = q.pop_front().unwrap_or(0);
        }
    }
}

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
    let audio_subsystem = sdl_context.audio().unwrap();

    let scalar = 3u32;
    let window = video_subsystem
        .window("emulator", 160 * scalar, 144 * scalar)
        .build()
        .unwrap();

    let mut canvas: Canvas<Window> = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    // Set up shared sample queue and SDL audio device
    let sample_queue: Arc<Mutex<VecDeque<i16>>> = Arc::new(Mutex::new(VecDeque::new()));

    let audio_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(2), // stereo interleaved [L, R, L, R, ...]
        samples: Some(512),
    };

    let audio_device = audio_subsystem
        .open_playback(None, &audio_spec, |_spec| SampleQueue {
            queue: Arc::clone(&sample_queue),
        })
        .unwrap();

    audio_device.resume();

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

        // Run CPU + GPU until a full frame (VBlank) is ready.
        // After each CPU step, tick the APU and push any new samples.
        let framebuffer = loop {
            let (time_increment, _) = cpu.step(&mut memory);
            cpu.handle_interrupts(&mut memory);

            // Tick APU with the T-cycle count this instruction took
            memory.tick_apu_into_queue(time_increment.t as u32, &sample_queue);

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
