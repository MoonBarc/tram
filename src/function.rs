use std::fmt::Debug;

use crate::{executor::VM, fe::ast::Value};

pub trait Callable: Debug {
    fn call(&self, vm: &mut VM, vals: Vec<Value>) -> Value;
    fn display(&self) -> String;
}

pub type NativeFunction = fn(vm: &mut VM, params: Vec<Value>) -> Value;

impl Callable for NativeFunction {
    fn call(&self, vm: &mut VM, vals: Vec<Value>) -> Value {
        self(vm, vals)
    }
    
    fn display(&self) -> String {
        "< native function >".to_string()
    }
}