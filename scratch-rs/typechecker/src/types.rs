use crate::types::MType::*;
use core::fmt::Debug;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Concrete {}

pub type ConcreteType = MType<MAtomic>;

#[derive(Debug, Eq, Clone, PartialEq)]
pub enum MAtomic {
    MNat,
    MInt,
    MString,
    MBool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum MType<T> {
    MPair(Box<(MType<T>, MType<T>)>),
    MList(Box<MType<T>>),
    MLambda(Box<(MType<T>, MType<T>)>),
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
}

#[derive(Debug, Clone)]
pub enum MValue {
    VNat(u32),
    VInt(i32),
    VBool(bool),
    VString(String),
    VPair(Box<(MValue, MValue)>),
    VList(Vec<MValue>),
    VLambda(Vec<CompoundInstruction<MValue>>),
}

#[derive(Debug, Clone)]
pub enum CompositeValue {
    CVPair(SomeValue, SomeValue),
    CVLambda(Vec<CompoundInstruction<SomeValue>>),
    CVList(Vec<SomeValue>),
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
    Other(Instruction<T>)
}

#[derive(Debug)]
pub enum ArgConstraint {
    CAtomic(MAtomic),
    CWarg(char),       // An type variable.
    CTypeArg(char),    // A argument that accept a type name, like Nat.
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

pub type StackState = Vec<ConcreteType>;

#[derive(Debug)]
pub struct InstructionDef {
    pub args: Vec<Constraint>,
    pub input_stack: Vec<StackArg>,
    pub output_stack: Vec<StackResult>,
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
        MLambda(b) => MLambda(map_mtype_boxed_pair(b, cb)),
        MList(l) => MList(Box::new(map_mtype(l, cb))),
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
