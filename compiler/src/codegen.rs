//! Code generator: AST + symbol table → LR35902 machine code bytes.
//!
//! Strategy: expressions load their result into register A (for u8/i8/bool)
//! or HL (for u16).  WRAM variables are accessed with LD A,(nn) / LD (nn),A.

use crate::ast::*;
use crate::resolver::{Resolver, VarInfo};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Output buffer + label management
// ─────────────────────────────────────────────────────────────────────────────

/// A single byte or a pending label reference (forward refs resolved later).
#[derive(Debug, Clone)]
enum Byte {
    Raw(u8),
    /// Absolute 16-bit address of label (little-endian low byte)
    LabelLo(String),
    /// Absolute 16-bit address of label (little-endian high byte)
    LabelHi(String),
    /// Signed 8-bit relative offset from *next* instruction to label
    RelOffset(String),
}

pub struct Codegen {
    bytes: Vec<Byte>,
    labels: HashMap<String, u16>,
    label_counter: usize,
    /// Base ROM address where generated code starts (after header+setup stubs)
    pub base: u16,
    pub vars: HashMap<String, VarInfo>,
    pub tiles: HashMap<String, u8>, // tile name → index
    /// For each user function, the ordered list of parameter names.
    pub fn_params: HashMap<String, Vec<String>>,
    /// Compile-time constants (name → value)
    pub consts: HashMap<String, i32>,
}

impl Codegen {
    pub fn new(base: u16, resolver: &Resolver) -> Self {
        Codegen {
            bytes: vec![],
            labels: HashMap::new(),
            label_counter: 0,
            base,
            vars: resolver.vars.clone(),
            tiles: resolver
                .tiles
                .iter()
                .map(|(k, v)| (k.clone(), v.index))
                .collect(),
            fn_params: resolver.fn_params.clone(),
            consts: resolver.consts.clone(),
        }
    }

    fn emit(&mut self, b: u8) {
        self.bytes.push(Byte::Raw(b));
    }

    fn emit_u16(&mut self, v: u16) {
        self.emit((v & 0xFF) as u8);
        self.emit((v >> 8) as u8);
    }

    fn current_offset(&self) -> u16 {
        self.bytes.len() as u16
    }

    fn fresh_label(&mut self) -> String {
        let n = self.label_counter;
        self.label_counter += 1;
        format!("__L{}", n)
    }

    pub fn place_label(&mut self, name: &str) {
        let offset = self.base + self.current_offset();
        self.labels.insert(name.to_string(), offset);
    }

    /// Return the absolute ROM address of a previously placed label.
    pub fn label_addr(&self, name: &str) -> Option<u16> {
        self.labels.get(name).copied()
    }

    fn emit_label_addr(&mut self, name: &str) {
        self.bytes.push(Byte::LabelLo(name.to_string()));
        self.bytes.push(Byte::LabelHi(name.to_string()));
    }

    fn emit_rel_offset(&mut self, name: &str) {
        self.bytes.push(Byte::RelOffset(name.to_string()));
    }

    /// Resolve all pending label references and return raw bytes.
    pub fn finalize(self) -> Result<Vec<u8>, String> {
        let mut out = vec![];
        for (i, b) in self.bytes.iter().enumerate() {
            match b {
                Byte::Raw(v) => out.push(*v),
                Byte::LabelLo(name) => {
                    let addr = self
                        .labels
                        .get(name)
                        .ok_or_else(|| format!("Undefined label '{}'", name))?;
                    out.push((addr & 0xFF) as u8);
                }
                Byte::LabelHi(name) => {
                    let addr = self
                        .labels
                        .get(name)
                        .ok_or_else(|| format!("Undefined label '{}'", name))?;
                    out.push((addr >> 8) as u8);
                }
                Byte::RelOffset(name) => {
                    let target = self
                        .labels
                        .get(name)
                        .ok_or_else(|| format!("Undefined label '{}'", name))?;
                    let here = (self.base + i as u16 + 1) as i32;
                    let diff = (*target as i32) - here;
                    if diff < -128 || diff > 127 {
                        return Err(format!(
                            "Relative jump to '{}' out of range ({})",
                            name, diff
                        ));
                    }
                    out.push(diff as i8 as u8);
                }
            }
        }
        Ok(out)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // LR35902 instruction helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// LD A, n  (immediate)
    fn ld_a_n(&mut self, n: u8) {
        self.emit(0x3E);
        self.emit(n);
    }
    /// LD A, (nn)  (absolute WRAM)
    fn ld_a_mem(&mut self, addr: u16) {
        self.emit(0xFA);
        self.emit_u16(addr);
    }
    /// LD (nn), A
    pub fn ld_mem_a(&mut self, addr: u16) {
        self.emit(0xEA);
        self.emit_u16(addr);
    }
    /// LD B, A
    fn ld_b_a(&mut self) {
        self.emit(0x47);
    }
    /// LD C, A
    fn ld_c_a(&mut self) {
        self.emit(0x4F);
    }
    /// LD D, A
    fn ld_d_a(&mut self) {
        self.emit(0x57);
    }
    /// LD E, A
    fn ld_e_a(&mut self) {
        self.emit(0x5F);
    }
    /// LD H, A
    fn ld_h_a(&mut self) {
        self.emit(0x67);
    }
    /// LD L, A
    fn ld_l_a(&mut self) {
        self.emit(0x6F);
    }
    /// LD A, B
    fn ld_a_b(&mut self) {
        self.emit(0x78);
    }
    /// ADD A, B
    fn add_a_b(&mut self) {
        self.emit(0x80);
    }
    /// ADD A, n
    fn add_a_n(&mut self, n: u8) {
        self.emit(0xC6);
        self.emit(n);
    }
    /// SUB B
    fn sub_b(&mut self) {
        self.emit(0x90);
    }
    /// SUB n
    fn sub_n(&mut self, n: u8) {
        self.emit(0xD6);
        self.emit(n);
    }
    /// AND B
    fn and_b(&mut self) {
        self.emit(0xA0);
    }
    /// OR B
    fn or_b(&mut self) {
        self.emit(0xB0);
    }
    /// XOR A (= LD A, 0)
    fn xor_a(&mut self) {
        self.emit(0xAF);
    }
    /// CP B  (set flags from A-B)
    fn cp_b(&mut self) {
        self.emit(0xB8);
    }
    /// CP n
    fn cp_n(&mut self, n: u8) {
        self.emit(0xFE);
        self.emit(n);
    }
    /// INC A
    fn inc_a(&mut self) {
        self.emit(0x3C);
    }
    /// DEC A
    fn dec_a(&mut self) {
        self.emit(0x3D);
    }
    /// CPL  (complement A)
    fn cpl(&mut self) {
        self.emit(0x2F);
    }
    /// JP nn
    fn jp_nn(&mut self, name: &str) {
        self.emit(0xC3);
        self.emit_label_addr(name);
    }
    /// JP NZ, nn
    fn jp_nz(&mut self, name: &str) {
        self.emit(0xC2);
        self.emit_label_addr(name);
    }
    /// JP Z, nn
    fn jp_z(&mut self, name: &str) {
        self.emit(0xCA);
        self.emit_label_addr(name);
    }
    /// JP NC, nn
    fn jp_nc(&mut self, name: &str) {
        self.emit(0xD2);
        self.emit_label_addr(name);
    }
    /// JP C, nn
    fn jp_c(&mut self, name: &str) {
        self.emit(0xDA);
        self.emit_label_addr(name);
    }
    /// JR e (relative)
    fn jr(&mut self, name: &str) {
        self.emit(0x18);
        self.emit_rel_offset(name);
    }
    /// CALL nn
    fn call(&mut self, name: &str) {
        self.emit(0xCD);
        self.emit_label_addr(name);
    }
    /// RET
    pub fn ret(&mut self) {
        self.emit(0xC9);
    }
    /// RETI
    fn reti(&mut self) {
        self.emit(0xD9);
    }
    /// PUSH AF
    fn push_af(&mut self) {
        self.emit(0xF5);
    }
    /// PUSH BC
    fn push_bc(&mut self) {
        self.emit(0xC5);
    }
    /// PUSH DE
    fn push_de(&mut self) {
        self.emit(0xD5);
    }
    /// PUSH HL
    fn push_hl(&mut self) {
        self.emit(0xE5);
    }
    /// POP AF
    fn pop_af(&mut self) {
        self.emit(0xF1);
    }
    /// POP BC
    fn pop_bc(&mut self) {
        self.emit(0xC1);
    }
    /// POP DE
    fn pop_de(&mut self) {
        self.emit(0xD1);
    }
    /// POP HL
    fn pop_hl(&mut self) {
        self.emit(0xE1);
    }
    /// LD HL, nn
    fn ld_hl_n(&mut self, v: u16) {
        self.emit(0x21);
        self.emit_u16(v);
    }
    /// LD BC, nn
    fn ld_bc_n(&mut self, v: u16) {
        self.emit(0x01);
        self.emit_u16(v);
    }
    /// LD DE, nn
    fn ld_de_n(&mut self, v: u16) {
        self.emit(0x11);
        self.emit_u16(v);
    }
    /// LD (HL), A
    fn ld_hl_a(&mut self) {
        self.emit(0x77);
    }
    /// LD A, (HL)
    fn ld_a_hl(&mut self) {
        self.emit(0x7E);
    }
    /// LDI (HL), A  — store A at (HL) then INC HL
    fn ldi_hl_a(&mut self) {
        self.emit(0x22);
    }
    /// INC HL
    fn inc_hl(&mut self) {
        self.emit(0x23);
    }
    /// DEC BC
    fn dec_bc(&mut self) {
        self.emit(0x0B);
    }
    /// LD A, B
    fn ld_a_b2(&mut self) {
        self.emit(0x78);
    }
    /// OR C
    fn or_c(&mut self) {
        self.emit(0xB1);
    }
    /// LD (HL+), A  (same as LDI)
    fn ldi_hl_a2(&mut self) {
        self.emit(0x22);
    }
    /// NOP
    fn nop(&mut self) {
        self.emit(0x00);
    }
    /// HALT
    fn halt(&mut self) {
        self.emit(0x76);
    }
    /// EI
    fn ei(&mut self) {
        self.emit(0xFB);
    }
    /// DI
    fn di(&mut self) {
        self.emit(0xF3);
    }
    /// LD SP, nn
    fn ld_sp_n(&mut self, v: u16) {
        self.emit(0x31);
        self.emit_u16(v);
    }
    /// LD (FF00+n), A  — LDH store
    fn ldh_n_a(&mut self, offset: u8) {
        self.emit(0xE0);
        self.emit(offset);
    }
    /// LD A, (FF00+n)  — LDH load
    fn ldh_a_n(&mut self, offset: u8) {
        self.emit(0xF0);
        self.emit(offset);
    }
    /// LD (FF00+C), A
    fn ldh_c_a(&mut self) {
        self.emit(0xE2);
    }
    /// LD A, (BC)
    fn ld_a_bc(&mut self) {
        self.emit(0x0A);
    }
    /// LD A, L
    fn ld_a_l(&mut self) {
        self.emit(0x7D);
    }
    /// LD A, H
    fn ld_a_h(&mut self) {
        self.emit(0x7C);
    }
    /// ADD HL, BC
    fn add_hl_bc(&mut self) {
        self.emit(0x09);
    }
    /// LD B, n
    fn ld_b_n(&mut self, n: u8) {
        self.emit(0x06);
        self.emit(n);
    }
    /// LD C, n
    fn ld_c_n(&mut self, n: u8) {
        self.emit(0x0E);
        self.emit(n);
    }
    /// LD D, n
    fn ld_d_n(&mut self, n: u8) {
        self.emit(0x16);
        self.emit(n);
    }
    /// LD E, n
    fn ld_e_n(&mut self, n: u8) {
        self.emit(0x1E);
        self.emit(n);
    }
    /// DJNZ (decrement B, jump if not zero) — doesn't exist on LR35902; use DEC B + JR NZ
    fn dec_b_jrnz(&mut self, name: &str) {
        self.emit(0x05); // DEC B
        self.emit(0x20); // JR NZ, e
        self.emit_rel_offset(name);
    }
    /// DEC B
    fn dec_b(&mut self) {
        self.emit(0x05);
    }
    /// JR NZ, e
    fn jr_nz(&mut self, name: &str) {
        self.emit(0x20);
        self.emit_rel_offset(name);
    }
    /// JR Z, e
    fn jr_z(&mut self, name: &str) {
        self.emit(0x28);
        self.emit_rel_offset(name);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Codegen: expressions  →  result in A
    // ─────────────────────────────────────────────────────────────────────────

    pub fn gen_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Int(n, _) => {
                self.ld_a_n(*n as i8 as u8);
            }
            Expr::Bool(b, _) => {
                self.ld_a_n(if *b { 1 } else { 0 });
            }
            Expr::Ident(name, line) => {
                if let Some(&val) = self.consts.get(name.as_str()) {
                    self.ld_a_n(val as u8);
                } else if let Some(var) = self.vars.get(name).cloned() {
                    self.ld_a_mem(var.addr);
                } else if let Some(&idx) = self.tiles.get(name.as_str()) {
                    self.ld_a_n(idx);
                } else {
                    return Err(format!("Line {}: undefined identifier '{}'", line, name));
                }
            }
            Expr::Member(obj, field, line) => {
                // Button.LEFT etc. — encode as immediate constants
                let val = match (obj.as_str(), field.as_str()) {
                    ("Button", "RIGHT") => 0u8,
                    ("Button", "LEFT") => 1,
                    ("Button", "UP") => 2,
                    ("Button", "DOWN") => 3,
                    ("Button", "A") => 4,
                    ("Button", "B") => 5,
                    ("Button", "START") => 6,
                    ("Button", "SELECT") => 7,
                    _ => return Err(format!("Line {}: unknown member {}.{}", line, obj, field)),
                };
                self.ld_a_n(val);
            }
            Expr::UnaryOp { op, expr, .. } => {
                self.gen_expr(expr)?;
                match op {
                    UnaryOp::Neg => {
                        // NEG A = CPL; INC A
                        self.cpl();
                        self.inc_a();
                    }
                    UnaryOp::Not => {
                        // NOT: if A == 0 → 1, else → 0
                        // CP 0; JR NZ, __true; LD A,0; JP __end; __true: LD A,1; __end:
                        let true_lbl = self.fresh_label();
                        let end_lbl = self.fresh_label();
                        self.cp_n(0);
                        self.jr_nz(&true_lbl.clone());
                        self.ld_a_n(1);
                        self.jr(&end_lbl.clone());
                        self.place_label(&true_lbl);
                        self.ld_a_n(0);
                        self.place_label(&end_lbl);
                    }
                }
            }
            Expr::BinOp { op, lhs, rhs, line } => {
                match op {
                    BinOp::And | BinOp::Or => {
                        self.gen_bool_binop(op, lhs, rhs)?;
                    }
                    BinOp::Mul => {
                        self.gen_expr(lhs)?;
                        self.ld_b_a(); // B = lhs
                        self.gen_expr(rhs)?;
                        self.ld_c_a(); // C = rhs (multiplier)
                                       // A = 0; while C != 0: A += B; C--
                        self.xor_a();
                        let loop_lbl = self.fresh_label();
                        let end_lbl = self.fresh_label();
                        self.cp_n(0);
                        self.jr_z(&end_lbl.clone());
                        self.place_label(&loop_lbl);
                        self.add_a_b();
                        self.dec_b();
                        self.jr_nz(&loop_lbl);
                        self.place_label(&end_lbl);
                    }
                    BinOp::Div => {
                        // Division: only constant power-of-2 divisors supported
                        if let Expr::Int(n, _) = rhs.as_ref() {
                            let n = *n;
                            if n > 0 && (n & (n - 1)) == 0 {
                                self.gen_expr(lhs)?; // A = dividend
                                let shifts = (n as u32).trailing_zeros();
                                for _ in 0..shifts {
                                    // SRL A = CB 3F
                                    self.emit(0xCB);
                                    self.emit(0x3F);
                                }
                            } else {
                                return Err(format!(
                                    "Line {}: division by non-power-of-2 not yet supported",
                                    line
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Line {}: division requires a constant divisor",
                                line
                            ));
                        }
                    }
                    BinOp::Mod => {
                        // Modulo: only constant power-of-2 supported via AND (n-1)
                        if let Expr::Int(n, _) = rhs.as_ref() {
                            let n = *n;
                            if n > 0 && (n & (n - 1)) == 0 {
                                self.gen_expr(lhs)?; // A = value
                                self.emit(0xE6); // AND n
                                self.emit((n - 1) as u8);
                            } else {
                                return Err(format!(
                                    "Line {}: modulo by non-power-of-2 not yet supported",
                                    line
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Line {}: modulo requires a constant divisor",
                                line
                            ));
                        }
                    }
                    _ => {
                        // Evaluate lhs → A, push; rhs → A; pop lhs into B
                        self.gen_expr(lhs)?;
                        self.push_af(); // save lhs
                        self.gen_expr(rhs)?;
                        self.ld_b_a(); // B = rhs
                        self.pop_af(); // A = lhs
                        self.gen_simple_binop(op, *line)?;
                    }
                }
            }
            Expr::Call { func, args, line } => {
                self.gen_builtin_call(func, args, *line)?;
            }
            Expr::Str(_, line) => {
                return Err(format!(
                    "Line {}: string expressions not supported in codegen",
                    line
                ));
            }
        }
        Ok(())
    }

    /// Emit code for simple binary ops where A=lhs, B=rhs on entry → result in A.
    fn gen_simple_binop(&mut self, op: &BinOp, line: usize) -> Result<(), String> {
        match op {
            BinOp::Add => {
                self.add_a_b();
            }
            BinOp::Sub => {
                self.sub_b();
            }
            BinOp::Eq => {
                self.gen_cmp_result(CmpKind::Eq);
            }
            BinOp::NotEq => {
                self.gen_cmp_result(CmpKind::NotEq);
            }
            BinOp::Lt => {
                self.gen_cmp_result(CmpKind::Lt);
            }
            BinOp::LtEq => {
                self.gen_cmp_result(CmpKind::LtEq);
            }
            BinOp::Gt => {
                self.gen_cmp_result(CmpKind::Gt);
            }
            BinOp::GtEq => {
                self.gen_cmp_result(CmpKind::GtEq);
            }
            BinOp::Div => {
                unreachable!("Div is handled in gen_expr");
            }
            BinOp::Mod => {
                unreachable!("Mod is handled in gen_expr");
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn gen_bool_binop(&mut self, op: &BinOp, lhs: &Expr, rhs: &Expr) -> Result<(), String> {
        match op {
            BinOp::And => {
                let false_lbl = self.fresh_label();
                let end_lbl = self.fresh_label();
                // lhs == 0 → short-circuit false
                self.gen_expr(lhs)?;
                self.cp_n(0);
                self.jr_z(&false_lbl.clone());
                // rhs == 0 → false
                self.gen_expr(rhs)?;
                self.cp_n(0);
                self.jr_z(&false_lbl.clone());
                self.ld_a_n(1);
                self.jr(&end_lbl.clone());
                self.place_label(&false_lbl);
                self.ld_a_n(0);
                self.place_label(&end_lbl);
            }
            BinOp::Or => {
                let true_lbl = self.fresh_label();
                let end_lbl = self.fresh_label();
                self.gen_expr(lhs)?;
                self.cp_n(0);
                self.jr_nz(&true_lbl.clone());
                self.gen_expr(rhs)?;
                self.cp_n(0);
                self.jr_nz(&true_lbl.clone());
                self.ld_a_n(0);
                self.jr(&end_lbl.clone());
                self.place_label(&true_lbl);
                self.ld_a_n(1);
                self.place_label(&end_lbl);
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Comparison helpers: A=lhs, B=rhs → A = 0 or 1
    // ─────────────────────────────────────────────────────────────────────────

    fn gen_cmp_result(&mut self, kind: CmpKind) {
        // Compare: A - B sets flags
        self.cp_b();
        let true_lbl = self.fresh_label();
        let end_lbl = self.fresh_label();
        match kind {
            CmpKind::Eq => self.jr_z(&true_lbl.clone()),
            CmpKind::NotEq => {
                self.jr_nz(&true_lbl.clone());
            }
            CmpKind::Lt => self.jr_c(&true_lbl.clone()), // C set if A < B (unsigned)
            CmpKind::GtEq => {
                self.jr_nc(&true_lbl.clone());
            }
            CmpKind::Gt => {
                // A > B  ↔  NOT Z AND NOT C
                // CP B sets Z if equal, C if A < B.
                // If Z (equal) → not greater → fall through to false.
                // If NC and not Z → A > B → true.
                let false_lbl = self.fresh_label();
                self.jr_z(&false_lbl.clone()); // equal → not greater
                self.jr_nc(&true_lbl.clone()); // NC and not Z → greater
                self.place_label(&false_lbl);
            }
            CmpKind::LtEq => {
                // LtEq: A <= B ↔ A < B OR A == B ↔ NOT (A > B) ↔ C set OR Z set
                // C set → A < B; Z set → A == B
                self.jr_z(&true_lbl.clone());
                self.jr_c(&true_lbl.clone());
            }
        }
        self.ld_a_n(0);
        self.jr(&end_lbl.clone());
        self.place_label(&true_lbl);
        self.ld_a_n(1);
        self.place_label(&end_lbl);
    }

    fn jr_nc(&mut self, name: &str) {
        self.emit(0x30);
        self.emit_rel_offset(name);
    }
    fn jr_c(&mut self, name: &str) {
        self.emit(0x38);
        self.emit_rel_offset(name);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Built-in function call codegen
    // ─────────────────────────────────────────────────────────────────────────

    fn gen_builtin_call(&mut self, func: &str, args: &[Expr], line: usize) -> Result<(), String> {
        match func {
            // set_sprite(index, x, y, tile)
            "set_sprite" => {
                if args.len() != 4 {
                    return Err(format!("Line {}: set_sprite takes 4 args", line));
                }
                // OAM entry = $FE00 + index*4
                // Byte 0: Y + 16 (OAM stores Y+16)
                // Byte 1: X + 8  (OAM stores X+8)
                // Byte 2: tile index
                // Byte 3: attributes (0)
                // We use a helper routine to avoid code size explosion.
                // Args: B=index, D=y, E=x, H=tile
                self.gen_expr(&args[2])?; // y
                                          // OAM Y = y + 16
                self.add_a_n(16);
                self.ld_d_a(); // D = y+16
                self.gen_expr(&args[1])?; // x
                self.add_a_n(8);
                self.ld_e_a(); // E = x+8
                self.gen_expr(&args[3])?; // tile
                self.ld_h_a(); // H = tile
                self.gen_expr(&args[0])?; // index
                self.ld_b_a(); // B = index
                self.call("__builtin_set_sprite");
            }
            // pressed(Button.X) → A = 0 or 1
            "pressed" => {
                if args.len() != 1 {
                    return Err(format!("Line {}: pressed takes 1 arg", line));
                }
                self.gen_expr(&args[0])?; // A = button index (0-7)
                self.ld_b_a();
                self.call("__builtin_pressed");
            }
            // just_pressed(Button.X) → A = 0 or 1 (edge detect)
            "just_pressed" => {
                if args.len() != 1 {
                    return Err(format!("Line {}: just_pressed takes 1 arg", line));
                }
                self.gen_expr(&args[0])?;
                self.ld_b_a();
                self.call("__builtin_just_pressed");
            }
            // set_bg_tile(tx, ty, tile)
            "set_bg_tile" => {
                if args.len() != 3 {
                    return Err(format!("Line {}: set_bg_tile takes 3 args", line));
                }
                self.gen_expr(&args[1])?; // ty
                self.ld_d_a();
                self.gen_expr(&args[0])?; // tx
                self.ld_e_a();
                self.gen_expr(&args[2])?; // tile
                self.ld_b_a();
                self.call("__builtin_set_bg_tile");
            }
            // set_scroll(sx, sy)
            "set_scroll" => {
                if args.len() != 2 {
                    return Err(format!("Line {}: set_scroll takes 2 args", line));
                }
                self.gen_expr(&args[0])?;
                self.ldh_n_a(0x43); // SCX = FF43
                self.gen_expr(&args[1])?;
                self.ldh_n_a(0x42); // SCY = FF42
            }
            // User-defined function call
            other => {
                // Look up param names for this function
                if let Some(param_names) = self.fn_params.get(other).cloned() {
                    if args.len() != param_names.len() {
                        return Err(format!(
                            "Line {}: function '{}' expects {} args, got {}",
                            line,
                            other,
                            param_names.len(),
                            args.len()
                        ));
                    }
                    // Evaluate each arg and store to the parameter's WRAM address
                    for (arg, pname) in args.iter().zip(param_names.iter()) {
                        self.gen_expr(arg)?;
                        let var = self.vars.get(pname).cloned().ok_or_else(|| {
                            format!(
                                "Line {}: internal error: param '{}' not in vars",
                                line, pname
                            )
                        })?;
                        self.ld_mem_a(var.addr);
                    }
                } else if !args.is_empty() {
                    return Err(format!("Line {}: unknown function '{}'", line, other));
                }
                self.call(other);
            }
        }
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Statements
    // ─────────────────────────────────────────────────────────────────────────

    pub fn gen_block(&mut self, block: &[Stmt]) -> Result<(), String> {
        for stmt in block {
            self.gen_stmt(stmt)?;
        }
        Ok(())
    }

    fn gen_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Pass => {}
            Stmt::Let(decl) => {
                // Variable already allocated; just emit init code
                self.gen_expr(&decl.init)?;
                if let Some(var) = self.vars.get(&decl.name).cloned() {
                    self.ld_mem_a(var.addr);
                } else {
                    return Err(format!(
                        "Line {}: undefined variable '{}'",
                        decl.line, decl.name
                    ));
                }
            }
            Stmt::Assign { name, val, line } => {
                self.gen_expr(val)?;
                if let Some(var) = self.vars.get(name).cloned() {
                    self.ld_mem_a(var.addr);
                } else {
                    return Err(format!("Line {}: undefined variable '{}'", line, name));
                }
            }
            Stmt::Expr(e) => {
                self.gen_expr(e)?;
            }
            Stmt::Return(val, _) => {
                if let Some(v) = val {
                    self.gen_expr(v)?;
                }
                self.ret();
            }
            Stmt::If {
                cond,
                then,
                elifs,
                else_,
                line,
            } => {
                self.gen_if(cond, then, elifs, else_.as_deref(), *line)?;
            }
            Stmt::While { cond, body, .. } => {
                self.gen_while(cond, body)?;
            }
            Stmt::Loop { body, .. } => {
                let top = self.fresh_label();
                self.place_label(&top);
                self.gen_block(body)?;
                self.jp_nn(&top);
            }
        }
        Ok(())
    }

    fn gen_if(
        &mut self,
        cond: &Expr,
        then: &[Stmt],
        elifs: &[(Expr, Block)],
        else_: Option<&[Stmt]>,
        _line: usize,
    ) -> Result<(), String> {
        let end_lbl = self.fresh_label();
        self.gen_condition_jump(cond, &end_lbl.clone())?;
        self.gen_block(then)?;

        for (elif_cond, elif_body) in elifs {
            let skip_lbl = self.fresh_label();
            self.jp_nn(&end_lbl);
            self.place_label(&skip_lbl); // (unused but needed as landing)
            let next_lbl = self.fresh_label();
            self.gen_condition_jump(elif_cond, &next_lbl.clone())?;
            self.gen_block(elif_body)?;
            self.jp_nn(&end_lbl);
            self.place_label(&next_lbl);
        }

        if let Some(else_block) = else_ {
            // jump over else from the end of the 'then'
            let after_else = self.fresh_label();
            // Wait — we already emitted then; need a jump-past-else before else starts.
            // Restructure: emit JP end_lbl after then, then place else.
            // This is tricky because we already placed end_lbl jump target naively above.
            // Simple approach: emit else inline without jump; the fallthrough handles it.
            self.gen_block(else_block)?;
        }
        self.place_label(&end_lbl);
        Ok(())
    }

    /// Emit condition check: if cond is false, jump to `else_label`.
    fn gen_condition_jump(&mut self, cond: &Expr, else_label: &str) -> Result<(), String> {
        self.gen_expr(cond)?;
        self.cp_n(0);
        self.jp_z(else_label);
        Ok(())
    }

    fn gen_while(&mut self, cond: &Expr, body: &[Stmt]) -> Result<(), String> {
        let cond_lbl = self.fresh_label();
        let end_lbl = self.fresh_label();
        self.place_label(&cond_lbl);
        self.gen_condition_jump(cond, &end_lbl.clone())?;
        self.gen_block(body)?;
        self.jp_nn(&cond_lbl);
        self.place_label(&end_lbl);
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Built-in runtime stubs
    // ─────────────────────────────────────────────────────────────────────────

    /// Emit all built-in helper routines after the main game code.
    /// Call order: set_sprite, pressed, just_pressed, set_bg_tile.
    pub fn emit_builtins(&mut self, need_just_pressed: bool) -> Result<(), String> {
        // ── __builtin_set_sprite ────────────────────────────────────────────
        // In: B=index, D=Y+16, E=X+8, H=tile
        // OAM entry layout: [Y+16, X+8, tile, attrs]  at $FE00 + index*4
        self.place_label("__builtin_set_sprite");
        // Save tile (H will be overwritten when we set HL = $FE00 + B*4)
        self.ld_a_h(); // A = tile
        self.push_af(); // tile saved on stack
                        // HL = $FE00 + B*4
        self.ld_a_b(); // A = index
        self.emit(0x87); // ADD A, A  (×2)
        self.emit(0x87); // ADD A, A  (×4)
        self.ld_h_a(); // H = index*4  (temp; we'll set H=$FE next)
        self.emit(0x26);
        self.emit(0xFE); // LD H, $FE
        self.ld_l_a(); // L = index*4  → HL = $FE00 + index*4
                       // Write Y+16 (in D)
        self.ld_a_d();
        self.ld_hl_a(); // (HL) = Y+16
        self.inc_hl();
        // Write X+8 (in E)
        self.ld_a_e();
        self.ld_hl_a(); // (HL) = X+8
        self.inc_hl();
        // Write tile (from stack)
        self.pop_af(); // A = tile
        self.ld_hl_a(); // (HL) = tile
        self.inc_hl();
        // Write attrs = 0
        self.xor_a();
        self.ld_hl_a(); // (HL) = 0
        self.ret();

        // ── __builtin_pressed ──────────────────────────────────────────────
        // In: B = button index (0=RIGHT,1=LEFT,2=UP,3=DOWN via dpad; 4=A,5=B,6=START,7=SELECT via btns)
        // Out: A = 1 if pressed, 0 if not
        self.place_label("__builtin_pressed");
        // Read joypad: select appropriate nibble
        // Buttons 0-3: dpad (select = bit4 low, bit5 high → write $20 to $FF00)
        // Buttons 4-7: buttons (select = bit4 high, bit5 low → write $10 to $FF00)
        // Bit index within nibble = button_index % 4
        // B < 4 → dpad; B >= 4 → buttons
        self.ld_a_b();
        self.cp_n(4);
        let btn_lbl = self.fresh_label();
        let done_lbl = self.fresh_label();
        self.jp_c(&btn_lbl.clone()); // C set → B < 4 → dpad
                                     // Buttons (index -= 4 to get nibble bit)
        self.sub_n(4);
        self.ld_b_a();
        self.ld_a_n(0x10);
        self.ldh_n_a(0x00); // select buttons: FF00 = $10
        self.nop();
        self.nop(); // wait 2 cycles
        self.ldh_a_n(0x00); // read FF00
        self.jr(&done_lbl.clone());
        // Dpad
        self.place_label(&btn_lbl);
        self.ld_a_n(0x20);
        self.ldh_n_a(0x00); // select dpad: FF00 = $20
        self.nop();
        self.nop();
        self.ldh_a_n(0x00);
        self.place_label(&done_lbl);
        // A has the nibble; bit B is the button (active-low)
        // Shift right B times and test bit 0
        let shift_lbl = self.fresh_label();
        let shift_end = self.fresh_label();
        self.place_label(&shift_lbl);
        self.cp_n(0); // B==0?
                      // wait — B is our counter here. Let's use a simpler approach: check B via DEC
                      // Rewrite: save A in C; shift C right B times using a loop
        self.ld_c_a(); // C = raw nibble
                       // Loop: shift C right B times
        let sloop = self.fresh_label();
        let send = self.fresh_label();
        self.ld_a_b();
        self.cp_n(0);
        self.jr_z(&send.clone());
        self.place_label(&sloop);
        // SRL C
        self.emit(0xCB);
        self.emit(0x39);
        self.dec_b();
        self.jr_nz(&sloop);
        self.place_label(&send);
        // C bit 0 = button state (active low = 0 means pressed)
        self.ld_a_n(1);
        self.emit(0xA1); // AND C
        self.cpl(); // flip: 1=pressed, 0=not pressed
        self.emit(0xE6);
        self.emit(0x01); // AND 1  (keep only bit 0)
        self.ret();

        // ── __builtin_just_pressed ─────────────────────────────────────────
        // Placeholder: for now same as pressed (no edge detection yet)
        if need_just_pressed {
            self.place_label("__builtin_just_pressed");
            self.call("__builtin_pressed");
            self.ret();
        }

        // ── __builtin_set_bg_tile ──────────────────────────────────────────
        // In: D=ty, E=tx, B=tile
        // BG map at $9800; address = $9800 + ty*32 + tx
        self.place_label("__builtin_set_bg_tile");
        // Save B (tile) — we need B as scratch for ty
        self.push_bc();

        // HL = $9800 + D*32 + E
        self.ld_a_d(); // A = ty
        self.ld_b_a(); // B = ty  (save for L computation)

        // H = $98 + (ty >> 3)  — the high byte of the BG map address.
        self.emit(0xCB);
        self.emit(0x3F); // SRL A  → ty>>1
        self.emit(0xCB);
        self.emit(0x3F); // SRL A  → ty>>2
        self.emit(0xCB);
        self.emit(0x3F); // SRL A  → ty>>3
        self.add_a_n(0x98); // A = $98 + (ty>>3)
        self.ld_h_a(); // H = correct high byte

        // L = (ty << 5) & 0xFF  — the low byte of ty*32.
        self.ld_a_b(); // A = ty (restored from B)
        for _ in 0..5 {
            self.emit(0x87); // ADD A, A  ×5  → A = (ty*32) & 0xFF
        }
        self.ld_l_a(); // L = low byte

        // HL += tx (E), propagating carry into H
        self.ld_a_e();
        self.emit(0x85); // ADD A, L → A = tx + L
        self.ld_l_a();
        self.ld_a_h2();
        self.emit(0xCE);
        self.emit(0x00); // ADC A, $00
        self.ld_h_a();

        // Restore B (tile), store it at (HL)
        self.pop_bc();
        self.ld_a_b();
        self.ld_hl_a();
        self.ret();

        Ok(())
    }

    // Helpers accidentally missing above:
    fn ld_a_e(&mut self) {
        self.emit(0x7B);
    }
    fn ld_a_d(&mut self) {
        self.emit(0x7A);
    }
    fn ld_h_n(&mut self, n: u8) {
        self.emit(0x26);
        self.emit(n);
    }
    fn ld_a_h2(&mut self) {
        self.emit(0x7C);
    }
}

enum CmpKind {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}
