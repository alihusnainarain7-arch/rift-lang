use crate::parser::{Expr, Op};
use std::collections::HashMap;
use std::io::{self, Write};
use std::fs;

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

pub struct Interpreter {
    pub vars: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    call_stack: Vec<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            vars: HashMap::new(),
            functions: HashMap::new(),
            call_stack: Vec::new(),
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
                Expr::Return(inner) => return self.eval(*inner),
                other => { self.eval(other); }
            }
        }
        Value::Null
    }

    fn error(&self, msg: &str) {
        if self.call_stack.is_empty() {
            eprintln!("RIFT Error: {}", msg);
        } else {
            eprintln!("RIFT Error in '{}': {}", self.call_stack.last().unwrap(), msg);
        }
    }

    fn val_to_string(&self, v: &Value) -> String {
        match v {
            Value::Number(n) => {
                if *n == n.floor() { format!("{}", *n as i64) }
                else { format!("{}", n) }
            }
            Value::Text(s) => s.clone(),
            Value::List(items) => {
                let parts: Vec<String> = items.iter().map(|i| self.val_to_string(i)).collect();
                parts.join(", ")
            }
            Value::Null => String::from("null"),
        }
    }

    fn eval(&mut self, expr: Expr) -> Value {
        match expr {
            Expr::Number(n) => Value::Number(n),
            Expr::StringLit(s) => Value::Text(s),
            Expr::Ident(name) => {
                match self.vars.get(&name).cloned() {
                    Some(v) => v,
                    None => {
                        self.error(&format!("'{}' variable nahi mila!", name));
                        Value::Null
                    }
                }
            }
            Expr::Show(inner) => {
                let val = self.eval(*inner);
                println!("{}", self.val_to_string(&val));
                val
            }
            Expr::ReadFile { path, store_in } => {
                let p = self.eval(*path);
                let path_str = self.val_to_string(&p);
                match fs::read_to_string(&path_str) {
                    Ok(content) => {
                        let val = Value::Text(content.trim().to_string());
                        self.vars.insert(store_in, val.clone());
                        val
                    }
                    Err(_) => {
                        self.error(&format!("'{}' file nahi parh saka!", path_str));
                        Value::Null
                    }
                }
            }
            Expr::WriteFile { path, content } => {
                let p = self.eval(*path);
                let c = self.eval(*content);
                let path_str = self.val_to_string(&p);
                let content_str = self.val_to_string(&c);
                match fs::write(&path_str, &content_str) {
                    Ok(_) => {
                        println!("'{}' mein likha gaya!", path_str);
                        Value::Text(content_str)
                    }
                    Err(_) => {
                        self.error(&format!("'{}' mein nahi likh saka!", path_str));
                        Value::Null
                    }
                }
            }
            Expr::AppendFile { path, content } => {
                let p = self.eval(*path);
                let c = self.eval(*content);
                let path_str = self.val_to_string(&p);
                let content_str = self.val_to_string(&c);
                let existing = fs::read_to_string(&path_str).unwrap_or_default();
                let new_content = existing + &content_str + "\n";
                match fs::write(&path_str, &new_content) {
                    Ok(_) => Value::Text(content_str),
                    Err(_) => {
                        self.error(&format!("'{}' mein append nahi ho saka!", path_str));
                        Value::Null
                    }
                }
            }
            Expr::FunctionDef { name, params, body } => {
                self.functions.insert(name, Function { params, body });
                Value::Null
            }
            Expr::FunctionCall { name, args } => {
                let func = self.functions.get(&name).cloned();
                if let Some(func) = func {
                    if args.len() != func.params.len() {
                        self.error(&format!(
                            "'{}' ko {} arguments chahiye, {} diye!",
                            name, func.params.len(), args.len()
                        ));
                        return Value::Null;
                    }
                    let old_vars = self.vars.clone();
                    let old_funcs = self.functions.clone();
                    self.call_stack.push(name.clone());
                    for (param, arg) in func.params.iter().zip(args.iter()) {
                        let val = self.eval(arg.clone());
                        self.vars.insert(param.clone(), val);
                    }
                    let result = self.run_body(func.body);
                    self.call_stack.pop();
                    self.vars = old_vars;
                    self.functions = old_funcs;
                    result
                } else {
                    self.error(&format!("'{}' function nahi mila!", name));
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
                        Op::Divide   => {
                            if b == 0.0 { self.error("Zero se divide nahi!"); Value::Null }
                            else { Value::Number(a / b) }
                        }
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
