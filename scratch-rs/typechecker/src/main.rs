#[macro_use]
extern crate lazy_static;

mod instruction;
mod types;
use types::*;

use std::collections::HashMap;
use std::convert::TryFrom;

use ArgValue as AV;
use AtomicValue::*;
use CompositeValue::*;
use Constraint::*;
use MNesting::*;
use MType::*;
use MValue::*;
use SomeValue::*;
use StackResult::*;

impl<T: Clone> Clone for MType<T> {
    fn clone(&self) -> Self {
        return map_mtype(self, |x| x);
    }
}

impl<T: Clone> Clone for MNesting<T> {
    fn clone(&self) -> Self {
        match self {
            Nested(a) => Nested(a.clone()),
            Other(a) => Other(a.clone()),
        }
    }
}

fn map_nesting<T: Clone, H>(nesting: &Box<MNesting<T>>, cb: fn(T) -> H) -> Box<MNesting<H>> {
    match nesting.as_ref() {
        Other(t) => {
            return Box::new(Other(cb((*t).clone())));
        }
        Nested(c) => Box::new(Nested(map_mtype(c, cb))),
    }
}

fn map_mtype<T: Clone, H>(ct: &MType<T>, cb: fn(T) -> H) -> MType<H> {
    match ct {
        MNat => MNat,
        MInt => MInt,
        MString => MString,
        MPair(l, r) => MPair(map_nesting(l, cb), map_nesting(r, cb)),
        MLambda(l, r) => MLambda(map_nesting(l, cb), map_nesting(r, cb)),
        MList(l) => MList(map_nesting(l, cb)),
    }
}

lazy_static! {
    static ref MICHELSON_INSTRUCTIONS: HashMap<String, InstructionDef> = HashMap::from([
        (
            String::from("DUP"),
            InstructionDef {
                args: Vec::from([]),
                input_stack: Vec::from([Warg('a')]),
                output_stack: Vec::from([SRArgRef('a'), SRArgRef('a')])
            }
        ),
        (
            String::from("DROP"),
            InstructionDef {
                args: Vec::from([]),
                input_stack: Vec::from([Warg('a')]),
                output_stack: Vec::from([])
            }
        ),
        (
            String::from("ADD"),
            InstructionDef {
                args: Vec::from([]),
                input_stack: Vec::from([Warg('a'), TypeArgRef('a')]),
                output_stack: Vec::from([SRArgRef('a')])
            }
        ),
        (
            String::from("CONS"),
            InstructionDef {
                args: Vec::from([]),
                input_stack: Vec::from([Warg('a'), Arg(MList(Box::new(Other(TypeArgRef('a')))))]),
                output_stack: Vec::from([SRMType(MList(Box::new(Other(SRArgRef('a')))))])
            }
        ),
        (
            String::from("PUSH"),
            InstructionDef {
                args: Vec::from([TypeArg('a'), TypeArgRef('a')]),
                input_stack: Vec::from([]),
                output_stack: Vec::from([SRArgRef('a')])
            }
        ),
        (
            String::from("PAIR"),
            InstructionDef {
                args: Vec::from([]),
                input_stack: Vec::from([Warg('a'), Warg('b')]),
                output_stack: Vec::from([SRMType(MPair(
                    Box::new(Other(SRArgRef('a'))),
                    Box::new(Other(SRArgRef('b')))
                ))])
            }
        ),
        (
            String::from("LAMBDA"),
            InstructionDef {
                args: Vec::from([
                    TypeArg('a'),
                    TypeArg('b'),
                    Arg(MLambda(
                        Box::new(Other(TypeArgRef('a'))),
                        Box::new(Other(TypeArgRef('b')))
                    ))
                ]),
                input_stack: Vec::from([]),
                output_stack: Vec::from([SRMType(MLambda(
                    Box::new(Other(SRArgRef('a'))),
                    Box::new(Other(SRArgRef('b')))
                ))])
            }
        )
    ]);
}

fn add_symbol<'a>(resolved: &mut HashMap<char, ConcreteType>, arg_con: char, type_: &ConcreteType) {
    resolved.insert(arg_con, type_.clone());
}

fn unify_arg_nested<'a>(
    resolved: &mut HashMap<char, ConcreteType>,
    arg: &Box<MNesting<Concrete>>,
    arg_con: &Box<MNesting<Constraint>>,
) -> Result<(), &'a str> {
    let constraint = match arg_con.as_ref() {
        Other(c) => c.clone(),
        Nested(arg_con_) => Arg(arg_con_.clone()),
    };
    match arg.as_ref() {
        Nested(arg_) => {
            return unify_concrete_arg(resolved, arg_, &constraint);
        }
        _ => panic!("Impossible!"),
    }
}

fn unify_args<'a>(
    args: &Vec<ArgValue<SomeValue>>,
    arg_cons: &Vec<Constraint>,
) -> Result<(HashMap<char, ConcreteType>, Vec<ArgValue<MValue>>), &'a str> {
    let mut resolved = HashMap::from([]);
    let mut args_ = vec![];
    for (arg, con) in args.iter().zip(arg_cons.iter()) {
        args_.push(unify_arg(&mut resolved, arg.clone(), con.clone())?);
    }
    return Result::Ok((resolved, args_));
}

fn unify_concrete_arg<'a>(
    resolved: &mut HashMap<char, ConcreteType>,
    arg: &ConcreteType,
    arg_con: &Constraint,
) -> Result<(), &'a str> {
    match arg_con {
        Warg(c) => {
            add_symbol(resolved, c.clone(), arg);
            return Result::Ok(());
        }
        TypeArg(c) => {
            add_symbol(resolved, c.clone(), arg);
            return Result::Ok(());
        }
        TypeArgRef(c) => match resolved.get(&c) {
            Some(tt) => {
                return unify_concrete_arg(resolved, arg, &Arg(coerce_concrete(tt)));
            }
            _ => {
                return Result::Err("Unknown type ref");
            }
        },
        Arg(c) => match c {
            MList(ic) => match arg {
                MList(iv) => {
                    return unify_arg_nested(resolved, iv, ic);
                }

                _ => {
                    return Result::Err("Expecting a list but got something else...");
                }
            },
            MLambda(vin, vout) => match arg {
                MLambda(cin, cout) => {
                    return unify_arg_nested(resolved, cin, vin)
                        .and_then(|_| unify_arg_nested(resolved, cout, vout));
                }
                _ => {
                    return Result::Err("Expecting a lambda but got something else...");
                }
            },
            MPair(cl, cr) => match arg {
                MPair(vl, vr) => {
                    return unify_arg_nested(resolved, vl, cl)
                        .and_then(|_| unify_arg_nested(resolved, vr, cr));
                }
                _ => {
                    return Result::Err("Expecting a pair but got something else...");
                }
            },
            MNat => match arg {
                MNat => {
                    return Result::Ok(());
                }
                _ => {
                    return Result::Err("Expecting a `Nat`, but found something else...");
                }
            },
            MInt => match arg {
                MInt => {
                    return Result::Ok(());
                }
                _ => {
                    return Result::Err("Expecting a `Int`, but found something else...");
                }
            },
            MString => match arg {
                MString => {
                    return Result::Ok(());
                }
                _ => {
                    return Result::Err("Expecting a `String`, but found something else...");
                }
            },
        },
    }
}

fn unify_arg<'a>(
    resolved: &mut HashMap<char, ConcreteType>,
    arg: ArgValue<SomeValue>,
    arg_con: Constraint,
) -> Result<ArgValue<MValue>, &'a str> {
    match arg {
        AV::TypeArg(ct) => match arg_con {
            TypeArg(c) => {
                add_symbol(resolved, c, &ct);
                return Result::Ok(AV::TypeArg(ct));
            }
            _ => {
                panic!("Unexpected type name argument");
            }
        },
        AV::ValueArg(some_val) => {
            let (m, ct): (MValue, ConcreteType) = match arg_con {
                TypeArg(_) => {
                    panic!("Unexpected value argument");
                }
                Warg(_) => {
                    panic!("Unexpected wildcard type encountered");
                }
                TypeArgRef(ref c) => match resolved.get(&c) {
                    Some(ct) => type_check_value(resolved, &some_val, ct)?,
                    None => panic!("Symbol resolution failed! {:?}", c),
                },
                Arg(_) => match constraint_to_concrete(resolved, &arg_con) {
                    Some(concrete_type) => type_check_value(resolved, &some_val, &concrete_type)?,
                    None => panic!("Couldnt resolve type"),
                },
            };
            unify_concrete_arg(resolved, &ct, &arg_con)?;
            return Ok(AV::ValueArg(m));
        }
    }
}

fn constraint_to_concrete(
    resolved: &HashMap<char, ConcreteType>,
    c: &Constraint,
) -> Option<ConcreteType> {
    match c {
        Arg(ctc) => match ctc {
            MInt => Some(MInt),
            MNat => Some(MNat),
            MString => Some(MString),
            MPair(l, r) => Some(MPair(
                constrain_to_concrete_nested(resolved, l)?,
                constrain_to_concrete_nested(resolved, r)?,
            )),
            MList(l) => Some(MList(constrain_to_concrete_nested(resolved, l)?)),
            MLambda(l, r) => Some(MLambda(
                constrain_to_concrete_nested(resolved, l)?,
                constrain_to_concrete_nested(resolved, r)?,
            )),
        },
        TypeArgRef(c) => match resolved.get(&c) {
            Some(ct) => Some(ct.clone()),
            None => None,
        },
        _ => None,
    }
}

fn constrain_to_concrete_nested(
    resolved: &HashMap<char, ConcreteType>,
    c: &MNesting<Constraint>,
) -> Option<Box<MNesting<Concrete>>> {
    match c {
        Other(c) => match constraint_to_concrete(resolved, c) {
            Some(x) => Some(Box::new(Nested(x))),
            None => None,
        },
        Nested(c) => match constraint_to_concrete(resolved, &Arg(c.clone())) {
            Some(x) => Some(Box::new(Nested(x))),
            None => None,
        },
    }
}

fn typecheck_value_<'a>(
    resolved: &HashMap<char, ConcreteType>,
    some_val: &SomeValue,
    target_box: Box<MNesting<Concrete>>,
) -> Result<(MValue, ConcreteType), &'a str> {
    match target_box.as_ref() {
        Nested(ctype) => type_check_value(resolved, some_val, ctype),
        _ => panic!("Impossible"),
    }
}

fn unwrap_ctbox(c: &MNesting<Concrete>) -> &ConcreteType {
    match c {
        Other(_) => panic!("Impossible!"),
        Nested(x) => return x,
    }
}

fn type_check_value<'a>(
    resolved: &HashMap<char, ConcreteType>,
    some_val: &SomeValue,
    target: &ConcreteType,
) -> Result<(MValue, ConcreteType), &'a str> {
    match (target, some_val) {
        (MNat, Atomic(AVNumber(n))) => match u32::try_from(*n) {
            Ok(n1) => Ok((VNat(n1), MNat)),
            Err(_) => Err("Expecting a Nat but found an Int"),
        },
        (MInt, Atomic(AVNumber(n))) => Ok((VInt(*n), MInt)),
        (MString, Atomic(AVString(s))) => Ok((VString(s.clone()), MString)),
        (MList(c), Composite(cv)) => match cv.as_ref() {
            CVList(items) => {
                let mut il: Vec<MValue> = vec![];
                for i in items {
                    let (mv, _) = typecheck_value_(resolved, i, c.clone())?;
                    il.push(mv);
                }
                return Ok((VList(il), MList(c.clone())));
            }
            _ => Err("Expecting a List but found something else..."),
        },
        (MPair(c1, c2), Composite(cv)) => match cv.as_ref() {
            CVPair(sv1, sv2) => {
                let (mv1, ct1) = typecheck_value_(resolved, sv1, c1.clone())?;
                let (mv2, ct2) = typecheck_value_(resolved, sv2, c2.clone())?;
                return Result::Ok((
                    VPair(Box::new(mv1), Box::new(mv2)),
                    MPair(Box::new(Nested(ct1)), Box::new(Nested(ct2))),
                ));
            }
            _ => Err("Expecting a Pair but found something else..."),
        },
        (MLambda(c1, c2), Composite(cv)) => match cv.as_ref() {
            CVLambda(instructions) => {
                let lambda_input = unwrap_ctbox(c1);
                let lambda_output = unwrap_ctbox(c2);
                let mut stack: StackState = Vec::from([lambda_input.clone()]);
                match typecheck(instructions, &mut stack) {
                    Ok(tins) => match stack[..] {
                        [ref real_out] => {
                            if (*real_out) == (*lambda_output) {
                                return Result::Ok((
                                    VLambda(tins),
                                    MLambda(
                                        Box::new(Nested(lambda_input.clone())),
                                        Box::new(Nested(lambda_output.clone())),
                                    ),
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

fn stack_result_to_concrete_type(
    resolved: &mut HashMap<char, ConcreteType>,
    sr: &StackResult,
) -> ConcreteType {
    match sr {
        SRArgRef(c) => match resolved.get(&c) {
            Some(ct) => {
                return (*ct).clone();
            }
            None => {
                panic!("Symbol resolution failed! {:?}", c)
            }
        },
        SRMType(ctype) => {
            return stack_result_to_concrete_type_(resolved, ctype);
        }
    }
}

fn stack_result_to_concrete_type_(
    resolved: &mut HashMap<char, ConcreteType>,
    ct: &MType<StackResult>,
) -> ConcreteType {
    match ct {
        MInt => MInt,
        MNat => MNat,
        MString => MString,
        MList(l) => MList(stack_result_to_concrete_nested(resolved, &l)),
        MPair(l, r) => MPair(
            stack_result_to_concrete_nested(resolved, &l),
            stack_result_to_concrete_nested(resolved, &r),
        ),
        MLambda(l, r) => MLambda(
            stack_result_to_concrete_nested(resolved, &l),
            stack_result_to_concrete_nested(resolved, &r),
        ),
    }
}

fn stack_result_to_concrete_nested(
    resolved: &mut HashMap<char, ConcreteType>,
    nesting: &MNesting<StackResult>,
) -> Box<MNesting<Concrete>> {
    Box::new(match nesting {
        Other(t) => Nested(stack_result_to_concrete_type(resolved, t)),
        Nested(c) => Nested(stack_result_to_concrete_type_(resolved, c)),
    })
}

fn make_resolved_stack<'a>(
    resolved: &mut HashMap<char, ConcreteType>,
    sem_stack_out: &Vec<StackResult>,
) -> Result<StackState, &'a str> {
    let mut resolved_stack: StackState = vec![];
    for i in sem_stack_out {
        resolved_stack.push(stack_result_to_concrete_type(resolved, &i));
    }
    return Result::Ok(resolved_stack);
}

fn unify_stack<'a>(
    resolved: &mut HashMap<char, ConcreteType>,
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
        unify_concrete_arg(resolved, &coerce_concrete(stack_elem), &constraint)?;
        stack_index = stack_index + 1;
    }
    let mut rs = make_resolved_stack(resolved, sem_stack_out)?;
    rs.append(&mut stack_state[stack_index..].to_vec());
    *stack_state = rs;
    return Result::Ok(());
}

fn coerce_nested<T>(nested: &Box<MNesting<Concrete>>) -> Box<MNesting<T>> {
    Box::new(Nested(coerce_concrete(unwrap_ctbox(nested))))
}

fn coerce_concrete<T>(c: &MType<Concrete>) -> MType<T> {
    match c {
        MInt => MInt,
        MNat => MNat,
        MString => MString,
        MPair(l, r) => MPair(coerce_nested(l), coerce_nested(r)),
        MLambda(l, r) => MLambda(coerce_nested(l), coerce_nested(r)),
        MList(l) => MList(coerce_nested(l)),
    }
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
    let mut stack = Vec::from([]);
    match instruction::InstructionListParser::new().parse(
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
    use crate::instruction::InstructionListParser;
    use crate::typecheck;
    use crate::Instruction;
    use crate::SomeValue;
    use crate::StackState;
    fn typecheck_<'a>(instructions: &Vec<Instruction<SomeValue>>) -> Result<StackState, &'a str> {
        let mut stack = Vec::from([]);
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

        // Stack result tests.
        assert_eq!(
            typecheck_(&parse("PUSH nat 5; PUSH nat 5;ADD"))
                .unwrap()
                .len(),
            1
        );
        assert_eq!(typecheck_(&parse("PUSH nat 5")).unwrap().len(), 1);
    }
}
