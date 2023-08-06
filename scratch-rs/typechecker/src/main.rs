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

    match parser::MDynParser::new().parse(
        "pair (int :a) nat",
    ) {
        Result::Ok(a) => println!("{:?}", a)
        ,
        Result::Err(s) => println!("{}", s),
    }
}
