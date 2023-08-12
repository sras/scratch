use crate::types::ConcreteType;
use crate::types::MType::*;

use crate::types::Attribute;
use crate::types::Attribute::*;

pub fn check_attributes(atrs: &[Attribute], ct: &ConcreteType) -> bool {
    for atr in atrs {
        if !check_attribute(atr, ct) {
            return false;
        }
    }
    true
}

pub fn check_attribute(atr: &Attribute, ct: &ConcreteType) -> bool {
    match ct {
        MWrapped(_) => true,

        MMap(b) => match atr {
            Comparable => false,
            _ => check_attribute(atr, &b.1),
        },
        MBigMap(b) => match atr {
            Passable => check_attribute(atr, &b.1),
            Storable => check_attribute(atr, &b.1),
            Duplicable => check_attribute(atr, &b.1),
            _ => false,
        },
        MPair(b) =>  {
            check_attribute(atr, &b.0) && check_attribute(atr, &b.1)
        },
        MOr(b) => {
            check_attribute(atr, &b.0) && check_attribute(atr, &b.1)
        },
        MTicket(_) => !matches!(atr, Comparable | Duplicable | Pushable | Passable),
        MList(b) => match atr {
            Comparable => false,
            _ => check_attribute(atr, b.as_ref()),
        },
        MSet(b) => match atr {
            Comparable => false,
            _ => check_attribute(atr, b.as_ref()),
        },

        MOption(b) => check_attribute(atr, b.as_ref()),
        MContract(_) => !matches!(atr, Comparable | Storable | Pushable | BigmapValue),

        MLambda(_) => !matches!(atr, Comparable),
    }
}
