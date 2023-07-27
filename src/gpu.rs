use crate::cpu::TimeIncrement;
use crate::MemoryAccess;
use rand::Rng;

#[derive(Debug)]
pub struct Gpu {
    scan_mode: ScanMode,
    mode_clock: usize,
    line: u8,
    framebuffer: Framebuffer,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone)]
pub struct Framebuffer(pub Vec<Rgba>);

fn gen_random_framebuffer() -> Framebuffer {
    let mut rng = rand::thread_rng();
    let mut pixels = Vec::new();
    for _ in 0..(160 * 144) {
        let n = rng.gen_range(0..=255);
        pixels.push(Rgba {
            r: 0,
            g: n,
            b: 0,
            a: 255,
        });
    }

    Framebuffer(pixels)
}

/// Write all GPU registers to their memory-mapped locations.
fn apply_memory_map(gpu: &Gpu, memory: &mut Box<dyn MemoryAccess>) {
    memory.write_byte(0xFF44, gpu.line)
}

fn get_pixel(n1: u8, n2: u8, bit: u8) -> bool {
    (((n1 >> bit) as u8) + (((n2 >> bit) as u8) << 1)) > 0
}

fn push_row(num: u8, buffer: &mut Vec<Rgba>) {
    println!("num: {:#02x}", num);
    println!("---");
    for i in 0..8 {
        if ((1 << i) & num) > 0 {
            println!("1");
            buffer.push(Rgba {
                r: 0, g: 0, b: 0, a: 255
            });
        } else {
            println!("0");
            buffer.push(Rgba {
                r: 255, g: 255, b: 255, a: 255
            });
        }
    }
    println!("---");
}

fn render_scan(gpu: &mut Gpu, memory: &Box<dyn MemoryAccess>) {
    gpu.framebuffer.0 = Vec::new();

    for _ in 0..11936 {
        gpu.framebuffer.0.push(Rgba {
            r: 255,
            g: 0,
            b: 255,
            a: 255,
        });
    }

    push_row(memory.read_byte(0x7f49), &mut gpu.framebuffer.0);
    for _ in 0..152 {
        gpu.framebuffer.0.push(Rgba {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        });
    }
    push_row(memory.read_byte(0x7f4a), &mut gpu.framebuffer.0);
    for _ in 0..152 {
        gpu.framebuffer.0.push(Rgba {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        });
    }
    push_row(memory.read_byte(0x7f4b), &mut gpu.framebuffer.0);
    for _ in 0..152 {
        gpu.framebuffer.0.push(Rgba {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        });
    }
    push_row(memory.read_byte(0x7f4c), &mut gpu.framebuffer.0);
    for _ in 0..152 {
        gpu.framebuffer.0.push(Rgba {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        });
    }
    push_row(memory.read_byte(0x7f4d), &mut gpu.framebuffer.0);
    for _ in 0..152 {
        gpu.framebuffer.0.push(Rgba {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        });
    }
    push_row(memory.read_byte(0x7f4e), &mut gpu.framebuffer.0);
    for _ in 0..152 {
        gpu.framebuffer.0.push(Rgba {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        });
    }
    push_row(memory.read_byte(0x7f4f), &mut gpu.framebuffer.0);
    for _ in 0..152 {
        gpu.framebuffer.0.push(Rgba {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        });
    }
    push_row(memory.read_byte(0x7f50), &mut gpu.framebuffer.0);
    for _ in 0..152 {
        gpu.framebuffer.0.push(Rgba {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        });
    }
    push_row(memory.read_byte(0x7f51), &mut gpu.framebuffer.0);
    
    for _ in 0..(11936 - 152*3) {
        gpu.framebuffer.0.push(Rgba {
            r: 255,
            g: 0,
            b: 255,
            a: 255,
        });
    }
}

fn step_mode(
    gpu: &mut Gpu,
    memory: &Box<dyn MemoryAccess>,
    time_increment: TimeIncrement,
) -> Option<Framebuffer> {
    gpu.mode_clock += (time_increment.t as usize);
    match gpu.scan_mode {
        ScanMode::AccessOam => {
            if gpu.mode_clock >= 80 {
                gpu.scan_mode = ScanMode::AccessVram;
                gpu.mode_clock = 0;
            }
            None
        }
        ScanMode::AccessVram => {
            if gpu.mode_clock >= 172 {
                gpu.scan_mode = ScanMode::HorizontalBlank;
                gpu.mode_clock = 0;
                render_scan(gpu, memory);
            }
            None
        }
        ScanMode::HorizontalBlank => {
            if gpu.mode_clock >= 204 {
                gpu.line += 1;
                gpu.mode_clock = 0;

                if gpu.line == 143 {
                    gpu.scan_mode = ScanMode::VerticalBlank;
                    return Some(gpu.framebuffer.clone());
                } else {
                    gpu.scan_mode = ScanMode::AccessOam;
                }
            }
            None
        }
        ScanMode::VerticalBlank => {
            if gpu.mode_clock >= 4560 {
                gpu.mode_clock = 0;
                gpu.scan_mode = ScanMode::AccessOam;
                gpu.line = 0;
            }
            None
        }
    }
}

impl Gpu {
    pub fn initialize() -> Self {
        Self {
            scan_mode: ScanMode::AccessOam,
            mode_clock: 0,
            line: 0,
            framebuffer: gen_random_framebuffer(),
        }
    }

    pub fn step(
        &mut self,
        time_increment: TimeIncrement,
        memory: &mut Box<dyn MemoryAccess>,
    ) -> Option<Framebuffer> {
        let framebuffer = step_mode(self, memory, time_increment);
        apply_memory_map(self, memory);
        framebuffer
    }
}

#[derive(Debug)]
enum ScanMode {
    AccessOam,
    AccessVram,
    HorizontalBlank,
    VerticalBlank,
}
