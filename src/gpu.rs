use crate::cpu::TimeIncrement;
use crate::MemoryAccess;

#[derive(Debug)]
pub struct Gpu {
    scan_mode: ScanMode,
    mode_clock: usize,
    line: u8,
    framebuffer: Framebuffer,
    window_line: u8,
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

// Game Boy monochrome palette: color IDs 0-3 → RGBA
const PALETTE: [Rgba; 4] = [
    Rgba { r: 255, g: 255, b: 255, a: 255 }, // 0 = white
    Rgba { r: 170, g: 170, b: 170, a: 255 }, // 1 = light gray
    Rgba { r:  85, g:  85, b:  85, a: 255 }, // 2 = dark gray
    Rgba { r:   0, g:   0, b:   0, a: 255 }, // 3 = black
];

fn gen_framebuffer() -> Framebuffer {
    Framebuffer(vec![PALETTE[0]; 160 * 144])
}

/// Decode a palette byte (BGP/OBP0/OBP1) for a given color_id.
fn decode_palette(palette: u8, color_id: u8) -> Rgba {
    let shade = (palette >> (color_id * 2)) & 0x3;
    PALETTE[shade as usize]
}

/// Read tile pixel color from VRAM at the given tile data address + row.
fn tile_pixel(memory: &Box<dyn MemoryAccess>, tile_data_addr: u16, pixel_x: u8, pixel_y: u8) -> u8 {
    let row_addr = tile_data_addr.wrapping_add((pixel_y as u16) * 2);
    let byte1 = memory.read_byte(row_addr);
    let byte2 = memory.read_byte(row_addr + 1);
    let bit = 7 - pixel_x;
    let lo = (byte1 >> bit) & 1;
    let hi = (byte2 >> bit) & 1;
    (hi << 1) | lo
}

/// Compute tile data address given tile index and LCDC bit 4.
fn tile_data_addr(tile_index: u8, lcdc: u8) -> u16 {
    if lcdc & 0x10 != 0 {
        // Unsigned: 0x8000 + index * 16
        0x8000u16.wrapping_add((tile_index as u16) * 16)
    } else {
        // Signed: base 0x9000, index is i8
        let signed = tile_index as i8 as i32;
        (0x9000i32 + signed * 16) as u16
    }
}

fn render_scan(gpu: &mut Gpu, memory: &mut Box<dyn MemoryAccess>) {
    let lcdc = memory.read_byte(0xFF40);

    let scroll_x = memory.read_byte(0xFF43);
    let scroll_y = memory.read_byte(0xFF42);
    let wy = memory.read_byte(0xFF4A); // Window Y position
    let wx = memory.read_byte(0xFF4B); // Window X position + 7
    let bgp = memory.read_byte(0xFF47); // BG palette
    let obp0 = memory.read_byte(0xFF48); // OBJ palette 0
    let obp1 = memory.read_byte(0xFF49); // OBJ palette 1
    let line = gpu.line;

    let line_start = line as usize * 160;

    // Track which pixels have non-transparent BG (for sprite priority)
    let mut bg_opaque = [false; 160];

    // ── Background ──────────────────────────────────────────────────────────
    if lcdc & 0x01 != 0 {
        let bg_map_base: u16 = if lcdc & 0x08 != 0 { 0x9C00 } else { 0x9800 };

        for pixel_x in 0u8..160 {
            let bg_x = pixel_x.wrapping_add(scroll_x);
            let bg_y = line.wrapping_add(scroll_y);

            let tile_col = bg_x / 8;
            let tile_row = bg_y / 8;
            let tile_px = bg_x % 8;
            let tile_py = bg_y % 8;

            let tile_idx = memory.read_byte(bg_map_base + (tile_row as u16) * 32 + (tile_col as u16));
            let addr = tile_data_addr(tile_idx, lcdc);
            let color_id = tile_pixel(memory, addr, tile_px, tile_py);
            bg_opaque[pixel_x as usize] = color_id != 0;
            gpu.framebuffer.0[line_start + pixel_x as usize] = decode_palette(bgp, color_id);
        }
    } else {
        // BG off: fill white
        for px in 0..160usize {
            gpu.framebuffer.0[line_start + px] = PALETTE[0];
        }
    }

    // ── Window ───────────────────────────────────────────────────────────────
    // Window is enabled by LCDC bit 5, shown when WY <= line and WX-7 < 160
    if lcdc & 0x20 != 0 && line >= wy {
        let win_map_base: u16 = if lcdc & 0x40 != 0 { 0x9C00 } else { 0x9800 };
        let win_x_screen = (wx as i16) - 7; // can be negative
        let win_py = gpu.window_line;

        let tile_row = win_py / 8;
        let tile_py = win_py % 8;

        for pixel_x in 0u8..160 {
            let screen_x = pixel_x as i16;
            if screen_x < win_x_screen { continue; }
            let win_px_abs = (screen_x - win_x_screen) as u16;
            let tile_col = (win_px_abs / 8) as u8;
            let tile_px = (win_px_abs % 8) as u8;

            let tile_idx = memory.read_byte(win_map_base + (tile_row as u16) * 32 + (tile_col as u16));
            let addr = tile_data_addr(tile_idx, lcdc);
            let color_id = tile_pixel(memory, addr, tile_px, tile_py);
            bg_opaque[pixel_x as usize] = color_id != 0;
            gpu.framebuffer.0[line_start + pixel_x as usize] = decode_palette(bgp, color_id);
        }
        gpu.window_line += 1;
    }

    // ── Sprites (OAM) ────────────────────────────────────────────────────────
    if lcdc & 0x02 != 0 {
        let sprite_height: i16 = if lcdc & 0x04 != 0 { 16 } else { 8 };

        // OAM: 40 sprites × 4 bytes at 0xFE00
        // Draw in reverse order so sprite 0 has highest priority
        // Collect visible sprites and sort by X (lower X = higher priority)
        let mut visible: Vec<(u8, i16, i16, u8, u8)> = Vec::new(); // (idx, sy, sx, tile, attr)
        for sprite in 0..40u16 {
            let base = 0xFE00 + sprite * 4;
            let sy = memory.read_byte(base) as i16 - 16;
            let sx = memory.read_byte(base + 1) as i16 - 8;
            let tile_idx = memory.read_byte(base + 2);
            let attr = memory.read_byte(base + 3);

            let ly = line as i16;
            if ly >= sy && ly < sy + sprite_height {
                visible.push((sprite as u8, sy, sx, tile_idx, attr));
                if visible.len() == 10 { break; } // max 10 sprites per line
            }
        }
        // Sort by X coordinate (lower X draws last = highest priority)
        visible.sort_by_key(|s| s.2);

        for (_, sy, sx, tile_idx, attr) in visible.iter().rev() {
            let flip_x = attr & 0x20 != 0;
            let flip_y = attr & 0x40 != 0;
            let behind_bg = attr & 0x80 != 0;
            let palette = if attr & 0x10 != 0 { obp1 } else { obp0 };

            // For 8x16 sprites, mask the lowest bit of tile index
            let tile = if sprite_height == 16 { tile_idx & 0xFE } else { *tile_idx };
            let sprite_addr = 0x8000u16 + (tile as u16) * 16;

            let mut row_in_sprite = (line as i16 - sy) as u16;
            if flip_y { row_in_sprite = (sprite_height as u16 - 1) - row_in_sprite; }

            // For 8x16, row >= 8 means lower tile
            let tile_addr = if row_in_sprite >= 8 {
                0x8000u16 + ((tile | 1) as u16) * 16
            } else {
                sprite_addr
            };
            let tile_row = (row_in_sprite % 8) as u8;

            for px in 0u8..8 {
                let screen_x = sx + px as i16;
                if screen_x < 0 || screen_x >= 160 { continue; }
                let tile_px = if flip_x { 7 - px } else { px };
                let color_id = tile_pixel(memory, tile_addr, tile_px, tile_row);
                if color_id == 0 { continue; } // transparent

                // Sprite behind BG: only draw if BG is transparent (color 0)
                if behind_bg && bg_opaque[screen_x as usize] { continue; }

                let idx = line_start + screen_x as usize;
                gpu.framebuffer.0[idx] = decode_palette(palette, color_id);
            }
        }
    }
}

fn step_mode(
    gpu: &mut Gpu,
    memory: &mut Box<dyn MemoryAccess>,
    time_increment: TimeIncrement,
) -> Option<Framebuffer> {
    gpu.mode_clock += time_increment.t as usize;

    match gpu.scan_mode {
        ScanMode::AccessOam => {
            if gpu.mode_clock >= 80 {
                gpu.mode_clock -= 80;
                gpu.scan_mode = ScanMode::AccessVram;
            }
            None
        }
        ScanMode::AccessVram => {
            if gpu.mode_clock >= 172 {
                gpu.mode_clock -= 172;
                gpu.scan_mode = ScanMode::HorizontalBlank;
                render_scan(gpu, memory);
            }
            None
        }
        ScanMode::HorizontalBlank => {
            if gpu.mode_clock >= 204 {
                gpu.mode_clock -= 204;
                gpu.line += 1;

                if gpu.line == 144 {
                    gpu.scan_mode = ScanMode::VerticalBlank;
                    // Set VBlank interrupt flag (bit 0 of IF at 0xFF0F)
                    let if_val = memory.read_byte(0xFF0F);
                    memory.write_byte(0xFF0F, if_val | 0x01);
                    let f = gpu.framebuffer.clone();
                    return Some(f);
                } else {
                    gpu.scan_mode = ScanMode::AccessOam;
                }
            }
            None
        }
        ScanMode::VerticalBlank => {
            if gpu.mode_clock >= 456 {
                gpu.mode_clock -= 456;
                gpu.line += 1;

                if gpu.line > 153 {
                    gpu.line = 0;
                    gpu.window_line = 0; // reset window line counter each frame
                    gpu.scan_mode = ScanMode::AccessOam;
                }
            }
            None
        }
    }
}

impl Gpu {
    pub fn initialize() -> Self {
        Self {
            scan_mode: ScanMode::HorizontalBlank,
            mode_clock: 0,
            line: 0,
            framebuffer: gen_framebuffer(),
            window_line: 0,
        }
    }

    pub fn step(
        &mut self,
        time_increment: TimeIncrement,
        memory: &mut Box<dyn MemoryAccess>,
    ) -> Option<Framebuffer> {
        let lcdc = memory.read_byte(0xFF40);
        if lcdc & 0x80 == 0 {
            // LCD is disabled: freeze LY at 0, halt GPU, mode stays at HBlank
            // Real GB hardware stops all GPU timing when LCD bit 7 = 0
            self.line = 0;
            self.mode_clock = 0;
            self.window_line = 0;
            self.scan_mode = ScanMode::HorizontalBlank;
            memory.write_byte(0xFF44, 0);
            return None;
        }
        // Write LY so CPU can read it; do NOT write scroll/palette registers back
        // (the CPU/game writes those, we only read them)
        memory.write_byte(0xFF44, self.line);
        step_mode(self, memory, time_increment)
    }
}

#[derive(Debug)]
enum ScanMode {
    AccessOam,
    AccessVram,
    HorizontalBlank,
    VerticalBlank,
}
