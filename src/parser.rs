use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Show(Box<Expr>),
    Number(f64),
    StringLit(String),
    Ident(String),
    Store { value: Box<Expr>, name: String },
    Add { value: Box<Expr>, list: String },
    Remove { value: Box<Expr>, list: String },
    If { condition: Box<Expr>, body: Vec<Expr>, else_body: Vec<Expr> },
    BinOp { left: Box<Expr>, op: Op, right: Box<Expr> },
    Input { prompt: String, store_in: String },
    Loop { count: Box<Expr>, body: Vec<Expr> },
    FunctionDef { name: String, params: Vec<String>, body: Vec<Expr> },
    FunctionCall { name: String, args: Vec<Expr> },
    Return(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Op {
    Plus, Minus, Multiply, Divide,
    Equals, Greater, Less,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }
    fn peek(&self) -> &Token { &self.tokens[self.pos] }
    fn next(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 { self.pos += 1; }
        t
    }
    fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline { self.next(); }
    }
    pub fn parse(&mut self) -> Vec<Expr> {
        let mut exprs = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek() == &Token::EOF { break; }
            exprs.push(self.parse_stmt());
        }
        exprs
    }
    fn parse_atom(&mut self) -> Expr {
        match self.next() {
            Token::Number(n) => Expr::Number(n),
            Token::StringLit(s) => Expr::StringLit(s),
            Token::Ident(s) => Expr::Ident(s),
            _ => Expr::Number(0.0),
        }
    }
    fn is_op(t: &Token) -> bool {
        matches!(t, Token::Plus | Token::Minus | Token::Multiply
            | Token::Divide | Token::Equals | Token::Greater | Token::Less)
    }
    fn token_to_op(t: Token) -> Op {
        match t {
            Token::Plus     => Op::Plus,
            Token::Minus    => Op::Minus,
            Token::Multiply => Op::Multiply,
            Token::Divide   => Op::Divide,
            Token::Equals   => Op::Equals,
            Token::Greater  => Op::Greater,
            Token::Less     => Op::Less,
            _ => Op::Plus,
        }
    }
    fn parse_expr(&mut self) -> Expr {
        let left = self.parse_atom();
        if Self::is_op(self.peek()) {
            let op = Self::token_to_op(self.next());
            let right = self.parse_atom();
            return Expr::BinOp {
                left: Box::new(left), op, right: Box::new(right)
            };
        }
        left
    }
    fn parse_stmt(&mut self) -> Expr {
        self.skip_newlines();
        match self.peek().clone() {
            Token::Show => {
                self.next();
                let val = self.parse_expr();
                Expr::Show(Box::new(val))
            }
            Token::Return => {
                self.next();
                Expr::Return(Box::new(self.parse_expr()))
            }
            Token::Function => {
                self.next();
                let name = if let Token::Ident(n) = self.next() { n }
                    else { String::new() };
                let mut params = Vec::new();
                if self.peek() == &Token::With {
                    self.next();
                    while let Token::Ident(p) = self.peek().clone() {
                        params.push(p); self.next();
                        if self.peek() == &Token::Do { break; }
                    }
                }
                if self.peek() == &Token::Do { self.next(); }
                self.skip_newlines();
                let mut body = Vec::new();
                while self.peek() != &Token::End && self.peek() != &Token::EOF {
                    body.push(self.parse_stmt());
                    self.skip_newlines();
                }
                self.next(); // end
                if self.peek() == &Token::Function { self.next(); }
                Expr::FunctionDef { name, params, body }
            }
            Token::Input => {
                self.next();
                let prompt = if let Token::StringLit(s) = self.peek().clone() {
                    self.next(); s
                } else { String::new() };
                self.next(); // store
                if let Token::Ident(name) = self.next() {
                    return Expr::Input { prompt, store_in: name };
                }
                Expr::Number(0.0)
            }
            Token::Loop => {
                self.next();
                let count = self.parse_expr();
                if self.peek() == &Token::Times { self.next(); }
                self.skip_newlines();
                let mut body = Vec::new();
                while self.peek() != &Token::End && self.peek() != &Token::EOF {
                    body.push(self.parse_stmt());
                    self.skip_newlines();
                }
                self.next(); // end
                if self.peek() == &Token::Loop { self.next(); }
                Expr::Loop { count: Box::new(count), body }
            }
            Token::Number(n) => {
                self.next();
                let val = if Self::is_op(self.peek()) {
                    let op = Self::token_to_op(self.next());
                    let right = self.parse_atom();
                    Expr::BinOp { left: Box::new(Expr::Number(n)), op, right: Box::new(right) }
                } else { Expr::Number(n) };
                if self.peek() == &Token::Store {
                    self.next();
                    if let Token::Ident(name) = self.next() {
                        return Expr::Store { value: Box::new(val), name };
                    }
                }
                val
            }
            Token::StringLit(s) => {
                self.next();
                let val = if self.peek() == &Token::Plus {
                    self.next();
                    let right = self.parse_atom();
                    Expr::BinOp { left: Box::new(Expr::StringLit(s)), op: Op::Plus, right: Box::new(right) }
                } else { Expr::StringLit(s) };
                if self.peek() == &Token::Store {
                    self.next();
                    if let Token::Ident(name) = self.next() {
                        return Expr::Store { value: Box::new(val), name };
                    }
                }
                val
            }
            Token::Ident(s) => {
                self.next();
                let val = if Self::is_op(self.peek()) {
                    let op = Self::token_to_op(self.next());
                    let right = self.parse_atom();
                    Expr::BinOp { left: Box::new(Expr::Ident(s.clone())), op, right: Box::new(right) }
                } else { Expr::Ident(s.clone()) };
                if self.peek() == &Token::Store {
                    self.next();
                    if let Token::Ident(name) = self.next() {
                        return Expr::Store { value: Box::new(val), name };
                    }
                }
                if matches!(val, Expr::Ident(_)) {
                    let mut args = Vec::new();
                    // args sirf same line pe — Newline aaye to stop
                    while !matches!(self.peek(),
                        Token::Newline | Token::EOF | Token::Store |
                        Token::Then | Token::Do | Token::End |
                        Token::Times | Token::Else)
                        && matches!(self.peek(),
                        Token::Number(_) | Token::StringLit(_) | Token::Ident(_))
                    {
                        args.push(self.parse_atom());
                    }
                    if self.peek() == &Token::Store {
                        self.next();
                        if let Token::Ident(var) = self.next() {
                            return Expr::Store {
                                value: Box::new(Expr::FunctionCall { name: s, args }),
                                name: var,
                            };
                        }
                    }
                    if !args.is_empty() {
                        return Expr::FunctionCall { name: s, args };
                    }
                }
                val
            }
            Token::Add => {
                self.next();
                let value = self.parse_expr();
                self.next(); // to
                if let Token::Ident(list) = self.next() {
                    return Expr::Add { value: Box::new(value), list };
                }
                Expr::Number(0.0)
            }
            Token::Remove => {
                self.next();
                let value = self.parse_expr();
                self.next(); // from
                if let Token::Ident(list) = self.next() {
                    return Expr::Remove { value: Box::new(value), list };
                }
                Expr::Number(0.0)
            }
            Token::If => {
                self.next();
                let cond = self.parse_expr();
                if self.peek() == &Token::Then { self.next(); }
                self.skip_newlines();
                let mut body = Vec::new();
                let mut else_body = Vec::new();
                while !matches!(self.peek(), Token::End | Token::Else | Token::EOF) {
                    body.push(self.parse_stmt());
                    self.skip_newlines();
                }
                if self.peek() == &Token::Else {
                    self.next();
                    self.skip_newlines();
                    while !matches!(self.peek(), Token::End | Token::EOF) {
                        else_body.push(self.parse_stmt());
                        self.skip_newlines();
                    }
                }
                self.next(); // end
                if self.peek() == &Token::If { self.next(); }
                Expr::If { condition: Box::new(cond), body, else_body }
            }
            _ => { self.next(); Expr::Number(0.0) }
        }
    }
}
