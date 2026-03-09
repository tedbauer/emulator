---
template: docs.html
title: Tiles
---

# Tiles

Tiles are 8×8 pixel art definitions embedded directly in your source code. They're the building blocks for both sprites and backgrounds on the Game Boy.

## Defining a tile

```
tile heart:
    ..3333..
    .333333.
    33333333
    33333333
    .333333.
    ..3333..
    ...33...
    ........
```

Each tile has exactly **8 rows of 8 characters**. Each character represents a pixel color:

| Character | Color | Shade |
|-----------|-------|-------|
| `.` | 0 | White (lightest) |
| `1` | 1 | Light gray |
| `2` | 2 | Dark gray |
| `3` | 3 | Black (darkest) |

## Using tiles

Tiles are referenced by name when calling sprite and background functions:

```
# As a sprite
set_sprite(0, x, y, heart)

# As a background tile
set_bg_tile(5, 10, heart)
```

## Tile 0 (blank)

Tile index 0 is always a blank (all white) tile, reserved by the system. Your defined tiles start at index 1. This means the background starts fully blank.

## Multiple tiles

You can define as many tiles as you need:

```
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
```

## Design tips

- Use `.` for transparency in sprites (color 0 is transparent for sprite objects)
- Background tiles are not transparent — all 4 colors are drawn
- You get up to ~384 tiles total (shared between sprites and backgrounds)
- Tiles are stored in VRAM and can be viewed with the **Tileset** debug panel in the IDE
