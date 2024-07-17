mod ast;
mod dsl;
mod parser;
mod printer;

use std::{
    io::{stdin, Read},
    process,
};

use parser::parse;

fn main() {
    let mut input: String = String::new();

    stdin()
        .read_to_string(&mut input)
        .expect("Failed to read STDIN.");

    match parse(&input) {
        Ok(json) => {
            println!("{json}");
        }
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };
}
