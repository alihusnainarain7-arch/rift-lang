mod lexer;
mod parser;
mod interpreter;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    let code = if args.len() > 1 {
        match fs::read_to_string(&args[1]) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("RIFT Error: '{}' file nahi mili!", args[1]);
                return;
            }
        }
    } else {
        eprintln!("RIFT: koi file nahi di! Use: rift myfile.rift");
        return;
    };

    let tokens = lexer::tokenize(&code);
    let mut p = parser::Parser::new(tokens);
    let ast = p.parse();
    let mut interp = interpreter::Interpreter::new();
    interp.run(ast);
}
