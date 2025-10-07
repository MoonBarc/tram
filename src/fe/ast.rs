use crate::value::Value;

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    Eq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    And,
    Or,
    Access
}

#[derive(Debug)]
pub enum UnOp {
    Not,
    Sub
}

pub type Ast = Box<AstNode>;

#[derive(Debug)]
pub enum AstNode {
    Call(Ast, Vec<Self>),
    Value(Box<Value>),
    Ident(String),
    Assign(String, Ast),
    Binary(BinOp, Ast, Ast),
    Unary(UnOp, Ast),
    If {
        cond: Ast,
        then: Ast,
        or: Option<Ast>
    },
    /// a list of statements and a
    /// bool describing if a new scope should be created
    Block(Vec<Statement>, bool),
    Loop {
        label: Option<String>,
        cond: Option<Ast>,
        run: Ast
    },
    Break(Option<String>),
    Error
}

#[derive(Debug)]
pub enum Statement {
    Expression(Ast),
}
