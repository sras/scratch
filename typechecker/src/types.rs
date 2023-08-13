#![allow(non_camel_case_types)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::upper_case_acronyms)]
use crate::types::MType::*;
use core::cmp::Eq;
use core::cmp::Ordering;
use core::fmt::Debug;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;

pub type ConcreteType = MType<MAtomic>;

#[derive(Debug, Hash, Eq, Clone, PartialEq)]
pub enum MAtomic {
    MChainId,
    MBytes,
    MAddress,
    MNat,
    MInt,
    MString,
    MBool,
    MKey,
    MKeyHash,
    MMutez,
    MTimestamp,
    MUnit,
    MOperation,
    MSignature,
}

#[derive(Debug, Eq, PartialEq)]
pub enum MType<T> {
    MTicket(Box<MType<T>>),
    MOption(Box<MType<T>>),
    MContract(Box<MType<T>>),
    MPair(Box<(MType<T>, MType<T>)>),
    MOr(Box<(MType<T>, MType<T>)>),
    MList(Box<MType<T>>),
    MLambda(Box<(MType<T>, MType<T>)>),
    MBigMap(Box<(MType<T>, MType<T>)>),
    MMap(Box<(MType<T>, MType<T>)>),
    MSet(Box<MType<T>>),
    MWrapped(T),
}

impl<T: Clone> Clone for MType<T> {
    fn clone(&self) -> Self {
        map_mtype(self, &|x| x.clone())
    }
}

#[derive(Debug, Clone)]
pub enum SomeValue {
    Atomic(AtomicValue),
    Composite(Box<CompositeValue>),
}

#[derive(Debug, Clone)]
pub enum AtomicValue {
    AVNumber(i32),
    AVString(String),
    AVBool(bool),
    AVUnit,
}

#[derive(Debug, Clone)]
pub enum MValue {
    VUnit,
    VNat(u32),
    VInt(i32),
    VBool(bool),
    VString(String),
    VPair(Box<(MValue, MValue)>),
    VRight(Box<MValue>),
    VLeft(Box<MValue>),
    VSome(Box<MValue>),
    VMutez(u32),
    VMap(BTreeMap<MValue, MValue>),
    VBigMap(BTreeMap<MValue, MValue>),
    VList(Vec<MValue>),
    VSet(BTreeSet<MValue>),
    VLambda(Vec<CompoundInstruction<MValue>>),
}

use MValue::*;

impl Eq for MValue {}

impl PartialEq for MValue {
    fn eq(&self, m2: &Self) -> bool {
        matches!(self.partial_cmp(m2), Some(std::cmp::Ordering::Equal))
    }
}

impl PartialOrd for MValue {
    fn partial_cmp(&self, m2: &Self) -> Option<Ordering> {
        match (self, m2) {
            (VNat(u1), VNat(u2)) => u1.partial_cmp(u2),
            (VInt(i1), VInt(i2)) => i1.partial_cmp(i2),
            (VBool(b1), VBool(b2)) => b1.partial_cmp(b2),
            (VString(s1), VString(s2)) => s1.partial_cmp(s2),
            (VPair(s1), VPair(s2)) => {
                s1.0.partial_cmp(&s2.0)
                    .partial_cmp(&(s1.1).partial_cmp(&s2.1))
            }
            _ => panic!("Uncomparable types!"),
        }
    }
}

impl Ord for MValue {
    fn cmp(&self, m2: &Self) -> Ordering {
        match self.partial_cmp(m2) {
            Some(x) => x,
            None => panic!("Uncomparable types!"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Contract<T> {
    pub parameter: ConcreteType,
    pub storage: ConcreteType,
    pub code: Vec<CompoundInstruction<T>>,
}

pub type SomeKeyValue = (SomeValue, SomeValue);

#[derive(Debug, Clone)]
pub enum SeqItem {
    SqValue(Vec<SomeValue>),
    SqInstr(Vec<CompoundInstruction<SomeValue>>),
}

use SeqItem::*;

impl SeqItem {
    pub fn len(&self) -> usize {
        match self {
            SqValue(s) => s.len(),
            SqInstr(s) => s.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CompositeValue {
    CVPair(SomeValue, SomeValue),
    CVSeq(SeqItem),
    CKVList(Vec<SomeKeyValue>),
    CVLeft(SomeValue),
    CVRight(SomeValue),
    CVSome(SomeValue),
    CVNone,
}

#[derive(Debug, Clone)]
pub enum ArgValue<T> {
    TypeArg(ConcreteType),
    ValueArg(T),
}

#[derive(Debug, Clone)]
pub struct Instruction<T> {
    pub location: usize,
    pub name: String,
    pub args: Vec<ArgValue<T>>,
}

#[derive(Debug, Clone)]
pub enum CompoundInstruction<T> {
    IF(Vec<CompoundInstruction<T>>, Vec<CompoundInstruction<T>>),
    IF_CONS(Vec<CompoundInstruction<T>>, Vec<CompoundInstruction<T>>),
    IF_SOME(Vec<CompoundInstruction<T>>, Vec<CompoundInstruction<T>>),
    IF_NONE(Vec<CompoundInstruction<T>>, Vec<CompoundInstruction<T>>),
    IF_LEFT(Vec<CompoundInstruction<T>>, Vec<CompoundInstruction<T>>),
    PAIR(usize),
    UNPAIR(usize),
    DIP(usize, Vec<CompoundInstruction<T>>),
    DUP(usize),
    DIG(usize),
    DUG(usize),
    DROP(usize),
    GET(usize),
    UPDATE(usize),
    MAP(Vec<CompoundInstruction<T>>),
    ITER(Vec<CompoundInstruction<T>>),
    LOOP(Vec<CompoundInstruction<T>>),
    LOOP_LEFT(Vec<CompoundInstruction<T>>),
    LAMBDA_REC(ConcreteType, ConcreteType, Vec<CompoundInstruction<T>>),
    SELF,
    FAIL,
    FAILWITH,
    Other(Instruction<T>),
}

pub struct TcEnv {
    pub self_type: ConcreteType,
}

#[derive(Debug)]
pub enum ArgConstraint {
    CAtomic(MAtomic),
    CWarg(char, Vec<Attribute>),    // An type variable.
    CTypeArg(char, Vec<Attribute>), // A argument that accept a type name, like Nat.
    CTypeArgRef(char), // A argument that accept a value of a type referred by previously encountered TypeArg.
}

pub type Constraint = MType<ArgConstraint>;

pub type StackArg = Constraint;

pub type StackResult = MType<StackResultElem>;

#[derive(Debug)]
pub enum StackResultElem {
    TRef(char),
    ElemType(MAtomic),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StackState<T> {
    LiveStack(VecDeque<MType<T>>),
    FailedStack,
}

use StackState::*;

pub enum StackDerived<T> {
    SdOk(T),
    SdFailed,
}

impl<T: Clone> StackDerived<T> {
    pub fn unwrap(self) -> T {
        match self {
            SdOk(t) => t,
            _ => panic!("Stack derived unwrapping failed!"),
        }
    }
}

use StackDerived::*;

#[macro_export]
macro_rules! ensure_stack_derived {
    ($n:expr, $s: expr, $f: expr) => {{
        match $n {
            StackDerived::SdOk(x) => {
                if x {
                } else {
                    return Result::Err($s);
                }
            }
            StackDerived::SdFailed => {
                return Result::Ok($f);
            }
        }
    }};
}

#[macro_export]
macro_rules! get_stack_derived_result {
    ($n:expr) => {{
        match $n {
            StackDerived::SdOk(Result::Ok(a)) => a,
            a => {
                return a;
            }
        }
    }};
}
#[macro_export]
macro_rules! get_stack_derived_result_handle_failed {
    ($n:expr, $f: expr) => {{
        match $n {
            StackDerived::SdOk(Result::Ok(a)) => a,
            StackDerived::SdOk(Result::Err(a)) => return Result::Err(a),
            StackDerived::SdFailed => return Result::Ok($f),
        }
    }};
}

pub enum StackCompResult {
    LeftFailed,
    RightFailed,
    BothFailed,
    NoMatch,
    Match,
}

use StackCompResult::*;

pub type ConcreteStack = StackState<MAtomic>;

impl<T: Eq + Clone> StackState<T> {
    pub fn ensure_non_empty(&self) -> StackDerived<bool> {
        self.ensure_stack_atleast(1)
    }
    pub fn len(&self) -> StackDerived<usize> {
        match self {
            LiveStack(v) => SdOk(v.len()),
            _ => SdFailed,
        }
    }
    pub fn ensure_stack_atleast(&self, l: usize) -> StackDerived<bool> {
        match self {
            LiveStack(v) => SdOk(v.len() >= l),
            FailedStack => SdFailed,
        }
    }
    pub fn get_index(&self, i: usize) -> StackDerived<Result<&MType<T>, String>> {
        match self {
            LiveStack(v) => {
                if v.is_empty() {
                    SdOk(Result::Err("Stack too short..".to_string()))
                } else {
                    SdOk(Result::Ok(&v[i]))
                }
            }
            FailedStack => SdFailed,
        }
    }
    pub fn push(&mut self, t: MType<T>) {
        match self {
            LiveStack(v) => v.push_front(t),
            FailedStack => {}
        }
    }

    pub fn pop(&mut self) -> StackDerived<Result<MType<T>, String>> {
        match self {
            LiveStack(v) => match v.pop_front() {
                Some(a) => SdOk(Result::Ok(a)),
                None => SdOk(Result::Err("Stack is empty".to_string())),
            },
            FailedStack => SdFailed,
        }
    }

    pub fn move_element(&mut self, f: usize, t: usize) {
        match self {
            LiveStack(v) => match v.remove(f) {
                Some(a) => {
                    v.insert(t, a);
                }
                None => {
                    panic!("Failed to move stack element!");
                }
            },
            FailedStack => {}
        }
    }

    pub fn replace_index(&mut self, i: usize, t: MType<T>) {
        match self {
            LiveStack(v) => {
                v[i] = t;
            }
            FailedStack => {}
        }
    }

    pub fn append_stack(&mut self, src: &mut Self) {
        match self {
            LiveStack(v) => match src {
                LiveStack(ref mut v1) => v.append(v1),
                FailedStack => {}
            },
            FailedStack => {}
        }
    }

    pub fn new() -> Self {
        LiveStack(VecDeque::new())
    }

    pub fn fail(&mut self) {
        *self = FailedStack;
    }

    pub fn from(v: Vec<MType<T>>) -> Self {
        LiveStack(VecDeque::from(v))
    }

    pub fn compare<'a>(&'a self, s: &'a Self) -> StackCompResult {
        match self {
            FailedStack => match s {
                FailedStack => BothFailed,
                _ => LeftFailed,
            },
            LiveStack(v) => match s {
                LiveStack(s_) => {
                    if *s_ == *v {
                        Match
                    } else {
                        NoMatch
                    }
                }
                FailedStack => RightFailed,
            },
        }
    }

    pub fn compare_singleton(&self, s: &MType<T>) -> bool {
        match self {
            FailedStack => true,
            LiveStack(v) => v[0] == *s,
        }
    }

    pub fn clone_tail(&mut self) -> Self {
        match self {
            LiveStack(v) => {
                let mut slice = v.clone();
                slice.pop_front();
                LiveStack(slice)
            }
            FailedStack => FailedStack,
        }
    }

    pub fn clone_tail_at(&self, l: usize) -> Self {
        match self {
            LiveStack(v) => LiveStack(v.iter().skip(l).cloned().collect()),
            FailedStack => FailedStack,
        }
    }

    pub fn clone_head_till(&self, l: usize) -> Self {
        match self {
            LiveStack(v) => LiveStack(v.iter().take(l).cloned().collect()),
            FailedStack => FailedStack,
        }
    }

    pub fn pop_front(&mut self) -> StackDerived<Result<MType<T>, String>> {
        match self {
            LiveStack(v) => match v.pop_front() {
                Some(x) => SdOk(Result::Ok(x)),
                None => SdOk(Result::Err("Stack is empty".to_string())),
            },
            FailedStack => SdFailed,
        }
    }

    pub fn push_front(&mut self, t: MType<T>) {
        match self {
            LiveStack(v) => {
                v.push_front(t);
            }
            FailedStack => {}
        }
    }
}

#[derive(Debug)]
pub struct InstructionDef {
    pub args: Vec<Constraint>,
    pub input_stack: Vec<StackArg>,
    pub output_stack: Vec<StackResult>,
}

#[derive(Debug)]
pub enum Attribute {
    Comparable,
    Passable,
    Pushable,
    Storable,
    Packable,
    BigmapValue,
    Duplicable,
}

// Parser helpers

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DynMType {
    DMAtomic(MAtomic),
    DMDyn(String),
}

use DynMType::*;

pub fn map_mtype_boxed_pair<T, H, F: Fn(&T) -> H>(
    b: &(MType<T>, MType<T>),
    cb: &F,
) -> Box<(MType<H>, MType<H>)> {
    let (f, s) = b;
    Box::new((map_mtype(f, cb), map_mtype(s, cb)))
}

pub fn update_n_pair<A: Clone>(
    n: &usize,
    src: &MType<A>,
    t: &mut MType<A>,
) -> Result<MType<A>, String> {
    let mut cb: bool = false;
    let mut cn: &mut MType<A> = t;
    for _ in 0..*n {
        if cb {
            match cn {
                MPair(b) => {
                    cn = &mut b.1;
                    cb = false;
                }
                _ => {
                    return Result::Err(
                        "Expected a Pair but got something else during GET n typecheck".to_string(),
                    );
                }
            }
        } else {
            cb = true;
        }
    }
    if cb {
        match cn {
            MPair(ref mut b) => {
                let (f, _) = b.as_mut();
                *f = src.clone();
                Result::Ok(f.clone())
            }
            _ => Result::Err("Expected a Pair but got something else".to_string()),
        }
    } else {
        Result::Ok(cn.clone())
    }
}

pub fn get_n_pair<'a, A: Clone>(n: &usize, t: &'a MType<A>) -> Result<&'a MType<A>, String> {
    let mut cb: bool = false;
    let mut cn: &MType<A> = t;
    for _ in 0..*n {
        if cb {
            match cn {
                MPair(b) => {
                    let (_, s) = b.as_ref();
                    cn = s;
                    cb = false;
                }
                _ => {
                    return Result::Err(
                        "Expected a Pair but got something else during GET n typecheck".to_string(),
                    );
                }
            }
        } else {
            cb = true;
        }
    }
    if cb {
        match cn {
            MPair(b) => {
                let (f, _) = b.as_ref();
                Result::Ok(f)
            }
            _ => Result::Err("Expected a Pair but got something else".to_string()),
        }
    } else {
        Result::Ok(cn)
    }
}

pub fn mk_pair<A: Clone + Eq>(
    tl: &mut StackState<A>,
    n: usize,
) -> StackDerived<Result<MType<A>, String>> {
    if n == 2 {
        let i1 = get_stack_derived_result!(tl.pop_front());
        let i2 = get_stack_derived_result!(tl.pop_front());
        SdOk(Result::Ok(MPair(Box::new((i1, i2)))))
    } else {
        let i1 = get_stack_derived_result!(tl.pop_front());
        let l2 = get_stack_derived_result!(mk_pair(tl, n - 1));
        SdOk(Result::Ok(MPair(Box::new((i1, l2)))))
    }
}

pub fn unmk_pair<A: Eq + Clone>(
    t: &MType<A>,
    n: usize,
    stack: &mut StackState<A>,
) -> Result<(), String> {
    if n == 1 {
        stack.push(t.clone());
        Result::Ok(())
    } else {
        match t {
            MPair(b) => {
                let (le, rp) = b.as_ref();
                let r = unmk_pair(rp, n - 1, stack);
                stack.push(le.clone());
                r
            }
            _ => Result::Err("Expecting a pair for UNPAIR but got something else".to_string()),
        }
    }
}

pub fn map_mtype<T, H, F: Fn(&T) -> H>(ct: &MType<T>, cb: &F) -> MType<H> {
    match ct {
        MPair(b) => MPair(map_mtype_boxed_pair(b, cb)),
        MOr(b) => MOr(map_mtype_boxed_pair(b, cb)),
        MLambda(b) => MLambda(map_mtype_boxed_pair(b, cb)),
        MList(l) => MList(Box::new(map_mtype(l, cb))),
        MTicket(l) => MTicket(Box::new(map_mtype(l, cb))),
        MContract(l) => MContract(Box::new(map_mtype(l, cb))),
        MOption(l) => MOption(Box::new(map_mtype(l, cb))),
        MSet(l) => MSet(Box::new(map_mtype(l, cb))),
        MMap(b) => MMap(map_mtype_boxed_pair(b, cb)),
        MBigMap(b) => MBigMap(map_mtype_boxed_pair(b, cb)),
        MWrapped(w) => MWrapped(cb(w)),
    }
}
pub fn mdyn_to_concrete(m: &MType<DynMType>) -> ConcreteType {
    map_mtype(m, &|x| dynm_to_matomic(x.clone()))
}

fn dynm_to_matomic(d: DynMType) -> MAtomic {
    match d {
        DMAtomic(a) => a,
        DMDyn(_) => panic!("Unexpected enum variant!"),
    }
}
