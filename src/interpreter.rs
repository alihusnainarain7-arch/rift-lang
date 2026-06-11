use crate::parser::{Expr, Op};
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Text(String),
    List(Vec<Value>),
    Null,
}

#[derive(Debug, Clone)]
struct Function {
    params: Vec<String>,
    body: Vec<Expr>,
}

enum RunResult {
    Value(Value),
    Return(Value),
}

pub struct Interpreter {
    pub vars: HashMap<String, Value>,
    functions: HashMap<String, Function>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            vars: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn run(&mut self, exprs: Vec<Expr>) {
        for expr in exprs {
            self.eval(expr);
        }
    }

    fn run_body(&mut self, exprs: Vec<Expr>) -> Value {
        for expr in exprs {
            match expr {
                Expr::Return(inner) => {
                    return self.eval(*inner);
                }
                other => { self.eval(other); }
            }
        }
        Value::Null
    }

    fn eval(&mut self, expr: Expr) -> Value {
        match expr {
            Expr::Number(n) => Value::Number(n),
            Expr::StringLit(s) => Value::Text(s),
            Expr::Ident(name) => {
                self.vars.get(&name).cloned().unwrap_or(Value::Null)
            }
            Expr::Show(inner) => {
                let val = self.eval(*inner);
                match &val {
                    Value::Number(n) => {
                        if *n == n.floor() { println!("{}", *n as i64); }
                        else { println!("{}", n); }
                    }
                    Value::Text(s) => println!("{}", s),
                    Value::List(items) => {
                        let parts: Vec<String> = items.iter().map(|i| match i {
                            Value::Number(n) => n.to_string(),
                            Value::Text(s) => s.clone(),
                            _ => String::new(),
                        }).collect();
                        println!("{}", parts.join(", "));
                    }
                    Value::Null => println!("null"),
                }
                val
            }
            Expr::FunctionDef { name, params, body } => {
                self.functions.insert(name, Function { params, body });
                Value::Null
            }
            Expr::FunctionCall { name, args } => {
                let func = self.functions.get(&name).cloned();
                if let Some(func) = func {
                    let old_vars = self.vars.clone();
                    let old_funcs = self.functions.clone();
                    for (param, arg) in func.params.iter().zip(args.iter()) {
                        let val = self.eval(arg.clone());
                        self.vars.insert(param.clone(), val);
                    }
                    let result = self.run_body(func.body);
                    self.vars = old_vars;
                    self.functions = old_funcs;
                    result
                } else {
                    println!("Error: '{}' function nahi mili!", name);
                    Value::Null
                }
            }
            Expr::Return(inner) => self.eval(*inner),
            Expr::Input { prompt, store_in } => {
                if !prompt.is_empty() {
                    print!("{} ", prompt);
                    io::stdout().flush().unwrap();
                }
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim().to_string();
                let val = if let Ok(n) = input.parse::<f64>() {
                    Value::Number(n)
                } else {
                    Value::Text(input)
                };
                self.vars.insert(store_in, val.clone());
                val
            }
            Expr::Store { value, name } => {
                let val = self.eval(*value);
                self.vars.insert(name, val.clone());
                val
            }
            Expr::Add { value, list } => {
                let val = self.eval(*value);
                let entry = self.vars.entry(list).or_insert(Value::List(Vec::new()));
                if let Value::List(v) = entry { v.push(val); }
                Value::Null
            }
            Expr::Remove { value, list } => {
                let val = self.eval(*value);
                if let Some(entry) = self.vars.get_mut(&list) {
                    if let Value::List(v) = entry {
                        v.retain(|x| match (x, &val) {
                            (Value::Text(a), Value::Text(b)) => a != b,
                            (Value::Number(a), Value::Number(b)) => a != b,
                            _ => true,
                        });
                    }
                }
                Value::Null
            }
            Expr::Loop { count, body } => {
                let n = match self.eval(*count) {
                    Value::Number(n) => n as i64,
                    _ => 0,
                };
                for _ in 0..n {
                    self.run(body.clone());
                }
                Value::Null
            }
            Expr::BinOp { left, op, right } => {
                let l = self.eval(*left);
                let r = self.eval(*right);
                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => match op {
                        Op::Plus     => Value::Number(a + b),
                        Op::Minus    => Value::Number(a - b),
                        Op::Multiply => Value::Number(a * b),
                        Op::Divide   => Value::Number(a / b),
                        Op::Equals   => Value::Number(if a == b { 1.0 } else { 0.0 }),
                        Op::Greater  => Value::Number(if a > b  { 1.0 } else { 0.0 }),
                        Op::Less     => Value::Number(if a < b  { 1.0 } else { 0.0 }),
                    },
                    (Value::Text(a), Value::Text(b)) => match op {
                        Op::Plus   => Value::Text(a + &b),
                        Op::Equals => Value::Number(if a == b { 1.0 } else { 0.0 }),
                        _ => Value::Null,
                    },
                    (Value::Text(a), Value::Number(b)) => match op {
                        Op::Plus => Value::Text(a + &b.to_string()),
                        _ => Value::Null,
                    },
                    (Value::Number(a), Value::Text(b)) => match op {
                        Op::Plus => Value::Text(a.to_string() + &b),
                        _ => Value::Null,
                    },
                    _ => Value::Null,
                }
            }
            Expr::If { condition, body, else_body } => {
                let cond = self.eval(*condition);
                let is_true = match cond {
                    Value::Number(n) => n != 0.0,
                    Value::Text(s) => !s.is_empty(),
                    Value::List(v) => !v.is_empty(),
                    Value::Null => false,
                };
                if is_true { self.run(body); }
                else { self.run(else_body); }
                Value::Null
            }
            _ => Value::Null,
        }
    }
}
