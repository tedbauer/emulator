//! Resolver: builds the symbol table and assigns WRAM addresses + tile indices.

use crate::ast::*;
use std::collections::HashMap;

/// A resolved global variable: its WRAM address and type.
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub addr: u16,
    pub ty: Type,
}

/// A resolved tile: its index in the VRAM tile table.
#[derive(Debug, Clone)]
pub struct TileInfo {
    pub index: u8,
    /// 8 rows of 8 pixels, each pixel is 0-3.
    pub pixels: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct Resolver {
    pub vars: HashMap<String, VarInfo>,
    pub tiles: HashMap<String, TileInfo>,
    next_wram: u16,
    next_tile: u8,
}

impl Resolver {
    const WRAM_BASE: u16 = 0xC000;

    pub fn new() -> Self {
        Resolver {
            vars: HashMap::new(),
            tiles: HashMap::new(),
            next_wram: Self::WRAM_BASE,
            next_tile: 1, // tile 0 reserved as blank for BG
        }
    }

    pub fn resolve(&mut self, prog: &Program) -> Result<(), String> {
        // Register tiles first (tile names may be used as arguments)
        for tile in &prog.tiles {
            self.register_tile(tile)?;
        }
        // Register globals
        for g in &prog.globals {
            self.register_var(g)?;
        }
        Ok(())
    }

    fn register_tile(&mut self, tile: &TileDef) -> Result<(), String> {
        if self.tiles.contains_key(&tile.name) {
            return Err(format!(
                "Line {}: duplicate tile '{}'",
                tile.line, tile.name
            ));
        }
        if self.next_tile == 255 {
            return Err("Too many tiles (max 255)".into());
        }
        let mut pixels: Vec<Vec<u8>> = Vec::new();
        for (ri, row) in tile.rows.iter().enumerate() {
            let mut pr = Vec::new();
            for (ci, ch) in row.chars().enumerate() {
                let v = match ch {
                    '.' | '0' => 0u8,
                    '1' => 1,
                    '2' => 2,
                    '3' => 3,
                    other => {
                        return Err(format!(
                            "Tile '{}' row {} col {}: invalid pixel '{}'",
                            tile.name, ri, ci, other
                        ));
                    }
                };
                pr.push(v);
            }
            pixels.push(pr);
        }
        self.tiles.insert(
            tile.name.clone(),
            TileInfo {
                index: self.next_tile,
                pixels,
            },
        );
        self.next_tile += 1;
        Ok(())
    }

    fn register_var(&mut self, decl: &LetDecl) -> Result<(), String> {
        if self.vars.contains_key(&decl.name) {
            return Err(format!(
                "Line {}: duplicate variable '{}'",
                decl.line, decl.name
            ));
        }
        // Infer type from literal if not annotated
        let ty = if let Some(t) = &decl.ty {
            t.clone()
        } else {
            infer_type(&decl.init)?
        };
        let size: u16 = match &ty {
            Type::U8 | Type::I8 | Type::Bool => 1,
            Type::U16 => 2,
        };
        let addr = self.next_wram;
        self.next_wram += size;
        self.vars.insert(decl.name.clone(), VarInfo { addr, ty });
        Ok(())
    }
}

fn infer_type(expr: &Expr) -> Result<Type, String> {
    match expr {
        Expr::Int(n, _) => {
            if *n < 0 {
                Ok(Type::I8)
            } else {
                Ok(Type::U8)
            }
        }
        Expr::Bool(_, _) => Ok(Type::Bool),
        _ => Ok(Type::U8), // default
    }
}
