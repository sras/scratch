#[macro_use]
extern crate lazy_static;

mod instructions;
mod parser;
mod parsers;
mod tests;
mod typechecker;
mod attributes;
mod types;

use typechecker::typecheck;

fn main() {
    let mut stack = Vec::new();

    match parser::InstructionListParser::new().parse(
        "LAMBDA nat (pair nat nat) {DUP;PAIR};PUSH nat 5;PUSH (pair nat int) (Pair 5 10);DROP",
    ) {
        Result::Ok(parsed_instructions) => match typecheck(&parsed_instructions, &mut stack) {
            Result::Ok(_) => {
                println!("{:?}", stack);
            }
            Err(s) => {
                println!("{}", s);
            }
        },
        Result::Err(s) => println!("{}", s),
    }
}
