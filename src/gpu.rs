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

fn read_tile_map(
    memory: &Box<dyn MemoryAccess>,
    tile_x: u8,
    tile_y: u8,
    pixel_x: u8,
    pixel_y: u8,
) -> Rgba {
    let tile_map_index = memory
        .read_byte(((tile_x as u16) * (tile_y as u16)) as u16 + 0x9800);
    let tile_set_index = (tile_map_index as u16) * 16 + ((pixel_y as u16) * 2) + 0x8000;
    println!("reading tileset index: {:#02x}", tile_set_index);
    let tile = memory.read_byte(tile_set_index as u16);

    if ((1 << pixel_x) & tile) > 0 {
        Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    } else {
        Rgba {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }
}

fn render_scan(gpu: &mut Gpu, memory: &Box<dyn MemoryAccess>) {
    println!("--------------------");
    for pixel in 0..160 {
        let tile_x = pixel / 8;
        let tile_y = gpu.line / 8;
        let tile_pixel_x = pixel % 8;
        let tile_pixel_y = gpu.line % 8;

        // println!("---Rendering pixel ({}, {}).---", pixel, gpu.line);
        // println!("  * Tile: ({}, {})", tile_x, tile_y);
        // println!("  * Pixel in tile: ({}, {})", pixel % 8, gpu.line % 8);
        // println!(
        //     "Reading tile at address {:#02x}]",
        //     ((tile_x as u16) * (tile_y as u16)) as u16 + 0x8000
        // );

        gpu.framebuffer.0.push(read_tile_map(
            memory,
            tile_x,
            tile_y,
            tile_pixel_x,
            tile_pixel_y,
        ));
    }
    println!("------------------");
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
                    let f = gpu.framebuffer.clone();
                    gpu.framebuffer = Framebuffer(Vec::new());
                    return Some(f);
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
