# Game Boy Emulator

A Game Boy emulator written in Rust.

I started working on this in 2023 as a hobby project, getting far enough to see the Nintendo boot screen, before shelving it. In 2026, I picked it back up again and used Claude Sonnet 4.6 and Gemini 3.1 Pro (High), in the Antigravity harness. They picked up where I left off and finished the CPU, GPU, and APU to the point of running full commercial games with input and sound.

## Building

Requires Rust and SDL2.

```bash
# macOS (Homebrew)
brew install sdl2

# Build (debug)
LIBRARY_PATH=/opt/homebrew/lib cargo build

# Build (release — much faster)
LIBRARY_PATH=/opt/homebrew/lib cargo build --release
```

## Running

```bash
# Debug build
LIBRARY_PATH=/opt/homebrew/lib ./target/debug/emulator roms/tetris.rom

# Release build
LIBRARY_PATH=/opt/homebrew/lib ./target/release/emulator roms/kirby_dream_land_game.rom
```

Place your ROM files in the `roms/` directory.

## Controls

| Key | Game Boy |
|-----|----------|
| Arrow keys | D-pad |
| `Z` | A button |
| `X` | B button |
| `Return` | Start |
| `Backspace` | Select |
| `Escape` | Quit |

## Supported Features

- **CPU**: Full LR35902 instruction set with correct flag behavior
- **GPU**: Background, Window, and Sprite (OBJ) layers; OAM DMA
- **APU**: All 4 audio channels (square ×2, wave table, noise) with envelope, sweep, and length counters
- **MBC1**: ROM bank switching (supports ROMs up to ~2MB)
- **Joypad**: D-pad and buttons via keyboard
- **BIOS**: DMG boot ROM (splash screen + header verification)
- **Timing**: VBlank-driven main loop capped at 59.7fps

## Architecture

```
src/
  cpu.rs     — LR35902 CPU: instruction table, execute/step, interrupt handling
  gpu.rs     — PPU: BG/Window/Sprite rendering, scanline timing, VBlank
  apu.rs     — APU: square wave, wave table, noise channels; stereo mixer; DC filter
  memory.rs  — Memory map, MBC1 bank switching, OAM DMA, joypad register
  main.rs    — SDL2 window + audio, frame-driven main loop
  lib.rs     — WASM bindings (WebAssembly frontend)
```

## Notes

- `docs/` contains development notes, debugging walkthrough, and BIOS disassembly reference
