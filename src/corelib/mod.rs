use std::{collections::HashMap, fs, io::Write, process, rc::Rc, thread, time::Duration};

use crate::{executor::{RuntimeError, VM}, fe::ast::Ast, function::NativeFunction, handle::Handle, value::Value};

fn assert_val_length(vals: &[Value], len: usize) -> Result<(), RuntimeError> {
    if vals.len() == len {
        Ok(())
    } else {
        Err(RuntimeError::IncorrectNumberOfArgs)
    }
}

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

pub fn prompt(_vm: &mut VM, vals: Vec<Value>) -> Result<Value, RuntimeError> {
    match vals.get(0) {
        Some(i) => {
            print!("{}", i.string()?.borrow());
            std::io::stdout().flush().unwrap();
        },
        None => {}
    };
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).unwrap();
    Ok(s.trim_end().into())
}

pub fn exit(_vm: &mut VM, vals: Vec<Value>) -> Result<Value, RuntimeError> {
    process::exit(vals.first()
        .map(|x| x.num().map(|x| x.round() as i32).unwrap_or(0))
        .unwrap_or(0));
}

pub fn corelib_type(_vm: &mut VM, vals: Vec<Value>) -> Result<Value, RuntimeError> {
    assert_val_length(&vals, 1)?;

    let val = &vals[0];

    let ty = match val {
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Bool(_) => "bool",
        Value::Array(_) => "array",
        Value::Map(_) => "map",
        Value::Function(_) => "func",
        Value::Nil => "nil",
    };

    Ok(ty.into())
}

pub fn run(vm: &mut VM, vals: Vec<Value>) -> Result<Value, RuntimeError> {
    assert_val_length(&vals, 1)?;

    let file = &vals[0];
    let Ok(s) = file.string() else {
        return Err(RuntimeError::NotAString)
    };
    let s = s.borrow();

    println!("--> loading file {}", s);
    let f = fs::read_to_string(&*s)
        .expect("failed to load file");

    let prog: Ast = match f.parse() {
        Ok(p) => p,
        Err(e) => {
            println!("encountered errors while running file");
            for e in e {
                e.log(Some(&f));
            }
            return Ok(Value::Bool(false))
        }
    };

    vm.locals.push();
    vm.execute(&prog)?;
    vm.locals.pop();

    Ok(Value::Bool(false))
}

pub fn sleep(_vm: &mut VM, vals: Vec<Value>) -> Result<Value, RuntimeError> {
    assert_val_length(&vals, 1)?;
    let m = &vals[0];

    thread::sleep(Duration::from_secs_f64(m.num()?));
    Ok(Value::Nil)
}

struct NativeLibModule {
    map: HashMap<Value, Value>
}

impl NativeLibModule {
    pub fn new() -> Self {
        NativeLibModule {
            map: HashMap::new(),
        }
    }

    pub fn export_fn(&mut self, name: impl Into<String>, func: NativeFunction) {
        self.map.insert(name.into().into(), Value::Function(Rc::new(func)));
    }

    pub fn export(&mut self, name: impl Into<String>, val: Value) {
        self.map.insert(name.into().into(), val);
    }
}

impl Into<Value> for NativeLibModule {
    fn into(self) -> Value {
        Value::Map(Handle::new(self.map))
    }
}

macro_rules! math_fns {
    ($math:expr, $($f:ident),*) => {
        mod __math_fns {
            use super::*;
            $(
            pub(super) fn $f(_: &mut VM, args: Vec<Value>) -> Result<Value, RuntimeError> {
                if args.len() != 1 {
                    return Err(RuntimeError::IncorrectNumberOfArgs)
                }
                let num = args.first().unwrap().num()?;
                Ok(Value::Number(num.$f()))
            }
            )*
        }

        $(
        $math.export_fn(stringify!($f), __math_fns::$f)
        );*
    };
}

pub fn math() -> Value {
    let mut math = NativeLibModule::new();

    math_fns!(
        math,
        sin, cos, tan,
        sinh, cosh, tanh,
        floor, ceil,
        ln,
        signum
    );

    math.export("pi", Value::Number(core::f64::consts::PI));
    math.export("e", Value::Number(core::f64::consts::E));

    math.into()
}
