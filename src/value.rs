use std::{collections::HashMap, fmt::{Debug, Display}, hash::Hash, rc::Rc};

use crate::{executor::RuntimeError, function::Callable, handle::Handle};

#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(Handle<String>),
    Bool(bool),
    Array(Handle<Vec<Self>>),
    Map(Handle<HashMap<Self, Self>>),
    Function(Rc<dyn Callable>),
    Nil
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Number(n) => n.to_string().hash(state),
            Self::String(s) => s.borrow().hash(state),
            Self::Bool(b) => b.hash(state),
            Self::Array(a) => a.hash(state),
            Self::Map(m) => m.hash(state),
            Self::Function(func) => std::ptr::hash(&*func, state),
            Self::Nil => {}
        }
    }
}

impl Value {
    pub fn truthy(&self) -> bool {
        match self {
            Self::Number(_) | Self::Map(_) | Self::String(_)
                | Self::Array(_) | Self::Function(_) => true,
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

    pub fn string(&self) -> Result<Handle<String>, RuntimeError> {
        Ok(match self {
            Self::String(s) => s.clone(),
            _ => return Err(RuntimeError::NotAString)
        })
    }

    pub fn map(&self) -> Result<Handle<HashMap<Self, Self>>, RuntimeError> {
        Ok(match self {
            Self::Map(m) => m.clone(),
            _ => return Err(RuntimeError::NotAMap)
        })
    }
}

// Todo: revisit this! this is poorly implemented :(
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l), Self::Number(r)) => l == r,
            (Self::String(l), Self::String(r)) => l == r,
            (Self::Bool(l), Self::Bool(r)) => l == r,
            (Self::Array(l), Self::Array(r)) => l == r,
            (Self::Function(f1), Self::Function(f2)) => core::ptr::eq(f1.as_ref(), f2.as_ref()),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

// This assertion is false!!
// but it makes HashMaps easier and the Hash implementation takes care of this
impl Eq for Value {}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n)?,
            Value::String(s) => write!(f, "{:?}", s.borrow())?,
            Value::Bool(b) => Display::fmt(b, f)?,
            Value::Array(a) => {
                write!(f, "[")?;
                let a = a.borrow();
                for (i, elem) in a.iter().enumerate() {
                    write!(f, "{}", elem.to_string())?;
                    if i != a.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")?;
            }
            Value::Map(m) => {
                let m = m.borrow();
                write!(f, "%{{\n")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    write!(f, "    {} => {}", k, v)?;
                    if i != m.len() - 1 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\n")?;
                }
                write!(f, "}}")?;
            }
            Value::Function(func) => write!(f, "{}", func.display())?,
            Value::Nil => write!(f, "nil")?
        };
        Ok(())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // strings can be printed directly
            Value::String(s) => write!(f, "{}", s.borrow())?,
            // fall back to debug implementation for everything else
            _ => Debug::fmt(&self, f)?
        };
        Ok(())
    }
}

impl<T: Into<String>> From<T> for Value {
    fn from(value: T) -> Self {
        Value::String(Handle::new(value.into()))
    }
}
