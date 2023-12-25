use crate::ast::Proc;
use nix::fcntl::{open, OFlag};
use nix::libc::{STDIN_FILENO, STDOUT_FILENO};
use nix::sys::stat::Mode;
use nix::sys::wait::wait;
use nix::unistd::{chdir, mkfifo, ForkResult};
use nix::unistd::{dup2, execvp, fork};
use std::ffi::{CStr, CString};
use std::fs::remove_file;

#[derive(Debug)]
pub enum Interrupt {
    ChildError(String),
    ExecError(String),
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
    match cmd {
        Proc::SubProc((cs, _)) if cs.is_empty() => {
            return Err(Interrupt::ExecError(format!("Error: Syntax error")));
        }
        _ => {}
    }
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
                            Err(Interrupt::ExecError(String::from("Syntax Error")))
                        }
                    } else {
                        // Default exit 0
                        Err(Interrupt::Exit(0))
                    }
                }
                "cd" if cmd.len() <= 2 => {
                    if let Some(path) = cmd.get(1) {
                        chdir(path.as_str()).map_err(|_| {
                            Interrupt::ExecError(format!("Cannot cd such file or directory"))
                        })?;
                        Ok(())
                    } else {
                        // Do nothing
                        Ok(())
                    }
                }
                "cz" => {
                    // Collect Zombie Processes
                    let mut has_child = true;
                    while let Ok(_) = wait() {
                        has_child = false;
                    }
                    if has_child {
                        Err(Interrupt::ExecError(format!("No child process found")))
                    } else {
                        Ok(())
                    }
                }
                _ => {
                    // Execute as normal commands
                    // Creating the child process
                    // println!(
                    //     "[DEBUG] Forking prog {:?}, input {:?}, output {:?}",
                    //     cmd, input, output
                    // );
                    let pres = unsafe { fork() }
                        .map_err(|_| Interrupt::ExecError(format!("Cannot fork")))?;
                    match pres {
                        ForkResult::Parent { child: _ } => {
                            // println!(
                            //     "[DEBUG] Parent process, waiting for the child (pid: {}) to complete...",
                            //     child.as_raw()
                            // );
                            if !is_background && !non_block {
                                wait().map_err(|e| {
                                    Interrupt::ExecError(format!("Cannot wait, {}", e.desc()))
                                })?;
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
                                    .map_err(|e| {
                                        Interrupt::ChildError(format!(
                                            "Subprocess {:?} error: {}",
                                            cmd,
                                            e.desc()
                                        ))
                                    })?;
                                    dup2(fd, STDOUT_FILENO).map_err(|e| {
                                        Interrupt::ChildError(format!(
                                            "Subprocess {:?} error: {}",
                                            cmd,
                                            e.desc()
                                        ))
                                    })?;
                                }
                                Output::Pipefile(path) => {
                                    // println!("[DEBUG] Setting output to {}", path);
                                    let fd = open(path.as_str(), OFlag::O_WRONLY, Mode::S_IWUSR)
                                        .map_err(|e| {
                                            Interrupt::ChildError(format!(
                                                "Subprocess {:?} pipefile {} open error: {}",
                                                cmd,
                                                path,
                                                e.desc()
                                            ))
                                        })?;
                                    dup2(fd, STDOUT_FILENO).map_err(|e| {
                                        Interrupt::ChildError(format!(
                                            "Subprocess {:?} error: {}",
                                            cmd,
                                            e.desc()
                                        ))
                                    })?;
                                }
                            }
                            match input {
                                Input::Stdin => {}
                                Input::File(path) => {
                                    let fd = open(path.as_str(), OFlag::O_RDONLY, Mode::S_IRUSR)
                                        .map_err(|e| {
                                            Interrupt::ChildError(format!(
                                                "Subprocess {:?} pipefile {} open error: {}",
                                                cmd,
                                                path,
                                                e.desc()
                                            ))
                                        })?;
                                    dup2(fd, STDIN_FILENO).map_err(|e| {
                                        Interrupt::ChildError(format!(
                                            "Subprocess {:?} error: {}",
                                            cmd,
                                            e.desc()
                                        ))
                                    })?;
                                }
                                Input::Pipefile(path) => {
                                    // println!("[DEBUG] Setting input to {}", path);
                                    let fd = open(path.as_str(), OFlag::O_RDONLY, Mode::S_IRUSR)
                                        .map_err(|e| {
                                            Interrupt::ChildError(format!(
                                                "Subprocess {:?} pipefile {} open error: {}",
                                                cmd,
                                                path,
                                                e.desc()
                                            ))
                                        })?;
                                    dup2(fd, STDIN_FILENO).map_err(|e| {
                                        Interrupt::ChildError(format!(
                                            "Subprocess {:?} error: {}",
                                            cmd,
                                            e.desc()
                                        ))
                                    })?;
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
                            execvp(pname, &pargs).map_err(|e| {
                                Interrupt::ChildError(format!(
                                    "Subprocess {:?} cannot run execvp: {}",
                                    cmd,
                                    e.desc()
                                ))
                            })?;
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
        Proc::Pipe(ps) => {
            if ps.len() <= 1 {
                // Invalid!
                panic!("Invalid Pipe detected");
            }
            for id in 0..(ps.len() - 1) {
                let fifo_path = format!("fifo{}.pipe", id);
                // println!("[DEBUG] Creating tmp pipe {}", fifo_path);
                mkfifo(fifo_path.as_str(), Mode::S_IRWXU)
                    .map_err(|e| Interrupt::ExecError(format!("Makefifo error, {}", e.desc())))?;
            }
            eval(
                ps.first().unwrap(),
                input,
                &Output::Pipefile(format!("fifo0.pipe")),
                true,
            )?;
            for id in 1..(ps.len() - 1) {
                let cps = &ps[id];
                eval(
                    cps,
                    &Input::Pipefile(format!("fifo{}.pipe", id - 1)),
                    &Output::Pipefile(format!("fifo{}.pipe", id)),
                    true,
                )?;
            }
            eval(
                ps.last().unwrap(),
                &Input::Pipefile(format!("fifo{}.pipe", ps.len() - 2)),
                output,
                true,
            )?;
            // Wait for all the processes to finish
            for _ in 0..ps.len() {
                wait().map_err(|e| Interrupt::ExecError(format!("Cannot wait, {}", e.desc())))?;
            }
            for id in 0..(ps.len() - 1) {
                let fifo_path = format!("fifo{}.pipe", id);
                remove_file(fifo_path).unwrap();
            }
            Ok(())
        }
    }
}
