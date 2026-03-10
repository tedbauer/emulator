from core import pressed, just_pressed, set_sprite, set_bg_tile, Button

# ── Tiles ──────────────────────────────────────────────

tile player:
    ..1111..
    .111111.
    .13.13..
    .111111.
    ..1111..
    .13.13..
    .1....1.
    ........

tile grass:
    ........
    ........
    ..1.....
    ........
    ........
    .....1..
    ........
    ........

tile path:
    11111111
    11111111
    11111111
    11111111
    11111111
    11111111
    11111111
    11111111

tile wall:
    33333333
    32222223
    32222223
    33333333
    33333333
    32222223
    32222223
    33333333

tile roof:
    33333333
    32323232
    23232323
    32323232
    23232323
    32323232
    23232323
    33333333

tile door:
    33333333
    32222223
    32222223
    32222223
    32222223
    32211223
    32211223
    33333333

tile floor:
    11111111
    1.1.1.1.
    11111111
    .1.1.1.1
    11111111
    1.1.1.1.
    11111111
    .1.1.1.1

tile table:
    22222222
    23333332
    23111132
    23111132
    23111132
    23111132
    23333332
    22222222

tile tree:
    ..2332..
    .233332.
    23333332
    23333332
    .233332.
    ..2332..
    ...33...
    ...33...

tile exit_mat:
    12121212
    21212121
    12121212
    21212121
    12121212
    21212121
    12121212
    21212121

tile bed:
    33333333
    31111113
    31111113
    31111113
    31111113
    32222223
    32222223
    33333333

tile rug:
    .222222.
    21111112
    21333312
    21311312
    21311312
    21333312
    21111112
    .222222.

tile window:
    33333333
    31.31.33
    31.31.33
    33333333
    31.31.33
    31.31.33
    33333333
    33333333

# ── Constants ──────────────────────────────────────────

const SCENE_TOWN = 0
const SCENE_H1 = 1
const SCENE_H2 = 2
const MOVE_COOLDOWN = 30

# ── Global state ───────────────────────────────────────

let px = 80
let py = 88
let nx = 80
let ny = 88
let scene = SCENE_TOWN
let can_go = 0
let cooldown = 0

# ── Map drawing ────────────────────────────────────────

fn clear_map():
    let y = 0
    while y < 18:
        let x = 0
        while x < 20:
            set_bg_tile(x, y, grass)
            x := x + 1
        y := y + 1

fn draw_town():
    clear_map()

    let i = 0
    while i < 20:
        set_bg_tile(i, 9, path)
        set_bg_tile(i, 10, path)
        i := i + 1

    set_bg_tile(4, 7, path)
    set_bg_tile(4, 8, path)
    set_bg_tile(15, 7, path)
    set_bg_tile(15, 8, path)

    set_bg_tile(0, 0, tree)
    set_bg_tile(1, 0, tree)
    set_bg_tile(7, 0, tree)
    set_bg_tile(8, 0, tree)
    set_bg_tile(11, 0, tree)
    set_bg_tile(12, 0, tree)
    set_bg_tile(18, 0, tree)
    set_bg_tile(19, 0, tree)

    set_bg_tile(0, 17, tree)
    set_bg_tile(3, 17, tree)
    set_bg_tile(9, 17, tree)
    set_bg_tile(10, 17, tree)
    set_bg_tile(16, 17, tree)
    set_bg_tile(19, 17, tree)

    set_bg_tile(0, 12, tree)
    set_bg_tile(9, 13, tree)
    set_bg_tile(18, 11, tree)
    set_bg_tile(1, 15, tree)
    set_bg_tile(17, 14, tree)

    # House 1
    set_bg_tile(2, 3, roof)
    set_bg_tile(3, 3, roof)
    set_bg_tile(4, 3, roof)
    set_bg_tile(5, 3, roof)
    set_bg_tile(6, 3, roof)
    set_bg_tile(2, 4, wall)
    set_bg_tile(3, 4, window)
    set_bg_tile(4, 4, wall)
    set_bg_tile(5, 4, window)
    set_bg_tile(6, 4, wall)
    set_bg_tile(2, 5, wall)
    set_bg_tile(3, 5, wall)
    set_bg_tile(4, 5, wall)
    set_bg_tile(5, 5, wall)
    set_bg_tile(6, 5, wall)
    set_bg_tile(2, 6, wall)
    set_bg_tile(3, 6, wall)
    set_bg_tile(4, 6, door)
    set_bg_tile(5, 6, wall)
    set_bg_tile(6, 6, wall)

    # House 2
    set_bg_tile(13, 3, roof)
    set_bg_tile(14, 3, roof)
    set_bg_tile(15, 3, roof)
    set_bg_tile(16, 3, roof)
    set_bg_tile(17, 3, roof)
    set_bg_tile(13, 4, wall)
    set_bg_tile(14, 4, window)
    set_bg_tile(15, 4, wall)
    set_bg_tile(16, 4, window)
    set_bg_tile(17, 4, wall)
    set_bg_tile(13, 5, wall)
    set_bg_tile(14, 5, wall)
    set_bg_tile(15, 5, wall)
    set_bg_tile(16, 5, wall)
    set_bg_tile(17, 5, wall)
    set_bg_tile(13, 6, wall)
    set_bg_tile(14, 6, wall)
    set_bg_tile(15, 6, door)
    set_bg_tile(16, 6, wall)
    set_bg_tile(17, 6, wall)

fn draw_house1():
    clear_map()
    let y = 1
    while y < 16:
        let x = 1
        while x < 19:
            set_bg_tile(x, y, floor)
            x := x + 1
        y := y + 1
    let i = 0
    while i < 20:
        set_bg_tile(i, 0, wall)
        i := i + 1
    i := 0
    while i < 20:
        set_bg_tile(i, 16, wall)
        i := i + 1
    let j = 0
    while j < 17:
        set_bg_tile(0, j, wall)
        set_bg_tile(19, j, wall)
        j := j + 1
    set_bg_tile(9, 16, exit_mat)
    set_bg_tile(10, 16, exit_mat)
    set_bg_tile(4, 0, window)
    set_bg_tile(8, 0, window)
    set_bg_tile(11, 0, window)
    set_bg_tile(15, 0, window)
    set_bg_tile(2, 2, table)
    set_bg_tile(3, 2, table)
    set_bg_tile(2, 3, table)
    set_bg_tile(3, 3, table)
    set_bg_tile(16, 2, bed)
    set_bg_tile(17, 2, bed)
    set_bg_tile(16, 3, bed)
    set_bg_tile(17, 3, bed)
    set_bg_tile(9, 8, rug)
    set_bg_tile(10, 8, rug)
    set_bg_tile(9, 9, rug)
    set_bg_tile(10, 9, rug)

fn draw_house2():
    clear_map()
    let y = 1
    while y < 16:
        let x = 1
        while x < 19:
            set_bg_tile(x, y, floor)
            x := x + 1
        y := y + 1
    let i = 0
    while i < 20:
        set_bg_tile(i, 0, wall)
        i := i + 1
    i := 0
    while i < 20:
        set_bg_tile(i, 16, wall)
        i := i + 1
    let j = 0
    while j < 17:
        set_bg_tile(0, j, wall)
        set_bg_tile(19, j, wall)
        j := j + 1
    set_bg_tile(9, 16, exit_mat)
    set_bg_tile(10, 16, exit_mat)
    set_bg_tile(5, 0, window)
    set_bg_tile(14, 0, window)
    set_bg_tile(7, 4, table)
    set_bg_tile(8, 4, table)
    set_bg_tile(9, 4, table)
    set_bg_tile(10, 4, table)
    set_bg_tile(11, 4, table)
    set_bg_tile(12, 4, table)
    set_bg_tile(7, 5, table)
    set_bg_tile(8, 5, table)
    set_bg_tile(9, 5, table)
    set_bg_tile(10, 5, table)
    set_bg_tile(11, 5, table)
    set_bg_tile(12, 5, table)
    set_bg_tile(17, 2, bed)
    set_bg_tile(18, 2, bed)
    set_bg_tile(17, 3, bed)
    set_bg_tile(18, 3, bed)
    set_bg_tile(17, 6, bed)
    set_bg_tile(18, 6, bed)
    set_bg_tile(17, 7, bed)
    set_bg_tile(18, 7, bed)
    set_bg_tile(9, 13, rug)
    set_bg_tile(10, 13, rug)
    set_bg_tile(9, 14, rug)
    set_bg_tile(10, 14, rug)

# ── Collision (uses nx/ny globals, sets can_go) ────────

fn check_town(cx: u8, cy: u8):
    can_go := 1
    let tx = cx / 8
    let ty = cy / 8
    if ty < 1:
        can_go := 0
    if ty > 16:
        can_go := 0
    if tx < 1:
        can_go := 0
    if tx > 18:
        can_go := 0
    # House 1 (tx 2-6, ty 3-6)
    if can_go == 1:
        if tx > 1:
            if tx < 7:
                if ty > 2:
                    if ty < 7:
                        can_go := 0
                        if tx == 4:
                            if ty == 6:
                                can_go := 1
    # House 2 (tx 13-17, ty 3-6)
    if can_go == 1:
        if tx > 12:
            if tx < 18:
                if ty > 2:
                    if ty < 7:
                        can_go := 0
                        if tx == 15:
                            if ty == 6:
                                can_go := 1
    # Tree collisions
    if can_go == 1:
        if tx == 0:
            if ty == 12:
                can_go := 0
        if tx == 9:
            if ty == 13:
                can_go := 0
        if tx == 18:
            if ty == 11:
                can_go := 0
        if tx == 1:
            if ty == 15:
                can_go := 0
        if tx == 17:
            if ty == 14:
                can_go := 0

fn check_house(cx: u8, cy: u8):
    can_go := 1
    let tx = cx / 8
    let ty = cy / 8
    if tx < 1:
        can_go := 0
    if tx > 18:
        can_go := 0
    if ty < 1:
        can_go := 0
    if ty > 15:
        can_go := 0
        if tx == 9:
            can_go := 1
        if tx == 10:
            can_go := 1

fn check_h1(cx: u8, cy: u8):
    check_house(cx, cy)
    if can_go == 1:
        let tx = cx / 8
        let ty = cy / 8
        if tx > 1:
            if tx < 4:
                if ty > 1:
                    if ty < 4:
                        can_go := 0
        if tx > 15:
            if tx < 18:
                if ty > 1:
                    if ty < 4:
                        can_go := 0

fn check_h2(cx: u8, cy: u8):
    check_house(cx, cy)
    if can_go == 1:
        let tx = cx / 8
        let ty = cy / 8
        if tx > 6:
            if tx < 13:
                if ty > 3:
                    if ty < 6:
                        can_go := 0
        if tx > 16:
            if ty > 1:
                if ty < 4:
                    can_go := 0
            if ty > 5:
                if ty < 8:
                    can_go := 0

# ── Collision dispatcher ──────────────────────────────

fn try_move(mx: u8, my: u8):
    if scene == SCENE_TOWN:
        check_town(mx, my)
    if scene == SCENE_H1:
        check_h1(mx, my)
    if scene == SCENE_H2:
        check_h2(mx, my)

# ── Scene transitions ─────────────────────────────────

fn check_enter():
    let tx = px / 8
    let ty = py / 8
    if scene == SCENE_TOWN:
        if tx == 4:
            if ty == 7:
                scene := SCENE_H1
                draw_house1()
                px := 76
                py := 112
                cooldown := MOVE_COOLDOWN
        if tx == 15:
            if ty == 7:
                scene := SCENE_H2
                draw_house2()
                px := 76
                py := 112
                cooldown := MOVE_COOLDOWN
    if scene == SCENE_H1:
        if ty > 15:
            if tx == 9:
                scene := SCENE_TOWN
                draw_town()
                px := 32
                py := 64
                cooldown := MOVE_COOLDOWN
            if tx == 10:
                scene := SCENE_TOWN
                draw_town()
                px := 32
                py := 64
                cooldown := MOVE_COOLDOWN
    if scene == SCENE_H2:
        if ty > 15:
            if tx == 9:
                scene := SCENE_TOWN
                draw_town()
                px := 120
                py := 64
                cooldown := MOVE_COOLDOWN
            if tx == 10:
                scene := SCENE_TOWN
                draw_town()
                px := 120
                py := 64
                cooldown := MOVE_COOLDOWN

# ── Main ───────────────────────────────────────────────

init:
    draw_town()
    set_sprite(0, px, py, player)

on vblank:
    if cooldown > 0:
        cooldown := cooldown - 1
        set_sprite(0, px, py, player)

    if cooldown == 0:
        # Horizontal movement
        nx := px
        if pressed(Button.LEFT):
            nx := px - 1
        if pressed(Button.RIGHT):
            nx := px + 1
        if nx != px:
            try_move(nx, py)
            if can_go == 1:
                px := nx

        # Vertical movement
        ny := py
        if pressed(Button.UP):
            ny := py - 1
        if pressed(Button.DOWN):
            ny := py + 1
        if ny != py:
            try_move(px, ny)
            if can_go == 1:
                py := ny

        check_enter()
        set_sprite(0, px, py, player)
