use nix::errno::Errno;
use nix::sys::wait::wait;
use nix::unistd::ForkResult;
use nix::unistd::{dup2, execvp, fork};
use std::ffi::{CStr, CString};
use std::{
    io::{self, BufRead, Write},
    process::exit,
};
use unix_shell::lex::{Token, lex};

#[derive(Debug)]
enum Interrupt {
    SyntaxError,
    ExecError(Errno),
    ForkError,
    Exit(i32),
}


fn execute(line: &String) -> Result<(), Interrupt> {
    let args = lex(line);
    let len = args.len();
    if len == 0 {
        return Ok(());
    }
    if let Token::Str(cmd0) = args[0] {
        if len == 1 && cmd0 == "exit" {
            Err(Interrupt::Exit(0))
        } else {
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
                    let pargs = args.clone();
                    let pargs: Vec<CString> =
                        pargs.iter().map(|x| CString::new(*x).unwrap()).collect();
                    let pargs: Vec<&CStr> = pargs.iter().map(|x| x.as_c_str()).collect();
                    execvp(pname, &pargs).map_err(|e| Interrupt::ExecError(e))?;
                }
            }
            Ok(())
        }
    }else {
        Err(Interrupt::SyntaxError)
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
