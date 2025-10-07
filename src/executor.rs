//! A basic, tree walking executor for the tram language

use std::rc::Rc;

use crate::{corelib, fe::ast::{AstNode, BinOp, Statement, UnOp}, function::NativeFunction, handle::Handle, value::Value};

#[derive(Debug)]
pub enum RuntimeError {
    CannotAdd,
    IncorrectNumberOfArgs,
    NotAFunction,
    NotANumber,
    NotAString,
    NotAMap
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
        for i in (pop .. self.locals.len()).rev() {
            self.locals.remove(i);
        }
    }

    pub fn set(&mut self, name: &str, val: Value) {
        let mut idx: Option<usize> = None;
        for (i, (lname, _)) in self.locals.iter().enumerate().rev() {
            if name == lname {
                idx = Some(i);
                break;
            }
        }
        if let Some(idx) = idx {
            self.locals[idx] = (self.locals[idx].0.to_owned(), val);
        } else {
            self.locals.push((name.to_owned(), val))
        }
    }
}

pub struct VM {
    pub locals: LocalStack,
    exit_flag: ExitFlag
}

enum ExitFlag {
    Continue,
    Exit,
    Break(Option<String>)
}

impl VM {
    pub fn new() -> Self {
        Self {
            locals: LocalStack::new(),
            exit_flag: ExitFlag::Continue
        }
    }

    pub fn register_stdlib(&mut self) {
        let funcs: &[(&str, NativeFunction)] = &[
            ("print", corelib::print),
            ("input", corelib::input),
            ("type", corelib::corelib_type),
            ("run", corelib::run),
            ("sleep", corelib::sleep),
        ];
        let funcs = funcs.into_iter()
            .map(|(n, f)| (*n, Value::Function(Rc::new(*f))));

        let objs = [
            ("math", corelib::math())
        ];

        let globals = objs.into_iter()
            .chain(funcs);

        for (name, global) in globals {
            self.locals.set(name, global);
        }
    }

    pub fn execute(&mut self, a: &AstNode) -> Result<Value, RuntimeError> {
        Ok(match a {
            AstNode::Call(func, args) => {
                let func = self.execute(func)?;
                let mut vargs = Vec::with_capacity(args.len());
                for a in args {
                    let computed = self.execute(a)?;
                    vargs.push(computed);
                }
                func.func()?.call(self, vargs)?
            },
            AstNode::Value(v) => (**v).clone(),
            AstNode::Ident(i) => {
                self.locals.get(&i)
            },
            AstNode::Assign(n, v) => {
                let val = self.execute(v)?;
                self.locals.set(&n, val);
                Value::Nil
            },
            AstNode::Binary(op, a, b) => {
                let a = self.execute(a)?;
                let b = self.execute(b)?;
                match op {
                    BinOp::Add => {
                        match (a, b) {
                            (Value::Array(a), Value::Array(b)) => {
                                let mut new = a.borrow().clone();
                                new.append(&mut b.borrow().clone());
                                Value::Array(Handle::new(new))
                            },
                            (Value::String(a), Value::String(b)) => {
                                let mut new = a.borrow().clone();
                                new.push_str(&b.borrow());
                                Value::String(Handle::new(new))
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
                    BinOp::Eq => Value::Bool(a == b),
                    BinOp::Gt => Value::Bool(a.num()? > b.num()?),
                    BinOp::GtEq => Value::Bool(a.num()? >= b.num()?),
                    BinOp::Lt => Value::Bool(a.num()? < b.num()?),
                    BinOp::LtEq => Value::Bool(a.num()? <= b.num()?),
                    BinOp::And => Value::Bool(a.truthy() && b.truthy()),
                    BinOp::Or => Value::Bool(a.truthy() || b.truthy()),
                    BinOp::Access => {
                        let map = a.map()?;
                        let map = map.borrow();
                        match map.get(&b) {
                            Some(v) => v.clone(),
                            None => {
                                Value::Nil
                            }
                        }
                    }
                }
            },
            AstNode::Unary(op, a) => {
                let val = self.execute(a)?;
                match op {
                    UnOp::Not => Value::Bool(!val.truthy()),
                    UnOp::Sub => Value::Number(-val.num()?)
                }
            },
            AstNode::If { cond, then, or } => {
                let cond = self.execute(cond)?;
                if cond.truthy() {
                    self.execute(then)?
                } else if let Some(or) = or {
                    self.execute(or)?
                } else {
                    Value::Nil
                }
            },
            AstNode::Block(stmts, scoped) => {
                if *scoped {
                    self.locals.push();
                }
                let mut out = Value::Nil;
                for stmt in stmts {
                    match stmt {
                        Statement::Expression(x) => { out = self.execute(x)?; }
                    }
                }
                if *scoped {
                    self.locals.pop();
                }
                out
            },
            AstNode::Loop { label, cond, run } => {
                loop {
                    let mut should_break = false;
                    if let ExitFlag::Break(elabel) = &self.exit_flag {
                        if let (Some(l1), Some(l2)) = (label, elabel) {
                            should_break = l1 == l2 
                        } else { should_break = true }
                    }
                    if should_break {
                        self.exit_flag = ExitFlag::Continue;
                        break
                    }
                    if let Some(c) = cond {
                        let v = self.execute(c)?;
                        if v.truthy() {
                            self.execute(run)?;
                        }
                    } else {
                        self.execute(run)?;
                    }
                }
                Value::Nil
            },
            AstNode::Break(label) => {
                self.exit_flag = ExitFlag::Break(label.clone());
                Value::Nil
            },
            AstNode::Error => {
                // this should never happen
                panic!("running poorly compiled code, encountered Error node.");
            }
        })
    }
}
