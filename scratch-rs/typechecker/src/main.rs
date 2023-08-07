#[macro_use]
extern crate lazy_static;
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
    println!("{:?}", typecheck_contract("parameter int;storage int #some comment\n;code { CDR; PUSH int 1; ADD; NIL operation; PAIR; }"));
    //println!("asdasd");
}
