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
    22222222
    ........
    ........
    ........
    ........
    ........

let px: u8 = 24
let py: u8 = 100
let vy: i8 = 0
let on_ground: u8 = 0
let jump_lock: u8 = 0
let tx: u8 = 0

init:
    while tx <= 19:
        set_bg_tile(tx, 16, ground)
        tx := tx + 1
    set_bg_tile(2, 12, platform)
    set_bg_tile(3, 12, platform)
    set_bg_tile(4, 12, platform)
    set_bg_tile(9, 10, platform)
    set_bg_tile(10, 10, platform)
    set_bg_tile(11, 10, platform)
    set_bg_tile(14, 8, platform)
    set_bg_tile(15, 8, platform)
    set_bg_tile(16, 8, platform)
    set_sprite(0, px, py, player)

on vblank:
    if on_ground == 0:
        vy := vy + 1
    if jump_lock >= 1:
        jump_lock := jump_lock - 1

    if on_ground == 1:
        if pressed(Button.A):
            vy := -12
            on_ground := 0
            jump_lock := 12

    if pressed(Button.LEFT):
        if px >= 2:
            px := px - 2
    if pressed(Button.RIGHT):
        if px <= 150:
            px := px + 2

    py := py + vy
    on_ground := 0

    if jump_lock == 0:
        if px >= 12:
            if px <= 42:
                if py >= 88:
                    if py <= 102:
                        py := 88
                        vy := 0
                        on_ground := 1

    if jump_lock == 0:
        if on_ground == 0:
            if px >= 68:
                if px <= 96:
                    if py >= 72:
                        if py <= 86:
                            py := 72
                            vy := 0
                            on_ground := 1

    if jump_lock == 0:
        if on_ground == 0:
            if px >= 108:
                if px <= 136:
                    if py >= 56:
                        if py <= 70:
                            py := 56
                            vy := 0
                            on_ground := 1

    if on_ground == 0:
        if py >= 120:
            py := 120
            vy := 0
            on_ground := 1

    set_sprite(0, px, py, player)
