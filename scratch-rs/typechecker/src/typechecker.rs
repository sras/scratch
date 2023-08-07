use std::collections::BTreeMap;
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::attributes;
use crate::attributes::check_attribute;
use crate::attributes::check_attributes;
use crate::instructions::MICHELSON_INSTRUCTIONS;
use crate::parsers::parse_contract;
use crate::types::map_mtype;
use crate::types::ArgConstraint::*;
use crate::types::ArgValue as AV;
use crate::types::ArgValue;
use crate::types::AtomicValue::*;
use crate::types::Attribute;
use crate::types::Attribute::*;
use crate::types::CompositeValue::*;
use crate::types::CompoundInstruction;
use crate::types::CompoundInstruction::*;
use crate::types::ConcreteType;
use crate::types::Constraint;
use crate::types::Contract;
use crate::types::Instruction;
use crate::types::MAtomic::*;
use crate::types::MType::*;
use crate::types::MValue;
use crate::types::SomeValue;
use crate::types::StackArg;
use crate::types::StackResult;
use crate::types::StackState;
use crate::types::TcEnv;

use crate::types::MValue::*;
use crate::types::SomeValue::*;
use crate::types::StackResultElem::*;

type ResolveCache = HashMap<char, ConcreteType>;

fn add_symbol<'ka>(resolved: &mut ResolveCache, arg_con: char, type_: &ConcreteType) {
    resolved.insert(arg_con, type_.clone());
}

fn unify_args(
    tcenv: &TcEnv,
    args: &Vec<ArgValue<SomeValue>>,
    arg_cons: &Vec<Constraint>,
) -> Result<(ResolveCache, Vec<ArgValue<MValue>>), String> {
    let mut resolved = HashMap::new();
    let mut args_ = Vec::new();
    for (arg, con) in args.iter().zip(arg_cons.iter()) {
        args_.push(unify_arg(tcenv, &mut resolved, arg, con.clone())?);
    }
    return Result::Ok((resolved, args_));
}

fn unify_concrete_arg<'a>(
    resolved: &mut ResolveCache,
    arg: &ConcreteType,
    arg_con: &Constraint,
) -> Result<(), &'a str> {
    match arg_con {
        MWrapped(CWarg(c, rattr)) => {
            add_symbol(resolved, c.clone(), arg);
            return Result::Ok(());
        }
        MWrapped(CTypeArg(c, rattr)) => {
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
        MTicket(ic) => match arg {
            MTicket(iv) => {
                return unify_concrete_arg(resolved, iv.as_ref(), ic);
            }

            _ => {
                return Result::Err("Expecting a Ticket but got something else...");
            }
        },
        MContract(ic) => match arg {
            MContract(iv) => {
                return unify_concrete_arg(resolved, iv.as_ref(), ic);
            }

            _ => {
                return Result::Err("Expecting a Ticket but got something else...");
            }
        },
        MOption(ic) => match arg {
            MOption(iv) => {
                return unify_concrete_arg(resolved, iv.as_ref(), ic);
            }

            _ => {
                return Result::Err("Expecting a Option but got something else...");
            }
        },
        MSet(ic) => match arg {
            MSet(iv) => {
                return unify_concrete_arg(resolved, iv.as_ref(), ic);
            }

            _ => {
                return Result::Err("Expecting a Option but got something else...");
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
        MOr(b) => match arg {
            MOr(b1) => {
                let (cl, cr) = b.as_ref();
                let (vl, vr) = b1.as_ref();
                return unify_concrete_arg(resolved, vl, cl)
                    .and_then(|_| unify_concrete_arg(resolved, vr, cr));
            }
            _ => {
                return Result::Err("Expecting a pair but got something else...");
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
        MBigMap(b) => match arg {
            MBigMap(b1) => {
                let (cl, cr) = b.as_ref();
                let (vl, vr) = b1.as_ref();
                return unify_concrete_arg(resolved, vl, cl)
                    .and_then(|_| unify_concrete_arg(resolved, vr, cr));
            }
            _ => {
                return Result::Err("Expecting a big_map but got something else...");
            }
        },
        MMap(b) => match arg {
            MMap(b1) => {
                let (cl, cr) = b.as_ref();
                let (vl, vr) = b1.as_ref();
                return unify_concrete_arg(resolved, vl, cl)
                    .and_then(|_| unify_concrete_arg(resolved, vr, cr));
            }
            _ => {
                return Result::Err("Expecting a map but got something else...");
            }
        },
        MWrapped(CAtomic(at)) => match arg {
            MWrapped(cn) => {
                if at == cn {
                    Result::Ok(())
                } else {
                    Result::Err("Expecting an atomic type, but found something else")
                }
            }
            _ => Result::Err("Expecting an atomic type, but found something else"),
        },
    }
}

fn unify_arg(
    tcenv: &TcEnv,
    resolved: &mut ResolveCache,
    arg: &ArgValue<SomeValue>,
    arg_con: &Constraint,
) -> Result<ArgValue<MValue>, String> {
    match arg {
        AV::TypeArg(ct) => match arg_con {
            MWrapped(CTypeArg(c, rattr)) => {
                if check_attributes(rattr, ct) {
                    add_symbol(resolved, *c, &ct);
                    return Result::Ok(AV::TypeArg((*ct).clone()));
                } else {
                    return Result::Err(String::from(
                        "Type does not meet the required constraints",
                    ));
                }
            }
            _ => {
                return Result::Err(String::from("Unexpected type name argument"));
            }
        },
        AV::ValueArg(some_val) => {
            let (m, ct): (MValue, ConcreteType) = match arg_con {
                MWrapped(CTypeArg(_, _)) => {
                    panic!("Unexpected value argument");
                }
                MWrapped(CWarg(_, _)) => {
                    panic!("Unexpected wildcard type encountered");
                }
                MWrapped(CTypeArgRef(ref c)) => match resolved.get(&c) {
                    Some(ct) => typecheck_value(tcenv, resolved, &some_val, ct)?,
                    None => panic!("Symbol resolution failed! {:?}", c),
                },
                _ => match constraint_to_concrete(resolved, &arg_con) {
                    Some(concrete_type) => {
                        typecheck_value(tcenv, resolved, &some_val, &concrete_type)?
                    }
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

fn typecheck_value(
    tcenv: &TcEnv,
    resolved: &ResolveCache,
    some_val: &SomeValue,
    target: &ConcreteType,
) -> Result<(MValue, ConcreteType), String> {
    match (target, some_val) {
        (MWrapped(MBool), Atomic(AVBool(n))) => Ok((VBool(*n), MWrapped(MBool))),
        (MWrapped(MNat), Atomic(AVNumber(n))) => match u32::try_from(*n) {
            Ok(n1) => Ok((VNat(n1), MWrapped(MNat))),
            Err(_) => Err(String::from("Expecting a Nat but found an Int")),
        },
        (MWrapped(MInt), Atomic(AVNumber(n))) => Ok((VInt(*n), MWrapped(MInt))),
        (MWrapped(MString), Atomic(AVString(s))) => Ok((VString(s.clone()), MWrapped(MString))),
        (MList(c), Composite(cv)) => match cv.as_ref() {
            CVList(items) => {
                let mut il: Vec<MValue> = vec![];
                for i in items {
                    let (mv, _) = typecheck_value(tcenv, resolved, i, c.as_ref())?;
                    il.push(mv);
                }
                return Ok((VList(il), MList(c.clone())));
            }
            _ => Err(String::from("Expecting a List but found something else...")),
        },
        (MMap(b), Composite(cv)) => match cv.as_ref() {
            CKVList(items) => {
                let mut hm: BTreeMap<MValue, MValue> = BTreeMap::new();
                let (kt, vt) = b.as_ref();
                if check_attribute(&Comparable, kt) {
                    for (k, v) in items {
                        let (mkv, _) = typecheck_value(tcenv, resolved, k, kt)?;
                        let (mvv, _) = typecheck_value(tcenv, resolved, v, vt)?;
                        hm.insert(mkv, mvv);
                    }
                    return Ok((VMap(Box::new(hm)), MMap(Box::new((kt.clone(), vt.clone())))));
                } else {
                    Err(String::from("Big map keys should be comparable"))
                }
            }
            _ => Err(String::from("Expecting a map but found something else...")),
        },
        (MBigMap(b), Composite(cv)) => match cv.as_ref() {
            CKVList(items) => {
                let mut hm: BTreeMap<MValue, MValue> = BTreeMap::new();
                let (kt, vt) = b.as_ref();
                if check_attribute(&Comparable, kt) {
                    if check_attribute(&BigmapValue, vt) {
                        for (k, v) in items {
                            let (mkv, _) = typecheck_value(tcenv, resolved, k, kt)?;
                            let (mvv, _) = typecheck_value(tcenv, resolved, v, vt)?;
                            hm.insert(mkv, mvv);
                        }
                        return Ok((
                            VBigMap(Box::new(hm)),
                            MBigMap(Box::new((kt.clone(), vt.clone()))),
                        ));
                    } else {
                        Err(String::from("Type not allowed to be a big_map value"))
                    }
                } else {
                    Err(String::from("Big map keys should be comparable"))
                }
            }
            _ => Err(String::from("Expecting a map but found something else...")),
        },
        (MPair(b), Composite(cv)) => match cv.as_ref() {
            CVPair(sv1, sv2) => {
                let (c1, c2) = b.as_ref();
                let (mv1, ct1) = typecheck_value(tcenv, resolved, sv1, c1)?;
                let (mv2, ct2) = typecheck_value(tcenv, resolved, sv2, c2)?;
                return Result::Ok((VPair(Box::new((mv1, mv2))), MPair(Box::new((ct1, ct2)))));
            }
            _ => Err(String::from("Expecting a Pair but found something else...")),
        },
        (MLambda(b), Composite(cv)) => match cv.as_ref() {
            CVLambda(instructions) => {
                let (c1, c2) = b.as_ref();
                let lambda_input = c1.clone();
                let lambda_output = c2.clone();
                let mut stack: StackState = Vec::from([lambda_input.clone()]);
                match typecheck(tcenv, instructions, &mut stack) {
                    Ok(tins) => match stack[..] {
                        [ref real_out] => {
                            if (*real_out) == lambda_output {
                                return Result::Ok((
                                    VLambda(tins),
                                    MLambda(Box::new((lambda_input, lambda_output))),
                                ));
                            } else {
                                return Err(String::from(
                                    "Lambda does not match the expected type",
                                ));
                            }
                        }
                        _ => {
                            return Err(String::from(
                                "Lambda produces more then one element on stack!",
                            ));
                        }
                    },
                    Err(s) => {
                        return Err(s);
                    }
                }
            }
            _ => Err(String::from(
                "Expecting a Lambda but found something else...",
            )),
        },
        _ => Err(String::from("Error type mismatch")),
    }
}

fn stack_result_to_concrete_type(resolved: &mut ResolveCache, sr: &StackResult) -> ConcreteType {
    match sr {
        MWrapped(wrp) => match wrp {
            ElemType(et) => MWrapped(et.clone()),
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
        MContract(l) => MContract(Box::new(stack_result_to_concrete_type(
            resolved,
            l.as_ref(),
        ))),
        MTicket(l) => MTicket(Box::new(stack_result_to_concrete_type(
            resolved,
            l.as_ref(),
        ))),
        MOption(l) => MOption(Box::new(stack_result_to_concrete_type(
            resolved,
            l.as_ref(),
        ))),
        MSet(l) => MSet(Box::new(stack_result_to_concrete_type(
            resolved,
            l.as_ref(),
        ))),
        MMap(b) => {
            let (l, r) = b.as_ref();
            MMap(Box::new((
                stack_result_to_concrete_type(resolved, &l),
                stack_result_to_concrete_type(resolved, &r),
            )))
        }
        MBigMap(b) => {
            let (l, r) = b.as_ref();
            MBigMap(Box::new((
                stack_result_to_concrete_type(resolved, &l),
                stack_result_to_concrete_type(resolved, &r),
            )))
        }
        MOr(b) => {
            let (l, r) = b.as_ref();
            MOr(Box::new((
                stack_result_to_concrete_type(resolved, &l),
                stack_result_to_concrete_type(resolved, &r),
            )))
        }
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

pub fn typecheck_contract<'a>(src: &'a str) -> Result<Contract<MValue>, String> {
    let contract = parse_contract(src);
    let mut stack = vec![MPair(Box::new((
        contract.parameter.clone(),
        contract.storage.clone(),
    )))];
    let tcenv = TcEnv {
        selfType: contract.parameter.clone(),
    };
    let tins = typecheck(&tcenv, &contract.code, &mut stack)?;
    let expected_stack = vec![MPair(Box::new((
        MList(Box::new(MWrapped(MOperation))),
        contract.storage.clone(),
    )))];
    if stack == expected_stack {
        return Result::Ok(Contract {
            parameter: contract.parameter.clone(),
            storage: contract.storage.clone(),
            code: tins,
        });
    } else {
        panic!(
            "Unexpected stack result {:?} while expecting {:>?}",
            stack, expected_stack
        );
    }
}

pub fn typecheck(
    tcenv: &TcEnv,
    instructions: &Vec<CompoundInstruction<SomeValue>>,
    stack: &mut StackState,
) -> Result<Vec<CompoundInstruction<MValue>>, String> {
    let mut resolved: Vec<CompoundInstruction<MValue>> = vec![];
    for instruction in instructions {
        resolved.push(typecheck_one(tcenv, instruction, stack)?);
    }
    return Result::Ok(resolved);
}

fn ensure_stack_atleast<'a>(stack: &StackState, l: usize) -> Result<(), String> {
    if stack.len() >= l {
        return Result::Ok(());
    } else {
        return Result::Err(String::from("Stack too short."));
    }
}

fn ensure_non_empty_stack<'a>(stack: &StackState) -> Result<(), String> {
    ensure_stack_atleast(stack, 1)
}

fn ensure_stack_head<'a>(stack: &mut StackState, t: ConcreteType) -> Result<(),String> {
    ensure_non_empty_stack(stack)?;
    if stack[0] == t {
        return Result::Ok(());
    } else {
        return Result::Err(String::from("Unexpected stack head"));
    }
}
fn ensure_iter_body(
    tcenv: &TcEnv,
    stack: &mut StackState,
    iter_item: &ConcreteType,
    instr: &Vec<CompoundInstruction<SomeValue>>,
) -> Result<Vec<CompoundInstruction<MValue>>, String> {
    let expected_stack = Vec::from(&stack[1..]);
    let mut start_stack: StackState = expected_stack.clone();
    start_stack.insert(0, iter_item.clone());
    let tinst = typecheck(tcenv, instr, &mut start_stack)?;
    if start_stack == expected_stack {
        *stack = start_stack;
        return Result::Ok(tinst);
    } else {
        return Result::Err(String::from("ITER body has unexpected type"));
    }
}

fn ensure_if_cons_body(
    tcenv: &TcEnv,
    stack: &mut StackState,
    (cs, ns): (
        &Vec<CompoundInstruction<SomeValue>>,
        &Vec<CompoundInstruction<SomeValue>>,
    ),
) -> Result<
    (
        Vec<CompoundInstruction<MValue>>,
        Vec<CompoundInstruction<MValue>>,
    ),
    String,
> {
    match stack[0].clone() {
        MList(x) => {
            let mut temp_stack_nil: StackState = Vec::from(&stack[1..]);
            let mut temp_stack_cons: StackState = Vec::from(&stack[1..]);
            temp_stack_cons.insert(0, MList(x.clone()));
            temp_stack_cons.insert(0, x.as_ref().clone());
            let cbtc = typecheck(tcenv, cs, &mut temp_stack_cons)?;
            let nbtc = typecheck(tcenv, ns, &mut temp_stack_nil)?;
            if temp_stack_cons == temp_stack_nil {
                *stack = temp_stack_cons;
                return Result::Ok((cbtc, nbtc));
            } else {
                return Result::Err(String::from("Type of IF_CONS branches differ"));
            }
        }
        m => {
            return Result::Err(format!("IF_CONS requires a list, but found {:?}", m));
        }
    }
}

fn ensure_if_none_body(
    tcenv: &TcEnv,
    stack: &mut StackState,
    (nb, sb): (
        &Vec<CompoundInstruction<SomeValue>>,
        &Vec<CompoundInstruction<SomeValue>>,
    ),
) -> Result<
    (
        Vec<CompoundInstruction<MValue>>,
        Vec<CompoundInstruction<MValue>>,
    ),
    String,
> {
    match stack[0].clone() {
        MOption(x) => {
            let mut temp_stack_none: StackState = Vec::from(&stack[1..]);
            let mut temp_stack_some: StackState = Vec::from(&stack[1..]);
            temp_stack_some.insert(0, x.as_ref().clone());
            let sbtc = typecheck(tcenv, sb, &mut temp_stack_some)?;
            let nbtc = typecheck(tcenv, nb, &mut temp_stack_none)?;
            if temp_stack_some == temp_stack_none {
                *stack = temp_stack_some;
                return Result::Ok((nbtc, sbtc));
            } else {
                return Result::Err(String::from("Type of IF_CONS branches differ"));
            }
        }
        m => {
            return Result::Err(format!("IF_CONS requires a list, but found {:?}", m));
        }
    }
}

fn ensure_same_lambda_type(
    tcenv: &TcEnv,
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
    String,
> {
    let mut temp_stack_t: StackState = Vec::from(&stack[1..]);
    let mut temp_stack_f: StackState = Vec::from(&stack[1..]);
    let tbtc = typecheck(tcenv, tb, &mut temp_stack_t)?;
    let fbtc = typecheck(tcenv, fb, &mut temp_stack_f)?;
    if temp_stack_t == temp_stack_f {
        *stack = temp_stack_t;
        return Result::Ok((tbtc, fbtc));
    } else {
        return Result::Err(String::from("Type of branches differ"));
    }
}

fn typecheck_one(
    tcenv: &TcEnv,
    cinstruction: &CompoundInstruction<SomeValue>,
    stack: &mut StackState,
) -> Result<CompoundInstruction<MValue>, String> {
    match cinstruction {
        Other(instruction) => match MICHELSON_INSTRUCTIONS.get(&instruction.name) {
            Some(variants) => {
                let mut last_error: &str;
                for s in variants {
                    let mut temp_stack = stack.clone();
                    match unify_args(tcenv, &instruction.args, &s.args) {
                        Result::Ok((mut resolved, args_)) => {
                            match unify_stack(
                                &mut resolved,
                                &s.input_stack,
                                &s.output_stack,
                                &mut temp_stack,
                            ) {
                                Result::Ok(_) => {
                                    *stack = temp_stack;
                                    return Result::Ok(Other(Instruction {
                                        args: args_,
                                        name: instruction.name.clone(),
                                    }));
                                }
                                Result::Err(s) => {
                                    continue;
                                }
                            }
                        }
                        Result::Err(s) => {
                            continue;
                        }
                    }
                }
                return Result::Err(format!(
                    "None of the instruction variants matched here for {} with stack {:?}",
                    &instruction.name, &stack
                ));
            }
            None => {
                return Result::Err(format!("Instruction {} not found", &instruction.name));
            }
        },
        SELF => {
            stack.insert(0, MContract(Box::new(tcenv.selfType.clone())));
            return Result::Ok(SELF);
        }
        FAIL => {
            return Result::Ok(FAIL);
        }
        ITER(ins) => {
            ensure_non_empty_stack(stack)?;
            match stack[0].clone() {
                MList(t) => {
                    let tinst = ensure_iter_body(tcenv, stack, t.as_ref(), ins)?;
                    return Result::Ok(ITER(tinst));
                }
                MSet(t) => {
                    let tinst = ensure_iter_body(tcenv, stack, t.as_ref(), ins)?;
                    return Result::Ok(ITER(tinst));
                }
                MMap(t) => {
                    let tinst = ensure_iter_body(tcenv, stack, &MPair(t.clone()), ins)?;
                    return Result::Ok(ITER(tinst));
                }
                m => {
                    return Result::Err(format!(
                        "ITER requires a list, set or map, but found {:?}",
                        m
                    ));
                }
            }
        }
        IF_CONS(tb, fb) => {
            ensure_non_empty_stack(stack)?;
            let (cbtc, nbtc) = ensure_if_cons_body(tcenv, stack, (tb, fb))?;
            return Result::Ok(IF_CONS(cbtc, nbtc));
        }
        IF_NONE(nb, sb) => {
            ensure_non_empty_stack(stack)?;
            let (nbtc, sbtc) = ensure_if_none_body(tcenv, stack, (nb, sb))?;
            return Result::Ok(IF_NONE(nbtc, sbtc));
        }
        IF_SOME(sb, nb) => {
            ensure_non_empty_stack(stack)?;
            let (nbtc, sbtc) = ensure_if_none_body(tcenv, stack, (nb, sb))?;
            return Result::Ok(IF_SOME(sbtc, nbtc));
        }
        IF_LEFT(tb, fb) => {
            panic!()
        }
        IF(tb, fb) => {
            ensure_stack_head(stack, MWrapped(MBool))?;
            let (tbtc, fbtc) = ensure_same_lambda_type(tcenv, stack, (tb, fb))?;
            return Result::Ok(IF(tbtc, fbtc));
        }
        DIP(n, instr) => {
            ensure_stack_atleast(stack, *n)?;
            let mut temp_stack: StackState = Vec::from(&stack[*n..]);
            let tins = typecheck(tcenv, instr, &mut temp_stack)?;
            let mut result_stack = Vec::from(&stack[0..*n]);
            result_stack.append(&mut temp_stack);
            *stack = result_stack;
            return Result::Ok(DIP(*n, tins));
        }
        LAMBDA_REC(it, ot, instr) => {
            let mut temp_stack: StackState =
                vec![it.clone(), MLambda(Box::new((it.clone(), ot.clone())))];
            let tins = typecheck(tcenv, instr, &mut temp_stack)?;
            if temp_stack.len() == 1 {
                if temp_stack[0] == *ot {
                    stack.insert(0, MLambda(Box::new((it.clone(), ot.clone()))));
                    return Result::Ok(LAMBDA_REC(it.clone(), ot.clone(), tins));
                } else {
                    return Result::Err(String::from(
                        "Unexpected output stack for lambda rec lambda",
                    ));
                }
            } else {
                return Result::Err(String::from("Output stack too short"));
            }
        }
    };
}
