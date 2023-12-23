use nix::errno::Errno;
use nix::sys::wait::wait;
use nix::unistd::ForkResult;
use nix::unistd::{dup2, execvp, fork};
use std::ffi::{CStr, CString};
use std::{
    io::{self, BufRead, Write},
    process::exit,
};
use unix_shell::ast::{parse, Proc};
use unix_shell::lex::{lex, Token};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[derive(Debug)]
enum Interrupt {
    SyntaxError,
    ExecError(Errno),
    ForkError,
    Exit(i32),
}

fn eval(cmd: &Proc) -> Result<(), Interrupt> {
    match cmd {
        Proc::SubProc(cmd) => {
            // Match Internal Commnads
            let cmd0 = cmd[0].as_str();
            match cmd0 {
                "exit" => {
                    if let Some(code) = cmd.get(1) {
                        if let Ok(code) = code.parse() {
                            Err(Interrupt::Exit(code))
                        } else {
                            Err(Interrupt::SyntaxError)
                        }
                    } else {
                        // Default exit 0
                        Err(Interrupt::Exit(0))
                    }
                }
                _ => {
                    // Execute as normal commands
                    // Creating the child process
                    let pres = unsafe { fork() }.map_err(|_| Interrupt::ForkError)?;
                    match pres {
                        ForkResult::Parent { .. } => {
                            // println!(
                            //     "Parent process, waiting for the child (pid: {}) to complete...",
                            //     child.as_raw()
                            // );
                            wait().map_err(|e| Interrupt::ExecError(e))?;
                            // println!("Child process {} exited!", child.as_raw());
                        }
                        ForkResult::Child => {
                            let pname = CString::new(cmd0).unwrap();
                            let pname = pname.as_c_str();
                            let pargs = cmd.clone();
                            let pargs: Vec<CString> = pargs
                                .iter()
                                .map(|x| CString::new(x.clone()).unwrap())
                                .collect();
                            let pargs: Vec<&CStr> = pargs.iter().map(|x| x.as_c_str()).collect();
                            execvp(pname, &pargs).map_err(|e| Interrupt::ExecError(e))?;
                        }
                    }
                    Ok(())
                }
            }
        }
        _ => Ok(()),
    }
}

fn execute(line: &String) -> Result<(), Interrupt> {
    let args = lex(line);
    let len = args.len();
    if len == 0 {
        return Ok(());
    }
    if let Some(ast) = parse(args) {
        println!("{:?}", ast);
        // eval(&ast)
        Ok(())
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
            },
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
                    exit(1);
                }
                Interrupt::ForkError => {
                    eprintln!("Fork Error!");
                    exit(1);
                }
                Interrupt::ExecError(e) => {
                    eprintln!("Exec Error: {}", e.desc());
                    exit(1);
                }
                Interrupt::Exit(code) => exit(code),
            }
        }
    }
}
