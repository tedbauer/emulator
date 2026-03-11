---
template: docs.html
title: Syntax
---

# Syntax Reference

Shrimp uses indentation-based blocks (like Python). Statements are separated by newlines. Comments start with `#`.

## Imports

```
from core import pressed, set_sprite, set_bg_tile, Button
```

The only module is `core`, which provides all built-in functions and the `Button` enum.

## Variables

### Declaration

```
let x = 10           # u8, inferred
let speed: i8 = -2   # explicit signed type
let flag: bool = true
```

Variables are always global. Type is inferred from the initializer unless explicitly annotated.

## Constants

```
const MAX_SPEED = 4
const SCENE_TOWN = 0
const TILE_SIZE = 8
```

Constants are inlined at compile time — no RAM is used. They must be integer literals. Use constants for magic numbers, state machine values, and configuration.

## Arrays

```
let map: [u8 * 20] = 0     # 20-byte array, initialized to 0
let grid: [u8 * 100] = 0
```

Access and assign by index:

```
map[5] := 42
let val = map[x]
```

Arrays are stored as contiguous WRAM. Index expressions can be variables or constants. Note the syntax uses `*` for array size: `[u8 * N]`.

### Assignment

```
x := x + 1
speed := -speed
```

Use `:=` for reassignment (not `=`).

## Operators

### Arithmetic

| Operator | Meaning |
|----------|---------|
| `+` | Add |
| `-` | Subtract (or unary negate) |
| `*` | Multiply |
| `/` | Divide (power-of-2 only) |
| `%` | Modulo (power-of-2 only) |

> `/` and `%` currently require the right-hand side to be a constant power of 2 (1, 2, 4, 8, 16, …). For example: `x / 8` and `frame % 4`.

### Comparison

| Operator | Meaning |
|----------|---------|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `<=` | Less or equal |
| `>` | Greater than |
| `>=` | Greater or equal |

### Bitwise

| Operator | Meaning |
|----------|---------|
| `&` | Bitwise AND |
| `\|` | Bitwise OR |
| `<<` | Shift left (constant count only) |
| `>>` | Shift right (constant count only) |

```
let masked = flags & 3
let high = value >> 4
let packed = base | (1 << 2)
```

> `<<` and `>>` require a constant shift amount. For example: `x << 3` or `val >> 2`.

### Logical

| Operator | Meaning |
|----------|---------|
| `and` | Logical AND |
| `or` | Logical OR |
| `not` | Logical NOT (unary) |

## Control flow

### If / elif / else

```
if x > 100:
    x := 0
elif x > 50:
    speed := 1
else:
    speed := 2
```

### Match / Case

```
match scene:
    case SCENE_TOWN:
        check_town()
    case SCENE_HOUSE:
        check_house()
    else:
        pass
```

Compares one expression against multiple constants. `else:` is optional (default branch). Cleaner than chains of `if`/`elif`.

### While loop

```
let i = 0
while i < 10:
    set_bg_tile(i, 0, my_tile)
    i := i + 1
```

### Loop (infinite)

```
loop:
    # runs forever
    x := x + 1
```

## Functions

```
fn clamp(val: u8, max: u8) -> u8:
    if val > max:
        return max
    return val
```

- Parameters must have type annotations
- Return type is specified with `->` (optional if no return value)
- Functions are called like: `clamp(x, 160)`
- Local `let` variables inside functions are **scoped** — `let tx` in two different functions won't collide

## Init and VBlank

```
init:
    # runs once at startup
    set_sprite(0, 80, 72, player)

on vblank:
    # runs every frame (~59.7 fps)
    # this is where your game loop goes
    x := x + 1
    set_sprite(0, x, 72, player)
```

The `init:` block runs once when the ROM starts. The `on vblank:` block is called by the GPU at the end of every frame.

## Comments

```
# This is a comment
let x = 10  # inline comment
```
