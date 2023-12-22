#[derive(Debug)]
pub enum Token {
    Str(String),
    Pipe,
    RightRedirect,
    LeftRedirect,
}

fn push_str(toks: &mut Vec<Token>, cur: &mut String) {
    if !cur.is_empty() {
        toks.push(Token::Str(cur.clone()));
        cur.clear();
    }
}

pub fn lex(line: &String) -> Vec<Token> {
    // A simple lexer
    let mut toks = Vec::new();
    let mut cur = String::new();
    let mut is_in_str = false;
    let mut it = line.chars();
    while let Some(c) = it.next() {
        match c {
            '\n' => {
                // EOL
                push_str(&mut toks, &mut cur);
            }
            '\\' => {
                let cn = it.next().unwrap();
                cur.push(cn);
                continue;
            }
            '"' => {
                if is_in_str {
                    is_in_str = false;
                    // Terminate string
                    push_str(&mut toks, &mut cur);
                } else {
                    is_in_str = true;
                }
            }
            _ if is_in_str => {
                cur.push(c);
            }
            ' ' => {
                push_str(&mut toks, &mut cur);
            }
            '|' => {
                // Pipe
                push_str(&mut toks, &mut cur);
                toks.push(Token::Pipe);
            }
            '>' => {
                // R-Redirect
                push_str(&mut toks, &mut cur);
                toks.push(Token::RightRedirect);
            }
            '<' => {
                // L-Redirect
                push_str(&mut toks, &mut cur);
                toks.push(Token::LeftRedirect);
            }
            '\0' => {
                // EOF
                push_str(&mut toks, &mut cur);
                break;
            }
            _ => {
                cur.push(c);
            }
        }
    }
    toks
}
