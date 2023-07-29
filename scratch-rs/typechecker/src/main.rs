#[macro_use]
extern crate lazy_static;

mod parser;
mod types;
use types::*;

use std::collections::HashMap;
use std::convert::TryFrom;

use parser::ConstraintParser;
use parser::MDynListParser;
use parser::StackResultElemParser;
use ArgConstraint::*;
use ArgValue as AV;
use AtomicValue::*;
use CompositeValue::*;
use DynMType::*;
use MAtomic::*;
use MType::*;
use MValue::*;
use SomeValue::*;
use StackResultElem::*;

type ResolveCache = HashMap<char, ConcreteType>;

impl<T: Clone> Clone for MType<T> {
    fn clone(&self) -> Self {
        return map_mtype(self, |x| x.clone());
    }
}

fn parse_mdyn_to<T, F: Fn(&MType<DynMType>) -> T>(cs: &str, cb: F) -> Vec<T> {
    if cs.len() == 0 {
        return Vec::new();
    } else {
        MDynListParser::new()
            .parse(cs)
            .unwrap()
            .iter()
            .map(cb)
            .collect()
    }
}

fn parse_constraints(cs: &str) -> Vec<Constraint> {
    return parse_mdyn_to(cs, mdyn_to_constraint);
}

fn parse_stack_results(cs: &str) -> Vec<StackResult> {
    return parse_mdyn_to(cs, mdyn_to_stack_result);
}

fn parse_concrete(cs: &str) -> Vec<ConcreteType> {
    return parse_mdyn_to(cs, mdyn_to_concrete);
}

macro_rules! mk_instr {
    ($n:expr, $arg: expr, $is: expr, $os: expr) => {
        (
            String::from($n),
            InstructionDef {
                args: parse_constraints($arg),
                input_stack: parse_constraints($is),
                output_stack: parse_stack_results($os),
            },
        )
    };
}

lazy_static! {
    static ref MICHELSON_INSTRUCTIONS: HashMap<String, InstructionDef> = HashMap::from([
        //mk_instr!("SWAP", "", "<w|a>;<w|b>", "<r|b>;<r|a>"),
        mk_instr!("DUP", "", "<w|a>", "<r|a>;<r|a>"),
        mk_instr!("DROP", "", "<w|a>", ""),
        mk_instr!("ADD", "", "<w|a>;<r|a>", "<r|a>"),
        mk_instr!("CONS", "", "<w|a>;list <r|a>", "list <r|a>"),
        mk_instr!("PUSH", "<t|a>;<r|a>", "", "<r|a>"),
        mk_instr!("PAIR", "", "<w|a>;<w|b>", "pair <r|a> <r|b>"),
        mk_instr!(
            "LAMBDA",
            "<t|a>;<t|b>;lambda <r|a> <r|b>",
            "",
            "lambda <r|a> <r|b>"
        ),
        mk_instr!("EXEC", "", "<w|a>;lambda <r|a> <w|b>", "<r|b>"),
    ]);
}

fn add_symbol<'a>(resolved: &mut ResolveCache, arg_con: char, type_: &ConcreteType) {
    resolved.insert(arg_con, type_.clone());
}

fn unify_args<'a>(
    args: &Vec<ArgValue<SomeValue>>,
    arg_cons: &Vec<Constraint>,
) -> Result<(ResolveCache, Vec<ArgValue<MValue>>), &'a str> {
    let mut resolved = HashMap::new();
    let mut args_ = Vec::new();
    for (arg, con) in args.iter().zip(arg_cons.iter()) {
        args_.push(unify_arg(&mut resolved, arg, con.clone())?);
    }
    return Result::Ok((resolved, args_));
}

fn unify_concrete_arg<'a>(
    resolved: &mut ResolveCache,
    arg: &ConcreteType,
    arg_con: &Constraint,
) -> Result<(), &'a str> {
    match arg_con {
        MWrapped(CWarg(c)) => {
            add_symbol(resolved, c.clone(), arg);
            return Result::Ok(());
        }
        MWrapped(CTypeArg(c)) => {
            add_symbol(resolved, c.clone(), arg);
            return Result::Ok(());
        }
        MWrapped(CTypeArgRef(c)) => match resolved.get(&c) {
            Some(tt) => {
                return unify_concrete_arg(resolved, arg, &map_mtype(tt, |x| CAtomic(x.clone())));
            }
            _ => {
                return Result::Err("Unknown type ref");
            }
        },
        MList(ic) => match arg {
            MList(iv) => {
                return unify_concrete_arg(resolved, iv.as_ref(), ic);
            }

            _ => {
                return Result::Err("Expecting a list but got something else...");
            }
        },
        MLambda(vin, vout) => match arg {
            MLambda(cin, cout) => {
                return unify_concrete_arg(resolved, cin, vin)
                    .and_then(|_| unify_concrete_arg(resolved, cout, vout));
            }
            _ => {
                return Result::Err("Expecting a lambda but got something else...");
            }
        },
        MPair(cl, cr) => match arg {
            MPair(vl, vr) => {
                return unify_concrete_arg(resolved, vl, cl)
                    .and_then(|_| unify_concrete_arg(resolved, vr, cr));
            }
            _ => {
                return Result::Err("Expecting a pair but got something else...");
            }
        },
        MWrapped(CAtomic(MNat)) => match arg {
            MWrapped(MNat) => {
                return Result::Ok(());
            }
            _ => {
                return Result::Err("Expecting a `Nat`, but found something else...");
            }
        },
        MWrapped(CAtomic(MInt)) => match arg {
            MWrapped(MInt) => {
                return Result::Ok(());
            }
            _ => {
                return Result::Err("Expecting a `Int`, but found something else...");
            }
        },
        MWrapped(CAtomic(MString)) => match arg {
            MWrapped(MString) => {
                return Result::Ok(());
            }
            _ => {
                return Result::Err("Expecting a `String`, but found something else...");
            }
        },
    }
}

fn unify_arg<'a>(
    resolved: &mut ResolveCache,
    arg: &ArgValue<SomeValue>,
    arg_con: &Constraint,
) -> Result<ArgValue<MValue>, &'a str> {
    match arg {
        AV::TypeArg(ct) => match arg_con {
            MWrapped(CTypeArg(c)) => {
                add_symbol(resolved, *c, &ct);
                return Result::Ok(AV::TypeArg((*ct).clone()));
            }
            _ => {
                panic!("Unexpected type name argument");
            }
        },
        AV::ValueArg(some_val) => {
            let (m, ct): (MValue, ConcreteType) = match arg_con {
                MWrapped(CTypeArg(_)) => {
                    panic!("Unexpected value argument");
                }
                MWrapped(CWarg(_)) => {
                    panic!("Unexpected wildcard type encountered");
                }
                MWrapped(CTypeArgRef(ref c)) => match resolved.get(&c) {
                    Some(ct) => typecheck_value(resolved, &some_val, ct)?,
                    None => panic!("Symbol resolution failed! {:?}", c),
                },
                _ => match constraint_to_concrete(resolved, &arg_con) {
                    Some(concrete_type) => typecheck_value(resolved, &some_val, &concrete_type)?,
                    None => panic!("Couldnt resolve type"),
                },
            };
            unify_concrete_arg(resolved, &ct, &arg_con)?;
            return Ok(AV::ValueArg(m));
        }
    }
}

fn constraint_to_concrete(resolved: &ResolveCache, c: &Constraint) -> Option<ConcreteType> {
    match c {
        MWrapped(CTypeArgRef(c)) => match resolved.get(&c) {
            Some(ct) => Some(ct.clone()),
            None => None,
        },
        MWrapped(CAtomic(x)) => Some(MWrapped(x.clone())),
        MPair(l, r) => Some(MPair(
            Box::new(constraint_to_concrete(resolved, l)?),
            Box::new(constraint_to_concrete(resolved, r)?),
        )),
        MList(l) => Some(MList(Box::new(constraint_to_concrete(resolved, l)?))),
        MLambda(l, r) => Some(MLambda(
            Box::new(constraint_to_concrete(resolved, l)?),
            Box::new(constraint_to_concrete(resolved, r)?),
        )),
        _ => None,
    }
}

fn typecheck_value<'a>(
    resolved: &ResolveCache,
    some_val: &SomeValue,
    target: &ConcreteType,
) -> Result<(MValue, ConcreteType), &'a str> {
    match (target, some_val) {
        (MWrapped(MNat), Atomic(AVNumber(n))) => match u32::try_from(*n) {
            Ok(n1) => Ok((VNat(n1), MWrapped(MNat))),
            Err(_) => Err("Expecting a Nat but found an Int"),
        },
        (MWrapped(MInt), Atomic(AVNumber(n))) => Ok((VInt(*n), MWrapped(MInt))),
        (MWrapped(MString), Atomic(AVString(s))) => Ok((VString(s.clone()), MWrapped(MString))),
        (MList(c), Composite(cv)) => match cv.as_ref() {
            CVList(items) => {
                let mut il: Vec<MValue> = vec![];
                for i in items {
                    let (mv, _) = typecheck_value(resolved, i, c.as_ref())?;
                    il.push(mv);
                }
                return Ok((VList(il), MList(c.clone())));
            }
            _ => Err("Expecting a List but found something else..."),
        },
        (MPair(c1, c2), Composite(cv)) => match cv.as_ref() {
            CVPair(sv1, sv2) => {
                let (mv1, ct1) = typecheck_value(resolved, sv1, c1.as_ref())?;
                let (mv2, ct2) = typecheck_value(resolved, sv2, c2.as_ref())?;
                return Result::Ok((
                    VPair(Box::new(mv1), Box::new(mv2)),
                    MPair(Box::new(ct1), Box::new(ct2)),
                ));
            }
            _ => Err("Expecting a Pair but found something else..."),
        },
        (MLambda(c1, c2), Composite(cv)) => match cv.as_ref() {
            CVLambda(instructions) => {
                let lambda_input = c1.as_ref().clone();
                let lambda_output = c2.as_ref().clone();
                let mut stack: StackState = Vec::from([lambda_input.clone()]);
                match typecheck(instructions, &mut stack) {
                    Ok(tins) => match stack[..] {
                        [ref real_out] => {
                            if (*real_out) == lambda_output {
                                return Result::Ok((
                                    VLambda(tins),
                                    MLambda(Box::new(lambda_input), Box::new(lambda_output)),
                                ));
                            } else {
                                return Err("Lambda does not match the expected type");
                            }
                        }
                        _ => {
                            return Err("Lambda produces more then one element on stack!");
                        }
                    },
                    Err(s) => {
                        return Err(s);
                    }
                }
            }
            _ => Err("Expecting a Lambda but found something else..."),
        },
        _ => Err("Error type mismatch"),
    }
}

fn stack_result_to_concrete_type(resolved: &mut ResolveCache, sr: &StackResult) -> ConcreteType {
    match sr {
        MWrapped(wrp) => match wrp {
            ElemType(et) => match et {
                MInt => MWrapped(MInt),
                MNat => MWrapped(MNat),
                MString => MWrapped(MString),
            },
            TRef(c) => match resolved.get(&c) {
                Some(ct) => {
                    return (*ct).clone();
                }
                None => {
                    panic!("Symbol resolution failed! {:?}", c)
                }
            },
        },
        MList(l) => MList(Box::new(stack_result_to_concrete_type(
            resolved,
            l.as_ref(),
        ))),
        MPair(l, r) => MPair(
            Box::new(stack_result_to_concrete_type(resolved, &l)),
            Box::new(stack_result_to_concrete_type(resolved, &r)),
        ),
        MLambda(l, r) => MLambda(
            Box::new(stack_result_to_concrete_type(resolved, &l)),
            Box::new(stack_result_to_concrete_type(resolved, &r)),
        ),
    }
}

fn make_resolved_stack<'a>(
    resolved: &mut ResolveCache,
    sem_stack_out: &Vec<StackResult>,
) -> Result<StackState, &'a str> {
    let mut resolved_stack: StackState = vec![];
    for i in sem_stack_out {
        resolved_stack.push(stack_result_to_concrete_type(resolved, &i));
    }
    return Result::Ok(resolved_stack);
}

fn unify_stack<'a>(
    resolved: &mut ResolveCache,
    sem_stack_in: &Vec<StackArg>,
    sem_stack_out: &Vec<StackResult>,
    stack_state: &mut StackState,
) -> Result<(), &'a str> {
    let mut stack_index: usize = 0;
    if stack_state.len() < sem_stack_in.len() {
        return Result::Err("Stack was found too small for the operation");
    }
    for constraint in sem_stack_in {
        let stack_elem = &stack_state[stack_index];
        unify_concrete_arg(resolved, &stack_elem, &constraint)?;
        stack_index = stack_index + 1;
    }
    let mut rs = make_resolved_stack(resolved, sem_stack_out)?;
    rs.append(&mut stack_state[stack_index..].to_vec());
    *stack_state = rs;
    return Result::Ok(());
}

fn typecheck<'a>(
    instructions: &Vec<Instruction<SomeValue>>,
    stack: &mut StackState,
) -> Result<Vec<Instruction<MValue>>, &'a str> {
    let mut resolved: Vec<Instruction<MValue>> = vec![];
    for instruction in instructions {
        resolved.push(typecheck_one(instruction, stack)?);
    }
    return Result::Ok(resolved);
}

fn typecheck_one<'a>(
    instruction: &Instruction<SomeValue>,
    stack: &mut StackState,
) -> Result<Instruction<MValue>, &'a str> {
    match MICHELSON_INSTRUCTIONS.get(&instruction.name) {
        Some(s) => {
            let (mut resolved, args_) = unify_args(&instruction.args, &s.args)?;
            unify_stack(&mut resolved, &s.input_stack, &s.output_stack, stack)?;
            return Result::Ok(Instruction {
                args: args_,
                name: instruction.name.clone(),
            });
        }
        _ => {
            return Result::Err("Instruction not found");
        }
    };
}

pub fn dynm_to_arg_constraint(d: DynMType) -> ArgConstraint {
    match d {
        DMDyn(s) => match ConstraintParser::new().parse(&s) {
            Result::Ok(s) => s,
            Result::Err(_) => panic!("Parsing of ArgConstraint failed!"),
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
    return map_mtype(m, |x| dynm_to_arg_constraint(x.clone()));
}

pub fn mdyn_to_stack_result(m: &MType<DynMType>) -> StackResult {
    return map_mtype(m, |x| dynm_to_stack_result(x.clone()));
}

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

mod tests {
    use crate::parse_concrete;
    use crate::parser::InstructionListParser;
    use crate::typecheck;
    use crate::Instruction;
    use crate::SomeValue;
    use crate::StackState;
    fn typecheck_<'a>(instructions: &Vec<Instruction<SomeValue>>) -> Result<StackState, &'a str> {
        let mut stack = Vec::new();
        typecheck(instructions, &mut stack)?;
        return Result::Ok(stack);
    }
    fn parse(src: &str) -> Vec<Instruction<SomeValue>> {
        let p = InstructionListParser::new();
        match p.parse(src) {
            Ok(s) => s,
            _ => panic!("Parse failed"),
        }
    }

    #[test]
    fn test_type_checking_simple() {
        // Type check behavior.

        assert!(Result::is_err(&typecheck_(&parse("PUSH nat \"asd\""))));
        assert!(Result::is_err(&typecheck_(&parse("PUSH (pair nat nat) 5"))));
        assert!(Result::is_err(&typecheck_(&parse(
            "PUSH (pair nat nat) (Pair 2 3);DROP;DROP"
        ))));
        assert!(Result::is_err(&typecheck_(&parse("PUSH nat 5;ADD"))));

        assert!(Result::is_err(&typecheck_(&parse(
            "LAMBDA nat (pair nat nat) {DUP;PAIR};PUSH int 5;EXEC"
        ))));

        assert!(Result::is_err(&typecheck_(&parse(
            "LAMBDA nat (pair nat nat) {DROP; PUSH int 1; DUP;PAIR};PUSH nat 5;EXEC"
        ))));

        // Stack result tests.

        assert_eq!(
            typecheck_(&parse("PUSH nat 5")).unwrap(),
            parse_concrete("nat")
        );
        assert_eq!(
            typecheck_(&parse("PUSH (pair nat nat) (Pair 2 3)")).unwrap(),
            parse_concrete("pair nat nat")
        );
        assert_eq!(
            typecheck_(&parse("PUSH (pair nat nat) (Pair 2 3);DROP")).unwrap(),
            parse_concrete("")
        );
        assert_eq!(
            typecheck_(&parse("PUSH nat 5; PUSH nat 5;ADD")).unwrap(),
            parse_concrete("nat")
        );

        assert_eq!(
            typecheck_(&parse("PUSH nat 5;DUP;DUP;DUP")).unwrap(),
            parse_concrete("nat;nat;nat;nat")
        );
        assert_eq!(
            typecheck_(&parse("PUSH nat 5;DUP;DROP")).unwrap(),
            parse_concrete("nat")
        );
        assert_eq!(
            typecheck_(&parse("PUSH (list nat) {5;6}")).unwrap(),
            parse_concrete("list nat")
        );
        assert_eq!(
            typecheck_(&parse(
                "LAMBDA nat (pair nat nat) {DUP;PAIR};PUSH nat 5;EXEC"
            ))
            .unwrap(),
            parse_concrete("pair nat nat")
        );
    }
}
