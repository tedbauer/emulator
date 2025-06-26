# Emulator

Gameboy emulator.

```
brew install sdl2
```

## Debugging tools

- Press `t` to generate `debug/tiles.png`, an image dump of the tileset loaded in memory.
- Press `m` to generate `debug/memory.txt`, a human-readable dump of the entire memory.

See hexdump of BIOS:

```sh
cat roms/bios.rom | hexdump
```
