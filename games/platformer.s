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

let px: u8 = 72
let py: u8 = 100
let vy: i8 = 0
let on_ground: u8 = 0
let tx: u8 = 0

init:
    while tx <= 19:
        set_bg_tile(tx, 16, ground)
        tx := tx + 1
    set_sprite(0, px, py, player)

on vblank:
    if on_ground == 0:
        vy := vy + 1

    if on_ground == 1:
        if pressed(Button.A):
            vy := -8
            on_ground := 0

    if pressed(Button.LEFT):
        if px >= 2:
            px := px - 2
    if pressed(Button.RIGHT):
        if px <= 150:
            px := px + 2

    py := py + vy

    if py >= 120:
        py := 120
        vy := 0
        on_ground := 1
    if py < 120:
        on_ground := 0

    set_sprite(0, px, py, player)
