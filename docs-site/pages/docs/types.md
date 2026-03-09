---
template: docs.html
title: Types
---

# Types

Shrimp is statically typed with type inference. Variables declared with `let` get their type from the initializer, or you can annotate explicitly.

## Available types

| Type | Size | Range | Use case |
|------|------|-------|----------|
| `u8` | 8 bits | 0 – 255 | Positions, counters, tile indices |
| `i8` | 8 bits | -128 – 127 | Velocities, signed offsets |
| `u16` | 16 bits | 0 – 65535 | Larger values |
| `bool` | 8 bits | `true` / `false` | Flags, conditions |

## Type inference

```
let x = 10          # u8 (positive integer literal)
let y = -3          # i8 (negative integer literal)
let big = 300       # u16 (value > 255)
let flag = true     # bool
```

## Explicit annotations

```
let speed: i8 = 0       # force signed even though 0 is positive
let counter: u16 = 0    # force 16-bit width
let on_ground: bool = false
```

## Overflow behavior

All arithmetic wraps on overflow, matching the Game Boy's native 8-bit behavior:

- `255 + 1` wraps to `0` for `u8`
- `127 + 1` wraps to `-128` for `i8`

This is useful for screen wrapping — a sprite that moves past x=160 will wrap around naturally.
