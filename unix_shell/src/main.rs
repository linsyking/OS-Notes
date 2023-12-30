use nix::sys::wait::wait;
use std::process::exit;
use unix_shell::ast::parse;
use unix_shell::eval::{check_prog, eval, Input, Interrupt, Output};
use unix_shell::lex::lex;

use reedline::{
    default_emacs_keybindings, DefaultPrompt, Emacs, FileBackedHistory, Reedline, Signal,
};

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
    let prompt = DefaultPrompt::default();
    let keybindings = default_emacs_keybindings();
    let edit_mode = Box::new(Emacs::new(keybindings));
    let history = Box::new(
        FileBackedHistory::with_file(1000, ".history".into())
            .expect("Error configuring history with file"),
    );
    let mut line_editor = Reedline::create()
        .with_edit_mode(edit_mode)
        .with_history(history);
    let mut exit_code = 0;
    loop {
        let sig = line_editor.read_line(&prompt);
        let line = match sig {
            Ok(Signal::Success(buffer)) => {
                line_editor.sync_history().unwrap();
                buffer
            }
            Ok(Signal::CtrlC) => {
                continue;
            }
            Ok(Signal::CtrlD) => {
                break;
            }
            x => {
                println!("Event: {:?}", x);
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
