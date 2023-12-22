use crate::lex::Token;

#[derive(Debug, Clone)]
pub enum Proc {
    SubProc(Vec<String>),
    Pipe(Box<Proc>, Box<Proc>),
    RRed(Box<Proc>, String),
    LRed(Box<Proc>, String),
}

fn read_until_op(it: &mut std::slice::Iter<Token>) -> Option<Vec<String>> {
    let mut res = Vec::new();
    while let Some(Token::Str(cmd)) = it.next() {
        res.push(cmd.clone());
    }
    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}

fn read_one_word(it: &mut std::slice::Iter<Token>) -> Option<String> {
    let m = it.next()?;
    if let Token::Str(s) = m {
        Some(s.clone())
    } else {
        None
    }
}

pub fn parse(toks: Vec<Token>) -> Option<Proc> {
    // Parse
    // All operators are left-associative and have the same precedence
    // p1 | p2 | p3 == (p1 | p2) | p3
    // p1 > p2 > p3 == (p1 > p2) > p3
    // p1 < p2 < p3 == (p1 < p2) < p3
    // p1 | p2 > p3 == (p1 | p2) > p3
    // p1 > p2 | p3 == (p1 > p2) | p3
    let mut cs = Vec::new();
    let mut cur = Proc::SubProc(Vec::new());
    let mut it = toks.iter();
    while let Some(tok) = it.next() {
        match tok {
            Token::Str(cmd) => cs.push(cmd.clone()),
            Token::Pipe => {
                cs.clear();
            }
            _ => todo!(),
        }
    }
    Some(cur)
}
