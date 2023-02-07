use runner::run;

mod ast;
mod lexing;
mod parsing;
mod runner;

fn main() {
    run("def main() {\nreturn 2+ 3\n}".to_string());
}
