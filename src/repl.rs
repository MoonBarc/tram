use std::io::Write;

use crate::{executor::VM, fe::parse::Parser};

pub fn run(vm: &mut VM) {
    loop {
        print!("> ");
        std::io::stdout().flush().expect("failed to flush stdout");
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)
            .expect("failed to read from stdin!");
        if buffer == "quit" { break }
        let prog = Parser::new(buffer).parse_all();
        if let Err(e) = vm.execute(&prog) {
            println!("== Runtime error from VM: {:?}", e);
        }
    }
    println!("kthxbye!")
}