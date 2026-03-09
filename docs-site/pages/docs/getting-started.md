---
template: docs.html
title: Getting Started
---

# Getting Started

## Using the browser IDE

The fastest way to try Shrimp is the online IDE at [polarbeardomestication.net/shrimp](https://polarbeardomestication.net/shrimp/).

1. Open the IDE
2. Write your code in the editor (or pick a demo from the 🦐 **Demos** menu)
3. Click **▶ Run**
4. Your game compiles and runs instantly in the emulator

### Controls

| Key | Game Boy |
|------|---------|
| Arrow keys | D-pad |
| `Z` | A button |
| `X` | B button |
| `Enter` | Start |
| `Shift` | Select |

## Using the CLI

You can also compile Shrimp programs from the command line.

### Build the compiler

```
cargo build -p compiler --release
```

### Compile a program

```
./target/release/shrimp games/pong.s -o games/pong.gb
```

### Run it

Load the `.gb` file in any Game Boy emulator, or use the built-in web emulator.

## Program structure

Every Shrimp program follows this structure:

```
from core import pressed, set_sprite, Button

tile my_tile:
    ........
    .333333.
    .3....3.
    .3....3.
    .3....3.
    .3....3.
    .333333.
    ........

let x = 80
let y = 72

init:
    set_sprite(0, x, y, my_tile)

on vblank:
    # game logic runs here, ~60 times per second
    set_sprite(0, x, y, my_tile)
```

### Key sections

- **Imports** — bring in built-in functions from `core`
- **Tiles** — define 8×8 pixel art inline
- **Global variables** — state that persists across frames
- **`init:`** — runs once at startup
- **`on vblank:`** — runs every frame (~59.7 fps)
