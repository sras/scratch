use core::fmt::Debug;
use crate::types::MType::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Concrete {}

pub type ConcreteType = MType<MAtomic>;

#[derive(Debug, Eq, Clone, PartialEq)]
pub enum MAtomic {
    MNat,
    MInt,
    MString,
}

#[derive(Debug, Eq, PartialEq)]
pub enum MType<T> {
    MPair(Box<MType<T>>, Box<MType<T>>),
    MList(Box<MType<T>>),
    MLambda(Box<MType<T>>, Box<MType<T>>),
    MWrapped(T)
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
}

#[derive(Debug, Clone)]
pub enum MValue {
    VNat(u32),
    VInt(i32),
    VString(String),
    VPair(Box<MValue>, Box<MValue>),
    VList(Vec<MValue>),
    VLambda(Vec<Instruction<MValue>>),
}

#[derive(Debug, Clone)]
pub enum CompositeValue {
    CVPair(SomeValue, SomeValue),
    CVLambda(Vec<Instruction<SomeValue>>),
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

#[derive(Debug)]
pub enum ArgConstraint {
    CAtomic(MAtomic),
    CWarg(char),             // An type variable.
    CTypeArg(char),          // A argument that accept a type name, like Nat.
    CTypeArgRef(char), // A argument that accept a value of a type referred by previously encountered TypeArg.
}

pub type Constraint = MType<ArgConstraint>;

pub type StackArg = Constraint;

pub type StackResult = MType<StackResultElem>;

#[derive(Debug)]
pub enum StackResultElem {
    TRef(char),
    ElemType(MAtomic)
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
  DMDyn(String)
}

use DynMType::*;

pub fn map_mtype<T: Clone, H>(ct: &MType<T>, cb: fn(&T) -> H) -> MType<H> {
    match ct {
        MPair(l, r) => MPair(Box::new(map_mtype(l, cb)), Box::new(map_mtype(r, cb))),
        MLambda(l, r) => MLambda(Box::new(map_mtype(l, cb)), Box::new(map_mtype(r, cb))),
        MList(l) => MList(Box::new(map_mtype(l, cb))),
        MWrapped(w) => MWrapped(cb(w)),
    }
}

pub fn mdyn_to_concrete(m: &MType<DynMType>) -> ConcreteType {
    return map_mtype(m, |x| dynm_to_matomic(x.clone()));
}

fn dynm_to_matomic(d: DynMType) -> MAtomic {
    match d {
        DMAtomic(a) => a,
        DMDyn(_) => panic!("Unexpected enum variant!")
    }
}
