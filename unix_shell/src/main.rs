use nix::sys::wait::wait;
use std::process::exit;
use unix_shell::ast::parse;
use unix_shell::eval::{check_prog, eval, Input, Interrupt, Output};
use unix_shell::lex::lex;

use rustyline::error::ReadlineError;

fn execute(line: &String) -> Result<(), Interrupt> {
    let args = lex(line);
    // println!("{:?}", args); // Print the lexer result
    let len = args.len();
    if len == 0 {
        return Ok(());
    }
    if let Some(ast) = parse(args) {
        // println!("{:?}", ast); // Print the AST
        check_prog(&ast)?;
        eval(&ast, &Input::Stdin, &Output::Stdout, false)
    } else {
        Err(Interrupt::ExecError(format!("Syntax error")))
    }
}

fn main() {
    let mut exit_code = 0;
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
                Interrupt::ChildError(e) => {
                    eprintln!("Sub-process Error: {}", e);
                    exit_code = 1;
                    break;
                }
                Interrupt::Exit(code) => exit_code = code,
                Interrupt::ExecError(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
    println!("[DEBUG] Wait for all child processes to quit...");
    match wait() {
        Ok(_) => {
            println!("[DEBUG] Gracefully shutdown")
        }
        Err(e) => {
            println!("[DEBUG] {}", e.desc())
        }
    }
    exit(exit_code);
}
