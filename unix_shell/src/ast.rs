use crate::lex::Token;

#[derive(Debug, Clone)]
pub enum Proc {
    SubProc(Vec<String>),
    Pipe(Box<Proc>, Box<Proc>),
    RRed(Box<Proc>, Box<Proc>),
    LRed(Box<Proc>, Box<Proc>),
}

fn fill(proc: &Proc, cs: Vec<String>) -> Proc {
    match proc {
        Proc::SubProc(_) => Proc::SubProc(cs),
        Proc::Pipe(lhs, _) => Proc::Pipe(lhs.clone(), Box::new(Proc::SubProc(cs))),
        Proc::RRed(lhs, _) => Proc::RRed(lhs.clone(), Box::new(Proc::SubProc(cs))),
        Proc::LRed(lhs, _) => Proc::LRed(lhs.clone(), Box::new(Proc::SubProc(cs))),
    }
}

pub fn parse(toks: Vec<Token>) -> Proc {
    // Parse
    // All operators are left-associative and have the same precedence
    // p1 | p2 | p3 == (p1 | p2) | p3
    // p1 > p2 > p3 == (p1 > p2) > p3
    // p1 < p2 < p3 == (p1 < p2) < p3
    // p1 | p2 > p3 == (p1 | p2) > p3
    // p1 > p2 | p3 == (p1 > p2) | p3
    let mut cs = Vec::new();
    let mut cur = Proc::SubProc(Vec::new());
    for tok in toks.iter() {
        match tok {
            Token::Str(cmd) => cs.push(cmd.clone()),
            Token::Pipe => {
                cur = fill(&cur, cs.clone());
                cs.clear();
            }
            _ => todo!(),
        }
    }
    cur
}
