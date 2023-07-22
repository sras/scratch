#[macro_use]
extern crate lazy_static;

mod instruction;
mod types;
use types::*;

use std::collections::HashMap;

use ArgValue as AV;
use CTBox::*;
use CType::*;
use Constraint::*;
use StackResult::*;

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

lazy_static! {
    static ref MICHELSON_INSTRUCTIONS: HashMap<&'static str, InstructionDef> = HashMap::from([
        (
            "DROP",
            InstructionDef {
                args: Vec::from([]),
                input_stack: Vec::from([Warg('a')]),
                output_stack: Vec::from([])
            }
        ),
        (
            "ADD",
            InstructionDef {
                args: Vec::from([]),
                input_stack: Vec::from([Warg('a'), TypeArgRef('a')]),
                output_stack: Vec::from([SRArgRef('a')])
            }
        ),
        (
            "CONS",
            InstructionDef {
                args: Vec::from([]),
                input_stack: Vec::from([Warg('a'), Arg(MList(Box::new(CTOther(TypeArgRef('a')))))]),
                output_stack: Vec::from([SRCType(MList(Box::new(CTOther(SRArgRef('a')))))])
            }
        ),
        (
            "PUSH",
            InstructionDef {
                args: Vec::from([TypeArg('a'), TypeArgRef('a')]),
                input_stack: Vec::from([]),
                output_stack: Vec::from([SRArgRef('a')])
            }
        ),
        (
            "PAIR",
            InstructionDef {
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
            InstructionDef {
                args: Vec::from([
                    TypeArg('a'),
                    TypeArg('b'),
                    Arg(MLambda(
                        Box::new(CTOther(TypeArgRef('a'))),
                        Box::new(CTOther(TypeArgRef('b')))
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

fn add_symbol<'a>(result: &mut HashMap<char, ConcreteType>, arg_con: char, type_: ConcreteType) {
    result.insert(arg_con, type_);
}

fn unify_arg_aux<'a>(
    result: &mut HashMap<char, ConcreteType>,
    arg: Box<CTBox<Concrete>>,
    arg_con: Box<CTBox<Constraint>>,
) -> Result<(), &'a str> {
    let constraint = match arg_con.as_ref() {
        CTOther(c) => c.clone(),
        CTSelf(arg_con_) => Arg(arg_con_.clone()),
    };
    match arg.as_ref() {
        CTSelf(arg_) => {
            return unify_arg(result, ArgValue::ValueArg((*arg_).clone()), constraint);
        }
        _ => panic!("Impossible!"),
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

fn unify_concrete_arg<'a>(
    result: &mut HashMap<char, ConcreteType>,
    arg: ConcreteType,
    arg_con: Constraint,
) -> Result<(), &'a str> {
    match arg_con {
        Warg(c) => {
            add_symbol(result, c, arg);
            return Result::Ok(());
        }
        TypeArg(c) => {
            add_symbol(result, c, arg.clone());
            return Result::Ok(());
        }
        TypeArgRef(c) => match result.get(&c) {
            Some(tt) => {
                return unify_concrete_arg(result, arg, Arg(coerce_ctype((*tt).clone())));
            }
            _ => {
                return Result::Err("Unknown type ref");
            }
        },
        Arg(c) => match c {
            MList(ic) => match arg {
                MList(iv) => {
                    return unify_arg_aux(result, iv, ic);
                }

                _ => {
                    return Result::Err("Expecting a list but got something else...");
                }
            },
            MLambda(vin, vout) => match arg {
                MLambda(cin, cout) => {
                    return unify_arg_aux(result, cin, vin)
                        .and_then(|_| unify_arg_aux(result, cout, vout));
                }
                _ => {
                    return Result::Err("Expecting a lambda but got something else...");
                }
            },
            MPair(cl, cr) => match arg {
                MPair(vl, vr) => {
                    return unify_arg_aux(result, vl, cl)
                        .and_then(|_| unify_arg_aux(result, vr, cr));
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
    result: &mut HashMap<char, ConcreteType>,
    arg: ArgValue,
    arg_con: Constraint,
) -> Result<(), &'a str> {
    match arg {
        AV::TypeArg(ct) => match arg_con {
            TypeArg(c) => {
                add_symbol(result, c, ct.clone());
                return Result::Ok(());
            }
            _ => {
                panic!("Unexpected type name argument");
            }
        },
        AV::ValueArg(ct) => {
            return unify_concrete_arg(result, ct, arg_con);
        }
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
    Box::new(match aux {
        CTOther(t) => CTSelf(stack_result_to_ctype(result, t)),
        CTSelf(c) => CTSelf(mk_ctype(result, c)),
    })
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
    for constraint in sem_stack_in {
        let stack_elem = stack_state[stack_index].clone();
        unify_concrete_arg(result, coerce_ctype(stack_elem), constraint)?;
        stack_index = stack_index + 1;
    }
    let mut rs = make_result_stack(result, sem_stack_out)?;
    rs.append(&mut stack_state[stack_index..].to_vec());
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
                ArgValue::ValueArg(MLambda(Box::new(CTSelf(MNat)), Box::new(CTSelf(MInt)))),
            ],
        },
        Instruction {
            name: "PUSH",
            args: vec![
                ArgValue::TypeArg(MPair(Box::new(CTSelf(MNat)), Box::new(CTSelf(MString)))),
                ArgValue::ValueArg(MPair(Box::new(CTSelf(MNat)), Box::new(CTSelf(MString)))),
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
        Instruction {
            name: "PUSH",
            args: vec![ArgValue::TypeArg(MNat), ArgValue::ValueArg(MNat)],
        },
        Instruction {
            name: "PUSH",
            args: vec![ArgValue::TypeArg(MInt), ArgValue::ValueArg(MInt)],
        },
        Instruction {
            name: "ADD",
            args: vec![],
        },
    ];
    let mut stack = Vec::from([]);
    let result = typecheck(instructions, &mut stack);
    println!("{:?} {:?}", result, stack);
    //println!("{:?}", instruction::MTypeParser::new().parse("pair int nat"));
}
