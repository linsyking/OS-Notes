#[derive(Debug)]
pub enum Token {
    Str(String),
    Pipe,
    RightRedirect,
    LeftRedirect,
}

pub fn lex(line: &String) -> Vec<Token> {
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
