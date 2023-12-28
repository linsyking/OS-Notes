use unix_shell::ast::parse;
use unix_shell::eval::check_prog;
use unix_shell::lex::lex;

fn run_test(s: &str, pass: bool) {
    let line = String::from(s);
    let l = lex(&line);
    let ast = parse(l).unwrap();
    if pass {
        check_prog(&ast).unwrap();
    } else {
        check_prog(&ast).unwrap_err();
    }
}

// #[cfg(test)]
#[test]
fn syntax() {
    run_test("ls | ls", true);
    run_test("ls > a > a", false);
    run_test("ls > a < a", true);
    run_test("ls > a | cat", false);
    run_test("ls < a < s", false);
    run_test("ls | cat > b | m", false);
    run_test("ls | cat < a", false);
}
