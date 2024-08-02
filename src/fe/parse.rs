use std::rc::Rc;

use crate::{fe::ast::BinOp, function::Function};

use super::{ast::{Ast, AstNode, Statement, Value}, lexer::Lexer, token::Token};

macro_rules! precs {
    ($($prec:ident: $num:expr),+) => {
        pub mod prec {
            $(
            pub const $prec: u8 = $num;
            )*
        }
    };
}

precs!(
    NONE: 0,
    ASSIGN: 1,
    OR: 2,
    AND: 3,
    EQ: 4,
    COMP: 5,
    TERM: 6,
    FACTOR: 7,
    POW: 8,
    UNARY: 9,
    CALL: 10,
    PRIMARY: 11
);

impl Token {
    fn prec(&self) -> u8 {
        use Token::*;
        match self {
            Add | Sub => prec::TERM,
            Mul | Div | Mod => prec::FACTOR,
            Pow => prec::POW,
            Gt | GtEq | Lt | LtEq => prec::COMP,
            Eq => prec::EQ,
            And => prec::AND,
            Or => prec::OR,
            Assign | AddEq | SubEq | MulEq | DivEq | PowEq | ModEq => prec::ASSIGN,
            LParen => prec::CALL,
            _ => prec::NONE
        }
    }

    pub fn infix(&self) -> Option<fn(&mut Parser, lhs: Ast, prec: u8) -> Ast> {
        match self.prec() {
            prec::NONE => None,
            prec::CALL => Some(Parser::call),
            prec::ASSIGN => Some(Parser::assign),
            _ => Some(Parser::binary)
        }
    }
}

pub fn parse(source: String) -> Vec<Statement> {
    let mut p = Parser::new(source);
    p.parse_all()
}

pub struct Parser {
    lexer: Lexer,
    current: Token,
    next: Token
}

impl Parser {
    /// The start of `lexed` must be a `Start` token,
    /// and the end must be an `Eof` token.
    pub fn new(source: String) -> Self {
        let mut lexer = Lexer::new(source);
        Self {
            current: Token::Start,
            next: lexer.next(),
            lexer
        }
    }

    pub fn parse_all(&mut self) -> Vec<Statement> {
        self.block(false)
    }

    pub fn statement(&mut self) -> Statement {
        Statement::Expression(self.expression())
    }

    pub fn expression(&mut self) -> Ast {
        self.parse_with_prec(prec::ASSIGN)
    }

    fn parse_with_prec(&mut self, prec: u8) -> Ast {
        self.advance();
        let mut node = match &self.current {
            Token::Number(..)
            | Token::String(..)
            | Token::True | Token::False | Token::Nil => self.literal(),
            Token::Identifier(..) => self.ident(),
            Token::Func => self.func(true),
            Token::If => self.if_expr(),
            t => panic!("unexpected token, {:?}", t)
        };
        while prec <= self.next.prec() {
            self.advance();
            node = if let Some(ifix) = self.current.infix() {
                ifix(self, node, self.current.prec() + 1)
            } else {
                panic!("{:?} has no infix value!", self.current)
            }
        }
        node
    }

    fn expect_ident(&mut self) -> String {
        match &self.current {
            Token::Identifier(s) => s.clone(),
            _ => panic!("expected an identifier")
        }
    }

    fn ident(&mut self) -> Ast {
        Ast::new(AstNode::Ident(self.expect_ident()))
    }

    fn call(&mut self, func: Ast, _prec: u8) -> Ast {
        let mut args = Vec::new();
        while !self.pick(&Token::RParen) {
            args.push(*self.expression());
            if self.next != Token::RParen {
                assert_eq!(self.next, Token::Comma);
                self.advance();
            }
        }
        Ast::new(AstNode::Call(func, args))
    }

    fn assign(&mut self, lhs: Ast, prec: u8) -> Ast {
        let name = match &*lhs {
            AstNode::Ident(s) => s.clone(),
            _ => panic!("invalid assignment target")
        };
        macro_rules! map {
            ($i:expr, $($token:ident => $binop:ident),*) => {
               match $i {
                $(Token::$token => Some(BinOp::$binop)),*,
                _ => None
               } 
            };
        }
        let op = map!(
            self.current,
            AddEq => Add,
            SubEq => Sub,
            MulEq => Mul,
            DivEq => Div,
            PowEq => Pow,
            ModEq => Mod
        );
        let rhs = self.parse_with_prec(prec);
        let value = if let Some(op) = op {
            Ast::new(AstNode::Binary(op, lhs, rhs))
        } else {
            rhs
        };
        Ast::new(AstNode::Assign(name, value))
    }

    fn func(&mut self, anon: bool) -> Ast {
        let mut name = None;
        if !anon {
            self.advance();
            name = match &self.current {
                Token::Identifier(s) => Some(s.clone()),
                _ => panic!("expected name of the function")
            };
        }
        if !self.pick(&Token::LParen) {
            panic!("expected `(` to start argument list")
        }
        let mut args = Vec::new();
        while !self.pick(&Token::RParen) {
            self.advance();
            args.push(self.expect_ident());
            if self.next != Token::RParen {
                assert_eq!(self.next, Token::Comma);
                self.advance();
            }
        }

        if !self.pick(&Token::LBrace) {
            panic!("expected `{{` to open the function block, got: {:?}", self.next);
        }
        let ast = self.block_expr(true);
        let func = Function {
            name,
            params: args,
            ast
        };
        Ast::new(AstNode::Value(Box::new(
            Value::Function(Rc::new(func))
        )))
    }

    fn if_expr(&mut self) -> Ast {
        let cond = self.expression();
        let then = self.block_expr(true);
        let mut or = None;
        if self.pick(&Token::Else) {
            if self.pick(&Token::RBracket) {
                or = Some(self.block_expr(true))
            } else if self.pick(&Token::If) {
                or = Some(self.if_expr())
            } else {
                panic!("expected block or if expression after `else`")
            }
        }
        Ast::new(AstNode::If {
            cond,
            then,
            or
        })
    }

    fn block_expr(&mut self, expect_end: bool) -> Ast {
        Ast::new(AstNode::Block(self.block(expect_end)))
    }

    fn block(&mut self, expect_end: bool) -> Vec<Statement> {
        let mut v = Vec::new();
        loop {
            if expect_end && self.pick(&Token::RBrace) {
                break;
            } else if self.pick(&Token::Eof) {
                if expect_end {
                    panic!("expected closing `}}`")
                } else {
                    break
                }
            }
            v.push(self.statement());
        }
        v
    }

    fn binary(&mut self, lhs: Ast, prec: u8) -> Ast {
        let op = match &self.current {
            Token::Add => BinOp::Add,
            Token::Sub => BinOp::Sub,
            Token::Mul => BinOp::Mul,
            Token::Div => BinOp::Div,
            Token::Mod => BinOp::Mod,
            Token::Pow => BinOp::Pow,
            x => panic!("no binary expression implemented for {:?}", x)
        };

        let rhs = self.parse_with_prec(prec);

        Ast::new(AstNode::Binary(op, lhs, rhs))
    }

    fn advance(&mut self) {
        let next = self.lexer.next();
        std::mem::swap(&mut self.current, &mut self.next);
        self.next = next;
    }

    fn pick(&mut self, tok: &Token) -> bool {
        if std::mem::discriminant(tok) == std::mem::discriminant(&self.next) {
            self.advance();
            true
        } else { false }
    }

    fn literal(&mut self) -> Ast {
        Ast::new(AstNode::Value(Box::new(match &self.current {
            Token::Number(n) => Value::Number(*n),
            Token::String(s) => Value::String(Rc::new(s.clone())),
            Token::True => Value::Bool(true),
            Token::False => Value::Bool(false),
            Token::Nil => Value::Nil,
            _ => unreachable!("thought it was a value but it wasn't")
        })))
    }
}