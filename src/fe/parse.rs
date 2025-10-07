use std::{borrow::Cow, rc::Rc, str::FromStr};

use crate::{fe::{ast::{BinOp, UnOp}, diagnostic::{ParseError, Span}}, function::Function, handle::Handle, value::Value};

use super::{ast::{Ast, AstNode, Statement}, lexer::Lexer, token::Token};

impl FromStr for Ast {
    type Err = Vec<ParseError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut p = Parser::new(s);
        let (ast, err) = p.parse_all();
        if err.is_empty() {
            Ok(ast)
        } else { Err(err) }
    }
}

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
    DOT: 11,
    PRIMARY: 12
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
            Dot => prec::DOT,
            _ => prec::NONE
        }
    }

    fn infix(&self) -> Option<fn(&mut Parser, lhs: Ast, prec: u8) -> Ast> {
        match self.prec() {
            prec::NONE => None,
            prec::CALL => Some(Parser::call),
            prec::DOT => Some(Parser::dot_expr),
            prec::ASSIGN => Some(Parser::assign),
            _ => Some(Parser::binary)
        }
    }
}

pub struct Parser {
    lexer: Lexer,
    current: Token,
    next: Token,
    next_span: Span,
    current_span: Span,
    errors: Vec<ParseError>
}

impl Parser {
    /// The start of `lexed` must be a `Start` token,
    /// and the end must be an `Eof` token.
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let (next, span) = lexer.next();
        Self {
            current: Token::Start,
            next,
            errors: vec![],
            lexer,
            current_span: Span::empty(),
            next_span: span
        }
    }

    pub fn parse_all(&mut self) -> (Ast, Vec<ParseError>) {
        let block = self.block(false, false);
        let errors = std::mem::take(&mut self.errors);
        (block, errors)
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
            Token::Func => self.func(),
            Token::If => self.if_expr(),
            Token::Loop => self.loop_expr(),
            Token::LBrace => self.block(true, true),
            Token::Break => Ast::new(AstNode::Break(None)),
            Token::Not | Token::Sub => self.unary(),
            t => self.error(format!("unexpected token {:?}", t))
        };
        while prec <= self.next.prec() {
            self.advance();
            node = if let Some(ifix) = self.current.infix() {
                ifix(self, node, self.current.prec() + 1)
            } else {
                return self.error(format!("{:?} has no infix value!", self.current))
            }
        }
        node
    }

    fn dot_expr(&mut self, lhs: Ast, _prec: u8) -> Ast {
        let Token::Identifier(i) = &self.next else {
            return self.error("identifier expected following `.`");
        };
        let st = AstNode::Value(Box::new(i.into()));
        self.advance();
        
        Ast::new(AstNode::Binary(
            BinOp::Access,
            lhs, 
            Ast::new(st)
        ))
    }

    fn access_expr(&mut self) -> Ast {
        self.error("access syntax unimplemented")
    }

    fn ident(&mut self) -> Ast {
        let Token::Identifier(s) = &self.current else {
            return self.error("expected an identifier");
        };
        Ast::new(AstNode::Ident(s.clone()))
    }

    fn call(&mut self, func: Ast, _prec: u8) -> Ast {
        let mut args = Vec::new();
        while !self.pick(&Token::RParen) {
            args.push(*self.expression());
            if self.next != Token::RParen {
                if !self.pick(&Token::Comma) {
                    return self.error("expected comma after expression");
                }
            }
        }
        Ast::new(AstNode::Call(func, args))
    }

    fn error(&mut self, message: impl Into<Cow<'static, str>>) -> Ast {
        let m = message.into();
        let error = ParseError {
            span: self.current_span,
            message: m.into()
        };
        self.errors.push(error);
        Ast::new(AstNode::Error)
    }

    fn assign(&mut self, lhs: Ast, prec: u8) -> Ast {
        let name = match &*lhs {
            AstNode::Ident(s) => s.clone(),
            _ => return self.error("invalid assignment target")
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

    fn func(&mut self) -> Ast {
        let name = match &self.next {
            Token::Identifier(s) => {
                let n = s.clone();
                self.advance();
                Some(n)
            }
            _ => None
        };
        if !self.pick(&Token::LParen) {
            return self.error("expected `(` to start argument list")
        }
        let mut args = Vec::new();
        while !self.pick(&Token::RParen) {
            self.advance();
            let Token::Identifier(id) = &self.current else {
                return self.error("expected identifier in argument list")
            };
            args.push(id.clone());
            if self.next != Token::RParen {
                assert_eq!(self.next, Token::Comma);
                self.advance();
            }
        }

        if !self.pick(&Token::LBrace) {
            return self.error(
                format!("expected `{{` to open the function block, got: {:?}", self.next));
        }
        let ast = self.block(true, true);
        let func = Function {
            name: name.clone(),
            params: args,
            ast
        };
        let fn_value = Ast::new(AstNode::Value(Box::new(
            Value::Function(Rc::new(func))
        )));

        if let Some(name) = name {
            // func hello() {} ==> hello = func hello() {}
            let assignment = Statement::Expression(
                Ast::new(AstNode::Assign(name.clone(), fn_value))
            );
            Ast::new(AstNode::Block(
                vec![
                    assignment,
                    Statement::Expression(Ast::new(AstNode::Ident(name)))
                ], false
            ))
        } else {
            fn_value
        }
    }

    fn if_expr(&mut self) -> Ast {
        let cond = self.expression();
        if !self.pick(&Token::LBrace) {
            return self.error("expected `{` to open then block after if condition")
        }
        let then = self.block(true, true);
        let or = if self.pick(&Token::Else) {
            if self.pick(&Token::LBrace) {
                Some(self.block(true, true))
            } else if self.pick(&Token::If) {
                Some(self.if_expr())
            } else {
                Some(self.error("expected block or if expression after `else`"))
            }
        } else { None };
        Ast::new(AstNode::If {
            cond,
            then,
            or
        })
    }

    fn block(&mut self, expect_end: bool, scoped: bool) -> Ast {
        let mut v = Vec::new();
        loop {
            if expect_end && self.pick(&Token::RBrace) {
                break;
            } else if self.pick(&Token::Eof) {
                if expect_end {
                    return self.error("expected closing `}`")
                } else {
                    break
                }
            }
            v.push(self.statement());
        }
        Ast::new(AstNode::Block(v, scoped))
    }

    fn binary(&mut self, lhs: Ast, prec: u8) -> Ast {
        let op = match &self.current {
            Token::Add => BinOp::Add,
            Token::Sub => BinOp::Sub,
            Token::Mul => BinOp::Mul,
            Token::Div => BinOp::Div,
            Token::Mod => BinOp::Mod,
            Token::Pow => BinOp::Pow,
            Token::Eq => BinOp::Eq,
            Token::Gt => BinOp::Gt,
            Token::GtEq => BinOp::GtEq,
            Token::Lt => BinOp::Lt,
            Token::LtEq => BinOp::LtEq,
            Token::And => BinOp::And,
            Token::Or => BinOp::Or,

            x => return self.error(format!("no binary expression implemented for {:?}", x))
        };

        let rhs = self.parse_with_prec(prec);

        Ast::new(AstNode::Binary(op, lhs, rhs))
    }

    fn unary(&mut self) -> Ast {
        let op = match &self.current {
            Token::Not => UnOp::Not,
            Token::Sub => UnOp::Sub,

            x => return self.error(format!("no unary expression implemented for {:?}", x))
        };
        let expr = self.expression();

        Ast::new(AstNode::Unary(op, expr))
    }

    fn loop_expr(&mut self) -> Ast {
        let cond: Option<Ast> = None;
        let label = None;
        if !self.pick(&Token::LBrace) {
            self.error(format!("expected `{{` to open loop, got {:?}", self.next));
        }
        let run = self.block(true, true);
        Ast::new(AstNode::Loop { cond, run, label })
    }

    fn advance(&mut self) {
        let (next, span) = self.lexer.next();
        std::mem::swap(&mut self.current, &mut self.next);
        std::mem::swap(&mut self.current_span, &mut self.next_span);
        self.next_span = span;
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
            Token::String(s) => Value::String(Handle::new(s.clone())),
            Token::True => Value::Bool(true),
            Token::False => Value::Bool(false),
            Token::Nil => Value::Nil,
            _ => unreachable!("thought it was a value but it wasn't")
        })))
    }
}
