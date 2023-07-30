use std::collections::HashMap;

use crate::parsers::parse_constraints;
use crate::parsers::parse_stack_results;
use crate::types::InstructionDef;

macro_rules! mk_instr {
    ($n:expr, $arg: expr, $is: expr, $os: expr) => {
        (
            String::from($n),
            InstructionDef {
                args: parse_constraints($arg),
                input_stack: parse_constraints($is),
                output_stack: parse_stack_results($os),
            },
        )
    };
}

lazy_static! {
    pub static ref MICHELSON_INSTRUCTIONS: HashMap<String, InstructionDef> = HashMap::from([
        mk_instr!("SWAP", "", "<w|a>;<w|b>", "<r|b>;<r|a>"),
        mk_instr!("DUP", "", "<w|a>", "<r|a>;<r|a>"),
        mk_instr!("DROP", "", "<w|a>", ""),
        mk_instr!("ADD", "", "<w|a>;<r|a>", "<r|a>"),
        mk_instr!("CONS", "", "<w|a>;list <r|a>", "list <r|a>"),
        mk_instr!("PUSH", "<t|a>;<r|a>", "", "<r|a>"),
        mk_instr!("PAIR", "", "<w|a>;<w|b>", "pair <r|a> <r|b>"),
        mk_instr!(
            "LAMBDA",
            "<t|a>;<t|b>;lambda <r|a> <r|b>",
            "",
            "lambda <r|a> <r|b>"
        ),
        mk_instr!("EXEC", "", "<w|a>;lambda <r|a> <w|b>", "<r|b>"),
    ]);
}
