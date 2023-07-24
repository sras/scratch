use core::fmt::Debug;

use Constraint::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Concrete {}

pub type ConcreteType = CType<Concrete>;

#[derive(Debug, Eq, PartialEq)]
pub enum CType<T> {
    MNat,
    MInt,
    MString,
    MPair(Box<CTBox<T>>, Box<CTBox<T>>),
    MList(Box<CTBox<T>>),
    MLambda(Box<CTBox<T>>, Box<CTBox<T>>),
}

pub enum MValue {
    VNat(u32),
    VInt(i32),
    VString(String),
    VPair(Box<MValue>, Box<MValue>),
    VList(Vec<MValue>),
    VLambda(Vec<Instruction>)
}

#[derive(Debug, Eq, PartialEq)]
pub enum CTBox<T> {
    CTOther(T),
    CTSelf(CType<T>),
}

#[derive(Debug, Clone)]
pub enum ArgValue {
    TypeArg(ConcreteType),
    ValueArg(ConcreteType),
}

#[derive(Debug)]
pub struct Instruction {
    pub name: String,
    pub args: Vec<ArgValue>,
}

#[derive(Debug)]
pub enum Constraint {
    Arg(CType<Constraint>), // An argument that accept a value of a certain type.
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
    SRCType(CType<StackResult>),
    SRArgRef(char),
}

pub type StackState = Vec<ConcreteType>;

#[derive(Debug)]
pub struct InstructionDef {
    pub args: Vec<Constraint>,
    pub input_stack: Vec<StackArg>,
    pub output_stack: Vec<StackResult>,
}
