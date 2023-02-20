use std::{fs::{File}, io::Read};

use runner::run;

mod ast;
mod lexing;
mod parsing;
mod runner;


fn load_file(relative_path: &str) -> String {
    let mut f = File::open(relative_path).expect("Missing file");

    let mut buf: String = "".to_string();

    f.read_to_string(&mut buf).unwrap();

    return buf;
}

fn main() {
    let file_path = "./test/main.txt";
    run(load_file(file_path));
}
