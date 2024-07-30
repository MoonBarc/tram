use crate::{executor::VM, fe::ast::Value};

pub fn print(_vm: &mut VM, vals: Vec<Value>) -> Value {
    let mut s = String::new();
    for val in vals {
        s.push_str(&val.to_string());
    }
    println!("{}", s);
    Value::Nil
}