use std::collections::HashMap;

use crate::types::ConcreteType;
use crate::types::MAtomic;
use crate::types::MAtomic::*;
use crate::types::MType;
use crate::types::MType::*;

use crate::types::Attribute;
use crate::types::Attribute::*;

pub fn check_attribute(atr: &Attribute, ct: &ConcreteType) -> bool {
    match ct {
        MWrapped(a) => true,
        MBigMap(b) => match b.as_ref() {
            (_, rt) => match atr {
                Passable => {
                    check_attribute(atr, rt)
                }
                Storable => {
                    check_attribute(atr, rt)
                }
                Duplicable => {
                    check_attribute(atr, rt)
                }
                _ => false
            }
        },
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
