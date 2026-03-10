//! AST node types for the Game Boy ROM language.

#[derive(Debug, Clone)]
pub struct Program {
    pub imports: Vec<Import>,
    pub tiles: Vec<TileDef>,
    pub globals: Vec<LetDecl>,
    pub consts: Vec<ConstDecl>,
    pub functions: Vec<FnDef>,
    pub init: Option<Block>,
    pub on_vblank: Option<Block>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub module: String,
    pub names: Vec<String>,
    pub line: usize,
}

/// Tile definition: 8×8 grid of 0-3 pixel values, or a pixel character map.
#[derive(Debug, Clone)]
pub struct TileDef {
    pub name: String,
    /// Exactly 8 rows of 8 characters each. '.' = 0, '1'=1, '2'=2, '3'=3.
    /// Spaces are not allowed inside the grid.
    pub rows: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct LetDecl {
    pub name: String,
    pub ty: Option<Type>,
    pub init: Expr,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub value: i32,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Option<Type>,
    pub body: Block,
    pub line: usize,
}

pub type Block = Vec<Stmt>;

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetDecl),
    Assign {
        name: String,
        val: Expr,
        line: usize,
    },
    If {
        cond: Expr,
        then: Block,
        elifs: Vec<(Expr, Block)>,
        else_: Option<Block>,
        line: usize,
    },
    While {
        cond: Expr,
        body: Block,
        line: usize,
    },
    Loop {
        body: Block,
        line: usize,
    },
    Return(Option<Expr>, usize),
    Expr(Expr),
    Pass,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i32, usize),
    Bool(bool, usize),
    Str(String, usize),
    Ident(String, usize),
    /// member access: Button.LEFT → Member("Button", "LEFT")
    Member(String, String, usize),
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        line: usize,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
        line: usize,
    },
    Call {
        func: String,
        args: Vec<Expr>,
        line: usize,
    },
}

impl Expr {
    pub fn line(&self) -> usize {
        match self {
            Expr::Int(_, l)
            | Expr::Bool(_, l)
            | Expr::Str(_, l)
            | Expr::Ident(_, l)
            | Expr::Member(_, _, l) => *l,
            Expr::BinOp { line, .. } | Expr::UnaryOp { line, .. } | Expr::Call { line, .. } => {
                *line
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    U8,
    I8,
    U16,
    Bool,
}
