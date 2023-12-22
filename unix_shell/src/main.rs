use nix::errno::Errno;
use nix::sys::wait::wait;
use nix::unistd::ForkResult;
use nix::unistd::{dup2, execvp, fork};
use std::ffi::{CStr, CString};
use std::{
    io::{self, BufRead, Write},
    process::exit,
};
#[derive(Debug)]
enum Interrupt {
    SyntaxError,
    ExecError(Errno),
    ForkError,
    Exit(i32),
}

#[derive(Debug)]
enum Token {
    Str(String),
    Pipe,
    RightRedirect,
    LeftRedirect,
}

fn lex(line: &String) -> Vec<Token> {
    // A simple lexer
    let mut toks = Vec::new();
    let mut cur = String::new();
    let mut is_in_str = false;
    let mut it = line.chars();
    while let Some(c) = it.next() {
        match c {
            '\\' => {
                let cn = it.next().unwrap();
                cur.push(c);
                cur.push(cn);
                continue;
            }
            '"' => {
                if is_in_str {
                    is_in_str = false;
                    // Terminate string
                    toks.push(Token::Str(cur.clone()));
                    cur.clear();
                } else {
                    is_in_str = true;
                }
            }
            _ if is_in_str => {
                cur.push(c);
            }
            ' ' => {
                toks.push(Token::Str(cur.clone()));
                cur.clear();
            }
            '|' => {
                // Pipe
                toks.push(Token::Str(cur.clone()));
                toks.push(Token::Pipe);
                cur.clear();
            }
            '>' => {
                // R-Redirect
                toks.push(Token::Str(cur.clone()));
                toks.push(Token::RightRedirect);
                cur.clear();
            }
            '<' => {
                // L-Redirect
                toks.push(Token::Str(cur.clone()));
                toks.push(Token::LeftRedirect);
                cur.clear();
            }
            '\0' => {
                // EOF
                toks.push(Token::Str(cur.clone()));
                cur.clear();
                break;
            }
            _ => {
                cur.push(c);
            }
        }
    }
    toks
}

fn execute(line: &String) -> Result<(), Interrupt> {
    let args: Vec<_> = line.trim().split(" ").collect();
    let len = args.len();
    if len == 0 {
        return Ok(());
    }
    match args[0] {
        ";;" | "exit" if len == 1 => Err(Interrupt::Exit(0)),
        _ => {
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
                    let pname = CString::new(args[0]).unwrap();
                    let pname = pname.as_c_str();
                    let pargs = args.clone();
                    let pargs: Vec<CString> =
                        pargs.iter().map(|x| CString::new(*x).unwrap()).collect();
                    let pargs: Vec<&CStr> = pargs.iter().map(|x| x.as_c_str()).collect();
                    execvp(pname, &pargs).map_err(|e| Interrupt::ExecError(e))?;
                }
            }
            Ok(())
        }
    }
}

fn main() {
    let stdin = io::stdin();
    loop {
        let mut line = String::new();
        print!("$> ");
        io::stdout().flush().unwrap();
        if let Ok(len) = stdin.lock().read_line(&mut line) {
            if len == 0 {
                // EOF
                return;
            } else {
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
        } else {
            // Error!
            exit(1);
        }
    }
}
