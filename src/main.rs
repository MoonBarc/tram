use crate::{value::Value, handle::Handle};

pub mod fe;
pub mod executor;
pub mod function;
pub mod corelib;
pub mod repl;
pub mod handle;
pub mod value;

fn main() {
    eprintln!("🚋 tram lang");
    let mut vm = executor::VM::new();
    vm.register_stdlib();

    if let Some(a) = std::env::args().nth(1) {
        let val = Value::String(Handle::new(a.trim().to_owned()));
        match corelib::run(&mut vm, vec![val]) {
            Ok(_) => {},
            Err(e) => eprintln!("VM Error: {:?}", e)
        }

        return;
    }

    repl::run(&mut vm);
}
