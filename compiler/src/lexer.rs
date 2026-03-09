//! Lexer: source text → Vec<Token>

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Int(i32),
    Ident(String),
    StringLit(String),

    // Keywords
    Let,
    Fn,
    Return,
    If,
    Elif,
    Else,
    While,
    Loop,
    From,
    Import,
    Tile,
    On,
    Init,
    Pass,
    And,
    Or,
    Not,
    True,
    False,

    // Symbols
    Colon,
    Comma,
    Dot,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,     // =
    Walrus, // :=
    EqEq,   // ==
    NotEq,  // !=
    Lt,     // <
    LtEq,   // <=
    Gt,     // >
    GtEq,   // >=
    LParen,
    RParen,
    LBracket,
    RBracket,

    // Type annotations
    TypeU8,
    TypeI8,
    TypeU16,
    TypeBool,

    // Structural
    Newline,
    Indent,
    Dedent,
    Eof,
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut indent_stack: Vec<usize> = vec![0];
    let mut line = 1usize;
    let mut pending_indent = false;

    let mut chars = src.chars().peekable();

    macro_rules! peek {
        () => {
            chars.peek().copied()
        };
    }
    macro_rules! bump {
        () => {
            chars.next()
        };
    }

    // Helper: emit token at current line
    macro_rules! tok {
        ($k:expr) => {
            Token { kind: $k, line }
        };
    }

    // We process the source line-by-line to handle indentation correctly.
    let lines: Vec<&str> = src.split('\n').collect();

    for (line_idx, raw_line) in lines.iter().enumerate() {
        line = line_idx + 1;

        // Skip blank lines and comment-only lines
        let trimmed = raw_line.trim_end();
        let stripped = if let Some(c) = trimmed.find('#') {
            trimmed[..c].trim_end()
        } else {
            trimmed
        };
        if stripped.is_empty() {
            continue;
        }

        // Measure indent (spaces only)
        let indent = stripped.len() - stripped.trim_start_matches(' ').len()
            + (raw_line.len() - raw_line.trim_start_matches(' ').len());
        // Simpler: count leading spaces in original (trimmed is already stripped of trailing)
        let indent = raw_line.len() - raw_line.trim_start_matches(' ').len();

        let current_indent = *indent_stack.last().unwrap();
        if indent > current_indent {
            indent_stack.push(indent);
            tokens.push(tok!(TokenKind::Indent));
        } else {
            while indent < *indent_stack.last().unwrap() {
                indent_stack.pop();
                tokens.push(tok!(TokenKind::Dedent));
            }
            if indent != *indent_stack.last().unwrap() {
                return Err(format!("Line {}: inconsistent indentation", line));
            }
        }

        // Tokenize the content of this line
        let content = stripped.trim_start_matches(' ');
        let mut ci = content.chars().peekable();

        loop {
            // Skip spaces (within line)
            while ci.peek() == Some(&' ') {
                ci.next();
            }
            match ci.peek() {
                None => break,
                Some('#') => break, // comment
                _ => {}
            }

            let ch = match ci.next() {
                None => break,
                Some(c) => c,
            };

            let kind = match ch {
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '[' => TokenKind::LBracket,
                ']' => TokenKind::RBracket,
                ',' => TokenKind::Comma,
                '.' => TokenKind::Dot,
                '+' => TokenKind::Plus,
                '-' => {
                    if ci.peek() == Some(&'>') {
                        ci.next();
                        // -> used in fn return types; treat as two tokens or special
                        // For now skip (not used in codegen yet)
                        continue;
                    }
                    TokenKind::Minus
                }
                '*' => TokenKind::Star,
                '/' => TokenKind::Slash,
                '%' => TokenKind::Percent,
                '=' => {
                    if ci.peek() == Some(&'=') {
                        ci.next();
                        TokenKind::EqEq
                    } else {
                        TokenKind::Eq
                    }
                }
                ':' => {
                    if ci.peek() == Some(&'=') {
                        ci.next();
                        TokenKind::Walrus
                    } else {
                        TokenKind::Colon
                    }
                }
                '!' => {
                    if ci.peek() == Some(&'=') {
                        ci.next();
                        TokenKind::NotEq
                    } else {
                        return Err(format!("Line {}: unexpected `!`", line));
                    }
                }
                '<' => {
                    if ci.peek() == Some(&'=') {
                        ci.next();
                        TokenKind::LtEq
                    } else {
                        TokenKind::Lt
                    }
                }
                '>' => {
                    if ci.peek() == Some(&'=') {
                        ci.next();
                        TokenKind::GtEq
                    } else {
                        TokenKind::Gt
                    }
                }
                '"' => {
                    let mut s = String::new();
                    loop {
                        match ci.next() {
                            None => return Err(format!("Line {}: unterminated string", line)),
                            Some('"') => break,
                            Some(c) => s.push(c),
                        }
                    }
                    TokenKind::StringLit(s)
                }
                c if c.is_ascii_digit() => {
                    let mut n = String::from(c);
                    while ci.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        n.push(ci.next().unwrap());
                    }
                    let v: i32 = n.parse().map_err(|_| format!("Line {}: bad int", line))?;
                    TokenKind::Int(v)
                }
                c if c.is_alphabetic() || c == '_' => {
                    let mut ident = String::from(c);
                    while ci
                        .peek()
                        .map(|c| c.is_alphanumeric() || *c == '_')
                        .unwrap_or(false)
                    {
                        ident.push(ci.next().unwrap());
                    }
                    match ident.as_str() {
                        "let" => TokenKind::Let,
                        "fn" => TokenKind::Fn,
                        "return" => TokenKind::Return,
                        "if" => TokenKind::If,
                        "elif" => TokenKind::Elif,
                        "else" => TokenKind::Else,
                        "while" => TokenKind::While,
                        "loop" => TokenKind::Loop,
                        "from" => TokenKind::From,
                        "import" => TokenKind::Import,
                        "tile" => TokenKind::Tile,
                        "on" => TokenKind::On,
                        "init" => TokenKind::Init,
                        "pass" => TokenKind::Pass,
                        "and" => TokenKind::And,
                        "or" => TokenKind::Or,
                        "not" => TokenKind::Not,
                        "true" => TokenKind::True,
                        "false" => TokenKind::False,
                        "u8" => TokenKind::TypeU8,
                        "i8" => TokenKind::TypeI8,
                        "u16" => TokenKind::TypeU16,
                        "bool" => TokenKind::TypeBool,
                        _ => TokenKind::Ident(ident),
                    }
                }
                c => {
                    return Err(format!("Line {}: unexpected character `{}`", line, c));
                }
            };
            tokens.push(tok!(kind));
        }

        tokens.push(tok!(TokenKind::Newline));
    }

    // Close remaining indent levels
    while indent_stack.last() != Some(&0) {
        indent_stack.pop();
        tokens.push(Token {
            kind: TokenKind::Dedent,
            line,
        });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        line,
    });
    Ok(tokens)
}
