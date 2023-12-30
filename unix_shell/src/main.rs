use nix::sys::wait::wait;
use rustyline::Config;
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
    let config = Config::builder().check_cursor_position(true).build();
    let mut exit_code = 0;
    let mut rl = rustyline::DefaultEditor::with_config(config).unwrap();
    let _ = rl.load_history(".history");
    loop {
        let readline = rl.readline("> ");
        let line = match readline {
            Ok(l) => {
                rl.add_history_entry(l.as_str()).unwrap();
                rl.save_history(".history").unwrap();
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
                Interrupt::Exit(code) => {
                    exit_code = code;
                    break;
                }
                Interrupt::ExecError(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
    // println!("[DEBUG] Wait for all child processes to quit...");
    while let Ok(_) = wait() {}
    exit(exit_code);
}
