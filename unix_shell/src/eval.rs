use crate::ast::Proc;
use nix::errno::Errno;
use nix::fcntl::{open, OFlag};
use nix::libc::{STDIN_FILENO, STDOUT_FILENO};
use nix::sys::stat::Mode;
use nix::sys::wait::wait;
use nix::unistd::{chdir, mkfifo, ForkResult};
use nix::unistd::{dup2, execvp, fork};
use std::ffi::{CStr, CString};
use tempfile::tempdir;

#[derive(Debug)]
pub enum Interrupt {
    SyntaxError,
    ExecError(Errno),
    ChildError(Errno),
    ForkError,
    OtherError(String),
    Exit(i32),
}

#[derive(Debug, Clone)]
pub enum Output {
    Stdout,
    File(String),
    Pipefile(String),
}

#[derive(Debug, Clone)]
pub enum Input {
    Stdin,
    File(String),
    Pipefile(String),
}

pub fn check_prog(cmd: &Proc) -> Result<(), Interrupt> {
    // Check if the command is valid
    // TO-DO
    Ok(())
}

pub fn eval(cmd: &Proc, input: &Input, output: &Output, non_block: bool) -> Result<(), Interrupt> {
    match cmd {
        Proc::SubProc((cmd, is_background)) => {
            if cmd.is_empty() {
                return Ok(());
            }
            // Match Internal Commands
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
                "cz" => {
                    // Collect Zombie Processes
                    while let Ok(_) = wait() {}
                    Ok(())
                }
                _ => {
                    // Execute as normal commands
                    // Creating the child process
                    let pres = unsafe { fork() }.map_err(|_| Interrupt::ForkError)?;
                    match pres {
                        ForkResult::Parent { child: _ } => {
                            // println!(
                            //     "[DEBUG] Parent process, waiting for the child (pid: {}) to complete...",
                            //     child.as_raw()
                            // );
                            if !is_background && !non_block {
                                wait().map_err(|e| Interrupt::ExecError(e))?;
                            }
                            // println!("[DEBUG] Child process {} exited!", child.as_raw());
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
                                    .map_err(|e| Interrupt::ChildError(e))?;
                                    dup2(fd, STDOUT_FILENO)
                                        .map_err(|e| Interrupt::ChildError(e))?;
                                }
                                Output::Pipefile(path) => {
                                    let fd = open(path.as_str(), OFlag::O_WRONLY, Mode::S_IWUSR)
                                        .map_err(|e| Interrupt::ChildError(e))?;
                                    dup2(fd, STDOUT_FILENO)
                                        .map_err(|e| Interrupt::ChildError(e))?;
                                }
                            }
                            match input {
                                Input::Stdin => {}
                                Input::File(path) => {
                                    let fd = open(path.as_str(), OFlag::O_RDONLY, Mode::S_IRUSR)
                                        .map_err(|e| Interrupt::ChildError(e))?;
                                    dup2(fd, STDIN_FILENO).map_err(|e| Interrupt::ChildError(e))?;
                                }
                                Input::Pipefile(path) => {
                                    let fd = open(path.as_str(), OFlag::O_RDONLY, Mode::S_IRUSR)
                                        .map_err(|e| Interrupt::ChildError(e))?;
                                    dup2(fd, STDIN_FILENO).map_err(|e| Interrupt::ChildError(e))?;
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
                            // When a process terminates, all of its open files are closed automatically by the kernel
                        }
                    }
                    Ok(())
                }
            }
        }
        Proc::RRed(proc, path) => {
            // proc > path
            eval(&proc, input, &Output::File(path.clone()), non_block)
        }
        Proc::LRed(proc, path) => {
            // proc < path
            eval(&proc, &Input::File(path.clone()), output, non_block)
        }
        Proc::Pipe(lhs, rhs) => {
            let tmp_dir = tempdir().unwrap();
            let fifo_path = tmp_dir.path().join("fifo.pipe");
            mkfifo(&fifo_path, Mode::S_IRWXU).map_err(|e| Interrupt::ExecError(e))?;
            let file_name = fifo_path.into_os_string().into_string().unwrap();
            // println!("[DEBUG] Creating tmp pipe {}", file_name);
            eval(lhs, input, &Output::Pipefile(file_name.clone()), true)?;
            eval(
                &Proc::SubProc(rhs.clone()),
                &Input::Pipefile(file_name),
                output,
                true,
            )?;
            // Wait for the two processes to finish
            wait().map_err(|e| Interrupt::ExecError(e))?;
            wait().map_err(|e| Interrupt::ExecError(e))?;
            // Pipe will be automatically deleted
            Ok(())
        }
    }
}
