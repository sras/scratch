#[macro_use]
extern crate lazy_static;

mod instructions;
mod parser;
mod parsers;
mod typechecker;
mod types;
use types::*;

use typechecker::typecheck;

fn main() {
    let mut stack = Vec::new();

    match parser::InstructionListParser::new().parse(
        "LAMBDA nat (pair nat nat) {DUP;PAIR};PUSH nat 5;PUSH (pair nat int) (Pair 5 10);DROP",
    ) {
        Result::Ok(parsed_instructions) => match typecheck(&parsed_instructions, &mut stack) {
            Result::Ok(_) => {
                println!("{:?}", stack);
            }
            Err(s) => {
                println!("{}", s);
            }
        },
        Result::Err(s) => println!("{}", s),
    }
}

mod tests {
    use crate::parser::InstructionListParser;
    use crate::parsers::parse_stack;
    use crate::typechecker::typecheck;
    use crate::Instruction;
    use crate::CompoundInstruction;
    use crate::SomeValue;
    use crate::StackState;
    fn typecheck_<'a>(instructions: &Vec<CompoundInstruction<SomeValue>>) -> Result<StackState, &'a str> {
        let mut stack = Vec::new();
        typecheck(instructions, &mut stack)?;
        return Result::Ok(stack);
    }
    fn parse(src: &str) -> Vec<CompoundInstruction<SomeValue>> {
        let p = InstructionListParser::new();
        match p.parse(src) {
            Ok(s) => s,
            Err(e) => panic!("Parse failed {}", e),
        }
    }

    #[test]
    fn test_type_checking_simple() {
        // Type check behavior.

        assert!(Result::is_err(&typecheck_(&parse("PUSH nat \"asd\""))));
        assert!(Result::is_err(&typecheck_(&parse("PUSH (pair nat nat) 5"))));
        assert!(Result::is_err(&typecheck_(&parse(
            "PUSH (pair nat nat) (Pair 2 3);DROP;DROP"
        ))));
        assert!(Result::is_err(&typecheck_(&parse("PUSH nat 5;ADD"))));

        assert!(Result::is_err(&typecheck_(&parse(
            "LAMBDA nat (pair nat nat) {DUP;PAIR};PUSH int 5;EXEC"
        ))));

        assert!(Result::is_err(&typecheck_(&parse(
            "LAMBDA nat (pair nat nat) {DROP; PUSH int 1; DUP;PAIR};PUSH nat 5;EXEC"
        ))));

        // Stack result tests.

        assert_eq!(
            typecheck_(&parse("PUSH nat 5")).unwrap(),
            parse_stack("nat")
        );
        assert_eq!(
            typecheck_(&parse("PUSH (pair nat nat) (Pair 2 3)")).unwrap(),
            parse_stack("pair nat nat")
        );
        assert_eq!(
            typecheck_(&parse("PUSH (pair nat nat) (Pair 2 3);DROP")).unwrap(),
            parse_stack("")
        );
        assert_eq!(
            typecheck_(&parse("PUSH nat 5; PUSH nat 5;ADD")).unwrap(),
            parse_stack("nat")
        );

        assert_eq!(
            typecheck_(&parse("PUSH nat 5;DUP;DUP;DUP")).unwrap(),
            parse_stack("nat;nat;nat;nat")
        );
        assert_eq!(
            typecheck_(&parse("PUSH nat 5;DUP;DROP")).unwrap(),
            parse_stack("nat")
        );
        assert_eq!(
            typecheck_(&parse("PUSH (list nat) {5;6}")).unwrap(),
            parse_stack("list nat")
        );
        assert_eq!(
            typecheck_(&parse(
                "LAMBDA nat (pair nat nat) {DUP;PAIR};PUSH nat 5;EXEC"
            ))
            .unwrap(),
            parse_stack("pair nat nat")
        );

        assert_eq!(
            typecheck_(&parse("PUSH int 1;PUSH nat 1;SWAP")).unwrap(),
            parse_stack("int;nat")
        );
    }
}
