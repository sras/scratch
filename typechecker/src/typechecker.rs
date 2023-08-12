use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::time::{Duration, Instant};

use crate::attributes;
use crate::attributes::check_attribute;
use crate::attributes::check_attributes;
use crate::get_stack_derived;
//use crate::get_stack_derived_2;
use crate::instructions::MICHELSON_INSTRUCTIONS;
use crate::parsers::parse_contract;
use crate::types::get_n_pair;
use crate::types::map_mtype;
use crate::types::mk_pair;
use crate::types::unmk_pair;
use crate::types::update_n_pair;
use crate::types::ArgConstraint::*;
use crate::types::ArgValue as AV;
use crate::types::ArgValue;
use crate::types::AtomicValue::*;
use crate::types::Attribute;
use crate::types::Attribute::*;
use crate::types::CompositeValue::*;
use crate::types::CompoundInstruction;
use crate::types::CompoundInstruction::*;
use crate::types::ConcreteStack;
use crate::types::ConcreteType;
use crate::types::Constraint;
use crate::types::Contract;
use crate::types::Instruction;
use crate::types::MAtomic;
use crate::types::MAtomic::*;
use crate::types::MType;
use crate::types::MType::*;
use crate::types::MValue;
use crate::types::SeqItem::*;
use crate::types::SomeValue;
use crate::types::StackArg;
use crate::types::StackCompResult::*;
use crate::types::StackDerived;
use crate::types::StackDerived::*;
use crate::types::StackResult;
use crate::types::StackState;
use crate::types::StackState::*;
use crate::types::TcEnv;

use crate::types::MValue::*;
use crate::types::SomeValue::*;
use crate::types::StackResultElem::*;

type ResolveCache = BTreeMap<char, ConcreteType>;

fn add_symbol<'ka>(resolved: &mut ResolveCache, arg_con: char, type_: &ConcreteType) {
    resolved.insert(arg_con, type_.clone());
}

fn unify_args(
    tcenv: &TcEnv,
    args: &Vec<ArgValue<SomeValue>>,
    arg_cons: &Vec<Constraint>,
) -> Result<(ResolveCache, Vec<ArgValue<MValue>>), String> {
    let mut resolved = BTreeMap::new();
    let mut args_ = Vec::new();
    for (arg, con) in args.iter().zip(arg_cons.iter()) {
        args_.push(unify_arg(tcenv, &mut resolved, arg, con.clone())?);
    }
    return Result::Ok((resolved, args_));
}

fn unify_concrete_arg(
    resolved: &mut ResolveCache,
    arg: &ConcreteType,
    arg_con: &Constraint,
) -> Result<(), String> {
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
                return Result::Err(String::from("Unknown type ref"));
            }
        },
        MList(ic) => match arg {
            MList(iv) => {
                return unify_concrete_arg(resolved, iv.as_ref(), ic);
            }

            _ => {
                return Result::Err(String::from("Expecting a list but got something else..."));
            }
        },
        MTicket(ic) => match arg {
            MTicket(iv) => {
                return unify_concrete_arg(resolved, iv.as_ref(), ic);
            }

            x => {
                return Result::Err(format!("Expecting a Ticket but got something else.{:?}", x));
            }
        },
        MContract(ic) => match arg {
            MContract(iv) => {
                return unify_concrete_arg(resolved, iv.as_ref(), ic);
            }

            c => {
                return Result::Err(format!(
                    "Expecting a Contract but got something else {:?}",
                    c
                ))
            }
        },
        MOption(ic) => match arg {
            MOption(iv) => {
                return unify_concrete_arg(resolved, iv.as_ref(), ic);
            }

            x => {
                return Result::Err(format!(
                    "Expecting a Option but got something else: {:?}",
                    x
                ));
            }
        },
        MSet(ic) => match arg {
            MSet(iv) => {
                return unify_concrete_arg(resolved, iv.as_ref(), ic);
            }

            _ => {
                return Result::Err(String::from("Expecting a Option but got something else..."));
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
                return Result::Err(String::from("Expecting a lambda but got something else..."));
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
                return Result::Err(String::from("Expecting a pair but got something else..."));
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
                return Result::Err(String::from("Expecting a pair but got something else..."));
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
                return Result::Err(String::from(
                    "Expecting a big_map but got something else...",
                ));
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
                return Result::Err(String::from("Expecting a map but got something else..."));
            }
        },
        MWrapped(CAtomic(at)) => match arg {
            MWrapped(cn) => {
                if at == cn {
                    Result::Ok(())
                } else {
                    Result::Err(format!(
                        "Expecting type {:?}, but found something else: {:?}",
                        at, cn
                    ))
                }
            }
            _ => Result::Err(String::from(
                "Expecting an atomic type, but found something else",
            )),
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
        (MWrapped(MUnit), Atomic(AVUnit)) => Ok((VUnit, MWrapped(MUnit))),
        (MWrapped(MBool), Atomic(AVBool(n))) => Ok((VBool(*n), MWrapped(MBool))),
        (MWrapped(MMutez), Atomic(AVNumber(n))) => match u32::try_from(*n) {
            Ok(n1) => Ok((VMutez(n1), MWrapped(MMutez))),
            Err(_) => Err(String::from("Expecting a Nat but found an Int")),
        },
        (MWrapped(MNat), Atomic(AVNumber(n))) => match u32::try_from(*n) {
            Ok(n1) => Ok((VNat(n1), MWrapped(MNat))),
            Err(_) => Err(String::from("Expecting a Nat but found an Int")),
        },
        (MWrapped(MInt), Atomic(AVNumber(n))) => Ok((VInt(*n), MWrapped(MInt))),
        (MWrapped(MString), Atomic(AVString(s))) => Ok((VString(s.clone()), MWrapped(MString))),
        (MSet(c), Composite(cv)) => match cv.as_ref() {
            CVSeq(SqValue(items)) => {
                let mut il: BTreeSet<MValue> = BTreeSet::new();
                for i in items {
                    let (mv, _) = typecheck_value(tcenv, resolved, i, c.as_ref())?;
                    il.insert(mv);
                }
                return Ok((VSet(il), MSet(c.clone())));
            }
            _ => Err(String::from(
                "Expecting a Sequence but found something else...",
            )),
        },
        (MList(c), Composite(cv)) => match cv.as_ref() {
            CVSeq(items) => {
                if items.len() == 0 {
                    return Ok((VList(vec![]), MList(c.clone())));
                } else {
                    match items {
                        SqValue(items) => {
                            let mut il: Vec<MValue> = vec![];
                            for i in items {
                                let (mv, _) = typecheck_value(tcenv, resolved, i, c.as_ref())?;
                                il.push(mv);
                            }
                            return Ok((VList(il), MList(c.clone())));
                        }
                        SqInstr(items) => Err(String::from(
                            "Expecting a list of values but found instructions..",
                        )),
                    }
                }
            }
            _ => Err(String::from("Expecting a List but found something else...")),
        },
        (MMap(b), Composite(cv)) => match cv.as_ref() {
            CVSeq(x) => {
                if x.len() == 0 {
                    let (kt, vt) = b.as_ref();
                    return Ok((
                        VMap(Box::new(BTreeMap::new())),
                        MMap(Box::new((kt.clone(), vt.clone()))),
                    ));
                } else {
                    return Err(String::from(
                        "Expecting a key/value list but found value list...",
                    ));
                }
            }
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
        (MOr(b), Composite(cv)) => {
            let (c1, c2) = b.as_ref();
            match cv.as_ref() {
                CVLeft(sv1) => {
                    let (mv1, ct1) = typecheck_value(tcenv, resolved, sv1, c1)?;
                    return Result::Ok((VLeft(Box::new(mv1)), MOr(b.clone())));
                }
                CVRight(sv1) => {
                    let (mv1, ct1) = typecheck_value(tcenv, resolved, sv1, c2)?;
                    return Result::Ok((VRight(Box::new(mv1)), MOr(b.clone())));
                }
                _ => Err(String::from(
                    "Expecting a Left/Right value but found something else...",
                )),
            }
        }
        (MLambda(b), Composite(cv)) => match cv.as_ref() {
            CVSeq(SqInstr(instructions)) => {
                let (c1, c2) = b.as_ref();
                let lambda_input = c1.clone();
                let lambda_output = c2.clone();
                let mut stack: ConcreteStack = StackState::from(vec![lambda_input.clone()]);
                match typecheck(tcenv, instructions, &mut stack) {
                    Ok(tins) => {
                        if stack.compare_singleton(&lambda_output) {
                            return Result::Ok((
                                VLambda(tins),
                                MLambda(Box::new((lambda_input, lambda_output))),
                            ));
                        } else {
                            return Err(String::from("Lambda does not match the expected type"));
                        }
                    }
                    Err(s) => {
                        return Err(s);
                    }
                }
            }
            _ => Err(String::from(
                "Expecting a Lambda but found something else...",
            )),
        },

        (MOption(b), Composite(cv)) => match cv.as_ref() {
            CVSome(v) => {
                let (tv, vt) = typecheck_value(tcenv, resolved, v, b.as_ref())?;
                return Result::Ok((VSome(Box::new(tv)), MOption(Box::new(vt))));
            }
            x => Err(format!(
                "Expecting an Option value but found something else: {:?}",
                x
            )),
        },
        (x, y) => Err(format!("Error type mismatch {:?} {:?}", x, y)),
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
) -> Result<VecDeque<ConcreteType>, &'a str> {
    let mut resolved_stack: VecDeque<ConcreteType> = VecDeque::new();
    for i in sem_stack_out {
        resolved_stack.push_front(stack_result_to_concrete_type(resolved, &i));
    }
    return Result::Ok(resolved_stack);
}

fn unify_stack(
    resolved: &mut ResolveCache,
    sem_stack_in: &Vec<StackArg>,
    sem_stack_out: &Vec<StackResult>,
    stack_state: &mut StackState<MAtomic>,
) -> Result<(), String> {
    match stack_state.len() {
        SdFailed => {
            return Result::Ok(());
        }
        SdOk(sslen) => {
            if sslen < sem_stack_in.len() {
                return Result::Err(format!(
                    "Stack was found too small for the operation: needed {}, found {}",
                    sem_stack_in.len(),
                    sslen
                ));
            }
            for constraint in sem_stack_in {
                match stack_state.pop_front() {
                    SdOk(Some(stack_elem)) => {
                        unify_concrete_arg(resolved, &stack_elem, &constraint)?;
                    }
                    SdOk(None) => {
                        return Result::Err("Too few values in stack".to_string());
                    }
                    SdFailed => {
                        return Result::Ok(());
                    }
                }
            }

            for i in sem_stack_out.iter().rev() {
                stack_state.push_front(stack_result_to_concrete_type(resolved, &i));
            }

            return Result::Ok(());
        }
    }
}

pub fn typecheck_contract(contract: Contract<SomeValue>) -> Result<Contract<MValue>, String> {
    let mut stack = StackState::from(vec![MPair(Box::new((
        contract.parameter.clone(),
        contract.storage.clone(),
    )))]);
    let tcenv = TcEnv {
        selfType: contract.parameter.clone(),
    };
    let tins = typecheck(&tcenv, &contract.code, &mut stack)?;
    let expected_stack_elem = MPair(Box::new((
        MList(Box::new(MWrapped(MOperation))),
        contract.storage.clone(),
    )));
    if stack.compare_singleton(&expected_stack_elem) {
        return Result::Ok(Contract {
            parameter: contract.parameter.clone(),
            storage: contract.storage.clone(),
            code: tins,
        });
    } else {
        panic!(
            "Unexpected stack result {:?} while expecting {:>?}",
            stack, expected_stack_elem
        );
    }
}

pub fn typecheck(
    tcenv: &TcEnv,
    instructions: &Vec<CompoundInstruction<SomeValue>>,
    stack: &mut StackState<MAtomic>,
) -> Result<Vec<CompoundInstruction<MValue>>, String> {
    let mut resolved: Vec<CompoundInstruction<MValue>> = Vec::with_capacity(instructions.len());
    for instruction in instructions {
        //println!("Instruction: {:?}", instruction);
        //println!("Stack in: {:?}", stack);
        resolved.push(typecheck_one(tcenv, instruction, stack)?);
        //println!("Stack out: {:?}", stack);
    }
    return Result::Ok(resolved);
}

fn ensure_stack_atleast<'a>(stack: &Vec<ConcreteType>, l: usize) -> Result<(), String> {
    if stack.len() >= l {
        return Result::Ok(());
    } else {
        return Result::Err(format!("Stack too short, req {} got {}", l, stack.len()));
    }
}

fn ensure_non_empty_stack<'a>(stack: &Vec<ConcreteType>) -> Result<(), String> {
    ensure_stack_atleast(stack, 1)
}

fn ensure_stack_head<'a>(stack: &Vec<ConcreteType>, t: ConcreteType) -> Result<(), String> {
    ensure_non_empty_stack(stack)?;
    if stack[0] == t {
        return Result::Ok(());
    } else {
        return Result::Err(String::from("Unexpected stack head"));
    }
}
fn ensure_iter_body(
    tcenv: &TcEnv,
    stack: &mut StackState<MAtomic>,
    m_iter_item: Option<&ConcreteType>,
    instr: &Vec<CompoundInstruction<SomeValue>>,
) -> Result<Vec<CompoundInstruction<MValue>>, String> {
    match m_iter_item {
        None => {
            let tinst = typecheck(tcenv, instr, &mut FailedStack)?;
            stack.fail();
            return Result::Ok(tinst);
        }
        Some(iter_item) => {
            let expected_stack = stack.clone_tail();
            let mut start_stack: ConcreteStack = expected_stack.clone();
            start_stack.push(iter_item.clone());
            let tinst = typecheck(tcenv, instr, &mut start_stack)?;
            match start_stack.compare(&expected_stack) {
                NoMatch => {
                    return Result::Err(String::from("ITER body has unexpected type"));
                }
                _ => {
                    *stack = start_stack;
                    return Result::Ok(tinst);
                }
            }
        }
    }
}
fn ensure_map_body<F: (Fn(ConcreteType) -> ConcreteType)>(
    tcenv: &TcEnv,
    stack: &mut StackState<MAtomic>,
    iter_item_info: StackDerived<(&ConcreteType, F)>,
    instr: &Vec<CompoundInstruction<SomeValue>>,
) -> Result<Vec<CompoundInstruction<MValue>>, String> {
    match iter_item_info {
        SdFailed => {
            let tinst = typecheck(tcenv, instr, &mut FailedStack)?;
            return Result::Ok(tinst);
        }
        SdOk((iter_item, to_result)) => {
            let expected_stack = stack.clone_tail();
            let mut start_stack: ConcreteStack = expected_stack.clone();
            start_stack.push(iter_item.clone());
            let tinst = typecheck(tcenv, instr, &mut start_stack)?;
            match start_stack.pop_front() {
                SdFailed => {
                    *stack = start_stack;
                    return Result::Ok(tinst);
                }
                SdOk(None) => {
                    return Result::Err("Map body returned too few values on stack.".to_string())
                }
                SdOk(Some(start_stack_head)) => match start_stack.compare(&expected_stack) {
                    NoMatch => {
                        return Result::Err(String::from(
                            "MAP body cannot mutated the tail of the stack.",
                        ));
                    }
                    _ => {
                        start_stack.push(to_result(start_stack_head));
                        *stack = start_stack;
                        return Result::Ok(tinst);
                    }
                },
            }
        }
    }
}
fn ensure_loop_body(
    tcenv: &TcEnv,
    stack: &mut StackState<MAtomic>,
    instr: &Vec<CompoundInstruction<SomeValue>>,
) -> Result<Vec<CompoundInstruction<MValue>>, String> {
    let expected_stack = stack.clone_tail();
    let mut start_stack: ConcreteStack = expected_stack.clone();
    start_stack.push(MWrapped(MBool));
    let tinst = typecheck(tcenv, instr, &mut start_stack)?;
    match start_stack.compare(&expected_stack) {
        NoMatch => {
            return Result::Err(String::from("LOOP body has unexpected type"));
        }
        _ => {
            *stack = expected_stack;
            return Result::Ok(tinst);
        }
    }
}

fn ensure_loop_left_body(
    tcenv: &TcEnv,
    stack: &mut StackState<MAtomic>,
    mleft: StackDerived<MType<MAtomic>>,
    mright: StackDerived<MType<MAtomic>>,
    instr: &Vec<CompoundInstruction<SomeValue>>,
) -> Result<Vec<CompoundInstruction<MValue>>, String> {
    match (mleft, mright) {
        (SdOk(left), SdOk(right)) => {
            let mut expected_stack = stack.clone_tail();
            let mut start_stack: ConcreteStack = expected_stack.clone();
            start_stack.push(left.clone());
            expected_stack.push(MOr(Box::new((left.clone(), right.clone()))));
            let tinst = typecheck(tcenv, instr, &mut start_stack)?;
            match start_stack.compare(&expected_stack) {
                NoMatch => {
                    return Result::Err(String::from("LOOP_LEFT body has unexpected type"));
                }
                _ => {
                    expected_stack.pop();
                    expected_stack.push(right.clone());
                    *stack = expected_stack;
                    return Result::Ok(tinst);
                }
            }
        }
        _ => {
            let tinst = typecheck(tcenv, instr, stack)?;
            return Result::Ok(tinst);
        }
    }
}

fn ensure_if_cons_body(
    tcenv: &TcEnv,
    stack_: &mut StackState<MAtomic>,
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
    match get_stack_derived!(stack_.get_index(0), (vec![FAIL], vec![FAIL])) {
        None => {
            return Result::Err("Empty stack found for IF_CONS body".to_string());
        }
        Some(stack_head) => match stack_head.clone() {
            MList(x) => {
                let mut temp_stack_nil: ConcreteStack = stack_.clone_tail();
                let mut temp_stack_cons: ConcreteStack = stack_.clone_tail();
                temp_stack_cons.push(MList(x.clone()));
                temp_stack_cons.push(x.as_ref().clone());
                let cbtc = typecheck(tcenv, cs, &mut temp_stack_cons)?;
                let nbtc = typecheck(tcenv, ns, &mut temp_stack_nil)?;
                match temp_stack_cons.compare(&temp_stack_nil) {
                    NoMatch => {
                        return Result::Err(format!(
                            "Type of IF_CONS branches differ {:?} {:?}",
                            temp_stack_cons, temp_stack_nil
                        ));
                    }
                    RightFailed => {
                        *stack_ = temp_stack_cons;
                        return Result::Ok((cbtc, nbtc));
                    }
                    LeftFailed => {
                        *stack_ = temp_stack_nil;
                        return Result::Ok((cbtc, nbtc));
                    }
                    Match => {
                        *stack_ = temp_stack_cons;
                        return Result::Ok((cbtc, nbtc));
                    }
                    BothFailed => {
                        stack_.fail();
                        return Result::Ok((vec![FAIL], vec![FAIL]));
                    }
                }
            }
            m => {
                return Result::Err(format!("IF_CONS requires a list, but found {:?}", m));
            }
        },
    }
}

fn ensure_if_left_body(
    tcenv: &TcEnv,
    stack_: &mut ConcreteStack,
    (lb, rb): (
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
    match get_stack_derived!(stack_.get_index(0), (vec![FAIL], vec![FAIL])) {
        None => {
            return Result::Err("Expecting a bool on stack top, but found empty stack".to_string());
        }
        Some(stack_head) => match stack_head.clone() {
            MOr(b) => {
                let mut temp_stack_left: ConcreteStack = stack_.clone_tail();
                let mut temp_stack_right: ConcreteStack = stack_.clone_tail();
                let (lt, rt) = b.as_ref();
                temp_stack_left.push(lt.clone());
                temp_stack_right.push(rt.clone());
                let lbtc = typecheck(tcenv, lb, &mut temp_stack_left)?;
                let rbtc = typecheck(tcenv, rb, &mut temp_stack_right)?;

                match temp_stack_left.compare(&temp_stack_right) {
                    RightFailed => {
                        *stack_ = temp_stack_left;
                        return Result::Ok((lbtc, rbtc));
                    }
                    LeftFailed => {
                        *stack_ = temp_stack_right;
                        return Result::Ok((lbtc, rbtc));
                    }
                    Match => {
                        *stack_ = temp_stack_right;
                        return Result::Ok((lbtc, rbtc));
                    }
                    NoMatch => {
                        return Result::Err(format!(
                            "Type of IF_LEFT branches differ {:?} {:?}",
                            temp_stack_left, temp_stack_right
                        ));
                    }
                    BothFailed => {
                        stack_.fail();
                        return Result::Ok((vec![FAIL], vec![FAIL]));
                    }
                }
            }
            m => {
                return Result::Err(format!(
                    "IF_LEFT requires an or, but found {:?}, {:?} {:?}",
                    m, lb, rb
                ));
            }
        },
    }
}

fn ensure_if_none_body(
    tcenv: &TcEnv,
    stack_: &mut ConcreteStack,
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
    match get_stack_derived!(stack_.get_index(0), (vec![FAIL], vec![FAIL])) {
        None => return Result::Err("Empty stack found for IF_NONE body!".to_string()),
        Some(stack_head) => match stack_head.clone() {
            MOption(x) => {
                let mut temp_stack_none: ConcreteStack = stack_.clone_tail();
                let mut temp_stack_some: ConcreteStack = stack_.clone_tail();
                temp_stack_some.push(x.as_ref().clone());
                let sbtc = typecheck(tcenv, sb, &mut temp_stack_some)?;
                let nbtc = typecheck(tcenv, nb, &mut temp_stack_none)?;
                match temp_stack_some.compare(&temp_stack_none) {
                    LeftFailed => {
                        *stack_ = temp_stack_none;
                        return Result::Ok((nbtc, sbtc));
                    }
                    RightFailed => {
                        *stack_ = temp_stack_some;
                        return Result::Ok((nbtc, sbtc));
                    }
                    Match => {
                        *stack_ = temp_stack_some;
                        return Result::Ok((nbtc, sbtc));
                    }
                    NoMatch => {
                        return Result::Err(String::from("Type of IF_NONE branches differ"));
                    }
                    BothFailed => {
                        stack_.fail();
                        return Result::Ok((vec![FAIL], vec![FAIL]));
                    }
                }
            }
            m => {
                return Result::Err(format!(
                    "IF_NONE requires an option, but found {:?}, {:?} {:?}",
                    m, sb, nb
                ));
            }
        },
    }
}

fn ensure_same_lambda_type(
    tcenv: &TcEnv,
    stack_: &mut ConcreteStack,
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
    let mut temp_stack_t: ConcreteStack = stack_.clone_tail();
    let mut temp_stack_f: ConcreteStack = stack_.clone_tail();
    let tbtc = typecheck(tcenv, tb, &mut temp_stack_t)?;
    let fbtc = typecheck(tcenv, fb, &mut temp_stack_f)?;

    match temp_stack_t.compare(&temp_stack_f) {
        RightFailed => {
            *stack_ = temp_stack_t;
            return Result::Ok((tbtc, fbtc));
        }
        LeftFailed => {
            *stack_ = temp_stack_f;
            return Result::Ok((tbtc, fbtc));
        }
        Match => {
            *stack_ = temp_stack_f;
            return Result::Ok((tbtc, fbtc));
        }
        NoMatch => {
            return Result::Err(String::from("Type of branches differ"));
        }
        BothFailed => {
            stack_.fail();
            return Result::Ok((vec![FAIL], vec![FAIL]));
        }
    }
}

fn typecheck_one(
    tcenv: &TcEnv,
    cinstruction: &CompoundInstruction<SomeValue>,
    stack: &mut ConcreteStack,
) -> Result<CompoundInstruction<MValue>, String> {
    match cinstruction {
        Other(instruction) => match MICHELSON_INSTRUCTIONS.get(&instruction.name) {
            Some(variants) => {
                let mut last_error: &str;
                let mut errors: String = String::new();
                for s in variants {
                    match unify_args(tcenv, &instruction.args, &s.args) {
                        Result::Ok((mut resolved, args_)) => {
                            let r = if variants.len() > 1 {
                                let mut temp_stack = stack.clone();
                                match unify_stack(
                                    &mut resolved,
                                    &s.input_stack,
                                    &s.output_stack,
                                    &mut temp_stack,
                                ) {
                                    Result::Ok(_) => {
                                        *stack = temp_stack;
                                        Result::Ok(())
                                    }
                                    e => e,
                                }
                            } else {
                                unify_stack(&mut resolved, &s.input_stack, &s.output_stack, stack)
                            };
                            match r {
                                Result::Ok(_) => {
                                    return Result::Ok(Other(Instruction {
                                        location: instruction.location.clone(),
                                        args: args_,
                                        name: instruction.name.clone(),
                                    }));
                                }
                                Result::Err(s) => {
                                    errors = format!("{};\n {}", errors, s);
                                    continue;
                                }
                            }
                        }
                        Result::Err(s) => {
                            errors = format!("{};\n {}", errors, s);
                            continue;
                        }
                    }
                }
                return Result::Err(format!(
                    "{}; None of the instruction variants matched here for {:?} with stack {:?}",
                    errors, &instruction, &stack
                ));
            }
            None => {
                return Result::Err(format!("Instruction {} not found", &instruction.name));
            }
        },
        SELF => {
            stack.push(MContract(Box::new(tcenv.selfType.clone())));
            return Result::Ok(SELF);
        }
        FAIL => {
            stack.fail();
            return Result::Ok(FAIL);
        }
        FAILWITH => {
            stack.fail();
            return Result::Ok(FAIL);
        }
        LOOP(ins) => match stack.get_index(0) {
            SdOk(Some(MWrapped(MBool))) => {
                let tinst = ensure_loop_body(tcenv, stack, ins)?;
                return Result::Ok(LOOP(tinst));
            }
            SdOk(Some(m)) => {
                return Result::Err(format!("LOOP requires a bool, but found {:?}", m));
            }
            SdOk(None) => {
                return Result::Err(format!("Stack can't be empty for LOOP"));
            }
            SdFailed => {
                let tinst = ensure_loop_body(tcenv, stack, ins)?;
                return Result::Ok(LOOP(tinst));
            }
        },
        LOOP_LEFT(ins) => match stack.get_index(0) {
            SdOk(Some(stack_head)) => match stack_head {
                MOr(b) => {
                    let (l, r) = b.as_ref();
                    let tinst =
                        ensure_loop_left_body(tcenv, stack, SdOk(l.clone()), SdOk(r.clone()), ins)?;
                    return Result::Ok(LOOP_LEFT(tinst));
                }
                m => {
                    return Result::Err(format!("LOOP_LEFT requires an or, but found {:?}", m));
                }
            },
            SdOk(None) => {
                return Result::Err(format!("LOOP_LEFT stack cannot be empty!"));
            }

            SdFailed => {
                let tinst = ensure_loop_left_body(tcenv, stack, SdFailed, SdFailed, ins)?;
                return Result::Ok(LOOP_LEFT(tinst));
            }
        },
        MAP(ins) => match stack.get_index(0) {
            SdOk(Some(stack_head)) => match stack_head.clone() {
                MList(t) => {
                    let tinst = ensure_map_body(
                        tcenv,
                        stack,
                        SdOk((&(t.clone()), |x| MList(Box::new(x)))),
                        ins,
                    )?;
                    return Result::Ok(MAP(tinst));
                }
                MOption(t) => {
                    let tinst = ensure_map_body(
                        tcenv,
                        stack,
                        SdOk((&(t.clone()), |x| MOption(Box::new(x)))),
                        ins,
                    )?;
                    return Result::Ok(MAP(tinst));
                }
                MMap(t) => {
                    let (k, v) = t.as_ref();
                    let tinst = ensure_map_body(
                        tcenv,
                        stack,
                        SdOk((&MPair(t.clone()), |x| MMap(Box::new((k.clone(), x))))),
                        ins,
                    )?;
                    return Result::Ok(MAP(tinst));
                }
                m => {
                    return Result::Err(format!(
                        "Map requires a list, option or map, but found {:?}",
                        m
                    ));
                }
            },
            SdOk(None) => {
                return Result::Err(format!("Map cannot work with Empty list"));
            }
            SdFailed => {
                return Result::Ok(FAIL);
            }
        },
        ITER(ins) => match get_stack_derived!(stack.get_index(0), FAIL) {
            None => {
                let tinst = ensure_iter_body(tcenv, stack, None, ins)?;
                return Result::Ok(ITER(tinst));
            }
            Some(stack_head) => match stack_head {
                MList(t) => {
                    let tinst = ensure_iter_body(tcenv, stack, Some(&(t.clone())), ins)?;
                    return Result::Ok(ITER(tinst));
                }
                MSet(t) => {
                    let tinst = ensure_iter_body(tcenv, stack, Some(&(t.clone())), ins)?;
                    return Result::Ok(ITER(tinst));
                }
                MMap(t) => {
                    let tinst = ensure_iter_body(tcenv, stack, Some(&MPair(t.clone())), ins)?;
                    return Result::Ok(ITER(tinst));
                }
                m => {
                    return Result::Err(format!(
                        "ITER requires a list, set or map, but found {:?}",
                        m
                    ));
                }
            },
        },
        IF_CONS(tb, fb) => {
            get_stack_derived!(stack.ensure_non_empty(), FAIL);
            let stack_head = get_stack_derived!(stack.get_index(0), FAIL);
            let (cbtc, nbtc) = ensure_if_cons_body(tcenv, stack, (tb, fb))?;
            return Result::Ok(IF_CONS(cbtc, nbtc));
        }
        IF_NONE(nb, sb) => {
            get_stack_derived!(stack.ensure_non_empty(), FAIL);
            let (nbtc, sbtc) = ensure_if_none_body(tcenv, stack, (nb, sb))?;
            return Result::Ok(IF_NONE(nbtc, sbtc));
        }
        IF_SOME(sb, nb) => {
            get_stack_derived!(stack.ensure_non_empty(), FAIL);
            let (nbtc, sbtc) = ensure_if_none_body(tcenv, stack, (nb, sb))?;
            return Result::Ok(IF_SOME(sbtc, nbtc));
        }

        IF_LEFT(lb, rb) => {
            get_stack_derived!(stack.ensure_non_empty(), FAIL);
            let (lbtc, rbtc) = ensure_if_left_body(tcenv, stack, (lb, rb))?;
            return Result::Ok(IF_LEFT(lbtc, rbtc));
        }
        IF(tb, fb) => {
            match get_stack_derived!(stack.get_index(0), FAIL) {
                Some(MWrapped(MBool)) => {}
                _ => {
                    return Result::Err(
                        "Expecting a bool on stack top, but found something else".to_string(),
                    );
                }
            }
            let (tbtc, fbtc) = ensure_same_lambda_type(tcenv, stack, (tb, fb))?;
            return Result::Ok(IF(tbtc, fbtc));
        }
        GET(n) => match get_stack_derived!(stack.get_index(0), FAIL) {
            Some(stack_head) => {
                let r = get_n_pair(&n, stack_head)?;
                stack.replace_index(0, r.clone());
                return Result::Ok(GET(*n));
            }
            _ => return Result::Ok(GET(*n)),
        },
        UPDATE(n) => {
            get_stack_derived!(stack.ensure_stack_atleast(2), FAIL);
            match get_stack_derived!(stack.get_index(0), FAIL).clone() {
                Some(stack_head) => match get_stack_derived!(stack.get_index(1), FAIL).clone() {
                    Some(u) => {
                        let mut update_target = u.clone();
                        update_n_pair(&n, &stack_head, &mut update_target)?;
                        stack.pop();
                        stack.replace_index(0, update_target);
                        return Result::Ok(UPDATE(*n));
                    }
                    None => {
                        return Result::Err("Stack unexpectedly found empty".to_string());
                    }
                },
                None => return Result::Ok(UPDATE(*n)),
            }
        }
        DUP(n) => {
            if *n > 0 {
                get_stack_derived!(stack.ensure_stack_atleast(*n), FAIL);
                match get_stack_derived!(stack.get_index(n - 1), FAIL) {
                    Some(target) => {
                        stack.push(target.clone());
                        return Result::Ok(DUP(*n));
                    }
                    None => { panic!() }
                }
            } else {
                return Result::Err("DUP(0) is forbidden".to_string());
            }
        }
        PAIR(n) => {
            get_stack_derived!(stack.ensure_stack_atleast(*n), FAIL);
            let pair = get_stack_derived!(mk_pair(stack, *n), FAIL);
            stack.push(pair);
            return Result::Ok(PAIR(*n));
        }
        UNPAIR(n) => {
            if *n >= 2 {
                get_stack_derived!(stack.ensure_stack_atleast(1), FAIL);
                match get_stack_derived!(stack.pop(), FAIL) {
                    Some(stack_head) => {
                        unmk_pair(&stack_head, *n, stack);
                        return Result::Ok(UNPAIR(*n));
                    }
                    None => { panic!() }
                }
            } else {
                return Result::Err("PAIR(<2) is forbidden".to_string());
            }
        }
        DIG(n) => {
            get_stack_derived!(stack.ensure_stack_atleast(*n + 1), FAIL);
            stack.move_element(*n, 0);
            return Result::Ok(DIG(*n));
        }

        DUG(n) => {
            get_stack_derived!(stack.ensure_stack_atleast(*n + 1), FAIL);
            stack.move_element(0, *n);
            return Result::Ok(DUG(*n));
        }

        DROP(n) => {
            get_stack_derived!(stack.ensure_stack_atleast(*n), FAIL);
            for i in 1..=*n {
                stack.pop();
            }
            return Result::Ok(DROP(*n));
        }
        DIP(n, instr) => {
            if n > &0 {
                get_stack_derived!(stack.ensure_stack_atleast(*n), FAIL);
                let mut temp_stack = stack.clone_tail_at(*n);
                let tins = typecheck(tcenv, instr, &mut temp_stack)?;
                let mut result_stack = stack.clone_head_till(*n);
                result_stack.append_stack(&mut temp_stack);
                *stack = result_stack;
                return Result::Ok(DIP(*n, tins));
            } else {
                return Result::Err("DIP instruction's argument cannot be zero".to_string());
            }
        }
        LAMBDA_REC(it, ot, instr) => {
            let mut temp_stack = StackState::from(vec![
                it.clone(),
                MLambda(Box::new((it.clone(), ot.clone()))),
            ]);
            let tins = typecheck(tcenv, instr, &mut temp_stack)?;
            let temp_stack_len = temp_stack.len();
            let temp_stack_head = temp_stack.get_index(0);
            match (temp_stack_len, temp_stack_head) {
                (SdOk(l), SdOk(Some(sh))) => {
                    if sh == ot && l == 1 {
                        stack.push(MLambda(Box::new((it.clone(), ot.clone()))));
                        return Result::Ok(LAMBDA_REC(it.clone(), ot.clone(), tins));
                    } else {
                        return Result::Err(String::from(
                            "Unexpected output stack for lambda rec lambda",
                        ));
                    }
                }
                _ => {
                    return Result::Ok(LAMBDA_REC(it.clone(), ot.clone(), tins));
                }
            }
        }

        _ => {
            return Result::Err(format!("Instruction {} not found", ""));
        }
    };
}
