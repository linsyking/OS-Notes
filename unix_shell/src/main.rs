use std::process::exit;
use unix_shell::ast::parse;
use unix_shell::eval::{eval, Input, Interrupt, Output};
use unix_shell::lex::lex;

use rustyline::error::ReadlineError;

fn execute(line: &String) -> Result<(), Interrupt> {
    let args = lex(line);
    let len = args.len();
    if len == 0 {
        return Ok(());
    }
    if let Some(ast) = parse(args) {
        // println!("{:?}", ast); // Print the AST
        eval(&ast, &Input::Stdin, &Output::Stdout)
    } else {
        Err(Interrupt::SyntaxError)
    }
}

fn main() {
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline("> ");
        let line = match readline {
            Ok(l) => {
                rl.add_history_entry(l.as_str()).unwrap();
                l
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(_) => {
                println!("invalid input");
                break;
            }
        };
        if let Err(e) = execute(&line) {
            match e {
                Interrupt::SyntaxError => {
                    eprintln!("Syntax Error!");
                }
                Interrupt::ForkError => {
                    eprintln!("Fork Error!");
                }
                Interrupt::ExecError(e) => {
                    eprintln!("Exec Error: {}", e.desc());
                }
                Interrupt::ChildError(e) => {
                    eprintln!("Sub-process Error: {}", e.desc());
                    exit(1);
                }
                Interrupt::Exit(code) => exit(code),
                Interrupt::OtherError(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
}
