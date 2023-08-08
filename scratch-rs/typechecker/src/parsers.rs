use regex::Regex;

use crate::parser::ConstraintParser;
use crate::parser::ContractParser;
use crate::parser::MDynListParser;
use crate::parser::StackResultElemParser;
use crate::types::map_mtype;
use crate::types::mdyn_to_concrete;
use crate::types::ArgConstraint;
use crate::types::ConcreteType;
use crate::types::Constraint;
use crate::types::Contract;
use crate::types::DynMType;
use crate::types::DynMType::*;
use crate::types::MType;
use crate::types::SomeValue;
use crate::types::StackResult;
use crate::types::StackResultElem;
use crate::types::StackState;
use crate::types::StackState::*;

fn parse_mdyn_to<T, F: Fn(&MType<DynMType>) -> T>(cs: &str, cb: F) -> Vec<T> {
    if cs.len() == 0 {
        return Vec::new();
    } else {
        match MDynListParser::new().parse(cs) {
            Result::Ok(s) => return s.iter().map(cb).collect(),
            Result::Err(s) => panic!("{} when parsing {}", s, cs),
        }
    }
}

pub fn dynm_to_arg_constraint(d: DynMType) -> ArgConstraint {
    match d {
        DMDyn(s) => match ConstraintParser::new().parse(&s) {
            Result::Ok(s) => s,
            Result::Err(e) => panic!("Parsing of ArgConstraint failed!: {} : {}", e, s),
        },
        _ => panic!("Unexpected enum variant during constraint parsing"),
    }
}

pub fn dynm_to_stack_result(d: DynMType) -> StackResultElem {
    match d {
        DMDyn(s) => match StackResultElemParser::new().parse(&s) {
            Result::Ok(s) => s,
            Result::Err(s) => panic!("Parsing of stack result failed! {:?}", s),
        },
        _ => panic!("Unexpected enum variant during stack result parsing"),
    }
}

pub fn mdyn_to_constraint(m: &MType<DynMType>) -> Constraint {
    return map_mtype(m, &|x| dynm_to_arg_constraint(x.clone()));
}

pub fn mdyn_to_stack_result(m: &MType<DynMType>) -> StackResult {
    return map_mtype(m, &|x| dynm_to_stack_result(x.clone()));
}

pub fn parse_constraints(cs: &str) -> Vec<Constraint> {
    return parse_mdyn_to(cs, mdyn_to_constraint);
}

pub fn parse_contract(src: &str) -> Contract<SomeValue> {
    let re = Regex::new(r"#[^\n\r]*[\n\r]*").unwrap();
    let src_sans_comments = re.replace_all(src, "");
    println!("{}", src_sans_comments);
    match ContractParser::new().parse(&src_sans_comments) {
        Result::Ok(s) => {
            return s;
        }
        Result::Err(s) => panic!("{}", s),
    }
}

pub fn parse_stack_results(cs: &str) -> Vec<StackResult> {
    return parse_mdyn_to(cs, mdyn_to_stack_result);
}

pub fn parse_stack(cs: &str) -> StackState {
    return LiveStack(parse_mdyn_to(cs, mdyn_to_concrete));
}
