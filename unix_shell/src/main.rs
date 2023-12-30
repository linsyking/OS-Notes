use nix::sys::wait::wait;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::env::current_dir;

use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Completer, Config, Helper, Hinter, Validator};
use std::process::exit;
use unix_shell::ast::parse;
use unix_shell::eval::{check_prog, eval, Input, Interrupt, Output};
use unix_shell::lex::lex;

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

#[derive(Helper, Completer, Hinter, Validator)]
struct MyHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[38;5;244m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}

fn main() {
    let config = Config::builder()
        .check_cursor_position(true)
        .completion_type(rustyline::CompletionType::List)
        .edit_mode(rustyline::EditMode::Emacs)
        .build();

    let h = MyHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter::new(),
        colored_prompt: "".to_owned(),
        validator: MatchingBracketValidator::new(),
    };
    let mut exit_code = 0;

    let mut rl = rustyline::Editor::with_config(config).unwrap();
    rl.set_helper(Some(h));
    let _ = rl.load_history("/tmp/.history");
    loop {
        let fname = current_dir().unwrap();
        let fname = fname.file_name().unwrap_or_default().to_str().unwrap();
        let p = format!("{}> ", fname);
        rl.helper_mut().expect("No helper").colored_prompt = format!("\x1b[1;32m{p}\x1b[0m");
        let readline = rl.readline(&p);
        let line = match readline {
            Ok(l) => {
                rl.add_history_entry(l.as_str()).unwrap();
                rl.save_history("/tmp/.history").unwrap();
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
