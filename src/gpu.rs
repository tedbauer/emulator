use crate::processor::TimeIncrement;
use crate::MemoryAccess;

#[derive(Debug)]
pub struct Gpu {
    scan_mode: ScanMode,
    mode_clock: usize,
    line: usize,
}

impl Gpu {
    pub fn initialize(memory: &mut Box<dyn MemoryAccess>) -> Self {
        Self { scan_mode: ScanMode::AccessOam, mode_clock: 0, line: 0 }
    }

    pub fn step(&mut self, time_increment: TimeIncrement) {
        println!("{:?}", self.scan_mode);
        println!("{}", self.mode_clock);

        self.mode_clock += (time_increment.t as usize);
        match self.scan_mode {
            ScanMode::AccessOam => {
                if self.mode_clock >= 80 {
                    self.scan_mode = ScanMode::AccessVram;
                    self.mode_clock = 0;
                }
            },
            ScanMode::AccessVram => {
                if self.mode_clock >= 172 {
                    self.scan_mode = ScanMode::VerticalBlank;
                    self.mode_clock = 0;
                    self.render_scan();
                }
            },
            ScanMode::HorizontalBlank => {
                if self.mode_clock >= 204 {
                    self.line += 1;
                    self.mode_clock = 0;

                    if self.line == 143 {
                        self.scan_mode = ScanMode::VerticalBlank;
                        // TODO: write framebuffer to screen.
                    } else {
                        self.scan_mode = ScanMode::AccessOam;
                    }
                }
            },
            ScanMode::VerticalBlank => {
                if self.mode_clock >= 4560 {
                    self.mode_clock = 0;
                    self.scan_mode = ScanMode::AccessOam;
                    self.line = 0;
                }
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
    VerticalBlank
}