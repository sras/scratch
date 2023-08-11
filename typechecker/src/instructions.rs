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
    pub static ref MICHELSON_INSTRUCTIONS: BTreeMap<String, Vec<InstructionDef>> =
        BTreeMap::from([
            mk_instr!(
                "APPLY",
                "",
                "<w|a>;lambda (pair <r|a> <w|b>) <w|c>",
                "lambda <r|b> <r|c>"
            ),
            mk_instr!("CAR", "", "pair <w|a> <w|b>", "<r|a>"),
            mk_instr!("CDR", "", "pair <w|a> <w|b>", "<r|b>"),
            mk_instr!("HASH_KEY", "", "<a|key>", "<a|key_hash>"),
            mk_instr!("IMPLICIT_ACCOUNT", "", "<a|key_hash>", "contract <a|unit>"),
            mk_instr!("SWAP", "", "<w|a>;<w|b>", "<r|b>;<r|a>"),
            mk_instr!("NIL", "<t|a>", "", "list <r|a>"),
            mk_instr!("CAST", "", "", ""),
            mk_instr!("VIEW", "<a|string>;<t|a>", "<w|b>;<a|address>", "option <r|a>"),
            mk_instr!("SENDER", "", "", "<a|address>"),
            mk_instr!("EMPTY_BIG_MAP", "<t|a>;<t|b>", "", "big_map <r|a> <r|b>"),
            mk_instr_poly!(
                "NOT",
                ("", "<a|bool>", "<a|bool>"),
                ("", "<a|nat>", "<a|int>"),
                ("", "<a|int>", "<a|int>"),
                ("", "<a|bytes>", "<a|bytes>")
            ),
            mk_instr_poly!(
                "MEM",
                ("", "<w|k>;set <r|k>", "<a|bool>"),
                ("", "<w|k>;map <r|k> <w|b>", "<a|bool>"),
                ("", "<w|k>;big_map <r|k> <w|b>", "<a|bool>")
            ),
            mk_instr_poly!(
                "MUL",
                ("", "<a|int>;<a|nat>", "<a|int>"),
                ("", "<a|nat>;<a|mutez>", "<a|mutez>"),
                ("", "<a|mutez>;<a|nat>", "<a|mutez>"),
                ("", "<a|int>;<a|int>", "<a|int>"),
                ("", "<a|nat>;<a|nat>", "<a|nat>"),
                ("", "<a|nat>;<a|int>", "<a|int>")
            ),
            mk_instr_poly!(
                "SIZE",
                ("", "set <w|b>", "<a|nat>"),
                ("", "map <w|a> <w|b>", "<a|nat>"),
                ("", "list <w|a>", "<a|nat>"),
                ("", "<a|string>", "<a|nat>"),
                ("", "<a|bytes>", "<a|nat>")
            ),
            mk_instr_poly!(
                "ADD",
                ("", "<a|mutez>;<a|mutez>", "<a|mutez>"),
                ("", "<a|int>;<a|timestamp>", "<a|timestamp>"),
                ("", "<a|timestamp>;<a|int>", "<a|timestamp>"),
                ("", "<a|int>;<a|nat>", "<a|int>"),
                ("", "<a|nat>;<a|int>", "<a|int>"),
                ("", "<a|nat>;<a|nat>", "<a|nat>"),
                ("", "<a|int>;<a|int>", "<a|int>")
            ),
            mk_instr_poly!(
                "AND",
                ("", "<a|bool>;<a|bool>", "<a|bool>"),
                ("", "<a|nat>;<a|nat>", "<a|nat>"),
                ("", "<a|int>;<a|nat>", "<a|nat>"),
                ("", "<a|bytes>;<a|bytes>", "<a|bytes>")
            ),
            mk_instr_poly!(
                "SUB",
                ("", "<a|nat>;<a|nat>", "<a|int>"),
                ("", "<a|int>;<a|int>", "<a|int>"),
                ("", "<a|int>;<a|nat>", "<a|int>"),
                ("", "<a|nat>;<a|int>", "<a|int>"),
                ("", "<a|timestamp>;<a|int>", "<a|timestamp>"),
                ("", "<a|timestamp>;<a|timestamp>", "<a|int>"),
                ("", "<a|mutez>;<a|mutez>", "option <a|mutez>")
            ),
            mk_instr_poly!(
                "OR",
                ("", "<a|bool>;<a|bool>", "<a|bool>"),
                ("", "<a|nat>;<a|nat>", "<a|nat>"),
                ("", "<a|bytes>;<a|bytes>", "<a|bytes>")
            ),
            mk_instr_poly!(
                "XOR",
                ("", "<a|bool>;<a|bool>", "<a|bool>"),
                ("", "<a|nat>;<a|nat>", "<a|nat>"),
                ("", "<a|bytes>;<a|bytes>", "<a|bytes>")
            ),
            mk_instr_poly!(
                "GET",
                ("", "<w|k>;map <r|k> <w|v>", "option <r|v>"),
                ("", "<w|k>;big_map <r|k> <w|v>", "option <r|v>")
            ),
            mk_instr_poly!(
                "EDIV",
                ("", "<a|nat>;<a|nat>", "option (pair <a|nat> <a|nat>)"),
                ("", "<a|nat>;<a|int>", "option (pair <a|int> <a|nat>)"),
                ("", "<a|int>;<a|nat>", "option (pair <a|int> <a|nat>)"),
                ("", "<a|int>;<a|int>", "option (pair <a|int> <a|nat>)"),
                ("", "<a|mutez>;<a|nat>", "option (pair <a|mutez> <a|mutez>)"),
                ("", "<a|mutez>;<a|mutez>", "option (pair <a|nat> <a|mutez>)")
            ),
            mk_instr_poly!("INT", ("", "<a|nat>", "<a|int>")),
            mk_instr!("SOME", "", "<w|a>", "option <r|a>"),
            mk_instr!("NONE", "<t|a>", "", "option <r|a>"),
            mk_instr_poly!(
                "UPDATE",
                ("", "<w|k>;option <w|v>;map <r|k> <r|v>", "map <r|k> <r|v>"),
                (
                    "",
                    "<w|k>;option <w|v>;big_map <r|k> <r|v>",
                    "big_map <r|k> <r|v>"
                ),
                ("", "<w|k>;<a|bool>;set <r|k>", "set <r|k>")
            ),
            mk_instr!("CONS", "", "<w|a>;list <r|a>", "list <r|a>"),
            mk_instr!("LEFT", "<t|a>", "<w|b>", "or <r|b> <r|a>"),
            mk_instr!("RIGHT", "<t|a>", "<w|b>", "or <r|a> <r|b>"),
            mk_instr!(
                "CONTRACT",
                "<t|a>",
                "<a|address>",
                "option (contract <r|a>)"
            ),
            mk_instr!("BLAKE2B", "", "<a|bytes>", "<a|bytes>"),
            mk_instr!("PUSH", "<t|a=>pushable>;<r|a>", "", "<r|a>"),
            mk_instr!("ADDRESS", "", "contract <w|a>", "<a|address>"),
            mk_instr!("CHAIN_ID", "", "", "<a|chain_id>"),
            mk_instr!("EQ", "", "<a|int>", "<a|bool>"),
            mk_instr!("GE", "", "<a|int>", "<a|bool>"),
            mk_instr!("GT", "", "<a|int>", "<a|bool>"),
            mk_instr!("NEQ", "", "<a|int>", "<a|bool>"),
            mk_instr!("ISNAT", "", "<a|int>", "option <a|nat>"),
            mk_instr!("LEVEL", "", "", "<a|nat>"),
            mk_instr!("COMPARE", "", "<w|a=>comparable>;<r|a>", "<a|int>"),
            mk_instr!("LT", "", "<a|int>", "<a|bool>"),
            mk_instr!(
                "CHECK_SIGNATURE",
                "",
                "<a|key>;<a|signature>;<a|bytes>",
                "<a|bool>"
            ),
            mk_instr!("PACK", "", "<w|a>", "<a|bytes>"),
            mk_instr!("UNPACK", "<t|a>", "<a|bytes>", "option <r|a>"),
            mk_instr!("SELF_ADDRESS", "", "", "<a|address>"),
            mk_instr!("SOURCE", "", "", "<a|address>"),
            mk_instr!("AMOUNT", "", "", "<a|mutez>"),
            mk_instr!("UNIT", "", "", "<a|unit>"),
            mk_instr!(
                "TRANSFER_TOKENS",
                "",
                "<w|a>;<a|mutez>;contract <r|a>",
                "<a|operation>"
            ),
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
