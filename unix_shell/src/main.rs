use nix::errno::Errno;
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::sys::wait::wait;
use nix::unistd::{chdir, ForkResult};
use nix::unistd::{dup2, execvp, fork};
use std::ffi::{CStr, CString};
use std::process::exit;
use unix_shell::ast::{parse, Proc};
use unix_shell::lex::lex;

use rustyline::error::ReadlineError;

#[derive(Debug)]
enum Interrupt {
    SyntaxError,
    ExecError(Errno),
    ChildError(Errno),
    ForkError,
    Exit(i32),
}

#[derive(Debug, Clone)]
enum Output {
    Stdout,
    File(String),
}

#[derive(Debug, Clone)]
enum Input {
    Stdin,
    File(String),
}

const STDIN_FILENO: i32 = 0;
const STDOUT_FILENO: i32 = 1;

fn eval(cmd: &Proc, input: &Input, output: &Output) -> Result<(), Interrupt> {
    match cmd {
        Proc::SubProc((cmd, is_background)) => {
            if cmd.is_empty() {
                return Ok(());
            }
            // Match Internal Commnads
            let cmd0 = cmd[0].as_str();
            match cmd0 {
                "exit" if cmd.len() <= 2 => {
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
                "cd" if cmd.len() <= 2 => {
                    if let Some(path) = cmd.get(1) {
                        chdir(path.as_str()).map_err(|e| Interrupt::ExecError(e))?;
                        Ok(())
                    } else {
                        // Do nothing
                        Ok(())
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
                            if !is_background {
                                wait().map_err(|e| Interrupt::ExecError(e))?;
                            }
                            // println!("Child process {} exited!", child.as_raw());
                        }
                        ForkResult::Child => {
                            match output {
                                Output::Stdout => {}
                                Output::File(path) => {
                                    // fd = open(path)
                                    // dup2(fd, stdout)
                                    let fd = open(
                                        path.as_str(),
                                        OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_TRUNC,
                                        Mode::S_IRUSR
                                            | Mode::S_IWUSR
                                            | Mode::S_IWGRP
                                            | Mode::S_IRGRP
                                            | Mode::S_IROTH,
                                    )
                                    .map_err(|e| Interrupt::ExecError(e))?;
                                    dup2(fd, STDOUT_FILENO).map_err(|e| Interrupt::ExecError(e))?;
                                }
                            }
                            match input {
                                Input::Stdin => {}
                                Input::File(path) => {
                                    let fd = open(
                                        path.as_str(),
                                        OFlag::O_RDONLY,
                                        Mode::S_IRUSR
                                            | Mode::S_IWUSR
                                            | Mode::S_IWGRP
                                            | Mode::S_IRGRP
                                            | Mode::S_IROTH,
                                    )
                                    .map_err(|e| Interrupt::ExecError(e))?;
                                    dup2(fd, STDIN_FILENO).map_err(|e| Interrupt::ExecError(e))?;
                                }
                            }
                            let pname = CString::new(cmd0).unwrap();
                            let pname = pname.as_c_str();
                            let pargs = cmd.clone();
                            let pargs: Vec<CString> = pargs
                                .iter()
                                .map(|x| CString::new(x.clone()).unwrap())
                                .collect();
                            let pargs: Vec<&CStr> = pargs.iter().map(|x| x.as_c_str()).collect();
                            execvp(pname, &pargs).map_err(|e| Interrupt::ChildError(e))?;

                            // This is not necessary.
                            // When a process terminates, all of its open files are closed automatically by the kernel.
                            // match output {
                            //     Output::File(_) if fd != 0 => {
                            //         close(fd).map_err(|e| Interrupt::ExecError(e))?;
                            //     }
                            //     _ => {}
                            // }
                        }
                    }
                    Ok(())
                }
            }
        }
        Proc::RRed(proc, path) => {
            // proc > path
            eval(&proc, input, &Output::File(path.clone()))
        }
        Proc::LRed(proc, path) => {
            // proc < path
            eval(&proc, &Input::File(path.clone()), output)
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
            }
        }
    }
}
