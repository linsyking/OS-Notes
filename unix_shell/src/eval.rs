use crate::ast::Proc;
use nix::errno::Errno;
use nix::fcntl::{open, OFlag};
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

fn pipe_output(output: &Output) -> bool {
    match output {
        Output::Pipefile(_) => true,
        _ => false,
    }
}

fn pipe_input(input: &Input) -> bool {
    match input {
        Input::Pipefile(_) => true,
        _ => false,
    }
}

const STDIN_FILENO: i32 = 0;
const STDOUT_FILENO: i32 = 1;

pub fn eval(cmd: &Proc, input: &Input, output: &Output) -> Result<(), Interrupt> {
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
                            if !is_background && !pipe_output(output) && !pipe_input(input) {
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
            eval(&proc, input, &Output::File(path.clone()))
        }
        Proc::LRed(proc, path) => {
            // proc < path
            eval(&proc, &Input::File(path.clone()), output)
        }
        Proc::Pipe(lhs, rhs) => {
            let tmp_dir = tempdir().unwrap();
            let fifo_path = tmp_dir.path().join(format!("{}.pipe", lhs.depth()));
            mkfifo(&fifo_path, Mode::S_IRWXU).map_err(|e| Interrupt::ExecError(e))?;
            let file_name = fifo_path.into_os_string().into_string().unwrap();
            // println!("Creating tmp pipe {}", file_name);
            eval(lhs, input, &Output::Pipefile(file_name.clone()))?;
            eval(
                &Proc::SubProc(rhs.clone()),
                &Input::Pipefile(file_name),
                output,
            )?;
            // Wait for the two processes to finish
            wait().map_err(|e| Interrupt::ExecError(e))?;
            tmp_dir
                .close()
                .map_err(|_| Interrupt::OtherError(String::from("Tmp dir close error")))?;
            Ok(())
        }
    }
}
