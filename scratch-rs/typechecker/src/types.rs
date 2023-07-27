use core::fmt::Debug;

use Constraint::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Concrete {}

pub type ConcreteType = MType<Concrete>;

#[derive(Debug, Eq, PartialEq)]
pub enum MType<T> {
    MNat,
    MInt,
    MString,
    MPair(Box<MNesting<T>>, Box<MNesting<T>>),
    MList(Box<MNesting<T>>),
    MLambda(Box<MNesting<T>>, Box<MNesting<T>>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum MNesting<T> {
    Other(T),
    Nested(MType<T>),
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
pub enum Constraint {
    Arg(MType<Constraint>), // An argument that accept a value of a certain type.
    Warg(char),             // An type variable.
    TypeArg(char),          // A argument that accept a type name, like Nat.
    TypeArgRef(char),       // A argument that accept a value of a type referred by
                            // previously encountered TypeArg.
}

impl Clone for Constraint {
    fn clone(&self) -> Self {
        match self {
            Arg(ct) => {
                return Arg(ct.clone());
            }
            Warg(c) => {
                return Warg(c.clone());
            }
            TypeArg(c) => {
                return TypeArg(c.clone());
            }
            TypeArgRef(c) => {
                return TypeArgRef(c.clone());
            }
        }
    }
}

pub type StackArg = Constraint;

#[derive(Debug, Clone)]
pub enum StackResult {
    SRMType(MType<StackResult>),
    SRArgRef(char),
}

pub type StackState = Vec<ConcreteType>;

#[derive(Debug)]
pub struct InstructionDef {
    pub args: Vec<Constraint>,
    pub input_stack: Vec<StackArg>,
    pub output_stack: Vec<StackResult>,
}
