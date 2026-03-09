# Game Boy Emulator — Debugging Session Walkthrough

## Where We Started

The emulator had a working skeleton: BIOS boot, a basic CPU with most opcodes, a GPU stub with BG tile rendering, and SDL2 for display. It could run [snake.rom](file:///Users/theodorebauer/Documents/emulator/roms/snake.rom) and display the Nintendo logo scroll animation via the BIOS. However:

- Tetris ([tetris.rom](file:///Users/theodorebauer/Documents/emulator/roms/tetris.rom)) → showed a **white screen** after the logo
- Kirby Dream Land ([kirby_dream_land_game.rom](file:///Users/theodorebauer/Documents/emulator/roms/kirby_dream_land_game.rom)) → showed **nothing at all**
- No ROM bank switching (MBC1) was implemented
- The ROM path was hardcoded
- The window was a small fixed size
- Input was mapped but never verified

---

## What We Fixed — In Order

### 1. QoL: ROM Path as CLI Argument + Window Scale

The ROM was hardcoded. We added a command-line argument so you can run:
```
./target/debug/emulator roms/tetris.rom
```
Also increased the window scale from 1× to 3× (480×432).

---

### 2. CPU Arithmetic Panics (`wrapping_sub`, half-carry flags)

Rust's debug builds panic on integer overflow. Several DEC/SUB instructions used plain `-` instead of `.wrapping_sub()`. Also corrected the half-carry flag logic for subtraction (based on bit 4 borrow, not bit 3).

---

### 3. Tetris White Screen — Part 1: LCDC Bit 7 / GPU Halt

Tetris disables the LCD (`LCDC = 0x03`, bit 7 = 0) during VRAM initialization. On real hardware, the GPU **completely stops** when the LCD is off — no VBlank fires, LY stays at 0. Our GPU kept ticking and generating interrupts through LCD-off periods. Fixed the GPU [step()](file:///Users/theodorebauer/Documents/emulator/src/gpu.rs#264-285) to freeze all state when `LCDC bit 7 = 0`.

---

### 4. MBC1 Bank Switching (for Kirby)

Kirby Dream Land uses a 256KB ROM with Memory Bank Controller 1 (MBC1). We implemented:
- Detecting the cartridge type from ROM header byte `0x0147`
- Reads from `0x4000–0x7FFF` now route through the active `rom_bank`
- Writes to `0x2000–0x3FFF` update `rom_bank` (clamped: bank 0 → bank 1)

---

### 5. Tetris White Screen — Part 2: `ADD HL,DE` Was a Stub ⭐

This was the **root cause** of Tetris's persistent white screen. After extensive tracing we found that Tetris uses a `RST 0x28` jump table pattern to dispatch game state:

```asm
; RST 0x28 vector at 0x0028:
ADD A,A       ; index *= 2
POP HL        ; HL = return address = table base
LD E,A        ; DE = byte offset
LD D,$00
ADD HL,DE     ; ← THIS WAS A STUB (only incremented PC)
LD E,(HL)     ; read function pointer from table
INC HL
LD D,(HL)
PUSH DE / POP HL / JP (HL)
```

Because `ADD HL,DE` (opcode `0x19`) was an empty stub, `HL` never moved. The dispatcher **always called table entry 0** regardless of the actual state index in `FFE1`. With `FFE1=0x24` (=36), it should have been calling the VRAM-init + LCDC-enable function at `0x0369`; instead it hit a function that immediately returned because of an unmet condition.

Also fixed `ADD HL,BC` (`0x09`) which was missing overflow-safe arithmetic and carry flags.

---

### 6. Kirby Black Screen — Critical WRAM Write Bug ⭐

[write_byte](file:///Users/theodorebauer/Documents/emulator/src/memory.rs#120-161) computed the `the_rest` index as:
```rust
let rest_idx = addr.saturating_sub(rom_len);  // WRONG
```
For Kirby's 256KB ROM, `rom_len = 0x40000`. Any write to WRAM (`0xC000`), VRAM (`0x8000`), HRAM (`0xFF80`), etc. would underflow and saturate to **index 0** — all writes clobbered the same single byte. VRAM, WRAM, OAM, and all IO registers were effectively read-only. Fixed to always use `0x8000` as the base:
```rust
let rest_idx = addr - 0x8000;  // correct, same as read_byte
```

---

### 7. Joypad Register (0xFF00)

The input system was wired but the joypad select bit logic was inverted at one point during debugging. Settled on the hardware-correct behavior:
- **P15 (bit 5) = 0**: selects button row (A, B, Select, Start)
- **P14 (bit 4) = 0**: selects d-pad row (Right, Left, Up, Down)

Key mapping:
| Key | Game Boy Button |
|-----|----------------|
| Arrow keys | D-pad |
| `Z` | A |
| `X` | B |
| `Return` | Start |
| `Backspace` | Select |

---

### 8. OAM DMA Transfer (0xFF46) ⭐

Sprites (Kirby, Tetris falling block) were completely invisible despite background tiles rendering correctly. The cause: **OAM DMA was not implemented**.

Games write a source address high byte to `0xFF46` to trigger an instant 160-byte copy from [(value << 8)](file:///Users/theodorebauer/Documents/emulator/src/cpu.rs#109-127) into OAM at `0xFE00`. Without it, all 40 sprite entries had `Y=0` → screen position `−16` → every sprite off-screen.

Implemented in [write_byte](file:///Users/theodorebauer/Documents/emulator/src/memory.rs#120-161):
```rust
} else if addr == 0xFF46 {
    let src = (value as u16) << 8;
    for i in 0..160u16 {
        let byte = self.read_byte(src + i);
        self.the_rest[(0xFE00 + i as usize) - 0x8000] = byte;
    }
}
```

---

### 9. Performance: Frame-Driven Main Loop

The original main loop ran **one CPU instruction per SDL event poll** — extremely slow and timing-inaccurate. Restructured to:

1. Poll SDL events **once per frame**
2. Run CPU + GPU steps in a tight inner loop **until VBlank** produces a complete frame
3. Render and blit that frame
4. Sleep if needed to cap at **59.7fps** (Game Boy native)

```rust
let framebuffer = loop {
    let (dt, _) = cpu.step(&mut memory);
    cpu.handle_interrupts(&mut memory);
    if let Some(fb) = gpu.step(dt, &mut memory) { break fb; }
};
// sleep to ~16.74ms per frame
```

---

## Final State

| ROM | Before | After |
|-----|--------|-------|
| [tetris.rom](file:///Users/theodorebauer/Documents/emulator/roms/tetris.rom) | White screen | ✅ Loads title screen, game plays, pieces visible |
| [kirby_dream_land_game.rom](file:///Users/theodorebauer/Documents/emulator/roms/kirby_dream_land_game.rom) | Black screen | ✅ Nintendo logo, title screen, level loads with Kirby visible |

Both games run at native Game Boy speed (~59.7fps) with working input.

---

## Debugging Techniques Used

- **CPU step sampler** — print `PC / LCDC / LY / FF85` every N million cycles to find stall points
- **Write tracing** — `eprintln!` in [write_byte](file:///Users/theodorebauer/Documents/emulator/src/memory.rs#120-161) for specific addresses (`0xFF85`, `0xDF7F`, `0xFFCC`) to track game state variable writes
- **PC traps** — trap first hit of specific addresses (`0x0BDE`, `0x0B94`) to confirm code paths execute
- **ROM static disassembly** — Python scripts to decode instruction streams directly from [.rom](file:///Users/theodorebauer/Documents/emulator/roms/bios.rom) files to understand game logic without a reference emulator

