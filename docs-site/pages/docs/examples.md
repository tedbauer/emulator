---
template: docs.html
title: Examples
---

# Examples

All of these demos are available in the Shrimp IDE under the **🦐 Demos** menu.

## Hello World

Display text on the Game Boy using custom letter tiles on the background layer.

```
from core import set_bg_tile

tile letter_h:
    1...1...
    1...1...
    1...1...
    11111...
    1...1...
    1...1...
    1...1...
    ........

tile letter_e:
    11111...
    1.......
    1.......
    1111....
    1.......
    1.......
    11111...
    ........

tile letter_l:
    1.......
    1.......
    1.......
    1.......
    1.......
    1.......
    11111...
    ........

tile letter_o:
    .111....
    1...1...
    1...1...
    1...1...
    1...1...
    1...1...
    .111....
    ........

init:
    set_bg_tile(6, 8, letter_h)
    set_bg_tile(7, 8, letter_e)
    set_bg_tile(8, 8, letter_l)
    set_bg_tile(9, 8, letter_l)
    set_bg_tile(10, 8, letter_o)
```

### Key concepts used

- **Background tiles**: text is rendered by placing letter tiles on the BG grid
- **No `on vblank` needed**: static content only requires `init:`
- The Game Boy has no built-in font — all text must be drawn with tiles

---

## Pong

A playable Pong game in ~50 lines. Arrow keys move the paddle.

```
from core import pressed, set_sprite, set_bg_tile, Button

tile ball:
    ..33....
    .3333...
    33333333
    33333333
    .3333...
    ..33....
    ........
    ........

tile paddle:
    33333333
    33333333
    33333333
    33333333
    33333333
    33333333
    33333333
    33333333

let bx = 80
let by = 72
let bvx: i8 = 1
let bvy: i8 = 1
let py = 120

init:
    set_sprite(0, bx, by, ball)
    set_sprite(1, py, 136, paddle)

on vblank:
    bx := bx + bvx
    by := by + bvy

    if bx > 155:
        bvx := -1
    if bx < 5:
        bvx := 1
    if by < 5:
        bvy := 1

    # Paddle collision
    if by > 130:
        if bx > py - 4:
            if bx < py + 12:
                bvy := -1

    # Ball falls off bottom — reset
    if by > 144:
        by := 10
        bvy := 1

    # Paddle movement
    if pressed(Button.Left):
        py := py - 2
    if pressed(Button.Right):
        py := py + 2

    set_sprite(0, bx, by, ball)
    set_sprite(1, py, 136, paddle)
```

### Key concepts used

- **Two sprites**: ball (slot 0) and paddle (slot 1)
- **Signed velocity** (`i8`): `bvx` and `bvy` can be negative
- **Collision detection**: simple bounding box check
- **Input polling**: `pressed()` for continuous paddle movement

---

## Platformer

A simple platformer with gravity, jumping, and floating platforms.

```
from core import pressed, set_sprite, set_bg_tile, Button

tile player:
    ..3333..
    .333333.
    .3.33.3.
    .333333.
    ..3333..
    .3.33.3.
    .3....3.
    ........

tile ground:
    33333333
    22222222
    22222222
    22222222
    22222222
    22222222
    22222222
    22222222

tile platform:
    33333333
    33333333
    ........
    ........
    ........
    ........
    ........
    ........

let px = 80
let py = 100
let vy: i8 = 0
let on_ground = 0
let jump_lock = 0

init:
    # Draw ground across bottom
    let i = 0
    while i < 20:
        set_bg_tile(i, 16, ground)
        i := i + 1

    # Floating platforms
    set_bg_tile(2, 12, platform)
    set_bg_tile(3, 12, platform)
    set_bg_tile(4, 12, platform)

    set_bg_tile(8, 10, platform)
    set_bg_tile(9, 10, platform)
    set_bg_tile(10, 10, platform)

    set_bg_tile(14, 8, platform)
    set_bg_tile(15, 8, platform)
    set_bg_tile(16, 8, platform)

    set_sprite(0, px, py, player)

on vblank:
    # Gravity
    vy := vy + 1
    if vy > 4:
        vy := 4
    py := py + vy

    # Ground collision
    if py > 120:
        py := 120
        vy := 0
        on_ground := 1
    
    # Platform collisions
    if jump_lock == 0:
        if vy > 0:
            # ... platform check logic
            pass

    # Horizontal movement
    if pressed(Button.Left):
        px := px - 1
    if pressed(Button.Right):
        px := px + 1

    # Jump
    if on_ground == 1:
        if pressed(Button.A):
            vy := -8
            on_ground := 0
            jump_lock := 12

    if jump_lock > 0:
        jump_lock := jump_lock - 1

    set_sprite(0, px, py, player)
```

### Key concepts used

- **Gravity**: `vy` increases each frame, simulating falling
- **Platform collision**: check if the player's feet align with a platform row
- **Jump lock**: prevents immediately locking onto a platform while jumping upward
- **Background tiles**: ground and platforms are drawn with `set_bg_tile()`
- **Sprite**: the player character is a single sprite
