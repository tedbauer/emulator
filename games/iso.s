from core import pressed, set_sprite, set_bg_tile, Button

# ── Isometric tiles ────────────────────────────────────
# Each tile is 8x8. We use a minimalist isometric style
# with diamond floor tiles and cube-shaped blocks.

# Floor tile: diamond pattern
tile floor1:
    ...33...
    ..3113..
    .311113.
    31111113
    31111113
    .311113.
    ..3113..
    ...33...

# Alternate floor for checkerboard
tile floor2:
    ...33...
    ..3223..
    .322223.
    32222223
    32222223
    .322223.
    ..3223..
    ...33...

# Cube top face
tile cube_top:
    ...33...
    ..3113..
    .311113.
    31111113
    31111113
    .311113.
    ..3113..
    ...33...

# Cube front-left face
tile cube_fl:
    ...33333
    ..322223
    .3222223
    32222223
    32222223
    .3222223
    ..322223
    ...33333

# Cube front-right face
tile cube_fr:
    33333...
    32222...
    32222...
    32222...
    32222...
    32222...
    32222...
    33333...

# Dark wall tile
tile wall:
    33333333
    32323232
    23232323
    33333333
    32323232
    23232323
    33333333
    32323232

# Tree top
tile tree_top:
    ..1221..
    .122221.
    12222221
    12222221
    12222221
    .122221.
    ..1221..
    ...33...

# Tree trunk
tile tree_trunk:
    ...33...
    ...33...
    ...33...
    ...33...
    ...33...
    ...33...
    ..3333..
    .333333.

# Water tile
tile water:
    ...11...
    ..1..1..
    .1....1.
    ........
    ........
    .1....1.
    ..1..1..
    ...11...

# Player sprite (isometric character)
tile player:
    ..1111..
    .133331.
    .131131.
    .133331.
    ..3333..
    .133331.
    .13..31.
    .1....1.

# Shadow under player
tile shadow:
    ........
    ........
    ........
    ........
    ........
    ...11...
    ..1111..
    ...11...

# ── Constants ──────────────────────────────────────────

const MAP_W = 10
const MAP_H = 9

# ── Global state ───────────────────────────────────────

let px = 80
let py = 64
let cam_x = 0
let cam_y = 0

# ── Map drawing ────────────────────────────────────────
# Draw a static isometric scene on the background layer

fn draw_map():
    # Floor: checkerboard diamond pattern
    let y = 0
    while y < MAP_H:
        let x = 0
        while x < MAP_W:
            if (x + y) % 2 == 0:
                set_bg_tile(x + 5, y + 4, floor1)
            else:
                set_bg_tile(x + 5, y + 4, floor2)
            x := x + 1
        y := y + 1

    # Walls along top edge
    let wx = 0
    while wx < MAP_W:
        set_bg_tile(wx + 5, 3, wall)
        wx := wx + 1

    # Walls along left edge
    let wy = 0
    while wy < MAP_H:
        set_bg_tile(4, wy + 4, wall)
        wy := wy + 1

    # Cube structures
    set_bg_tile(7, 6, cube_top)
    set_bg_tile(7, 7, cube_fl)
    set_bg_tile(8, 6, cube_top)
    set_bg_tile(8, 7, cube_fr)

    set_bg_tile(12, 8, cube_top)
    set_bg_tile(12, 9, cube_fl)

    # Trees
    set_bg_tile(10, 5, tree_top)
    set_bg_tile(10, 6, tree_trunk)

    set_bg_tile(14, 5, tree_top)
    set_bg_tile(14, 6, tree_trunk)

    # Water pond
    set_bg_tile(11, 10, water)
    set_bg_tile(12, 10, water)
    set_bg_tile(11, 11, water)
    set_bg_tile(12, 11, water)

# ── Collision ──────────────────────────────────────────

let can_go = 1

fn check_bounds(mx: u8, my: u8):
    can_go := 1
    # Keep within floor area
    if mx < 48:
        can_go := 0
    if mx > 112:
        can_go := 0
    if my < 40:
        can_go := 0
    if my > 96:
        can_go := 0

    # Block cubes
    let tx = mx / 8
    let ty = my / 8

    # Cube at (7,6)-(8,7)
    if tx > 6:
        if tx < 9:
            if ty > 5:
                if ty < 8:
                    can_go := 0

    # Cube at (12,8)-(12,9)
    if tx == 12:
        if ty > 7:
            if ty < 10:
                can_go := 0

    # Trees at (10,5-6) and (14,5-6)
    if tx == 10:
        if ty > 4:
            if ty < 7:
                can_go := 0
    if tx == 14:
        if ty > 4:
            if ty < 7:
                can_go := 0

    # Water at (11-12, 10-11)
    if tx > 10:
        if tx < 13:
            if ty > 9:
                if ty < 12:
                    can_go := 0

# ── Main ───────────────────────────────────────────────

init:
    draw_map()
    set_sprite(0, px, py, player)
    set_sprite(1, px, py + 4, shadow)

on vblank:
    let nx = px
    let ny = py

    if pressed(Button.LEFT):
        nx := px - 1
    if pressed(Button.RIGHT):
        nx := px + 1
    if pressed(Button.UP):
        ny := py - 1
    if pressed(Button.DOWN):
        ny := py + 1

    if nx != px:
        check_bounds(nx, py)
        if can_go == 1:
            px := nx
    if ny != py:
        check_bounds(px, ny)
        if can_go == 1:
            py := ny

    set_sprite(0, px, py, player)
    set_sprite(1, px, py + 4, shadow)
