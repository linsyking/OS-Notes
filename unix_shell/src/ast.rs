use crate::lex::Token;

type Command = (Vec<String>, bool);

#[derive(Debug, Clone)]
pub enum Proc {
    SubProc(Command), // Whether run in background
    Pipe(Box<Proc>, Command),
    RRed(Box<Proc>, String),
    LRed(Box<Proc>, String),
}

#[derive(Debug, Clone)]
enum Op {
    Pipe,
    RRed,
    LRed,
}

fn read_until_op(it: &mut std::slice::Iter<Token>) -> Option<(Option<Op>, Command)> {
    let mut res = Vec::new();
    let mut is_background = false;
    let mut op = None;
    while let Some(tok) = it.next() {
        match tok {
            Token::Str(cmd) => res.push(cmd.clone()),
            Token::Pipe => {
                op = Some(Op::Pipe);
                break;
            }
            Token::LeftRedirect => {
                op = Some(Op::LRed);
                break;
            }
            Token::RightRedirect => {
                op = Some(Op::RRed);
                break;
            }
            Token::Background => {
                is_background = true;
                break;
            }
        }
    }
    if res.is_empty() {
        None
    } else {
        Some((op, (res, is_background)))
    }
}

pub fn parse(toks: Vec<Token>) -> Option<Proc> {
    // Parse
    // All operators are left-associative and have the same precedence
    // p1 | p2 > p3 == (p1 | p2) > p3
    // p1 > p2 | p3 == (p1 > p2) | p3
    let mut cur = Proc::SubProc((Vec::new(), false));
    let mut it = toks.iter();
    let mut cur_op = None;
    while let Some((op, tok)) = read_until_op(&mut it) {
        match cur_op {
            None => {
                // Start
                cur = Proc::SubProc(tok.clone());
            }
            Some(Op::Pipe) => {
                // cur | tok
                cur = Proc::Pipe(Box::new(cur), tok.clone());
            }
            Some(Op::RRed) => {
                // cur > tok
                if tok.0.len() != 1 {
                    // Not a file
                    return None;
                } else {
                    // A file
                    cur = Proc::RRed(Box::new(cur), tok.0[0].clone());
                }
            }
            Some(Op::LRed) => {
                // cur < tok
                if tok.0.len() != 1 {
                    // Not a file
                    return None;
                } else {
                    // A file
                    cur = Proc::LRed(Box::new(cur), tok.0[0].clone());
                }
            }
        }
        cur_op = op;
    }
    if cur_op.is_none() {
        // Correct
        Some(cur)
    } else {
        None
    }
}
