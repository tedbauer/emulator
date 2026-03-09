from core import pressed, set_sprite, Button

# ── Tiles ────────────────────────────────────────────────────────────────────

tile ball:
    .333....
    3333....
    33333333
    33333333
    33333333
    33333333
    3333....
    .333....

tile paddle:
    33333333
    33333333
    33333333
    33333333
    ........
    ........
    ........
    ........

# ── Game state ────────────────────────────────────────────────────────────────

let bx = 80
let by = 72
let vx: i8 = 1
let vy: i8 = 1
let px = 64

# ── Init ──────────────────────────────────────────────────────────────────────

init:
    set_sprite(0, bx, by, ball)
    set_sprite(1, px, 136, paddle)
    set_sprite(2, px + 8, 136, paddle)
    set_sprite(3, px + 16, 136, paddle)

# ── Game loop ─────────────────────────────────────────────────────────────────

on vblank:
    # Move ball
    bx := bx + vx
    by := by + vy

    # Left/right wall bounce
    if bx <= 8:
        vx := 1
    if bx >= 152:
        vx := -1

    # Top wall bounce
    if by <= 16:
        vy := 1

    # Paddle collision (simple AABB)
    if by >= 126:
        if by <= 136:
            if bx >= px:
                if bx <= px + 24:
                    vy := -1

    # Ball off bottom — reset
    if by >= 144:
        bx := 80
        by := 72
        vx := 1
        vy := 1

    # Move paddle left/right
    if pressed(Button.LEFT):
        if px > 8:
            px := px - 2
    if pressed(Button.RIGHT):
        if px < 128:
            px := px + 2

    # Update sprites
    set_sprite(0, bx, by, ball)
    set_sprite(1, px, 136, paddle)
    set_sprite(2, px + 8, 136, paddle)
    set_sprite(3, px + 16, 136, paddle)
