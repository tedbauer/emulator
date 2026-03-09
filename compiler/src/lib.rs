//! compiler — entry point: tokenize → parse → resolve → codegen → ROM bytes.

pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod resolver;
pub mod rom;

use codegen::Codegen;
use resolver::Resolver;
use rom::{encode_tile, RomWriter};

const GAME_CODE_BASE: u16 = 0x0200;

/// Compile GBScript source → 32-byte Game Boy ROM binary.
pub fn compile(src: &str) -> Result<Vec<u8>, String> {
    // 1. Lex
    let tokens = lexer::tokenize(src)?;

    // 2. Parse
    let program = parser::parse(tokens)?;

    // 3. Resolve symbols (WRAM layout, tile indices)
    let mut resolver = Resolver::new();
    resolver.resolve(&program)?;

    // 4. Encode tile pixel data → 2bpp bytes
    let mut tile_data: Vec<u8> = Vec::new();
    // Tiles are stored in resolver order (by index)
    let mut ordered_tiles: Vec<(&String, &resolver::TileInfo)> = resolver.tiles.iter().collect();
    ordered_tiles.sort_by_key(|(_, v)| v.index);
    for (_, tile_info) in &ordered_tiles {
        tile_data.extend(encode_tile(&tile_info.pixels));
    }

    // 5. Codegen
    let mut cg = Codegen::new(GAME_CODE_BASE, &resolver);

    // We emit: init_fn, vblank_fn, then builtins.
    // The SETUP code at $0150 calls init_fn at $0200 directly.
    // The VBlank ISR calls vblank_fn at its label.
    //
    // Layout:
    //   $0200: __init_fn ...  RET
    //   ????:  __vblank_fn ... RET
    //   ????:  builtin stubs

    cg.place_label("__init_fn");
    if let Some(init_block) = &program.init {
        cg.gen_block(init_block)?;
    }
    cg.ret();

    cg.place_label("__vblank_fn");
    if let Some(vblank_block) = &program.on_vblank {
        cg.gen_block(vblank_block)?;
    }
    cg.ret();

    // User-defined functions
    for func in &program.functions {
        cg.place_label(&func.name);
        cg.gen_block(&func.body)?;
        cg.ret();
    }

    // Check whether just_pressed is used
    let src_lower = src;
    let need_just_pressed = src.contains("just_pressed");

    cg.emit_builtins(need_just_pressed)?;

    let game_code = cg.finalize()?;

    let has_vblank = program.on_vblank.is_some();

    // 6. Build ROM
    let mut writer = RomWriter::new();
    let rom = writer.build(&game_code, &tile_data, has_vblank);

    Ok(rom.to_vec())
}
