//! Parser: token stream → AST
//! Uses a recursive-descent approach with a manual indent/dedent stack.

use crate::ast::*;
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // -----------------------------------------------------------------------
    // Utility helpers
    // -----------------------------------------------------------------------

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn peek_line(&self) -> usize {
        self.tokens[self.pos].line
    }

    fn bump(&mut self) -> &TokenKind {
        let k = &self.tokens[self.pos].kind;
        self.pos += 1;
        k
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<(), String> {
        if self.peek() == expected {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!(
                "Line {}: expected {:?}, got {:?}",
                self.peek_line(),
                expected,
                self.peek()
            ))
        }
    }

    /// Skip Newline tokens
    fn skip_newlines(&mut self) {
        while self.peek() == &TokenKind::Newline {
            self.pos += 1;
        }
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    // -----------------------------------------------------------------------
    // Top-level
    // -----------------------------------------------------------------------

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut prog = Program {
            imports: vec![],
            tiles: vec![],
            globals: vec![],
            functions: vec![],
            init: None,
            on_vblank: None,
        };

        self.skip_newlines();
        while !self.at_eof() {
            match self.peek().clone() {
                TokenKind::From => prog.imports.push(self.parse_import()?),
                TokenKind::Tile => prog.tiles.push(self.parse_tile()?),
                TokenKind::Let => prog.globals.push(self.parse_let_decl()?),
                TokenKind::Fn => prog.functions.push(self.parse_fn()?),
                TokenKind::Init => {
                    self.bump(); // consume 'init'
                    self.expect(&TokenKind::Colon)?;
                    self.skip_newlines();
                    prog.init = Some(self.parse_block()?);
                }
                TokenKind::On => {
                    self.bump(); // consume 'on'
                    // Expect 'vblank'
                    if let TokenKind::Ident(name) = self.peek().clone() {
                        if name == "vblank" {
                            self.bump();
                            self.expect(&TokenKind::Colon)?;
                            self.skip_newlines();
                            prog.on_vblank = Some(self.parse_block()?);
                        } else {
                            return Err(format!(
                                "Line {}: unknown event '{}'",
                                self.peek_line(),
                                name
                            ));
                        }
                    } else {
                        return Err(format!(
                            "Line {}: expected event name after 'on'",
                            self.peek_line()
                        ));
                    }
                }
                TokenKind::Newline => {
                    self.bump();
                }
                other => {
                    return Err(format!(
                        "Line {}: unexpected top-level token {:?}",
                        self.peek_line(),
                        other
                    ));
                }
            }
        }
        Ok(prog)
    }

    // -----------------------------------------------------------------------
    // Import: `from core import a, b, c`
    // -----------------------------------------------------------------------

    fn parse_import(&mut self) -> Result<Import, String> {
        let line = self.peek_line();
        self.expect(&TokenKind::From)?;
        let module = if let TokenKind::Ident(m) = self.peek().clone() {
            self.bump();
            m
        } else {
            return Err(format!("Line {}: expected module name", line));
        };
        self.expect(&TokenKind::Import)?;
        let mut names = vec![];
        loop {
            if let TokenKind::Ident(n) = self.peek().clone() {
                self.bump();
                names.push(n);
            } else {
                break;
            }
            if self.peek() == &TokenKind::Comma {
                self.bump();
            } else {
                break;
            }
        }
        self.skip_newlines();
        Ok(Import { module, names, line })
    }

    // -----------------------------------------------------------------------
    // Tile: `tile name:\n    INDENT rows DEDENT`
    // -----------------------------------------------------------------------

    fn parse_tile(&mut self) -> Result<TileDef, String> {
        let line = self.peek_line();
        self.expect(&TokenKind::Tile)?;
        let name = if let TokenKind::Ident(n) = self.peek().clone() {
            self.bump();
            n
        } else {
            return Err(format!("Line {}: expected tile name", line));
        };
        self.expect(&TokenKind::Colon)?;
        self.skip_newlines();
        self.expect(&TokenKind::Indent)?;

        // Each indented line is a row of pixel characters.
        // We reconstruct rows from Ident tokens and Int tokens that
        // make up pixel row strings like ".333...." or "33333333".
        let mut rows: Vec<String> = vec![];
        loop {
            match self.peek().clone() {
                TokenKind::Dedent | TokenKind::Eof => break,
                TokenKind::Newline => {
                    self.bump();
                }
                // A tile row is an identifier like ".333...." or "33333333"
                TokenKind::Ident(s) => {
                    self.bump();
                    rows.push(s);
                    // might have trailing newline
                    if self.peek() == &TokenKind::Newline {
                        self.bump();
                    }
                }
                // Rows starting with a digit get lexed as Int then Ident parts
                TokenKind::Int(n) => {
                    // row like "3333...." — accumulate adjacent int/dot/ident tokens
                    let mut row = n.to_string();
                    self.bump();
                    loop {
                        match self.peek().clone() {
                            TokenKind::Ident(s) => { row.push_str(&s); self.bump(); }
                            TokenKind::Int(n2)  => { row.push_str(&n2.to_string()); self.bump(); }
                            TokenKind::Dot      => { row.push('.'); self.bump(); }
                            _ => break,
                        }
                    }
                    if self.peek() == &TokenKind::Newline {
                        self.bump();
                    }
                    rows.push(row);
                }
                // Dot at start of row
                TokenKind::Dot => {
                    let mut row = String::from(".");
                    self.bump();
                    // accumulate rest
                    loop {
                        match self.peek().clone() {
                            TokenKind::Dot => { self.bump(); row.push('.'); }
                            TokenKind::Int(n) => { row.push_str(&n.to_string()); self.bump(); }
                            TokenKind::Ident(s) => { row.push_str(&s); self.bump(); }
                            _ => break,
                        }
                    }
                    if self.peek() == &TokenKind::Newline {
                        self.bump();
                    }
                    rows.push(row);
                }
                other => {
                    return Err(format!("Line {}: unexpected in tile body: {:?}", self.peek_line(), other));
                }
            }
        }
        self.expect(&TokenKind::Dedent)?;
        self.skip_newlines();

        if rows.len() != 8 {
            return Err(format!("Line {}: tile '{}' must have exactly 8 rows, got {}", line, name, rows.len()));
        }
        for (i, row) in rows.iter().enumerate() {
            if row.len() != 8 {
                return Err(format!("Line {}: tile '{}' row {} must be 8 chars wide, got {}", line, name, i, row.len()));
            }
        }
        Ok(TileDef { name, rows, line })
    }

    // -----------------------------------------------------------------------
    // Let declaration: `let name[: type] = expr`
    // -----------------------------------------------------------------------

    fn parse_let_decl(&mut self) -> Result<LetDecl, String> {
        let line = self.peek_line();
        self.expect(&TokenKind::Let)?;
        let name = if let TokenKind::Ident(n) = self.peek().clone() {
            self.bump();
            n
        } else {
            return Err(format!("Line {}: expected variable name after 'let'", line));
        };

        let ty = if self.peek() == &TokenKind::Colon {
            self.bump();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(&TokenKind::Eq)?;
        let init = self.parse_expr(0)?;
        self.skip_newlines();
        Ok(LetDecl { name, ty, init, line })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let line = self.peek_line();
        match self.peek().clone() {
            TokenKind::TypeU8 => { self.bump(); Ok(Type::U8) }
            TokenKind::TypeI8 => { self.bump(); Ok(Type::I8) }
            TokenKind::TypeU16 => { self.bump(); Ok(Type::U16) }
            TokenKind::TypeBool => { self.bump(); Ok(Type::Bool) }
            other => Err(format!("Line {}: expected type, got {:?}", line, other)),
        }
    }

    // -----------------------------------------------------------------------
    // Function definition
    // -----------------------------------------------------------------------

    fn parse_fn(&mut self) -> Result<FnDef, String> {
        let line = self.peek_line();
        self.expect(&TokenKind::Fn)?;
        let name = if let TokenKind::Ident(n) = self.peek().clone() {
            self.bump();
            n
        } else {
            return Err(format!("Line {}: expected function name", line));
        };
        self.expect(&TokenKind::LParen)?;
        let mut params = vec![];
        while self.peek() != &TokenKind::RParen {
            let pname = if let TokenKind::Ident(n) = self.peek().clone() {
                self.bump();
                n
            } else {
                return Err(format!("Line {}: expected parameter name", self.peek_line()));
            };
            self.expect(&TokenKind::Colon)?;
            let ty = self.parse_type()?;
            params.push((pname, ty));
            if self.peek() == &TokenKind::Comma { self.bump(); }
        }
        self.expect(&TokenKind::RParen)?;

        // Optional return type (skipped for now — unused in Phase 1)
        let ret = None;

        self.expect(&TokenKind::Colon)?;
        self.skip_newlines();
        let body = self.parse_block()?;
        Ok(FnDef { name, params, ret, body, line })
    }

    // -----------------------------------------------------------------------
    // Block (indented sequence of statements)
    // -----------------------------------------------------------------------

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(&TokenKind::Indent)?;
        let mut stmts = vec![];
        loop {
            self.skip_newlines();
            match self.peek().clone() {
                TokenKind::Dedent | TokenKind::Eof => break,
                _ => stmts.push(self.parse_stmt()?),
            }
        }
        self.expect(&TokenKind::Dedent)?;
        Ok(stmts)
    }

    // -----------------------------------------------------------------------
    // Statements
    // -----------------------------------------------------------------------

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let line = self.peek_line();
        match self.peek().clone() {
            TokenKind::Let => Ok(Stmt::Let(self.parse_let_decl()?)),
            TokenKind::Return => {
                self.bump();
                if self.peek() == &TokenKind::Newline || self.peek() == &TokenKind::Eof {
                    self.skip_newlines();
                    Ok(Stmt::Return(None, line))
                } else {
                    let e = self.parse_expr(0)?;
                    self.skip_newlines();
                    Ok(Stmt::Return(Some(e), line))
                }
            }
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::Loop => {
                self.bump();
                self.expect(&TokenKind::Colon)?;
                self.skip_newlines();
                let body = self.parse_block()?;
                Ok(Stmt::Loop { body, line })
            }
            TokenKind::Pass => {
                self.bump();
                self.skip_newlines();
                Ok(Stmt::Pass)
            }
            // Assignment: `name :=` or function call starting with an identifier
            TokenKind::Ident(name) => {
                // Peek ahead for :=
                if self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Walrus) {
                    self.bump(); // name
                    self.bump(); // :=
                    let val = self.parse_expr(0)?;
                    self.skip_newlines();
                    Ok(Stmt::Assign { name, val, line })
                } else {
                    // Expression statement (function call)
                    let e = self.parse_expr(0)?;
                    self.skip_newlines();
                    Ok(Stmt::Expr(e))
                }
            }
            other => Err(format!("Line {}: unexpected statement token {:?}", line, other)),
        }
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        let line = self.peek_line();
        self.expect(&TokenKind::If)?;
        let cond = self.parse_expr(0)?;
        self.expect(&TokenKind::Colon)?;
        self.skip_newlines();
        let then = self.parse_block()?;

        let mut elifs = vec![];
        let mut else_ = None;

        loop {
            self.skip_newlines();
            match self.peek().clone() {
                TokenKind::Elif => {
                    self.bump();
                    let c = self.parse_expr(0)?;
                    self.expect(&TokenKind::Colon)?;
                    self.skip_newlines();
                    let b = self.parse_block()?;
                    elifs.push((c, b));
                }
                TokenKind::Else => {
                    self.bump();
                    self.expect(&TokenKind::Colon)?;
                    self.skip_newlines();
                    else_ = Some(self.parse_block()?);
                    break;
                }
                _ => break,
            }
        }
        Ok(Stmt::If { cond, then, elifs, else_, line })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        let line = self.peek_line();
        self.expect(&TokenKind::While)?;
        let cond = self.parse_expr(0)?;
        self.expect(&TokenKind::Colon)?;
        self.skip_newlines();
        let body = self.parse_block()?;
        Ok(Stmt::While { cond, body, line })
    }

    // -----------------------------------------------------------------------
    // Expressions (Pratt / precedence climbing)
    // -----------------------------------------------------------------------

    fn parse_expr(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut lhs = self.parse_unary()?;
        loop {
            let (prec, right_assoc) = match self.peek() {
                TokenKind::Or => (1, false),
                TokenKind::And => (2, false),
                TokenKind::EqEq | TokenKind::NotEq
                | TokenKind::Lt | TokenKind::LtEq
                | TokenKind::Gt | TokenKind::GtEq => (3, false),
                TokenKind::Plus | TokenKind::Minus => (4, false),
                TokenKind::Star | TokenKind::Slash | TokenKind::Percent => (5, false),
                _ => break,
            };
            if prec < min_prec {
                break;
            }
            let line = self.peek_line();
            let op = match self.bump().clone() {
                TokenKind::Or => BinOp::Or,
                TokenKind::And => BinOp::And,
                TokenKind::EqEq => BinOp::Eq,
                TokenKind::NotEq => BinOp::NotEq,
                TokenKind::Lt => BinOp::Lt,
                TokenKind::LtEq => BinOp::LtEq,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::GtEq => BinOp::GtEq,
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => unreachable!(),
            };
            let next_prec = if right_assoc { prec } else { prec + 1 };
            let rhs = self.parse_expr(next_prec)?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs), line };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        let line = self.peek_line();
        match self.peek().clone() {
            TokenKind::Minus => {
                self.bump();
                let e = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Neg, expr: Box::new(e), line })
            }
            TokenKind::Not => {
                self.bump();
                let e = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Not, expr: Box::new(e), line })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let line = self.peek_line();
        match self.peek().clone() {
            TokenKind::Int(n) => { self.bump(); Ok(Expr::Int(n, line)) }
            TokenKind::True => { self.bump(); Ok(Expr::Bool(true, line)) }
            TokenKind::False => { self.bump(); Ok(Expr::Bool(false, line)) }
            TokenKind::StringLit(s) => { self.bump(); Ok(Expr::Str(s, line)) }
            TokenKind::LParen => {
                self.bump();
                let e = self.parse_expr(0)?;
                self.expect(&TokenKind::RParen)?;
                Ok(e)
            }
            TokenKind::Ident(name) => {
                self.bump();
                // Check for member access: Name.field
                if self.peek() == &TokenKind::Dot {
                    self.bump();
                    if let TokenKind::Ident(field) = self.peek().clone() {
                        self.bump();
                        return Ok(Expr::Member(name, field, line));
                    } else {
                        return Err(format!("Line {}: expected field name after '.'", line));
                    }
                }
                // Check for function call: name(args)
                if self.peek() == &TokenKind::LParen {
                    self.bump();
                    let mut args = vec![];
                    while self.peek() != &TokenKind::RParen {
                        args.push(self.parse_expr(0)?);
                        if self.peek() == &TokenKind::Comma {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    return Ok(Expr::Call { func: name, args, line });
                }
                Ok(Expr::Ident(name, line))
            }
            other => Err(format!("Line {}: unexpected token in expression: {:?}", line, other)),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, String> {
    let mut p = Parser::new(tokens);
    p.parse_program()
}
