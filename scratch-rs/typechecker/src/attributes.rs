use std::collections::HashMap;

use crate::types::ConcreteType;
use crate::types::MAtomic;
use crate::types::MAtomic::*;
use crate::types::MType;
use crate::types::MType::*;

pub enum Attribute {
    Comparable,
    Passable,
    Pushable,
    Storable,
    Packable,
    BigmapValue,
    Duplicable,
}
use Attribute::*;

fn check_attribute(atr: &Attribute, ct: &ConcreteType) -> bool {
    match ct {
        MWrapped(a) => true,
        MPair(b) => match b.as_ref() {
            (lt, rt) => check_attribute(atr, lt) && check_attribute(atr, rt),
        },
        MList(b) => match atr {
            Comparable => false,
            _ => check_attribute(atr, b.as_ref()),
        },

        MLambda(b) => match atr {
            Comparable => false,
            _ => true,
        },
    }
}
