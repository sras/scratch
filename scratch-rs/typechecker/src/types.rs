use crate::types::MType::*;
use core::cmp::Eq;
use core::cmp::Ordering;
use core::fmt::Debug;
use std::collections::BTreeMap;

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
        return map_mtype(self, &|x| x.clone());
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
    VMap(Box<BTreeMap<MValue, MValue>>),
    VBigMap(Box<BTreeMap<MValue, MValue>>),
    VList(Vec<MValue>),
    VLambda(Vec<CompoundInstruction<MValue>>),
}

use MValue::*;

impl Eq for MValue {}

impl PartialEq for MValue {
    fn eq(&self, m2: &Self) -> bool {
        match self.partial_cmp(m2) {
            Some(std::cmp::Ordering::Equal) => true,
            _ => false,
        }
    }
}

impl PartialOrd for MValue {
    fn partial_cmp(&self, m2: &Self) -> Option<Ordering> {
        match (self, m2) {
            (VNat(u1), VNat(u2)) => u1.partial_cmp(u2),
            (VInt(i1), VInt(i2)) => i1.partial_cmp(i2),
            (VBool(b1), VBool(b2)) => b1.partial_cmp(b2),
            (VString(s1), VString(s2)) => s1.partial_cmp(s2),
            (VPair(s1), VPair(s2)) => match (s1.as_ref(), s2.as_ref()) {
                ((l1, r1), (l2, r2)) => l1.partial_cmp(l2).partial_cmp(&r1.partial_cmp(r2)),
            },
            _ => panic!("Uncomparable types!"),
        }
    }
}

impl Ord for MValue {
    fn cmp(&self, m2: &Self) -> Ordering {
        panic!()
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
pub enum CompositeValue {
    CVPair(SomeValue, SomeValue),
    CVLambda(Vec<CompoundInstruction<SomeValue>>),
    CVList(Vec<SomeValue>),
    CKVList(Vec<SomeKeyValue>),
}

#[derive(Debug, Clone)]
pub enum ArgValue<T> {
    TypeArg(ConcreteType),
    ValueArg(T),
}

#[derive(Debug, Clone)]
pub struct Instruction<T> {
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
    DIP(usize, Vec<CompoundInstruction<T>>),
    ITER(Vec<CompoundInstruction<T>>),
    LAMBDA_REC(ConcreteType, ConcreteType, Vec<CompoundInstruction<T>>),
    SELF,
    FAIL,
    Other(Instruction<T>),
}

pub struct TcEnv {
    pub selfType: ConcreteType,
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
pub enum StackState {
    LiveStack(Vec<ConcreteType>),
    FailedStack,
}

use StackState::*;

impl StackState {
    pub fn push(&mut self, t: ConcreteType) {
        match self {
            LiveStack(v) => v.insert(0, t),
            FailedStack => {}
        }
    }

    pub fn get_live(&self) -> Option<&Vec<ConcreteType>> {
        match self {
            LiveStack(v) => Some(&v),
            FailedStack => None,
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
        LiveStack(Vec::new())
    }

    pub fn fail(&mut self) {
        *self = FailedStack;
    }

    pub fn from(v: Vec<ConcreteType>) -> Self {
        LiveStack(Vec::from(v))
    }

    pub fn compare(&self, s: &Self) -> Option<Self> {
        match self {
            FailedStack => match s {
                FailedStack => Some(FailedStack),
                _ => Some(s.clone()),
            },
            LiveStack(v) => match s {
                LiveStack(s_) => {
                    if s_ == v {
                        Some(self.clone())
                    } else {
                        None
                    }
                }
                FailedStack => Some(self.clone()),
            },
        }
    }

    pub fn compare_singleton(&self, s: &ConcreteType) -> bool {
        match self {
            FailedStack => true,
            LiveStack(v) => match v[..] {
                [ref si] => *si == *s,
                _ => false,
            },
        }
    }

    pub fn clone_tail(&self) -> Self {
        match self {
            LiveStack(v) => LiveStack(Vec::from(&v[1..])),
            FailedStack => FailedStack,
        }
    }

    pub fn clone_tail_at(&self, l: usize) -> Self {
        match self {
            LiveStack(v) => LiveStack(Vec::from(&v[l..])),
            FailedStack => FailedStack,
        }
    }

    pub fn clone_head_till(&self, l: usize) -> Self {
        match self {
            LiveStack(v) => LiveStack(Vec::from(&v[0..l])),
            FailedStack => FailedStack,
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

#[derive(Debug, Clone)]
pub enum DynMType {
    DMAtomic(MAtomic),
    DMDyn(String),
}

use DynMType::*;

pub fn map_mtype_boxed_pair<T, H, F: Fn(&T) -> H>(
    b: &Box<(MType<T>, MType<T>)>,
    cb: &F,
) -> Box<(MType<H>, MType<H>)> {
    let (f, s) = b.as_ref();
    return Box::new((map_mtype(f, cb), map_mtype(s, cb)));
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
    return map_mtype(m, &|x| dynm_to_matomic(x.clone()));
}

fn dynm_to_matomic(d: DynMType) -> MAtomic {
    match d {
        DMAtomic(a) => a,
        DMDyn(_) => panic!("Unexpected enum variant!"),
    }
}
