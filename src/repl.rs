use std::io::Write;

use crate::{executor::VM, fe::ast::Ast};

pub fn run(vm: &mut VM) {
    loop {
        print!("> ");
        std::io::stdout().flush().expect("failed to flush stdout");
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)
            .expect("failed to read from stdin!");
        if buffer == "" {
            // no return must mean EOF
            break
        }
        if buffer.trim() == "quit" { break }
        let prog: Ast = match buffer.parse() {
            Ok(p) => p,
            Err(e) => {
                for err in e {
                    err.log(Some(&buffer));
                }
                continue
            }
        };
        match vm.execute(&prog) {
            Err(e) => println!("== Runtime error from VM: {:?}", e),
            Ok(v) => {
                println!("\x1b[36m{:?}\x1b[0m", v)
            }
        }
    }
    println!("\nbye!")
}
