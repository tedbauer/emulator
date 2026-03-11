from core import pressed, just_pressed, set_sprite, set_bg_tile, Button

# ═══════════════════════════════════════════════════════
# Player dino: 4 directional head tiles
# 1=body, 2=belly, 3=outline/eye
# ═══════════════════════════════════════════════════════

# Facing RIGHT
tile hr_tl:
    ....3...
    ...31333
    ..311111
    ..313111
    ..311111
    ...31111
    ...31111
    ..312211

tile hr_tr:
    ........
    3.......
    13......
    1133....
    13......
    13......
    1133....
    1133....

# Facing LEFT (mirrored)
tile hl_tl:
    ........
    .......3
    ......31
    ....3311
    ......31
    ......31
    ....3311
    ....3311

tile hl_tr:
    ...3....
    33313...
    111113..
    111313..
    111113..
    11113...
    11113...
    112213..

# Facing DOWN (front view)
tile hd_tl:
    ....3333
    ...31111
    ..311111
    .3131111
    .3111111
    .3121111
    ..311111
    ..312211

tile hd_tr:
    3333....
    11113...
    111113..
    11113131
    1111113.
    1111213.
    111113..
    112213..

# Facing UP (back view)
tile hu_tl:
    ....3333
    ...31111
    ..311111
    ..311111
    ..311111
    ...31111
    ...31111
    ..312211

tile hu_tr:
    3333....
    11113...
    111113..
    111113..
    111113..
    11113...
    11113...
    112213..

# ── Body tiles (shared all directions) ──

tile idle_bl:
    ..312211
    ...31113
    ...3113.
    ....31..
    ....31..
    ....33..
    ........
    ........

tile idle_br:
    1113....
    .11113..
    ..113...
    ...3....
    ........
    ........
    ........
    ........

tile w1_bl:
    ..312211
    ...31113
    ...3113.
    ...31...
    ....31..
    ...33.3.
    ........
    ........

tile w1_br:
    1113....
    .11113..
    ..113...
    ..3.....
    ...3....
    ........
    ........
    ........

tile w2_bl:
    ..312211
    ...31113
    ...3113.
    ....31..
    ...31...
    ..3..33.
    ........
    ........

tile w2_br:
    1113....
    .11113..
    ..113...
    ...3....
    ....3...
    ........
    ........
    ........

# ═══════════════════════════════════════════════════════
# NPC dino (darker, color 2 body)
# ═══════════════════════════════════════════════════════

tile npc_tl:
    ....3333
    ...32222
    ..322222
    .3232222
    .3222222
    .3212222
    ..322222
    ..321122

tile npc_tr:
    3333....
    22223...
    222223..
    22223231
    2222213.
    2222213.
    222223..
    221123..

tile npc_bl:
    ..321122
    ...32223
    ...3223.
    ....32..
    ....32..
    ....33..
    ........
    ........

tile npc_br:
    2223....
    .22223..
    ..223...
    ...3....
    ........
    ........
    ........
    ........

# ═══════════════════════════════════════════════════════
# World + UI
# ═══════════════════════════════════════════════════════

tile grass:
    11112111
    11121112
    11111121
    21111111
    11211111
    11111211
    12111111
    11111112

tile box_tl:
    33333333
    3.......
    3.......
    3.......
    3.......
    3.......
    3.......
    3.......

tile box_tr:
    33333333
    .......3
    .......3
    .......3
    .......3
    .......3
    .......3
    .......3

tile box_bl:
    3.......
    3.......
    3.......
    3.......
    3.......
    3.......
    3.......
    33333333

tile box_br:
    .......3
    .......3
    .......3
    .......3
    .......3
    .......3
    .......3
    33333333

tile box_h:
    33333333
    ........
    ........
    ........
    ........
    ........
    ........
    ........

tile box_hb:
    ........
    ........
    ........
    ........
    ........
    ........
    ........
    33333333

tile box_v:
    3.......
    3.......
    3.......
    3.......
    3.......
    3.......
    3.......
    3.......

tile box_vr:
    .......3
    .......3
    .......3
    .......3
    .......3
    .......3
    .......3
    .......3

tile arrow_down:
    ........
    ........
    .333333.
    ..33333.
    ...333..
    ....3...
    ........
    ........

# ═══════════════════════════════════════════════════════
# State
# ═══════════════════════════════════════════════════════

let px = 100
let py = 64
let frame = 0
let timer = 0
let moving = 0
let dir = 0
let talking = 0
let wait_release = 0
let text_pos = 0
let text_timer = 0
let arrow_blink = 0

const DIR_DOWN = 0
const DIR_UP = 1
const DIR_LEFT = 2
const DIR_RIGHT = 3
const NPC_X = 40
const NPC_Y = 64
const TEXT_LEN = 18

# ═══════════════════════════════════════════════════════
# Drawing
# ═══════════════════════════════════════════════════════

fn draw_head():
    if dir == DIR_RIGHT:
        set_sprite(0, px, py, hl_tl)
        set_sprite(1, px + 8, py, hl_tr)
    if dir == DIR_LEFT:
        set_sprite(0, px, py, hr_tl)
        set_sprite(1, px + 8, py, hr_tr)
    if dir == DIR_DOWN:
        set_sprite(0, px, py, hd_tl)
        set_sprite(1, px + 8, py, hd_tr)
    if dir == DIR_UP:
        set_sprite(0, px, py, hu_tl)
        set_sprite(1, px + 8, py, hu_tr)

fn draw_idle():
    draw_head()
    set_sprite(2, px, py + 8, idle_bl)
    set_sprite(3, px + 8, py + 8, idle_br)

fn draw_w1():
    draw_head()
    set_sprite(2, px, py + 8, w1_bl)
    set_sprite(3, px + 8, py + 8, w1_br)

fn draw_w2():
    draw_head()
    set_sprite(2, px, py + 8, w2_bl)
    set_sprite(3, px + 8, py + 8, w2_br)

fn draw_npc():
    set_sprite(4, NPC_X, NPC_Y, npc_tl)
    set_sprite(5, NPC_X + 8, NPC_Y, npc_tr)
    set_sprite(6, NPC_X, NPC_Y + 8, npc_bl)
    set_sprite(7, NPC_X + 8, NPC_Y + 8, npc_br)

fn draw_box():
    set_bg_tile(1, 14, box_tl)
    let tx = 2
    while tx < 18:
        set_bg_tile(tx, 14, box_h)
        tx := tx + 1
    set_bg_tile(18, 14, box_tr)
    set_bg_tile(1, 15, box_v)
    set_bg_tile(18, 15, box_vr)
    set_bg_tile(1, 16, box_v)
    set_bg_tile(18, 16, box_vr)
    let cx = 2
    while cx < 18:
        set_bg_tile(cx, 15, 0)
        set_bg_tile(cx, 16, 0)
        cx := cx + 1
    set_bg_tile(1, 17, box_bl)
    tx := 2
    while tx < 18:
        set_bg_tile(tx, 17, box_hb)
        tx := tx + 1
    set_bg_tile(18, 17, box_br)

fn hide_dialog():
    let gy = 14
    while gy < 18:
        let gx = 1
        while gx < 19:
            set_bg_tile(gx, gy, grass)
            gx := gx + 1
        gy := gy + 1

fn check_near() -> bool:
    let near = 0
    if px > 20:
        if px < 68:
            if py > 44:
                if py < 88:
                    near := 1
    return near

fn scroll_char():
    if text_pos == 0:
        print(3, 15, "H")
    if text_pos == 1:
        print(4, 15, "i")
    if text_pos == 2:
        print(5, 15, " ")
    if text_pos == 3:
        print(6, 15, "t")
    if text_pos == 4:
        print(7, 15, "h")
    if text_pos == 5:
        print(8, 15, "e")
    if text_pos == 6:
        print(9, 15, "r")
    if text_pos == 7:
        print(10, 15, "e")
    if text_pos == 8:
        print(11, 15, "!")
    if text_pos == 9:
        print(3, 16, "N")
    if text_pos == 10:
        print(4, 16, "i")
    if text_pos == 11:
        print(5, 16, "c")
    if text_pos == 12:
        print(6, 16, "e")
    if text_pos == 13:
        print(7, 16, " ")
    if text_pos == 14:
        print(8, 16, "d")
    if text_pos == 15:
        print(9, 16, "a")
    if text_pos == 16:
        print(10, 16, "y")
    if text_pos == 17:
        print(11, 16, "!")

# ═══════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════

init:
    let gy = 0
    while gy < 18:
        let gx = 0
        while gx < 20:
            set_bg_tile(gx, gy, grass)
            gx := gx + 1
        gy := gy + 1
    draw_idle()
    draw_npc()

on vblank:
    let a_held = pressed(Button.A)
    if a_held == 0:
        wait_release := 0

    if talking == 1:
        arrow_blink := arrow_blink + 1
        if arrow_blink == 15:
            set_bg_tile(17, 16, arrow_down)
        if arrow_blink == 30:
            set_bg_tile(17, 16, 0)
            arrow_blink := 0
        if a_held == 1:
            if wait_release == 0:
                talking := 0
                hide_dialog()
                wait_release := 1
                text_pos := 0
                text_timer := 0
                arrow_blink := 0
                beep()

    if talking == 0:
        if a_held == 1:
            if wait_release == 0:
                let near = check_near()
                if near == 1:
                    talking := 2
                    text_pos := 0
                    text_timer := 0
                    arrow_blink := 0
                    draw_box()
                    wait_release := 1
                    beep()

    if talking == 2:
        text_timer := text_timer + 1
        if text_timer == 3:
            text_timer := 0
            if text_pos < TEXT_LEN:
                scroll_char()
                text_pos := text_pos + 1
            if text_pos == TEXT_LEN:
                talking := 1
                set_bg_tile(17, 16, arrow_down)

    if talking == 0:
        moving := 0

        if pressed(Button.LEFT):
            px := px - 1
            moving := 1
            dir := DIR_LEFT
        if pressed(Button.RIGHT):
            px := px + 1
            moving := 1
            dir := DIR_RIGHT
        if pressed(Button.UP):
            py := py - 1
            moving := 1
            dir := DIR_UP
        if pressed(Button.DOWN):
            py := py + 1
            moving := 1
            dir := DIR_DOWN

        if moving == 1:
            timer := timer + 1
        if moving == 0:
            timer := 0
            frame := 0
            draw_idle()

        if timer == 1:
            draw_w1()
            frame := 1
        if timer == 6:
            draw_idle()
            frame := 0
        if timer == 11:
            draw_w2()
            frame := 2
        if timer == 16:
            draw_idle()
            frame := 0
            timer := 0

        if moving == 1:
            if frame == 0:
                draw_idle()
            if frame == 1:
                draw_w1()
            if frame == 2:
                draw_w2()
