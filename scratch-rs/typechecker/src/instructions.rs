use std::collections::BTreeMap;

use crate::parsers::parse_constraints;
use crate::parsers::parse_stack_results;
use crate::types::InstructionDef;

macro_rules! mk_instr {
    ($n:expr, $arg: expr, $is: expr, $os: expr) => {
        (
            String::from($n),
            vec![InstructionDef {
                args: parse_constraints($arg),
                input_stack: parse_constraints($is),
                output_stack: parse_stack_results($os),
            }],
        )
    };
}

macro_rules! mk_instr_poly {
    ($n:expr, $(($arg: expr, $is: expr, $os: expr)),*) => {
        (String::from($n), vec![$(InstructionDef {args: parse_constraints($arg), input_stack: parse_constraints($is), output_stack: parse_stack_results($os)} ), *])
    };
}

lazy_static! {
    pub static ref MICHELSON_INSTRUCTIONS: BTreeMap<String, Vec<InstructionDef>> = BTreeMap::from([
        mk_instr!(
            "APPLY",
            "",
            "<w|a>;lambda (pair <r|a> <w|b>) <w|c>",
            "lambda <r|b> <r|c>"
        ),
        mk_instr!("CAR", "", "pair <w|a> <w|b>", "<r|a>"),
        mk_instr!("CDR", "", "pair <w|a> <w|b>", "<r|b>"),
        mk_instr!("SWAP", "", "<w|a>;<w|b>", "<r|b>;<r|a>"),
        mk_instr!("DUP", "", "<w|a>", "<r|a>;<r|a>"),
        mk_instr!("DROP", "", "<w|a>", ""),
        mk_instr!("NIL", "<t|a>", "", "list <r|a>"),
        mk_instr_poly!(
            "ADD",
            ("", "<a|nat>;<a|nat>", "<a|nat>"),
            ("", "<a|int>;<a|int>", "<a|int>")
        ),
        mk_instr!("CONS", "", "<w|a>;list <r|a>", "list <r|a>"),
        mk_instr!("PUSH", "<t|a=>pushable>;<r|a>", "", "<r|a>"),
        mk_instr!("PAIR", "", "<w|a>;<w|b>", "pair <r|a> <r|b>"),
        mk_instr!("ADDRESS", "", "contract <w|a>", "<a|address>"),
        mk_instr!("CHAIN_ID", "", "", "<a|chain_id>"),
        mk_instr!("CHECK_SIGNATURE", "", "<a|key>;<a|signature>;<a|bytes>", "<a|bool>"),
        mk_instr!("PACK", "", "<w|a>", "<a|bytes>"),
        mk_instr!("SELF_ADDRESS", "", "<a|address>", ""),
        mk_instr!("UNPAIR", "", "pair <w|a> <w|b>", "<r|a>;<r|b>"),
        mk_instr!("UNIT", "", "", "<a|unit>"),
        mk_instr!("TRANSFER_TOKENS", "", "<w|a>;<a|mutez>;contract <r|a>", "<a|operation>"),
        mk_instr!("SET_DELEGATE", "", "option <a|key_hash>", "<a|operation>"),
        mk_instr!(
            "LAMBDA",
            "<t|a>;<t|b>;lambda <r|a> <r|b>",
            "",
            "lambda <r|a> <r|b>"
        ),
        mk_instr!("EXEC", "", "<w|a>;lambda <r|a> <w|b>", "<r|b>"),
        mk_instr!("ASSERT", "", "<a|bool>", ""),
        mk_instr!("ASSERT_CMPEQ", "", "<w|a>;<w|b>", ""),
        mk_instr!("ASSERT_CMPLE", "", "<w|a>;<w|b>", ""),
    ]);
}
