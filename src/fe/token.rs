#[derive(PartialEq, Debug)]
pub enum Token {
    // keywords
    Let,
    Const,
    Pub,
    Use,
    Func,
    Enum,
    Struct,
    If,
    Else,

    // Literals
    String(String),
    Number(f64),
    True,
    False,
    Nil,

    // Symbols
    Arrow,
    Assign,
    Eq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    Dot,
    Question,
    At,
    Comma,
    Semicolon,
    Not,
    NotEq,
    Add,
    AddEq,
    Sub,
    SubEq,
    Mul,
    MulEq,
    Div,
    DivEq,
    Pow,
    PowEq,
    Mod,
    ModEq,
    And,
    Or,

    // Misc
    Identifier(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Start,
    Eof
}