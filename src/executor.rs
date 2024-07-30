//! A basic, tree walking executor for the tram language

use std::rc::Rc;

use crate::{corelib, fe::ast::{AstNode, BinOp, Statement, UnOp, Value}, function::NativeFunction};

#[derive(Debug)]
pub enum RuntimeError {
    NotANumber,
    CannotAdd,
    NotAFunction
}

pub struct LocalStack {
    markers: Vec<usize>,
    locals: Vec<(String, Value)>
}

impl LocalStack {
    pub fn new() -> Self {
        Self {
            markers: Vec::new(),
            locals: Vec::new()
        }
    }

    pub fn get(&self, key: &str) -> Value {
        for l in self.locals.iter().rev() {
            if l.0 == key {
                return l.1.clone();
            }
        }
        return Value::Nil;
    }

    pub fn exists(&self, key: &str) -> bool {
        for l in self.locals.iter().rev() {
            if l.0 == key {
                return true
            }
        }
        return false
    }

    pub fn push(&mut self) {
        self.markers.push(self.locals.len());
    }

    pub fn pop(&mut self) {
        let pop = self.markers.pop().expect("popped nonexistant scope");
        for i in (pop .. self.locals.len() - 1).rev() {
            self.locals.remove(i);
        }
    }

    pub fn set(&mut self, name: &str, val: Value) {
        self.locals.push((name.to_owned(), val))
    }
}

pub struct VM {
    pub locals: LocalStack
}

impl VM {
    pub fn new() -> Self {
        Self {
            locals: LocalStack::new()
        }
    }

    pub fn register_stdlib(&mut self) {
        self.locals.set("print", Value::Function(Rc::new(corelib::print as NativeFunction)));
    }

    pub fn execute(&mut self, prog: Vec<Statement>) -> Result<(), RuntimeError> {
        for stmt in prog.into_iter() {
            match stmt {
                Statement::Expression(x) => { self.execute_ast(*x)?; }
            }
        }
        Ok(())
    }

    pub fn execute_ast(&mut self, a: AstNode) -> Result<Value, RuntimeError> {
        Ok(match a {
            AstNode::Call(func, args) => {
                let func = self.execute_ast(*func)?;
                let mut vargs = Vec::with_capacity(args.len());
                for a in args {
                    let computed = self.execute_ast(a)?;
                    vargs.push(computed);
                }
                func.func()?.call(self, vargs)
            },
            AstNode::Value(v) => *v,
            AstNode::Ident(i) => {
                self.locals.get(&i)
            },
            AstNode::Assign(n, v) => {
                let val = self.execute_ast(*v)?;
                self.locals.set(&n, val);
                Value::Nil
            },
            AstNode::Binary(op, a, b) => {
                let a = self.execute_ast(*a)?;
                let b = self.execute_ast(*b)?;
                match op {
                    BinOp::Add => {
                        match (a, b) {
                            (Value::Array(a), Value::Array(b)) => {
                                let mut new = a.clone();
                                new.append(&mut b.clone());
                                Value::Array(new)
                            },
                            (Value::String(a), Value::String(b)) => {
                                let mut new = a.clone();
                                new.push_str(&b);
                                Value::String(new)
                            }
                            (Value::Number(a), Value::Number(b)) => {
                                Value::Number(a + b)
                            },
                            _ => return Err(RuntimeError::CannotAdd)
                        }
                    },
                    BinOp::Sub => a.num_op(&b, |a, b| Ok(a - b))?,
                    BinOp::Mul => a.num_op(&b, |a, b| Ok(a * b))?,
                    BinOp::Div => a.num_op(&b, |a, b| Ok(a / b))?,
                    BinOp::Pow => a.num_op(&b, |a, b| Ok(a.powf(b)))?,
                    BinOp::Mod => a.num_op(&b, |a, b| Ok(a % b))?,
                }
            },
            AstNode::Unary(op, a) => {
                let val = self.execute_ast(*a)?;
                match op {
                    UnOp::Not => Value::Bool(!val.truthy()),
                }
            },
            AstNode::If { cond, then, or } => {
                let cond = self.execute_ast(*cond)?;
                if cond.truthy() {
                    self.execute_ast(*then)?
                } else {
                    self.execute_ast(*or)?
                }
            },
            AstNode::Block(stmt) => {
                self.locals.push();
                self.execute(stmt);
                self.locals.pop();
                Value::Nil
            }
        })
    }
}