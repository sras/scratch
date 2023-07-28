#[macro_use]
extern crate lazy_static;

mod parser;
mod types;
use types::*;

use std::collections::HashMap;
use std::convert::TryFrom;

use ArgConstraint::*;
use ArgValue as AV;
use AtomicValue::*;
use MAtomic::*;
use CompositeValue::*;
use StackResultElem::*;
use MType::*;
use MValue::*;
use SomeValue::*;

type ResolveCache = HashMap<char, ConcreteType>;

impl<T: Clone> Clone for MType<T> {
    fn clone(&self) -> Self {
        return map_mtype(self, |x| x.clone());
    }
}

fn map_mtype<T: Clone, H>(ct: &MType<T>, cb: fn(&T) -> H) -> MType<H> {
    match ct {
        MPair(l, r) => MPair(Box::new(map_mtype(l, cb)), Box::new(map_mtype(r, cb))),
        MLambda(l, r) => MLambda(Box::new(map_mtype(l, cb)), Box::new(map_mtype(r, cb))),
        MList(l) => MList(Box::new(map_mtype(l, cb))),
        MWrapped(w) => MWrapped(cb(w)),
    }
}

lazy_static! {
    static ref MICHELSON_INSTRUCTIONS: HashMap<String, InstructionDef> = HashMap::from([
        (
            String::from("DUP"),
            InstructionDef {
                args: Vec::new(),
                input_stack: Vec::from([MWrapped(Warg('a'))]),
                output_stack: Vec::from([MWrapped(TRef('a')), MWrapped(TRef('a'))])
            }
        ),
        (
            String::from("DROP"),
            InstructionDef {
                args: Vec::new(),
                input_stack: Vec::from([MWrapped(Warg('a'))]),
                output_stack: Vec::new()
            }
        ),
        (
            String::from("ADD"),
            InstructionDef {
                args: Vec::new(),
                input_stack: Vec::from([MWrapped(Warg('a')), MWrapped(TypeArgRef('a'))]),
                output_stack: Vec::from([MWrapped(TRef('a'))])
            }
        ),
        (
            String::from("CONS"),
            InstructionDef {
                args: Vec::new(),
                input_stack: Vec::from([
                    MWrapped(Warg('a')),
                    MList(Box::new(MWrapped(TypeArgRef('a'))))
                ]),
                output_stack: Vec::from([MList(Box::new(MWrapped(TRef('a'))))])
            }
        ),
        (
            String::from("PUSH"),
            InstructionDef {
                args: Vec::from([MWrapped(TypeArg('a')), MWrapped(TypeArgRef('a'))]),
                input_stack: Vec::new(),
                output_stack: Vec::from([MWrapped(TRef('a'))])
            }
        ),
        (
            String::from("PAIR"),
            InstructionDef {
                args: Vec::new(),
                input_stack: Vec::from([MWrapped(Warg('a')), MWrapped(Warg('b'))]),
                output_stack: Vec::from([MPair(Box::new(MWrapped(TRef('a'))), Box::new(MWrapped(TRef('b'))))])
            }
        ),
        (
            String::from("LAMBDA"),
            InstructionDef {
                args: Vec::from([
                    MWrapped(TypeArg('a')),
                    MWrapped(TypeArg('b')),
                    MLambda(
                        Box::new(MWrapped(TypeArgRef('a'))),
                        Box::new(MWrapped(TypeArgRef('b')))
                    )
                ]),
                input_stack: Vec::new(),
                output_stack: Vec::from([MLambda(
                    Box::new(MWrapped(TRef('a'))),
                    Box::new(MWrapped(TRef('b')))
                )])
            }
        ),
        (
            String::from("EXEC"),
            InstructionDef {
                args: Vec::new(),
                input_stack: Vec::from([
                    MWrapped(Warg('a')),
                    MLambda(
                        Box::new(MWrapped(TypeArgRef('a'))),
                        Box::new(MWrapped(Warg('b')))
                    )
                ]),
                output_stack: Vec::from([MWrapped(TRef('b'))])
            }
        )
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
        MWrapped(Warg(c)) => {
            add_symbol(resolved, c.clone(), arg);
            return Result::Ok(());
        }
        MWrapped(TypeArg(c)) => {
            add_symbol(resolved, c.clone(), arg);
            return Result::Ok(());
        }
        MWrapped(TypeArgRef(c)) => match resolved.get(&c) {
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
            MWrapped(TypeArg(c)) => {
                add_symbol(resolved, *c, &ct);
                return Result::Ok(AV::TypeArg((*ct).clone()));
            }
            _ => {
                panic!("Unexpected type name argument");
            }
        },
        AV::ValueArg(some_val) => {
            let (m, ct): (MValue, ConcreteType) = match arg_con {
                MWrapped(TypeArg(_)) => {
                    panic!("Unexpected value argument");
                }
                MWrapped(Warg(_)) => {
                    panic!("Unexpected wildcard type encountered");
                }
                MWrapped(TypeArgRef(ref c)) => match resolved.get(&c) {
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
        MWrapped(TypeArgRef(c)) => match resolved.get(&c) {
            Some(ct) => Some(ct.clone()),
            None => None,
        },
        MWrapped(CAtomic(MInt)) => Some(MWrapped(MInt)),
        MWrapped(CAtomic(MNat)) => Some(MWrapped(MNat)),
        MWrapped(CAtomic(MString)) => Some(MWrapped(MString)),
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
        MWrapped(ElemType(MInt)) => MWrapped(MInt),
        MWrapped(ElemType(MNat)) => MWrapped(MNat),
        MWrapped(ElemType(MString)) => MWrapped(MString),
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
        MWrapped(TRef(c)) => match resolved.get(&c) {
            Some(ct) => {
                return (*ct).clone();
            }
            None => {
                panic!("Symbol resolution failed! {:?}", c)
            }
        },
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
        assert!(Result::is_ok(&typecheck_(&parse("PUSH nat 5"))));
        assert!(Result::is_ok(&typecheck_(&parse(
            "PUSH (pair nat nat) (Pair 2 3)"
        ))));
        assert!(Result::is_ok(&typecheck_(&parse(
            "PUSH (pair nat nat) (Pair 2 3);DROP"
        ))));
        assert!(Result::is_ok(&typecheck_(&parse(
            "PUSH nat 5; PUSH nat 5;ADD"
        ))));

        assert!(Result::is_err(&typecheck_(&parse("PUSH nat \"5\""))));
        assert!(Result::is_err(&typecheck_(&parse("PUSH (pair nat nat) 5"))));
        assert!(Result::is_err(&typecheck_(&parse(
            "PUSH (pair nat nat) (Pair 2 3);DROP;DROP"
        ))));
        assert!(Result::is_err(&typecheck_(&parse("PUSH nat 5;ADD"))));

        assert!(Result::is_err(&typecheck_(&parse(
            "LAMBDA nat (pair nat nat) {DUP;PAIR};PUSH int 5;EXEC"
        ))));

        // Stack result tests.
        assert_eq!(
            typecheck_(&parse("PUSH nat 5; PUSH nat 5;ADD"))
                .unwrap()
                .len(),
            1
        );
        assert_eq!(typecheck_(&parse("PUSH nat 5")).unwrap().len(), 1);
        assert_eq!(
            typecheck_(&parse("PUSH nat 5;DUP;DUP;DUP")).unwrap().len(),
            4
        );
        assert_eq!(typecheck_(&parse("PUSH nat 5;DUP;DROP")).unwrap().len(), 1);
        assert_eq!(
            typecheck_(&parse("PUSH (list nat) {5;6}")).unwrap().len(),
            1
        );
        assert_eq!(
            typecheck_(&parse(
                "LAMBDA nat (pair nat nat) {DUP;PAIR};PUSH nat 5;EXEC"
            ))
            .unwrap()
            .len(),
            1
        );
    }
}
