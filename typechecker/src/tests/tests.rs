use crate::parser::InstructionListParser;
use crate::parser::MDynListParser;
use crate::parsers::parse_contract;
use crate::parsers::parse_stack;
use crate::typechecker::typecheck;
use crate::typechecker::typecheck_contract;
use crate::types::mdyn_to_concrete;
use crate::types::CompoundInstruction;
use crate::types::ConcreteType;
use crate::types::MAtomic::*;
use crate::types::MType::*;
use crate::types::SomeValue;
use crate::types::StackState;
use crate::types::StackState::*;
use crate::types::TcEnv;
fn typecheck_<'a>(
    instructions: &Vec<CompoundInstruction<SomeValue>>,
) -> Result<StackState, String> {
    let mut stack = StackState::new();
    let tcenv: TcEnv = TcEnv {
        selfType: MWrapped(MUnit),
    };
    typecheck(&tcenv, instructions, &mut stack)?;
    return Result::Ok(stack);
}
fn parse(src: &str) -> Vec<CompoundInstruction<SomeValue>> {
    let p = InstructionListParser::new();
    match p.parse(src) {
        Ok(s) => s,
        Err(e) => panic!("Parse failed {}", e),
    }
}

fn parse_type(src: &str) -> ConcreteType {
    match parse_stack(src).get_live().unwrap()[..] {
        [ref x] => x.clone(),
        ref x => panic!("Got many types {:?}", x),
    }
}

#[test]
fn test_paring_behavior() {
    assert_eq!(parse_type("nat"), MWrapped(MNat));
    assert_eq!(parse_type("pair nat int"), MPair(Box::new((MWrapped(MNat), MWrapped(MInt)))));
    assert_eq!(parse_type("(pair nat int string)"), MPair(Box::new((MWrapped(MNat), MPair(Box::new((MWrapped(MInt), MWrapped(MString))))))));
    parse("PUSH nat 5");
    parse("PUSH string \"5 3\"");
    parse("IF { PUSH nat 0} {}");
}

fn test_type_checking_simple() {
    // Type checking behavior.

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

    assert!(Result::is_err(&typecheck_(&parse(
        "PUSH bool True; IF { PUSH nat 5 } { PUSH int 10 }"
    ))));

    assert!(Result::is_err(&typecheck_(&parse(
        "PUSH nat 1; IF { PUSH nat 5 } { PUSH nat 10 }"
    ))));
    assert!(Result::is_err(&typecheck_(&parse(
        "LAMBDA_REC nat nat { PUSH nat 1; ADD;};"
    ))));

    assert!(Result::is_err(&typecheck_(&parse(
        "PUSH (big_map nat nat) {Elt 2 3};"
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

    assert_eq!(
        typecheck_(&parse("PUSH bool True;")).unwrap(),
        parse_stack("bool")
    );

    assert_eq!(
        typecheck_(&parse("PUSH nat 5;DIP \n {PUSH bool True;}")).unwrap(),
        parse_stack("nat;bool")
    );

    assert_eq!(
        typecheck_(&parse("PUSH nat 5; PUSH int 10; DIP 2 {PUSH bool True;}")).unwrap(),
        parse_stack("int; nat ; bool")
    );

    assert_eq!(
        typecheck_(&parse("PUSH nat 5; PUSH int 10; DIIP {PUSH bool True;}")).unwrap(),
        parse_stack("int; nat ; bool")
    );

    assert_eq!(
        typecheck_(&parse("PUSH bool True; IF { PUSH nat 5 } { PUSH nat 10 }")).unwrap(),
        parse_stack("nat")
    );

    assert_eq!(
        typecheck_(&parse(
            "LAMBDA_REC nat nat { PUSH nat 1; ADD; DIP { DROP }};"
        ))
        .unwrap(),
        parse_stack("lambda nat nat")
    );

    assert_eq!(
        typecheck_(&parse(
            "LAMBDA (pair int nat) int { CAR }; PUSH int 10; APPLY;"
        ))
        .unwrap(),
        parse_stack("lambda nat int")
    );

    assert_eq!(
        typecheck_(&parse("PUSH nat 1; DUP; ADD")).unwrap(),
        parse_stack("nat")
    );

    assert_eq!(
        typecheck_(&parse("PUSH int 1; DUP; ADD")).unwrap(),
        parse_stack("int")
    );

    assert_eq!(
        typecheck_(&parse("PUSH (map nat nat) {Elt 2 3};")).unwrap(),
        parse_stack("map nat nat")
    );

    assert_eq!(
        typecheck_(&parse("PUSH nat :a 1;")).unwrap(),
        parse_stack("nat")
    );

    assert_eq!(
        typecheck_(&parse("PUSH %a %b %c nat :a 1;")).unwrap(),
        parse_stack("nat")
    );

    assert_eq!(
        typecheck_(&parse("SELF;")).unwrap(),
        parse_stack("contract unit")
    );

    assert_eq!(
        typecheck_(&parse(
            "PUSH %a %b %c (pair :point (nat %x) (nat %y)) (Pair 1 1);"
        ))
        .unwrap(),
        parse_stack("pair nat nat")
    );
}
