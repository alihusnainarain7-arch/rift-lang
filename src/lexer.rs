#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Show, Store, Add, To, From, Remove,
    If, Else, Then, End, Input, Loop, Times,
    Function, With, Do, Return,
    Read, Write, Append,
    Plus, Minus, Multiply, Divide,
    Equals, Greater, Less,
    Newline,
    Number(f64),
    StringLit(String),
    Ident(String),
    EOF,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            '\n' => { tokens.push(Token::Newline); chars.next(); }
            ' ' | '\t' | '\r' => { chars.next(); }
            '+' => { tokens.push(Token::Plus); chars.next(); }
            '-' => { tokens.push(Token::Minus); chars.next(); }
            '*' => { tokens.push(Token::Multiply); chars.next(); }
            '/' => { tokens.push(Token::Divide); chars.next(); }
            '=' => { tokens.push(Token::Equals); chars.next(); }
            '>' => { tokens.push(Token::Greater); chars.next(); }
            '<' => { tokens.push(Token::Less); chars.next(); }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '"' { chars.next(); break; }
                    s.push(c); chars.next();
                }
                tokens.push(Token::StringLit(s));
            }
            '0'..='9' => {
                let mut num = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num.push(c); chars.next();
                    } else { break; }
                }
                tokens.push(Token::Number(num.parse().unwrap()));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut word = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        word.push(c); chars.next();
                    } else { break; }
                }
                let tok = match word.as_str() {
                    "show"     => Token::Show,
                    "store"    => Token::Store,
                    "add"      => Token::Add,
                    "to"       => Token::To,
                    "from"     => Token::From,
                    "remove"   => Token::Remove,
                    "if"       => Token::If,
                    "else"     => Token::Else,
                    "then"     => Token::Then,
                    "end"      => Token::End,
                    "input"    => Token::Input,
                    "loop"     => Token::Loop,
                    "times"    => Token::Times,
                    "function" => Token::Function,
                    "with"     => Token::With,
                    "do"       => Token::Do,
                    "return"   => Token::Return,
                    "read"     => Token::Read,
                    "write"    => Token::Write,
                    "append"   => Token::Append,
                    _          => Token::Ident(word),
                };
                tokens.push(tok);
            }
            _ => { chars.next(); }
        }
    }
    tokens.push(Token::EOF);
    tokens
}
