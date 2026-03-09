---
template: docs.html
title: Overview
---

# 🦐 Shrimp

Shrimp is a high-level scripting language for writing Game Boy games. It compiles directly to LR35902 machine code — the CPU inside the original Game Boy — and produces standard `.gb` ROM files.

## What you can do

- **Define pixel art** as tile literals right in your source code
- **Handle input** with `pressed()` and `just_pressed()`
- **Draw sprites and backgrounds** with `set_sprite()` and `set_bg_tile()`
- **Write game logic** with variables, loops, conditionals, and functions
- **Run instantly** in the browser IDE with a built-in emulator

## Quick taste

```
from core import pressed, set_sprite, Button

tile ball:
    ..33....
    .3333...
    33333333
    33333333
    .3333...
    ..33....
    ........
    ........

let x = 80

init:
    set_sprite(0, x, 72, ball)

on vblank:
    if pressed(Button.Right):
        x := x + 1
    if pressed(Button.Left):
        x := x - 1
    set_sprite(0, x, 72, ball)
```

## How it works

1. You write `.s` source files in the Shrimp language
2. The compiler tokenizes, parses, resolves symbols, and emits LR35902 machine code
3. A ROM writer packages the code + tile data into a valid 32KB Game Boy ROM
4. The emulator (or any Game Boy emulator) runs it

## Next steps

- [Getting Started](getting-started.html) — write your first program
- [Syntax](syntax.html) — full language reference
- [Built-in Functions](builtins.html) — sprites, tiles, input, and scrolling
- [Examples](examples.html) — Pong and Platformer walkthroughs
