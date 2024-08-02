use std::{fmt::{Debug, Display}, rc::Rc};

use crate::{executor::RuntimeError, function::Callable};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
    Bool(bool),
    Array(Vec<Self>),
    Function(Rc<dyn Callable>),
    Nil
}

impl Value {
    pub fn truthy(&self) -> bool {
        match self {
            Self::Number(_) | Self::String(_) | Self::Array(_) | Self::Function(_) => true,
            Self::Bool(b) => *b,
            Self::Nil => false
        }
    }

    pub fn num(&self) -> Result<f64, RuntimeError> {
        match self {
            Self::Number(n) => Ok(*n),
            _ => Err(RuntimeError::NotANumber)
        }
    }

    pub fn num_op(&self, other: &Value, op: fn(f64, f64) -> Result<f64, RuntimeError>)
        -> Result<Value, RuntimeError> {
        let a = self.num()?;
        let b = other.num()?;
        op(a, b).map(|num| Value::Number(num))
    }

    pub fn func(&self) -> Result<Rc<dyn Callable>, RuntimeError> {
        Ok(match self {
            Self::Function(c) => c.clone(),
            _ => return Err(RuntimeError::NotAFunction)
        })
    }

    pub fn string(&self) -> Result<Rc<String>, RuntimeError> {
        Ok(match self {
            Self::String(s) => s.clone(),
            _ => return Err(RuntimeError::NotAString)
        })
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n)?,
            Value::String(s) => write!(f, "{}", s)?,
            Value::Bool(b) => Display::fmt(b, f)?,
            Value::Array(a) => {
                write!(f, "[")?;
                for elem in a {
                    write!(f, "{}", elem.to_string())?
                }
                write!(f, "]")?;
            }
            Value::Function(func) => write!(f, "{}", func.display())?,
            Value::Nil => write!(f, "nil")?
        };
        Ok(())
    }
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod
}

#[derive(Debug)]
pub enum UnOp {
    Not
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
    Block(Vec<Statement>)
}

#[derive(Debug)]
pub enum Statement {
    Expression(Ast),
}