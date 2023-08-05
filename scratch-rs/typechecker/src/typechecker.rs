use std::collections::HashMap;
use std::convert::TryFrom;

use crate::instructions::MICHELSON_INSTRUCTIONS;
use crate::map_mtype;
use crate::ArgConstraint::*;
use crate::ArgValue as AV;
use crate::ArgValue;
use crate::AtomicValue::*;
use crate::CompositeValue::*;
use crate::CompoundInstruction;
use crate::CompoundInstruction::*;
use crate::ConcreteType;
use crate::Constraint;
use crate::Instruction;
use crate::MAtomic::*;
use crate::MType::*;
use crate::MValue;
use crate::SomeValue;
use crate::StackArg;
use crate::StackResult;
use crate::StackState;

use crate::MValue::*;
use crate::SomeValue::*;
use crate::StackResultElem::*;

type ResolveCache = HashMap<char, ConcreteType>;

fn add_symbol<'ka>(resolved: &mut ResolveCache, arg_con: char, type_: &ConcreteType) {
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
                return unify_concrete_arg(resolved, arg, &map_mtype(tt, &|x| CAtomic(x.clone())));
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
        MLambda(b) => match arg {
            MLambda(b1) => {
                let (vin, vout) = b.as_ref();
                let (cin, cout) = b1.as_ref();
                return unify_concrete_arg(resolved, cin, vin)
                    .and_then(|_| unify_concrete_arg(resolved, cout, vout));
            }
            _ => {
                return Result::Err("Expecting a lambda but got something else...");
            }
        },
        MPair(b) => match arg {
            MPair(b1) => {
                let (cl, cr) = b.as_ref();
                let (vl, vr) = b1.as_ref();
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
        MWrapped(CAtomic(MBool)) => match arg {
            MWrapped(MBool) => {
                return Result::Ok(());
            }
            _ => {
                return Result::Err("Expecting a `Bool`, but found something else...");
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
        MPair(b) => {
            let (l, r) = b.as_ref();
            Some(MPair(Box::new((
                constraint_to_concrete(resolved, l)?,
                constraint_to_concrete(resolved, r)?,
            ))))
        }
        MList(l) => Some(MList(Box::new(constraint_to_concrete(resolved, l)?))),
        MLambda(b) => {
            let (l, r) = b.as_ref();
            Some(MLambda(Box::new((
                constraint_to_concrete(resolved, l)?,
                constraint_to_concrete(resolved, r)?,
            ))))
        }
        _ => None,
    }
}

fn typecheck_value<'a>(
    resolved: &ResolveCache,
    some_val: &SomeValue,
    target: &ConcreteType,
) -> Result<(MValue, ConcreteType), &'a str> {
    match (target, some_val) {
        (MWrapped(MBool), Atomic(AVBool(n))) => Ok((VBool(*n), MWrapped(MBool))),
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
        (MPair(b), Composite(cv)) => match cv.as_ref() {
            CVPair(sv1, sv2) => {
                let (c1, c2) = b.as_ref();
                let (mv1, ct1) = typecheck_value(resolved, sv1, c1)?;
                let (mv2, ct2) = typecheck_value(resolved, sv2, c2)?;
                return Result::Ok((VPair(Box::new((mv1, mv2))), MPair(Box::new((ct1, ct2)))));
            }
            _ => Err("Expecting a Pair but found something else..."),
        },
        (MLambda(b), Composite(cv)) => match cv.as_ref() {
            CVLambda(instructions) => {
                let (c1, c2) = b.as_ref();
                let lambda_input = c1.clone();
                let lambda_output = c2.clone();
                let mut stack: StackState = Vec::from([lambda_input.clone()]);
                match typecheck(instructions, &mut stack) {
                    Ok(tins) => match stack[..] {
                        [ref real_out] => {
                            if (*real_out) == lambda_output {
                                return Result::Ok((
                                    VLambda(tins),
                                    MLambda(Box::new((lambda_input, lambda_output))),
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
                MBool => MWrapped(MBool),
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
        MPair(b) => {
            let (l, r) = b.as_ref();
            MPair(Box::new((
                stack_result_to_concrete_type(resolved, &l),
                stack_result_to_concrete_type(resolved, &r),
            )))
        }
        MLambda(b) => {
            let (l, r) = b.as_ref();
            MLambda(Box::new((
                stack_result_to_concrete_type(resolved, &l),
                stack_result_to_concrete_type(resolved, &r),
            )))
        }
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

pub fn typecheck<'a>(
    instructions: &Vec<CompoundInstruction<SomeValue>>,
    stack: &mut StackState,
) -> Result<Vec<CompoundInstruction<MValue>>, &'a str> {
    let mut resolved: Vec<CompoundInstruction<MValue>> = vec![];
    for instruction in instructions {
        resolved.push(typecheck_one(instruction, stack)?);
    }
    return Result::Ok(resolved);
}

fn ensure_non_empty_stack<'a>(stack: &StackState) -> Result<(), &'a str> {
    if stack.len() > 0 {
        return Result::Ok(());
    } else {
        return Result::Err("Stack too short.");
    }
}

fn ensure_stack_head<'a>(stack: &StackState, t: ConcreteType) -> Result<(), &'a str> {
    ensure_non_empty_stack(stack)?;
    if stack[0] == t {
        return Result::Ok(());
    } else {
        return Result::Err("Unexpected stack head");
    }
}

fn ensure_same_instr_type<'a>(
    stack: &mut StackState,
    (tb, fb): (
        &Vec<CompoundInstruction<SomeValue>>,
        &Vec<CompoundInstruction<SomeValue>>,
    ),
) -> Result<
    (
        Vec<CompoundInstruction<MValue>>,
        Vec<CompoundInstruction<MValue>>,
    ),
    &'a str,
> {
    let mut temp_stack_t: StackState = Vec::from(&stack[1..]);
    let mut temp_stack_f: StackState = Vec::from(&stack[1..]);
    let tbtc = typecheck(tb, &mut temp_stack_t)?;
    let fbtc = typecheck(fb, &mut temp_stack_f)?;
    if temp_stack_t == temp_stack_f {
        *stack = temp_stack_t;
        return Result::Ok((tbtc, fbtc));
    } else {
        return Result::Err("Type of branches differ");
    }
}

fn typecheck_one<'a>(
    cinstruction: &CompoundInstruction<SomeValue>,
    stack: &mut StackState,
) -> Result<CompoundInstruction<MValue>, &'a str> {
    match cinstruction {
        Other(instruction) => match MICHELSON_INSTRUCTIONS.get(&instruction.name) {
            Some(s) => {
                let (mut resolved, args_) = unify_args(&instruction.args, &s.args)?;
                unify_stack(&mut resolved, &s.input_stack, &s.output_stack, stack)?;
                return Result::Ok(Other(Instruction {
                    args: args_,
                    name: instruction.name.clone(),
                }));
            }
            _ => {
                return Result::Err("Instruction not found");
            }
        },
        IF(tb, fb) => {
            ensure_stack_head(stack, MWrapped(MBool))?;
            let (tbtc, fbtc) = ensure_same_instr_type(stack, (tb, fb))?;
            return Result::Ok(IF(tbtc, fbtc));
        }
        DIP(instr) => {
            ensure_non_empty_stack(stack)?;
            let mut temp_stack: StackState = Vec::from(&stack[1..]);
            let tins = typecheck(instr, &mut temp_stack)?;
            temp_stack.insert(0, stack[0].clone());
            *stack = temp_stack;
            return Result::Ok(DIP(tins));
        }
        LAMBDA_REC(it, ot, instr) => {
            let mut temp_stack: StackState =
                vec![it.clone(), MLambda(Box::new((it.clone(), ot.clone())))];
            let tins = typecheck(instr, &mut temp_stack)?;
            if temp_stack.len() == 1 {
                if temp_stack[0] == *ot {
                    stack.insert(0, MLambda(Box::new((it.clone(), ot.clone()))));
                    return Result::Ok(LAMBDA_REC(it.clone(), ot.clone(), tins));
                } else {
                    return Result::Err("Unexpected output stack for lambda rec lambda");
                }
            } else {
                return Result::Err("Output stack too short");
            }
        }
    };
}
