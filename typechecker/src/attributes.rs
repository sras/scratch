use std::collections::HashMap;

use crate::types::ConcreteType;
use crate::types::MAtomic;
use crate::types::MAtomic::*;
use crate::types::MType;
use crate::types::MType::*;

use crate::types::Attribute;
use crate::types::Attribute::*;

pub fn check_attributes(atrs: &[Attribute], ct: &ConcreteType) -> bool {
    for atr in atrs {
        if !check_attribute(atr, ct) {
            return false;
        }
    }
    return true;
}

pub fn check_attribute(atr: &Attribute, ct: &ConcreteType) -> bool {
    match ct {
        MWrapped(a) => true,

        MMap(b) => match b.as_ref() {
            (_, rt) => match atr {
                Comparable => false,
                _ => check_attribute(atr, rt),
            },
        },
        MBigMap(b) => match b.as_ref() {
            (_, rt) => match atr {
                Passable => check_attribute(atr, rt),
                Storable => check_attribute(atr, rt),
                Duplicable => check_attribute(atr, rt),
                _ => false,
            },
        },
        MPair(b) => match b.as_ref() {
            (lt, rt) => check_attribute(atr, lt) && check_attribute(atr, rt),
        },
        MOr(b) => match b.as_ref() {
            (lt, rt) => check_attribute(atr, lt) && check_attribute(atr, rt),
        },
        MTicket(b) => match atr {
            Comparable => false,
            Duplicable => false,
            Pushable => false,
            Passable => false,
            _ => true,
        },
        MList(b) => match atr {
            Comparable => false,
            _ => check_attribute(atr, b.as_ref()),
        },
        MSet(b) => match atr {
            Comparable => false,
            _ => check_attribute(atr, b.as_ref()),
        },

        MOption(b) => check_attribute(atr, b.as_ref()),
        MContract(b) => match atr {
            Comparable => false,
            Storable => false,
            Pushable => false,
            BigmapValue => false,
            _ => true,
        },

        MLambda(b) => match atr {
            Comparable => false,
            _ => true,
        },
    }
}
