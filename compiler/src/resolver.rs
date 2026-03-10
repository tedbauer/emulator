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
    /// For each user function, the ordered list of parameter names.
    pub fn_params: HashMap<String, Vec<String>>,
    /// Compile-time constants (inlined, no WRAM allocation)
    pub consts: HashMap<String, i32>,
    next_wram: u16,
    next_tile: u8,
}

impl Resolver {
    const WRAM_BASE: u16 = 0xC000;

    pub fn new() -> Self {
        Resolver {
            vars: HashMap::new(),
            tiles: HashMap::new(),
            fn_params: HashMap::new(),
            consts: HashMap::new(),
            next_wram: Self::WRAM_BASE,
            next_tile: 1, // tile 0 reserved as blank for BG
        }
    }

    pub fn resolve(&mut self, prog: &Program) -> Result<(), String> {
        // Register tiles first (tile names may be used as arguments)
        for tile in &prog.tiles {
            self.register_tile(tile)?;
        }
        // Register constants (inlined at compile time)
        for c in &prog.consts {
            if self.consts.contains_key(&c.name) {
                return Err(format!("Line {}: duplicate constant '{}'", c.line, c.name));
            }
            self.consts.insert(c.name.clone(), c.value);
        }
        // Register globals
        for g in &prog.globals {
            self.register_var(g)?;
        }
        // Register function parameters as WRAM variables
        for func in &prog.functions {
            let mut param_names = Vec::new();
            for (pname, pty) in &func.params {
                // Mangle: fn_name$$param_name to avoid conflicts between functions
                let mangled = format!("{}$${}", func.name, pname);
                let size: u16 = match pty {
                    Type::U8 | Type::I8 | Type::Bool => 1,
                    Type::U16 => 2,
                };
                let addr = self.next_wram;
                self.next_wram += size;
                self.vars.insert(
                    mangled.clone(),
                    VarInfo {
                        addr,
                        ty: pty.clone(),
                    },
                );
                param_names.push(mangled);
            }
            self.fn_params.insert(func.name.clone(), param_names);
        }
        // Register `let` declarations inside all blocks
        if let Some(init) = &prog.init {
            self.register_block_vars(init)?;
        }
        if let Some(vblank) = &prog.on_vblank {
            self.register_block_vars(vblank)?;
        }
        for func in &prog.functions {
            self.register_block_vars(&func.body)?;
        }
        Ok(())
    }

    fn register_block_vars(&mut self, block: &Block) -> Result<(), String> {
        for stmt in block {
            match stmt {
                Stmt::Let(decl) => {
                    if !self.vars.contains_key(&decl.name) {
                        self.register_var(decl)?;
                    }
                }
                Stmt::If {
                    then, elifs, else_, ..
                } => {
                    self.register_block_vars(then)?;
                    for (_, elif_block) in elifs {
                        self.register_block_vars(elif_block)?;
                    }
                    if let Some(e) = else_ {
                        self.register_block_vars(e)?;
                    }
                }
                Stmt::While { body, .. } | Stmt::Loop { body, .. } => {
                    self.register_block_vars(body)?;
                }
                _ => {}
            }
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
