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
