#[macro_use]
extern crate lazy_static;
use std::io;
use crate::typechecker::typecheck_contract;

mod instructions;
mod parser;
mod parsers;
mod tests;
mod typechecker;
mod attributes;
mod types;

use typechecker::typecheck;

fn main() {
    match std::io::read_to_string(io::stdin()) {
        Result::Ok(i) => {
        match typecheck_contract(&i) {
            Result::Ok(i) => {
                println!("{:?}", i);
            }
            Result::Err(s) => {
                println!("{}", s);
            }
        }
        }
        Result::Err(s) => panic!("{}", s)
    }
}
