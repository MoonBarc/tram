use super::token::Token;

pub struct Lexer {
    at: usize,
    tok_start: usize,
    source: Vec<char>
}

// lazy, inefficient lexer that does two passes over the source program
impl Lexer {
    pub fn new(source: String) -> Self {
        // space at the front
        let chars = [' '].into_iter().chain(source.chars()).collect();
        
        Self {
            at: 0,
            tok_start: 0,
            source: chars
        }
    }

    pub fn next(&mut self) -> Token {
        self.skip_whitespace();
        use Token::*;
        let nxt = self.advance();
        self.tok_start = self.at;
        match nxt {
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
            '0'..='9' => self.number(),
            '-' => if self.pick('>') { Arrow } else { self.eq_or(Sub, SubEq) },
            '=' => self.eq_or(Assign, Eq),
            '>' => self.eq_or(Gt, GtEq),
            '<' => self.eq_or(Lt, LtEq),
            '.' => Dot,
            '?' => Question,
            '@' => At,
            ',' => Comma,
            ';' => Semicolon,
            '!' => self.eq_or(Not, NotEq),
            '+' => self.eq_or(Add, AddEq),
            '*' => if self.pick('*') {
                self.eq_or(Pow, PowEq)
            } else { self.eq_or(Mul, MulEq) },
            '/' => self.eq_or(Div, DivEq),
            '%' => self.eq_or(Mod, ModEq),
            '&' if self.pick('&') => And,
            '|' if self.pick('|') => Or,

            '(' => LParen,
            ')' => RParen,
            '{' => LBrace,
            '}' => RBrace,
            '[' => LBracket,
            ']' => RBracket,
            '\0' => Eof,

            '"' => self.string(),

            n => panic!("unknown character {}", n)
        }
    }

    fn eq_or(&mut self, without: Token, with: Token) -> Token {
        if self.pick('=') { with } else { without }
    }

    fn at_end(&self) -> bool {
        self.peek() == '\0'
    }

    fn skip_whitespace(&mut self) {
        while self.pick(' ') || self.pick('\t') || self.pick('\n') {}
    }

    fn advance(&mut self) -> char {
        self.at += 1;
        self.at()
    }

    fn at(&self) -> char {
        self.peek_n(0)
    }

    fn peek_n(&self, n: usize) -> char {
        if self.at + n >= self.source.len() {
            '\0'
        } else {
            self.source[self.at + n]
        }
    }

    fn peek(&self) -> char {
        self.peek_n(1)
    }

    fn pick(&mut self, ch: char) -> bool {
        if self.peek() == ch {
            self.advance();
            true
        } else { false }
    }

    fn eat_through(&mut self, ch: char) {
        while !self.pick(ch) {
            self.advance();
        }
    }

    fn lexeme(&self) -> String {
        self.source[self.tok_start..=self.at.min(self.source.len())]
            .into_iter()
            .collect()
    }

    fn string(&mut self) -> Token {
        self.eat_through('"');
        let string = self.lexeme();
        Token::String(string[1..string.len() - 1].to_owned())
    }

    fn number(&mut self) -> Token {
        loop {
            match self.peek() {
                '0'..='9' | '.' => {
                    self.advance();
                },
                _ => break
            }
        }
        Token::Number(self.lexeme().parse().expect("failed to parse number"))
    }

    fn identifier(&mut self) -> Token {
        loop {
            match self.peek() {
                '0'..'9'
                | 'a'..'z'
                | 'A'..'Z'
                | '_' => self.advance(),
                _ => break
            };
        };
        let str = self.lexeme();
        use Token::*;
        match str.as_str() {
            "let" => Let,
            "const" => Const,
            "pub" => Pub,
            "use" => Use,
            "func" => Func,
            "enum" => Enum,
            "struct" => Struct,
            "if" => If,
            "else" => Else,

            "true" => True,
            "false" => False,
            "nil" => Nil,

            _ => Token::Identifier(str)
        }
    }
}