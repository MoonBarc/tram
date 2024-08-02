use std::fs;

use crate::{executor::{RuntimeError, VM}, fe::{ast::Value, parse}};

pub fn print(_vm: &mut VM, vals: Vec<Value>) -> Result<Value, RuntimeError> {
    let mut s = String::new();
    for val in vals {
        if s.len() > 0 {
            s.push(' ');
        }
        s.push_str(&val.to_string());
    }
    println!("{}", s);
    Ok(Value::Nil)
}

pub fn run(vm: &mut VM, vals: Vec<Value>) -> Result<Value, RuntimeError> {
    let Some(file) = vals.get(0) else {
        panic!("no file specified!");
    };
    let Ok(s) = file.string() else {
        panic!("file is not a string");
    };
    println!("--> loading file {}", s);
    let f = fs::read_to_string(s.as_str())
        .expect("failed to load file");

    let prog = parse::parse(f);
    vm.locals.push();
    vm.execute(&prog)?;
    vm.locals.pop();

    Ok(Value::Nil)
}