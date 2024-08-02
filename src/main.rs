pub mod fe;
pub mod executor;
pub mod function;
pub mod corelib;
pub mod repl;

fn main() {
    println!("🚋 tram lang");
    let mut vm = executor::VM::new();
    vm.register_stdlib();

    repl::run(&mut vm);
}
