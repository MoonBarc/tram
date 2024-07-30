use fe::ast::{Ast, AstNode, Statement};

pub mod fe;
pub mod executor;
pub mod function;
pub mod corelib;

fn main() {
    println!("🚋 tram lang");
    let mut vm = executor::VM::new();
    vm.register_stdlib();
    let prog = Vec::from([
        Statement::Expression(Ast::new(
            AstNode::Call(Ast::new(
                AstNode::Ident("print".to_string())
            ), Vec::from([
                AstNode::Ident("print".to_string())
            ]))
        ))
    ]);

    vm.execute(prog).expect("failed to run program!");
}
