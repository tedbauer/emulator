use crate::processor::TimeIncrement;
use crate::MemoryAccess;
use rand::Rng;

#[derive(Debug)]
pub struct Gpu {
    scan_mode: ScanMode,
    mode_clock: usize,
    line: usize,
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

impl Gpu {
    pub fn initialize(memory: &mut Box<dyn MemoryAccess>) -> Self {
        Self {
            scan_mode: ScanMode::AccessOam,
            mode_clock: 0,
            line: 0,
            framebuffer: Framebuffer(
                [Rgba {
                    r: 0,
                    g: 255,
                    b: 0,
                    a: 255,
                }; 160 * 144]
                    .to_vec(),
            ),
        }
    }

    pub fn step(&mut self, time_increment: TimeIncrement) -> Option<Framebuffer> {
        return Some(gen_random_framebuffer());
        // return Some(self.framebuffer.clone());
        println!("{:?}", self.scan_mode);
        println!("{}", self.mode_clock);

        self.mode_clock += (time_increment.t as usize);
        match self.scan_mode {
            ScanMode::AccessOam => {
                if self.mode_clock >= 80 {
                    self.scan_mode = ScanMode::AccessVram;
                    self.mode_clock = 0;
                }
                None
            }
            ScanMode::AccessVram => {
                if self.mode_clock >= 172 {
                    self.scan_mode = ScanMode::VerticalBlank;
                    self.mode_clock = 0;
                    self.render_scan();
                }
                None
            }
            ScanMode::HorizontalBlank => {
                if self.mode_clock >= 204 {
                    self.line += 1;
                    self.mode_clock = 0;

                    if self.line == 143 {
                        self.scan_mode = ScanMode::VerticalBlank;
                        return Some(self.framebuffer.clone());
                    } else {
                        self.scan_mode = ScanMode::AccessOam;
                    }
                }
                None
            }
            ScanMode::VerticalBlank => {
                if self.mode_clock >= 4560 {
                    self.mode_clock = 0;
                    self.scan_mode = ScanMode::AccessOam;
                    self.line = 0;
                }
                None
            }
        }
    }

    fn render_scan(&self) {}
}

#[derive(Debug)]
enum ScanMode {
    AccessOam,
    AccessVram,
    HorizontalBlank,
    VerticalBlank,
}
