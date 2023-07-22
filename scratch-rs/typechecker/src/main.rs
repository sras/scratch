#[macro_use]
extern crate lazy_static;

use core::fmt::Debug;
use std::collections::HashMap;

use CType::*;
use CTBox::*;
use Constraint as CON;
use Constraint::*;
use StackResult::*;

#[derive(Debug, Clone, Eq, PartialEq)]
enum Concrete {}

type ConcreteType = CType<Concrete>;

#[derive(Debug, Eq, PartialEq)]
enum CType<T> {
    MNat,
    MInt,
    MString,
    MPair(Box<CTBox<T>>, Box<CTBox<T>>),
    MList(Box<CTBox<T>>),
    MLambda(Box<CTBox<T>>, Box<CTBox<T>>),
}

#[derive(Debug, Eq, PartialEq)]
enum CTBox<T> {
    CTOther(T),
    CTSelf(CType<T>),
}

#[derive(Debug, Clone)]
enum ArgValue {
    TypeArg(ConcreteType),
    ValueArg(CType<ArgValue>),
}

fn map_box_aux<T: Clone, H>(aux: &Box<CTBox<T>>, cb: fn(T) -> H) -> Box<CTBox<H>> {
    match aux.as_ref() {
        CTOther(t) => {
            return Box::new(CTOther(cb((*t).clone())));
        }
        CTSelf(c) => Box::new(CTSelf(map_ctype(c, cb))),
    }
}

fn map_ctype<T: Clone, H>(ct: &CType<T>, cb: fn(T) -> H) -> CType<H> {
    match ct {
        MNat => MNat,
        MInt => MInt,
        MString => MString,
        MPair(l, r) => MPair(map_box_aux(l, cb), map_box_aux(r, cb)),
        MLambda(l, r) => MLambda(map_box_aux(l, cb), map_box_aux(r, cb)),
        MList(l) => MList(map_box_aux(l, cb)),
    }
}

impl<T: Clone> Clone for CType<T> {
    fn clone(&self) -> Self {
        return map_ctype(self, |x| x);
    }
}

impl<T: Clone> Clone for CTBox<T> {
    fn clone(&self) -> Self {
        match self {
            CTSelf(a) => CTSelf(a.clone()),
            CTOther(a) => CTOther(a.clone()),
        }
    }
}

struct Instruction<'a> {
    name: &'a str,
    args: Vec<ArgValue>,
}

#[derive(Debug)]
enum Constraint {
    Arg(CType<Constraint>), // An argument that accept a value of a certain type.
    Warg(char),             // An type variable.
    TypeArg(char),          // A argument that accept a type name, like Nat.
    TypeArgRef(char),       // A argument that accept a value of a type referred by
                            // previously encountered TypeArg.
}

impl Clone for Constraint {
    fn clone(&self) -> Self {
        match self {
            CON::Arg(ct) => {
                return CON::Arg(ct.clone());
            }
            CON::Warg(c) => {
                return CON::Warg(c.clone());
            }
            CON::TypeArg(c) => {
                return CON::TypeArg(c.clone());
            }
            CON::TypeArgRef(c) => {
                return CON::TypeArgRef(c.clone());
            }
        }
    }
}

type StackArg = Constraint;

#[derive(Debug, Clone)]
enum StackResult {
    SRCType(CType<StackResult>),
    SRArgRef(char),
}

type StackState = Vec<ConcreteType>;

#[derive(Debug)]
struct InstructionSem {
    args: Vec<Constraint>,
    input_stack: Vec<StackArg>,
    output_stack: Vec<StackResult>,
}

lazy_static! {
    static ref MICHELSON_INSTRUCTIONS: HashMap<&'static str, InstructionSem> = HashMap::from([
        (
            "DROP",
            InstructionSem {
                args: Vec::from([]),
                input_stack: Vec::from([Warg('a')]),
                output_stack: Vec::from([])
            }
        ),
        (
            "CONS",
            InstructionSem {
                args: Vec::from([]),
                input_stack: Vec::from([
                    Warg('a'),
                    Arg(MList(Box::new(CTOther(TypeArgRef('a')))))
                ]),
                output_stack: Vec::from([SRCType(MList(Box::new(CTOther(SRArgRef('a')))))])
            }
        ),
        (
            "PUSH",
            InstructionSem {
                args: Vec::from([TypeArg('a'), CON::TypeArgRef('a')]),
                input_stack: Vec::from([]),
                output_stack: Vec::from([SRArgRef('a')])
            }
        ),
        (
            "PAIR",
            InstructionSem {
                args: Vec::from([]),
                input_stack: Vec::from([Warg('a'), Warg('b')]),
                output_stack: Vec::from([SRCType(MPair(
                    Box::new(CTOther(SRArgRef('a'))),
                    Box::new(CTOther(SRArgRef('b')))
                ))])
            }
        ),
        (
            "LAMBDA",
            InstructionSem {
                args: Vec::from([
                    TypeArg('a'),
                    TypeArg('b'),
                    CON::Arg(MLambda(
                        Box::new(CTOther(CON::TypeArgRef('a'))),
                        Box::new(CTOther(CON::TypeArgRef('b')))
                    ))
                ]),
                input_stack: Vec::from([]),
                output_stack: Vec::from([SRCType(MLambda(
                    Box::new(CTOther(SRArgRef('a'))),
                    Box::new(CTOther(SRArgRef('b')))
                ))])
            }
        )
    ]);
}

fn add_symbol<'a>(
    result: &mut HashMap<char, ConcreteType>,
    arg_con: char,
    type_: ConcreteType,
) {
    result.insert(arg_con, type_);
}

fn concrete_to_arg_constraint(c: ConcreteType) -> Constraint {
    return wrap_ctype(c, |ct| CON::Arg(ct));
}

fn unify_arg_aux<'a>(
    result: &mut HashMap<char, ConcreteType>,
    arg: Box<CTBox<ArgValue>>,
    arg_con: Box<CTBox<Constraint>>,
) -> Result<(), &'a str> {
    let constraint = match arg_con.as_ref() {
        CTOther(c) => c.clone(),
        CTSelf(arg_con_) => CON::Arg(arg_con_.clone()),
    };
    match arg.as_ref() {
        CTSelf(arg_) => {
            return unify_arg(result, ArgValue::ValueArg((*arg_).clone()), constraint);
        }
        CTOther(arg_) => {
            return unify_arg(result, (*arg_).clone(), constraint);
        }
    }
}

fn unify_args<'a>(
    args: Vec<ArgValue>,
    arg_cons: Vec<Constraint>,
) -> Result<HashMap<char, ConcreteType>, &'a str> {
    let mut result = HashMap::from([]);
    for (arg, con) in args.iter().zip(arg_cons.iter()) {
        unify_arg(&mut result, arg.clone(), con.clone())?;
    }
    return Result::Ok(result);
}

fn unify_arg<'a>(
    result: &mut HashMap<char, ConcreteType>,
    arg: ArgValue,
    arg_con: Constraint,
) -> Result<(), &'a str> {
    match arg_con {
        CON::Warg(c) => {
            add_symbol(result, c, arg_value_to_concrete(arg));
            return Result::Ok(());
        }
        CON::TypeArg(c) => match arg {
            ArgValue::TypeArg(ct) => {
                add_symbol(result, c, ct.clone());
                return Result::Ok(());
            }
            _ => return Result::Err("Expecting a type name, but found something else..."),
        },
        CON::TypeArgRef(c) => match result.get(&c) {
            Some(tt) => {
                return unify_arg(result, arg, concrete_to_arg_constraint((*tt).clone()));
            }
            _ => {
                return Result::Err("Unknown type ref");
            }
        },
        CON::Arg(c) => match c {
            MList(ic) => match arg {
                ArgValue::ValueArg(CType::MList(iv)) => {
                    return unify_arg_aux(result, iv, ic);
                }

                _ => {
                    return Result::Err("Expecting a list but got something else...");
                }
            },
            MLambda(vin, vout) => match arg {
                ArgValue::ValueArg(CType::MLambda(cin, cout)) => {
                    return unify_arg_aux(result, cin, vin)
                        .and_then(|_| unify_arg_aux(result, cout, vout));
                }
                _ => {
                    return Result::Err("Expecting a lambda but got something else...");
                }
            },
            MPair(cl, cr) => match arg {
                ArgValue::ValueArg(CType::MPair(vl, vr)) => {
                    return unify_arg_aux(result, vl, cl)
                        .and_then(|_| unify_arg_aux(result, vr, cr));
                }
                _ => {
                    return Result::Err("Expecting a pair but got something else...");
                }
            },
            CType::MNat => match arg {
                ArgValue::ValueArg(CType::MNat) => {
                    return Result::Ok(());
                }
                _ => {
                    return Result::Err("Expecting a `Nat`, but found something else...");
                }
            },
            CType::MInt => match arg {
                ArgValue::ValueArg(CType::MInt) => {
                    return Result::Ok(());
                }
                _ => {
                    return Result::Err("Expecting a `Int`, but found something else...");
                }
            },
            CType::MString => match arg {
                ArgValue::ValueArg(CType::MString) => {
                    return Result::Ok(());
                }
                _ => {
                    return Result::Err("Expecting a `String`, but found something else...");
                }
            },
        },
    }
}

fn stack_result_to_ctype(
    result: &mut HashMap<char, ConcreteType>,
    sr: &StackResult,
) -> ConcreteType {
    match sr {
        SRArgRef(c) => match result.get(&c) {
            Some(ct) => {
                return (*ct).clone();
            }
            None => {
                panic!("Symbol resolution failed! {:?}", c)
            }
        },
        SRCType(ctype) => {
            return mk_ctype(result, ctype);
        }
    }
}

fn mk_ctype(result: &mut HashMap<char, ConcreteType>, ct: &CType<StackResult>) -> ConcreteType {
    match ct {
        MInt => MInt,
        MNat => MNat,
        MString => MString,
        MList(l) => MList(stack_result_aux_to_ctype_aux(result, &l)),
        MPair(l, r) => MPair(
            stack_result_aux_to_ctype_aux(result, &l),
            stack_result_aux_to_ctype_aux(result, &r),
        ),
        MLambda(l, r) => MLambda(
            stack_result_aux_to_ctype_aux(result, &l),
            stack_result_aux_to_ctype_aux(result, &r),
        ),
    }
}

fn stack_result_aux_to_ctype_aux(
    result: &mut HashMap<char, ConcreteType>,
    aux: &CTBox<StackResult>,
) -> Box<CTBox<Concrete>> {
    match aux {
        CTOther(t) => {
            return Box::new(CTSelf(stack_result_to_ctype(result, t)));
        }
        CTSelf(c) => Box::new(CTSelf(mk_ctype(result, c))),
    }
}

fn make_result_stack<'a>(
    result: &mut HashMap<char, ConcreteType>,
    sem_stack_out: Vec<StackResult>,
) -> Result<StackState, &'a str> {
    let mut result_stack: StackState = vec![];
    for i in sem_stack_out {
        result_stack.push(stack_result_to_ctype(result, &i));
    }
    return Result::Ok(result_stack);
}

fn unify_stack<'a>(
    result: &mut HashMap<char, ConcreteType>,
    sem_stack_in: Vec<StackArg>,
    sem_stack_out: Vec<StackResult>,
    stack_state: &mut StackState,
) -> Result<(), &'a str> {
    let mut stack_index: usize = 0;
    let mut s_tail: StackState;
    for constraint in sem_stack_in {
        let stack_elem = stack_state[stack_index].clone();
        unify_arg(
            result,
            ArgValue::ValueArg(coerce_ctype(stack_elem)),
            constraint,
        )?;
        stack_index = stack_index + 1;
    }
    s_tail = stack_state[stack_index..].to_vec();

    let mut rs = make_result_stack(result, sem_stack_out)?;
    rs.append(&mut s_tail);
    *stack_state = rs;
    return Result::Ok(());
}

fn coerce_box_auxct<T>(aux: Box<CTBox<Concrete>>) -> Box<CTBox<T>> {
    match aux.as_ref() {
        CTOther(_) => {
            panic!("Impossible!")
        }
        CTSelf(c) => Box::new(CTSelf(coerce_ctype(c.clone()))),
    }
}

fn coerce_ctype<T>(c: CType<Concrete>) -> CType<T> {
    match c {
        MInt => MInt,
        MNat => MNat,
        MString => MString,
        MPair(l, r) => MPair(coerce_box_auxct(l), coerce_box_auxct(r)),
        MLambda(l, r) => MLambda(coerce_box_auxct(l), coerce_box_auxct(r)),
        MList(l) => MList(coerce_box_auxct(l)),
    }
}

fn argvalue_to_concrete_unsafe(c: &CType<ArgValue>) -> ConcreteType {
    match c {
        MInt => MInt,
        MNat => MNat,
        MString => MString,
        MList(l) => MList(box_aux_argvalue_to_concrete_unsafe(l)),
        MPair(l, r) => MPair(
            box_aux_argvalue_to_concrete_unsafe(l),
            box_aux_argvalue_to_concrete_unsafe(r),
        ),
        MLambda(l, r) => MLambda(
            box_aux_argvalue_to_concrete_unsafe(l),
            box_aux_argvalue_to_concrete_unsafe(r),
        ),
    }
}

fn box_aux_argvalue_to_concrete_unsafe(aux: &Box<CTBox<ArgValue>>) -> Box<CTBox<Concrete>> {
    match aux.as_ref() {
        CTOther(av) => match av {
            ArgValue::ValueArg(ct) => Box::new(CTSelf(argvalue_to_concrete_unsafe(ct))),
            ArgValue::TypeArg(_) => panic!("Type arg unexpected here"),
        },
        CTSelf(c) => Box::new(CTSelf(argvalue_to_concrete_unsafe(c))),
    }
}

fn arg_value_to_concrete(c: ArgValue) -> ConcreteType {
    match c {
        ArgValue::TypeArg(_) => panic!("Unexpected"),
        ArgValue::ValueArg(ct) => argvalue_to_concrete_unsafe(&ct),
    }
}

fn wrap_ctype<T>(ct: ConcreteType, cb: fn(CType<T>) -> T) -> T {
    match ct {
        MNat => cb(MNat),
        MInt => cb(MInt),
        MString => cb(MString),
        MList(l) => cb(MList(coerce_box_auxct(l))),
        MPair(l, r) => cb(MPair(coerce_box_auxct(l), coerce_box_auxct(r))),
        MLambda(l, r) => cb(MLambda(coerce_box_auxct(l), coerce_box_auxct(r))),
    }
}

fn typecheck<'a>(
    instructions: Vec<Instruction<'a>>,
    stack: &mut StackState,
) -> Result<(), &'a str> {
    for instruction in instructions {
        typecheck_one(instruction, stack)?
    }
    return Result::Ok(());
}

fn typecheck_one<'a>(instruction: Instruction<'a>, stack: &mut StackState) -> Result<(), &'a str> {
    match MICHELSON_INSTRUCTIONS.get(instruction.name) {
        Some(s) => {
            let mut result = unify_args(instruction.args, s.args.clone())?;
            unify_stack(
                &mut result,
                s.input_stack.clone(),
                s.output_stack.clone(),
                stack,
            )?;
            return Result::Ok(());
        }
        _ => {
            return Result::Err("Instruction not found");
        }
    };
}

fn main() {
    let instructions: Vec<Instruction> = vec![
        Instruction {
            name: "PUSH",
            args: vec![
                ArgValue::TypeArg(MPair(
                    Box::new(CTSelf(MNat)),
                    Box::new(CTSelf(MPair(
                        Box::new(CTSelf(MNat)),
                        Box::new(CTSelf(MNat)),
                    ))),
                )),
                ArgValue::ValueArg(MPair(
                    Box::new(CTSelf(MNat)),
                    Box::new(CTSelf(MPair(
                        Box::new(CTSelf(MNat)),
                        Box::new(CTSelf(MNat)),
                    ))),
                )),
            ],
        },
        Instruction {
            name: "LAMBDA",
            args: vec![
                ArgValue::TypeArg(MNat),
                ArgValue::TypeArg(MInt),
                ArgValue::ValueArg(MLambda(
                    Box::new(CTSelf(MNat)),
                    Box::new(CTSelf(MInt)),
                )),
            ],
        },
        Instruction {
            name: "PUSH",
            args: vec![
                ArgValue::TypeArg(MPair(
                    Box::new(CTSelf(MNat)),
                    Box::new(CTSelf(MString)),
                )),
                ArgValue::ValueArg(MPair(
                    Box::new(CTSelf(MNat)),
                    Box::new(CTSelf(MString)),
                )),
            ],
        },
        Instruction {
            name: "DROP",
            args: vec![],
        },
        Instruction {
            name: "DROP",
            args: vec![],
        },
        Instruction {
            name: "DROP",
            args: vec![],
        },
        Instruction {
            name: "PUSH",
            args: vec![ArgValue::TypeArg(MNat), ArgValue::ValueArg(MNat)],
        },
        Instruction {
            name: "PUSH",
            args: vec![ArgValue::TypeArg(MInt), ArgValue::ValueArg(MInt)],
        },
        Instruction {
            name: "PAIR",
            args: vec![],
        },
        Instruction {
            name: "PUSH",
            args: vec![
                ArgValue::TypeArg(MList(Box::new(CTSelf(MNat)))),
                ArgValue::ValueArg(MList(Box::new(CTSelf(MNat)))),
            ],
        },
        Instruction {
            name: "PUSH",
            args: vec![ArgValue::TypeArg(MNat), ArgValue::ValueArg(MNat)],
        },
        Instruction {
            name: "CONS",
            args: vec![],
        },
    ];
    let mut stack = Vec::from([]);
    let result = typecheck(instructions, &mut stack);
    println!("{:?} {:?}", result, stack);
}
