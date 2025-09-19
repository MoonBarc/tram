pub mod fe;
pub mod executor;
pub mod function;
pub mod corelib;
pub mod repl;

fn main() {
    println!("🚋 tram lang");
    let mut vm = executor::VM::new();
    vm.register_stdlib();

    if let Some(a) = std::env::args().nth(1) {
        println!("> running {}", a.trim());
        let code = std::fs::read_to_string(a.trim())
            .expect("file should exist");

        let mut parser = fe::parse::Parser::new(code);
        let stmts = parser.parse_all();
        vm.execute(&stmts).unwrap();
        return;
    }

    repl::run(&mut vm);
}
