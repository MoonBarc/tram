use std::fmt::Debug;

use crate::{executor::{RuntimeError, VM}, fe::ast::Ast, value::Value};

pub trait Callable: Debug {
    fn call(&self, vm: &mut VM, vals: Vec<Value>) -> Result<Value, RuntimeError>;
    fn display(&self) -> String;
}

pub type NativeFunction = fn(vm: &mut VM, params: Vec<Value>) -> Result<Value, RuntimeError>;

impl Callable for NativeFunction {
    fn call(&self, vm: &mut VM, vals: Vec<Value>) -> Result<Value, RuntimeError> {
        self(vm, vals)
    }
    
    fn display(&self) -> String {
        "< native func >".to_string()
    }
}

#[derive(Debug)]
pub struct Function {
    pub ast: Ast,
    pub name: Option<String>,
    pub params: Vec<String>
}

impl Callable for Function {
    fn call(&self, vm: &mut VM, vals: Vec<Value>) -> Result<Value, RuntimeError> {
        vm.locals.push();
        if vals.len() != self.params.len() {
            return Err(RuntimeError::IncorrectNumberOfArgs)
        }
        for p in self.params.iter().enumerate() {
            vm.locals.set(&p.1, vals[p.0].clone())
        }
        let val = vm.execute(&*self.ast);
        vm.locals.pop();
        val
    }

    fn display(&self) -> String {
        if let Some(name) = &self.name {
            format!("< func {} >", name)
        } else {
            "< anonymous func >".to_owned()
        }
    }
}
