from core import pressed, set_sprite, set_bg_tile, Button

# ── Character sprites: 4 animation frames ──────────────────

# Idle frame
tile idle:
    ..1111..
    .133331.
    .131131.
    .133331.
    ..3333..
    .133331.
    .13..31.
    ..1..1..

# Walk frame 1 (left leg forward)
tile walk1:
    ..1111..
    .133331.
    .131131.
    .133331.
    ..3333..
    .133331.
    .1.33.1.
    .1....1.

# Walk frame 2 (right leg forward)
tile walk2:
    ..1111..
    .133331.
    .131131.
    .133331.
    ..3333..
    .133331.
    .1.33.1.
    1......1

# Walk frame (arms swinging)
tile walk3:
    ..1111..
    .133331.
    .131131.
    .133331.
    ..3333..
    1.3333.1
    ..1..1..
    .1....1.

# ── World tiles ─────────────────────────────────────────────

tile grass:
    21221122
    12112211
    21221122
    12112211
    21221122
    12112211
    21221122
    12112211

tile path:
    11111111
    11111111
    11111111
    11111111
    11111111
    11111111
    11111111
    11111111

tile flowers:
    21221122
    12312211
    21221122
    12112231
    21231122
    12112211
    21221132
    12112211

tile tree_top:
    ..2222..
    .222222.
    22222222
    22322232
    22222222
    .222222.
    ..2222..
    ...33...

tile tree_trunk:
    ...33...
    ...33...
    ...33...
    ...33...
    ...33...
    ..3333..
    .222222.
    22222222

tile rock:
    ...222..
    ..22222.
    .2233222
    22233322
    22222222
    .222222.
    ..2222..
    ........

tile water:
    ........
    .1....1.
    ..1111..
    ........
    ....1...
    ..11.1..
    ........
    ........

tile fence:
    .3....3.
    .3....3.
    33333333
    .3....3.
    .3....3.
    33333333
    .3....3.
    .3....3.

# ── State ───────────────────────────────────────────────────

let px = 80
let py = 72
let frame = 0
let anim_timer = 0
let moving = 0
let dir = 0

const ANIM_SPEED = 8
const RIGHT = 0
const LEFT = 1

# ── Map drawing ─────────────────────────────────────────────

fn draw_world():
    # Fill with grass
    let y = 0
    while y < 18:
        let x = 0
        while x < 20:
            set_bg_tile(x, y, grass)
            x := x + 1
        y := y + 1

    # Path across the middle
    let px2 = 0
    while px2 < 20:
        set_bg_tile(px2, 8, path)
        set_bg_tile(px2, 9, path)
        set_bg_tile(px2, 10, path)
        px2 := px2 + 1

    # Vertical path
    let py2 = 0
    while py2 < 18:
        set_bg_tile(9, py2, path)
        set_bg_tile(10, py2, path)
        py2 := py2 + 1

    # Trees
    set_bg_tile(2, 2, tree_top)
    set_bg_tile(2, 3, tree_trunk)
    set_bg_tile(5, 1, tree_top)
    set_bg_tile(5, 2, tree_trunk)
    set_bg_tile(16, 3, tree_top)
    set_bg_tile(16, 4, tree_trunk)
    set_bg_tile(14, 1, tree_top)
    set_bg_tile(14, 2, tree_trunk)

    # Flowers
    set_bg_tile(3, 5, flowers)
    set_bg_tile(7, 4, flowers)
    set_bg_tile(12, 5, flowers)
    set_bg_tile(15, 6, flowers)
    set_bg_tile(4, 13, flowers)
    set_bg_tile(17, 12, flowers)

    # Rocks
    set_bg_tile(1, 12, rock)
    set_bg_tile(6, 14, rock)
    set_bg_tile(15, 14, rock)

    # Water pond
    set_bg_tile(13, 12, water)
    set_bg_tile(14, 12, water)
    set_bg_tile(13, 13, water)
    set_bg_tile(14, 13, water)

    # Fences
    set_bg_tile(3, 7, fence)
    set_bg_tile(4, 7, fence)
    set_bg_tile(5, 7, fence)
    set_bg_tile(6, 7, fence)

# ── Animation ───────────────────────────────────────────────

fn update_anim():
    if moving == 1:
        anim_timer := anim_timer + 1
        if anim_timer > ANIM_SPEED:
            anim_timer := 0
            frame := frame + 1
            if frame > 2:
                frame := 0
    else:
        frame := 0
        anim_timer := 0

fn get_sprite():
    # Return the right tile based on animation frame
    if frame == 0:
        set_sprite(0, px, py, walk1)
    elif frame == 1:
        set_sprite(0, px, py, walk2)
    else:
        set_sprite(0, px, py, walk3)

fn show_idle():
    set_sprite(0, px, py, idle)

# ── Main ────────────────────────────────────────────────────

init:
    draw_world()
    set_sprite(0, px, py, idle)

on vblank:
    moving := 0

    if pressed(Button.LEFT):
        px := px - 1
        moving := 1
        dir := LEFT
    if pressed(Button.RIGHT):
        px := px + 1
        moving := 1
        dir := RIGHT
    if pressed(Button.UP):
        py := py - 1
        moving := 1
    if pressed(Button.DOWN):
        py := py + 1
        moving := 1

    # Clamp to screen
    if px < 4:
        px := 4
    if px > 152:
        px := 152
    if py < 4:
        py := 4
    if py > 136:
        py := 136

    update_anim()

    if moving == 1:
        get_sprite()
    else:
        show_idle()
