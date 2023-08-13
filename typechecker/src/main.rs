#[macro_use]
extern crate lazy_static;
use crate::instructions::MICHELSON_INSTRUCTIONS;
use crate::typechecker::typecheck_contract;
use std::io;
use std::time::{Instant};

mod attributes;
mod instructions;
mod parser;
mod parsers;
mod tests;
mod typechecker;
mod types;
use crate::parsers::parse_contract;

fn main() {
    // Force the evaluation of instructions.
    let _ = MICHELSON_INSTRUCTIONS.get("PUSH");
    match std::io::read_to_string(io::stdin()) {
        Result::Ok(i) => {
            let mut start_time = Instant::now();
            let contract = parse_contract(&i);
            println!(
                "Parsed in {} mills..",
                Instant::now().duration_since(start_time).as_millis()
            );
            start_time = Instant::now();
            match typecheck_contract(contract) {
                Result::Ok(_) => {
                    println!(
                        "Successful typecheck in {} millis..",
                        Instant::now().duration_since(start_time).as_millis()
                    );
                }
                Result::Err(s) => {
                    println!("{}", s);
                }
            }
        }
        Result::Err(s) => panic!("{}", s),
    }
}
