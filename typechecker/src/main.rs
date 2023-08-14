use std::io;
use std::time::Instant;

use typechecker::attributes::*;
use typechecker::types::*;
use typechecker::instructions::*;
use typechecker::parser::*;
use typechecker::parsers::*;
use typechecker::typechecker::*;

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
