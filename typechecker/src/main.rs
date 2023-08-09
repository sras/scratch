#[macro_use]
extern crate lazy_static;
use crate::instructions::MICHELSON_INSTRUCTIONS;
use crate::typechecker::typecheck_contract;
use std::io;
use std::time::{Duration, Instant};

mod attributes;
mod instructions;
mod parser;
mod parsers;
mod tests;
mod typechecker;
mod types;
use crate::parsers::parse_contract;

use typechecker::typecheck;

fn main() {
    // Force the evaluation of instructions.
    match MICHELSON_INSTRUCTIONS.get("PUSH") {
        Some(_) => {}
        None => {}
    }
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
                Result::Ok(i) => {
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
