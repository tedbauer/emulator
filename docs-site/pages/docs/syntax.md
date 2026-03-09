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
| `/` | Divide |
| `%` | Modulo |

### Comparison

| Operator | Meaning |
|----------|---------|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `<=` | Less or equal |
| `>` | Greater than |
| `>=` | Greater or equal |

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
