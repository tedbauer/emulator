---
template: docs.html
title: Built-in Functions
---

# Built-in Functions

All built-ins are imported from the `core` module:

```
from core import pressed, just_pressed, set_sprite, set_bg_tile, set_scroll, Button
```

---

## Input

### `pressed(button) -> bool`

Returns `true` if the button is **currently held down**.

```
if pressed(Button.Right):
    x := x + 1
```

### `just_pressed(button) -> bool`

Returns `true` only on the **first frame** the button is pressed. Useful for actions that should trigger once (like jumping).

```
if just_pressed(Button.A):
    vy := -8
```

### Button enum

| Value | Game Boy button |
|-------|----------------|
| `Button.Up` | D-pad up |
| `Button.Down` | D-pad down |
| `Button.Left` | D-pad left |
| `Button.Right` | D-pad right |
| `Button.A` | A button |
| `Button.B` | B button |
| `Button.Start` | Start |
| `Button.Select` | Select |

---

## Sprites

### `set_sprite(index, x, y, tile)`

Places a sprite on screen.

| Param | Type | Description |
|-------|------|-------------|
| `index` | `u8` | Sprite slot (0–39) |
| `x` | `u8` | X position in pixels |
| `y` | `u8` | Y position in pixels |
| `tile` | tile name | Which tile graphic to use |

```
set_sprite(0, 80, 72, player)
```

> The Game Boy can display up to 40 sprites, with a limit of 10 per scanline. Sprite coordinates place the top-left corner of the 8×8 tile at (x, y). A sprite at (0, 0) is at the top-left of the screen.

---

## Background

### `set_bg_tile(tx, ty, tile)`

Places a tile on the background layer.

| Param | Type | Description |
|-------|------|-------------|
| `tx` | `u8` | Tile column (0–31) |
| `ty` | `u8` | Tile row (0–31) |
| `tile` | tile name | Which tile graphic to draw |

```
# Fill row 16 with ground tiles
let i = 0
while i < 20:
    set_bg_tile(i, 16, ground)
    i := i + 1
```

> The background is a 32×32 grid of 8×8 tiles. The visible screen shows a 20×18 window into this grid (160×144 pixels). Use `set_scroll` to pan the view.

---

## Scrolling

### `set_scroll(sx, sy)`

Sets the background scroll position.

| Param | Type | Description |
|-------|------|-------------|
| `sx` | `u8` | Horizontal scroll (pixels) |
| `sy` | `u8` | Vertical scroll (pixels) |

```
let scroll_x = 0

on vblank:
    scroll_x := scroll_x + 1
    set_scroll(scroll_x, 0)
```

> Scrolling wraps around at 256 pixels in both directions. The background layer scrolls independently of sprites.

---

## Text

### `print(tx, ty, "text")`

Renders a string to the background layer using a **built-in 8×8 ASCII font** (96 characters, printable ASCII 32–127).

| Param | Type | Description |
|-------|------|-------------|
| `tx` | `u8` | Starting tile column |
| `ty` | `u8` | Tile row |
| `"text"` | string literal | Text to display |

```
print(2, 4, "HELLO WORLD!")
print(3, 8, "Score: 0")
```

> The built-in font is loaded automatically when `print()` is used. No tile definitions or imports are needed. The font occupies tiles after your user-defined tiles.

---

## Large Sprites

### `set_sprite_16(index, x, y, top_tile, bottom_tile)`

Places a 16-pixel-tall sprite using two consecutive OAM slots.

| Param | Type | Description |
|-------|------|-------------|
| `index` | `u8` | Sprite slot for top half (bottom = index+1) |
| `x` | `u8` | X position |
| `y` | `u8` | Y position of top half |
| `top_tile` | tile name | Top 8×8 tile |
| `bottom_tile` | tile name | Bottom 8×8 tile |

```
set_sprite_16(0, px, py, player_top, player_bottom)
```

> Uses two standard sprite slots. The bottom half is drawn 8 pixels below the top.

---

## Tile Expressions

Tile arguments in `set_sprite()`, `set_bg_tile()`, etc. accept **any expression**, not just tile names:

```
# Tile name (resolves to its index)
set_bg_tile(x, y, grass)

# Arithmetic on tile indices
set_bg_tile(x, y, grass + 1)

# Numeric tile index directly
set_bg_tile(x, y, 5)
```

This lets you compute tile indices at runtime for animation frames, tile variations, or font rendering.

---

## Sound

### `beep()`

Plays a short blip sound (A5, ~880 Hz) with fast volume decay. Good for UI feedback.

```
beep()  # plays a quick blip
```

### `play_tone(freq_lo, freq_hi)`

Plays a tone on the Game Boy's Channel 1 square wave. Takes the raw 11-bit frequency register value split into low and high bytes.

| Param | Type | Description |
|-------|------|-------------|
| `freq_lo` | `u8` | Low 8 bits of frequency register |
| `freq_hi` | `u8` | High 3 bits of frequency register (0-7) |

```
play_tone(0x6B, 0x07)   # ~880 Hz (A5)
play_tone(0x0C, 0x06)   # ~262 Hz (C4)
```

The frequency formula is: `freq_register = 2048 - (131072 / Hz)`. Common note values:

| Note | Hz | freq_lo | freq_hi |
|------|----|---------|---------|
| C4 | 262 | 0x0C | 0x06 |
| D4 | 294 | 0x42 | 0x06 |
| E4 | 330 | 0x73 | 0x06 |
| G4 | 392 | 0xB2 | 0x06 |
| A4 | 440 | 0xD6 | 0x06 |
| C5 | 523 | 0x05 | 0x07 |
| E5 | 659 | 0x4E | 0x07 |
| A5 | 880 | 0x6B | 0x07 |
