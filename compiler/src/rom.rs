//! ROM writer: assembles the full 32KB Game Boy binary from its parts.
//!
//! Memory map of generated ROM:
//!   $0000-$003F  Interrupt vectors (RETI stubs + VBlank dispatch)
//!   $0040        VBlank ISR stub → calls user vblank handler
//!   $0100-$0103  Cartridge entry point (NOP; JP $0150)
//!   $0104-$0133  Nintendo logo
//!   $0134-$014F  ROM header
//!   $0150        LCD init, tile VRAM copy, OAM clear, user init call, EI, HALT loop
//!   $0200+       Compiled user code (init fn + vblank fn + builtins)
//!   Near end     Tile pixel data

const ROM_SIZE: usize = 32768;

/// The 48-byte Nintendo logo that must appear at $0104–$0133.
const NINTENDO_LOGO: [u8; 48] = [
    0xCE,0xED,0x66,0x66,0xCC,0x0D,0x00,0x0B,
    0x03,0x73,0x00,0x83,0x00,0x0C,0x00,0x0D,
    0x00,0x08,0x11,0x1F,0x88,0x89,0x00,0x0E,
    0xDC,0xCC,0x6E,0xE6,0xDD,0xDD,0xD9,0x99,
    0xBB,0xBB,0x67,0x63,0x6E,0x0E,0xEC,0xCC,
    0xDD,0xDC,0x99,0x9F,0xBB,0xB9,0x33,0x3E,
];

pub struct RomWriter {
    rom: [u8; ROM_SIZE],
}

impl RomWriter {
    pub fn new() -> Self {
        RomWriter { rom: [0u8; ROM_SIZE] }
    }

    /// Write a single byte at an absolute ROM address.
    fn w(&mut self, addr: usize, val: u8) {
        assert!(addr < ROM_SIZE, "ROM address out of range: ${:04X}", addr);
        self.rom[addr] = val;
    }

    fn w16(&mut self, addr: usize, val: u16) {
        self.w(addr,     (val & 0xFF) as u8);
        self.w(addr + 1, (val >> 8)   as u8);
    }

    fn write_slice(&mut self, addr: usize, data: &[u8]) {
        for (i, &b) in data.iter().enumerate() {
            self.w(addr + i, b);
        }
    }

    /// Build the complete ROM.
    ///
    /// * `init_code`   — compiled bytes for `init:` block + vblank handler
    /// * `tile_data`   — 2bpp tile bytes (16 bytes per tile)
    /// * `n_tiles`     — number of tiles
    /// * `has_vblank`  — whether to wire up the VBlank ISR
    pub fn build(
        &mut self,
        game_code: &[u8],
        tile_data: &[u8],
        has_vblank: bool,
    ) -> &[u8] {
        const VBLANK_ISR:    usize = 0x0040;
        const ENTRY_POINT:   usize = 0x0100;
        const LOGO:          usize = 0x0104;
        const TITLE:         usize = 0x0134;
        const CART_TYPE:     usize = 0x0147;
        const ROM_SIZE_BYTE: usize = 0x0148;
        const RAM_SIZE_BYTE: usize = 0x0149;
        const DEST_CODE:     usize = 0x014A;
        const OLD_LIC:       usize = 0x014B;
        const HDR_CKSUM:     usize = 0x014D;
        const SETUP_START:   usize = 0x0150;
        const GAME_CODE_START: usize = 0x0200;

        // ── Interrupt vectors ──────────────────────────────────────────────
        // $0000-$003F: all RETI (RST handlers; we don't use them)
        for i in (0..0x40).step_by(8) {
            self.w(i,     0xD9); // RETI
            for j in 1..8 { self.w(i + j, 0x00); }
        }

        // $0040: VBlank ISR
        if has_vblank {
            // PUSH AF; PUSH BC; PUSH DE; PUSH HL; CALL user_vblank; POP HL; POP DE; POP BC; POP AF; RETI
            let vblank_fn_addr = (GAME_CODE_START + find_label_offset(game_code, "__vblank_fn")) as u16;
            let code: &[u8] = &[
                0xF5,       // PUSH AF
                0xC5,       // PUSH BC
                0xD5,       // PUSH DE
                0xE5,       // PUSH HL
                0xCD,       // CALL nn
                (vblank_fn_addr & 0xFF) as u8,
                (vblank_fn_addr >> 8)   as u8,
                0xE1,       // POP HL
                0xD1,       // POP DE
                0xC1,       // POP BC
                0xF1,       // POP AF
                0xD9,       // RETI
            ];
            self.write_slice(VBLANK_ISR, code);
        } else {
            self.w(VBLANK_ISR, 0xD9); // RETI
        }

        // ── Entry point ($0100) ───────────────────────────────────────────
        // NOP; JP $0150
        self.w(ENTRY_POINT,     0x00); // NOP
        self.w(ENTRY_POINT + 1, 0xC3); // JP nn
        self.w16(ENTRY_POINT + 2, SETUP_START as u16);

        // ── Nintendo logo ($0104-$0133) ───────────────────────────────────
        self.write_slice(LOGO, &NINTENDO_LOGO);

        // ── Title ($0134-$013E) ───────────────────────────────────────────
        let title = b"GBSCRIPT    ";
        self.write_slice(TITLE, &title[..11]);

        // ── Cartridge type / size / dest ─────────────────────────────────
        self.w(CART_TYPE,     0x00); // ROM only
        self.w(ROM_SIZE_BYTE, 0x00); // 32KB
        self.w(RAM_SIZE_BYTE, 0x00); // no RAM
        self.w(DEST_CODE,     0x01); // non-Japanese
        self.w(OLD_LIC,       0x33); // use new licensee

        // ── Header checksum ($014D) ────────────────────────────────────────
        let cksum = self.rom[0x0134..=0x014C]
            .iter()
            .fold(0u8, |acc, &b| acc.wrapping_sub(b).wrapping_sub(1));
        self.w(HDR_CKSUM, cksum);

        // ── Setup code ($0150) ────────────────────────────────────────────
        // DI; LD SP,$FFFE; turn off LCD; copy tiles; clear OAM; load palettes;
        // enable LCD; call user init; EI; HALT loop.
        const VRAM_TILE_BASE: u16 = 0x8000;
        let n_tiles = tile_data.len() / 16;
        let tile_end_addr = VRAM_TILE_BASE + (tile_data.len() as u16);

        let mut setup: Vec<u8> = vec![];
        // DI
        setup.push(0xF3);
        // LD SP, $FFFE
        setup.extend_from_slice(&[0x31, 0xFE, 0xFF]);
        // Wait for VBlank before touching VRAM: poll LY >= 144
        //   vblank_wait: LD A,(FF44); CP 144; JR C, vblank_wait
        setup.extend_from_slice(&[0xF0, 0x44, 0xFE, 0x90, 0x38, 0xFB]); // LDH A,(44); CP 144; JR C,-5
        // LCD off: LD A,0; LD (FF40), A
        setup.extend_from_slice(&[0x3E, 0x00, 0xE0, 0x40]);

        if !tile_data.is_empty() {
            // Copy tile data to VRAM $8000
            // LD HL, tile_data_rom_addr; LD DE, $8000; LD BC, len; CALL __memcpy
            let tile_rom_addr = ROM_SIZE as u16 - tile_data.len() as u16;
            setup.extend_from_slice(&[0x21]);
            setup.extend_from_slice(&tile_rom_addr.to_le_bytes());
            setup.extend_from_slice(&[0x11, 0x00, 0x80]); // LD DE, $8000
            let len = tile_data.len() as u16;
            setup.extend_from_slice(&[0x01]);
            setup.extend_from_slice(&len.to_le_bytes());  // LD BC, len
            // Inline memcpy loop:
            //   copy_loop: LD A,(HL+); LD (DE),A; INC DE; DEC BC; LD A,B; OR C; JR NZ,copy_loop
            setup.extend_from_slice(&[0x2A, 0x12, 0x13, 0x0B, 0x78, 0xB1, 0x20, 0xF9]);
        }

        // Clear OAM ($FE00-$FE9F): 160 bytes of $00
        // LD HL,$FE00; LD B,160; XOR A; oam_loop: LD (HL+),A; DEC B; JR NZ,oam_loop
        setup.extend_from_slice(&[0x21, 0x00, 0xFE, 0x06, 0xA0, 0xAF, 0x22, 0x05, 0x20, 0xFD]);

        // Set palettes: BGP=$E4, OBP0=$E4, OBP1=$E4
        setup.extend_from_slice(&[0x3E, 0xE4, 0xE0, 0x47]); // LD A,$E4; LDH (47),A  BGP
        setup.extend_from_slice(&[0xE0, 0x48]);              // LDH (48),A  OBP0
        setup.extend_from_slice(&[0xE0, 0x49]);              // LDH (49),A  OBP1

        // LCD on: LD A,$91; LDH (40),A   (LCD on, BG tile at $8000, BG on, Sprites on)
        setup.extend_from_slice(&[0x3E, 0x91, 0xE0, 0x40]);

        // Enable VBlank interrupt: LD A,$01; LD (FFFF),A
        setup.extend_from_slice(&[0x3E, 0x01, 0xEA, 0xFF, 0xFF]);

        // Call user init (at GAME_CODE_START)
        let init_addr = GAME_CODE_START as u16;
        setup.extend_from_slice(&[0xCD]);
        setup.extend_from_slice(&init_addr.to_le_bytes());

        // EI
        setup.push(0xFB);
        // Halt loop: HALT; JR -2
        setup.extend_from_slice(&[0x76, 0x18, 0xFE]);

        self.write_slice(SETUP_START, &setup);

        // ── Game code ($0200) ─────────────────────────────────────────────
        self.write_slice(GAME_CODE_START, game_code);

        // ── Tile data at end of ROM ───────────────────────────────────────
        if !tile_data.is_empty() {
            let tile_start = ROM_SIZE - tile_data.len();
            self.write_slice(tile_start, tile_data);
        }

        &self.rom
    }
}

/// Placeholder: in the current design game_code already has absolute addresses
/// embedded by the codegen; we don't need to search for a label.
/// This returns 0 because the vblank fn is just at the beginning of game_code.
fn find_label_offset(_code: &[u8], _label: &str) -> usize {
    0
}

// ─────────────────────────────────────────────────────────────────────────────
// Tile encoder: pixel grid → 2bpp Game Boy format
// ─────────────────────────────────────────────────────────────────────────────

/// Encode 8×8 pixel values (0-3 each) into 16-byte 2bpp Game Boy tile format.
/// Each row = 2 bytes: plane0 (LSB) and plane1 (MSB).
pub fn encode_tile(pixels: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::with_capacity(16);
    for row in pixels {
        let mut plane0 = 0u8;
        let mut plane1 = 0u8;
        for (i, &px) in row.iter().enumerate() {
            let bit = 7 - i;
            plane0 |= ((px & 1) as u8)       << bit;
            plane1 |= (((px >> 1) & 1) as u8) << bit;
        }
        out.push(plane0);
        out.push(plane1);
    }
    out
}
