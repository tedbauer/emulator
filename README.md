# Game Boy Emulator

A Game Boy emulator written in Rust.

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

Place your ROM files in the `roms/` directory (not included — provide your own legally-obtained copies).

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
- **MBC1**: ROM bank switching (supports ROMs up to ~2MB)
- **Joypad**: D-pad and buttons via keyboard
- **BIOS**: DMG boot ROM (splash screen + header verification)
- **Timing**: VBlank-driven main loop capped at 59.7fps

## Architecture

```
src/
  cpu.rs     — LR35902 CPU: instruction table, execute/step, interrupt handling
  gpu.rs     — PPU: BG/Window/Sprite rendering, scanline timing, VBlank
  memory.rs  — Memory map, MBC1 bank switching, OAM DMA, joypad register
  main.rs    — SDL2 window, frame-driven main loop
  lib.rs     — WASM bindings (WebAssembly frontend)
```

## Notes

- `docs/` contains development notes, debugging walkthrough, and BIOS disassembly reference
