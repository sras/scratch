use crate::parser::InstructionListParser;
use crate::parsers::parse_stack;
use crate::typechecker::typecheck;
use crate::types::CompoundInstruction;
use crate::types::ConcreteType;
use crate::types::MAtomic;
use crate::types::MAtomic::*;
use crate::types::MType::*;
use crate::types::SomeValue;
use crate::types::StackDerived::*;
use crate::types::StackState;
use crate::types::TcEnv;
fn typecheck_(
    instructions: &Vec<CompoundInstruction<SomeValue>>,
) -> Result<StackState<MAtomic>, String> {
    let mut stack = StackState::new();
    let tcenv: TcEnv = TcEnv {
        self_type: MWrapped(MUnit),
    };
    typecheck(&tcenv, instructions, &mut stack)?;
    Result::Ok(stack)
}
fn parse(src: &str) -> Vec<CompoundInstruction<SomeValue>> {
    let p = InstructionListParser::new();
    match p.parse(src) {
        Ok(s) => s,
        Err(e) => panic!("Parse failed {}", e),
    }
}

fn parse_type(src: &str) -> ConcreteType {
    match parse_stack(src).get_index(0) {
        SdOk(Result::Ok(x)) => x.clone(),
        _ => panic!("Unexpected stack after parsing"),
    }
}

#[test]
fn dummy_2() {
    assert_eq!(
        typecheck_(&parse("PUSH nat 5; PUSH nat 5;ADD")).unwrap(),
        parse_stack("nat")
    );
}

#[test]
fn dummy() {
    let r = typecheck_(&parse(" LAMBDA
         (pair (pair (lambda (pair nat nat) nat)
                     (lambda (pair nat (pair nat nat) nat nat) (pair (pair nat nat) nat nat))
                     (lambda (pair nat (pair nat nat) nat nat) (pair (pair nat nat) nat nat)))
               (pair (pair nat address)
                     nat
                     (pair (pair (pair (pair address (pair (pair (pair (pair nat nat) int int) (pair int nat) nat nat) int))
                                       (big_map (pair address address) unit)
                                       (pair (pair (pair (pair nat nat) nat mutez)
                                                   (pair mutez (set address))
                                                   (big_map string string)
                                                   (big_map string bytes))
                                             nat
                                             nat))
                                 (pair (big_map address (pair (pair nat nat) nat nat)) nat)
                                 nat
                                 (pair address nat))
                           (pair (pair address (big_map string bytes))
                                 (option (pair bytes bytes (big_map (pair bytes bool) bytes)))
                                 address)
                           (pair nat
                                 (big_map bytes (pair (pair (pair nat bytes) address nat) (pair nat nat) nat nat)))
                           (pair (pair nat nat) nat)
                           (big_map (pair address bytes) nat))
                     nat))
         (pair (pair (pair (pair (pair address (pair (pair (pair (pair nat nat) int int) (pair int nat) nat nat) int))
                                 (big_map (pair address address) unit)
                                 (pair (pair (pair (pair nat nat) nat mutez)
                                             (pair mutez (set address))
                                             (big_map string string)
                                             (big_map string bytes))
                                       nat
                                       nat))
                           (pair (big_map address (pair (pair nat nat) nat nat)) nat)
                           nat
                           (pair address nat))
                     (pair (pair address (big_map string bytes))
                           (option (pair bytes bytes (big_map (pair bytes bool) bytes)))
                           address)
                     (pair nat
                           (big_map bytes (pair (pair (pair nat bytes) address nat) (pair nat nat) nat nat)))
                     (pair (pair nat nat) nat)
                     (big_map (pair address bytes) nat))
               nat)
         { UNPAIR ;
           UNPAIR 3 ;
           DIG 3 ;
           UNPAIR ;
           UNPAIR ;
           DIG 2 ;
           UNPAIR ;
           DUP 2 ;
           CDR ;
           PAIR ;
           DIG 4 ;
           SWAP ;
           EXEC ;
           DUP 3 ;
           DUP 3 ;
           CAR ;
           CDR ;
           CDR ;
           CDR ;
           CAR ;
           CDR ;
           ADD ;
           DUP 3 ;
           CAR ;
           CAR ;
           CDR ;
           CAR ;
           CAR ;
           DUP 6 ;
           GET ;
           IF_NONE
             { SWAP ;
               DIG 4 ;
               DIG 5 ;
               DIG 6 ;
               DROP 4 ;
               PUSH nat 0 ;
               DIG 3 ;
               COMPARE ;
               EQ ;
               IF { DUP 2 ; CAR ; CAR ; CDR ; CAR ; CAR } { PUSH nat 114 ; FAILWITH } }
             { DIG 2 ;
               PAIR ;
               DIG 5 ;
               SWAP ;
               EXEC ;
               DUP 4 ;
               PAIR ;
               DIG 5 ;
               SWAP ;
               EXEC ;
               DIG 3 ;
               DUP 2 ;
               CDR ;
               CDR ;
               ADD ;
               DUP 2 ;
               CDR ;
               CAR ;
               PAIR ;
               SWAP ;
               CAR ;
               PAIR ;
               DUP 3 ;
               CAR ;
               CAR ;
               CDR ;
               CAR ;
               CAR ;
               SWAP ;
               SOME ;
               DIG 4 ;
               UPDATE } ;
           DUP 3 ;
           CDR ;
           DUP 4 ;
           CAR ;
           CDR ;
           DUP 5 ;
           CAR ;
           CAR ;
           CDR ;
           CDR ;
           DUP 6 ;
           CAR ;
           CAR ;
           CDR ;
           CAR ;
           CDR ;
           DIG 4 ;
           PAIR ;
           PAIR ;
           DUP 5 ;
           CAR ;
           CAR ;
           CAR ;
           PAIR ;
           PAIR ;
           PAIR ;
           DUP ;
           CDR ;
           DUP 2 ;
           CAR ;
           CDR ;
           CDR ;
           CDR ;
           CDR ;
           DIG 3 ;
           DIG 4 ;
           CAR ;
           CDR ;
           CDR ;
           CDR ;
           CAR ;
           CAR ;
           PAIR ;
           PAIR ;
           DUP 3 ;
           CAR ;
           CDR ;
           CDR ;
           CAR ;
           PAIR ;
           DUP 3 ;
           CAR ;
           CDR ;
           CAR ;
           PAIR ;
           DIG 2 ;
           CAR ;
           CAR ;
           PAIR ;
           PAIR }"));
    match r {
        Ok(_) => {}
        Err(s) => println!("{}", s),
    }
}

#[test]
fn test_paring_behavior() {
    assert_eq!(parse_type("nat"), MWrapped(MNat));
    assert_eq!(
        parse_type("pair nat int"),
        MPair(Box::new((MWrapped(MNat), MWrapped(MInt))))
    );
    assert_eq!(
        parse_type("(pair nat int string)"),
        MPair(Box::new((
            MWrapped(MNat),
            MPair(Box::new((MWrapped(MInt), MWrapped(MString))))
        )))
    );
    parse("PUSH nat 5");
    parse("PUSH string \"5 3\"");
    parse("PUSH %something string \"5 3\"");
    parse("UNPAIR");
    parse("CAR %something 2");
    parse("UNPAIR %something %something 2");
    parse("DUP");
    parse("DUP 2");
    parse("IF { PUSH nat 0} {}");
    parse(
        r#"LAMBDA (pair (pair (set address) (list address)) (lambda (pair (set address) address) (set address))) (set address) {}"#,
    );
    parse_type("(pair nat int string)");
    parse_type("(nat %counter)");
    assert_eq!(
        parse_type(
            r#"(pair (pair (set address) (list address)) (lambda (pair (set address) address) (set address)))"#
        ),
        MPair(Box::new((
            MPair(Box::new((
                MSet(Box::new(MWrapped(MAddress))),
                MList(Box::new(MWrapped(MAddress)))
            ))),
            MLambda(Box::new((
                MPair(Box::new((
                    MSet(Box::new(MWrapped(MAddress))),
                    MWrapped(MAddress)
                ))),
                MSet(Box::new(MWrapped(MAddress)))
            )))
        )))
    );
}

#[test]
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
        typecheck_(&parse("SELF_ADDRESS")).unwrap(),
        parse_stack("address")
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
        typecheck_(&parse("PUSH nat 5; PUSH nat 5")).unwrap(),
        parse_stack("nat; nat")
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

    assert_eq!(
        typecheck_(&parse("PUSH bool True; IF { PUSH int 1 } { PUSH int 5 }")).unwrap(),
        parse_stack("int")
    );

    assert_eq!(
        typecheck_(&parse("PUSH bool False; IF { PUSH int 1 } { FAIL }")).unwrap(),
        parse_stack("int")
    );

    assert!(Result::is_err(&typecheck_(&parse(
        "PUSH int 1; IF { PUSH int 1 } { FAIL }"
    ))));

    assert_eq!(
        typecheck_(&parse(
            "PUSH (option nat) (Some 1); IF_SOME { DROP; PUSH int 1 } { PUSH int 5 }"
        ))
        .unwrap(),
        parse_stack("int")
    );

    assert_eq!(
        typecheck_(&parse(
            "PUSH (option nat) (Some 1); IF_NONE  { PUSH int 5 } { DROP; PUSH int 1 }"
        ))
        .unwrap(),
        parse_stack("int")
    );

    assert_eq!(
        typecheck_(&parse("PUSH (or int nat) (Left 1); IF_LEFT  { PUSH int 1; ADD; DROP; PUSH int 1; } { PUSH nat 1; ADD; DROP; PUSH int 2; }")).unwrap(),
        parse_stack("int")
    );

    assert_eq!(
        typecheck_(&parse(
            "PUSH int 1; PUSH (list nat) {1;2;3}; ITER { PUSH nat 1; ADD; DROP; } "
        ))
        .unwrap(),
        parse_stack("int")
    );

    assert_eq!(
        typecheck_(&parse(
            "PUSH int 1; PUSH (set nat) {1;2;3}; ITER { PUSH nat 1; ADD; DROP; } "
        ))
        .unwrap(),
        parse_stack("int")
    );

    assert_eq!(
        typecheck_(&parse("PUSH int 1; PUSH nat 1; PUSH mutez 0;DIG 2;")).unwrap(),
        parse_stack("int;mutez;nat")
    );

    assert_eq!(
        typecheck_(&parse(r#"PUSH int 1; PUSH (map nat string) {Elt 1 "one";Elt 2 "two";Elt 3 "Three"}; ITER { CAR; PUSH nat 1; ADD; DROP; } "#)).unwrap(),
        parse_stack("int")
    );

    assert_eq!(typecheck_(&parse("LAMBDA
         (pair (pair (set address) (list address)) (lambda (pair (set address) address) (set address)))
         (set address)
         { UNPAIR @counter ;
           UNPAIR ;
           SWAP ;
           ITER { SWAP ; PAIR ; DUP 2 ; SWAP ; EXEC } ;
           SWAP ;
           DROP }")).unwrap(), parse_stack("lambda (pair (pair (set address) (list address)) (lambda (pair (set address) address) (set address))) (set address)"));

    assert_eq!(
        typecheck_(&parse(
            "PUSH nat 1; PUSH int 1; PUSH mutez 0; PUSH nat 5; PAIR 3;"
        ))
        .unwrap(),
        parse_stack("pair nat (pair mutez int);nat")
    );

    assert_eq!(
        typecheck_(&parse(
            r#"PUSH (pair nat (pair int (pair string mutez))) (Pair 0 (Pair 1 (Pair "some" 2))); UNPAIR 2;"#
        ))
        .unwrap(),
        parse_stack("nat; pair int (pair string mutez)")
    );

    assert_eq!(
        typecheck_(&parse(
            r#"PUSH (pair nat  (pair int (pair string mutez))) (Pair 0 (Pair 1 (Pair "some" 2))); UNPAIR 3;"#
        ))
        .unwrap(),
        parse_stack("nat; int; pair string mutez")
    );

    assert_eq!(
        typecheck_(&parse("PUSH nat 1; PUSH int 1; PUSH mutez 0; DROP 1;")).unwrap(),
        parse_stack("int; nat")
    );

    assert_eq!(
        typecheck_(&parse("PUSH nat 1; PUSH int 1; PUSH mutez 0; DROP 2;")).unwrap(),
        parse_stack("nat")
    );

    assert_eq!(
        typecheck_(&parse(r#"PUSH (pair nat (pair int (pair string mutez))) (Pair 0 (Pair 1 (Pair "some" 2))); GET 0"#)).unwrap(),
        parse_stack("pair nat (pair int (pair string mutez))")
    );

    assert_eq!(
        typecheck_(&parse(r#"PUSH (pair nat (pair int (pair string mutez))) (Pair 0 (Pair 1 (Pair "some" 2))); GET 1"#)).unwrap(),
        parse_stack("nat")
    );

    assert_eq!(
        typecheck_(&parse(r#"PUSH (pair nat (pair int (pair string mutez))) (Pair 0 (Pair 1 (Pair "some" 2))); GET 2"#)).unwrap(),
        parse_stack("pair int (pair string mutez)")
    );

    assert_eq!(
        typecheck_(&parse(r#"PUSH (pair nat (pair int (pair string mutez))) (Pair 0 (Pair 1 (Pair "some" 2))); GET 3"#)).unwrap(),
        parse_stack("int")
    );

    assert_eq!(
        typecheck_(&parse(r#"PUSH (pair nat (pair int (pair string mutez))) (Pair 0 (Pair 1 (Pair "some" 2))); GET 4"#)).unwrap(),
        parse_stack("pair string mutez")
    );

    assert_eq!(
        typecheck_(&parse(r#"PUSH (pair nat (pair int (pair string mutez))) (Pair 0 (Pair 1 (Pair "some" 2))); GET 5"#)).unwrap(),
        parse_stack("string")
    );

    assert_eq!(
        typecheck_(&parse(r#"PUSH (pair nat (pair int (pair string mutez))) (Pair 0 (Pair 1 (Pair "some" 2))); GET 6"#)).unwrap(),
        parse_stack("mutez")
    );

    assert_eq!(
        typecheck_(&parse(
            r#"PUSH (or nat int) (Left 10); LOOP_LEFT {DROP; PUSH int 10; RIGHT nat;}"#
        ))
        .unwrap(),
        parse_stack("int")
    );

    assert_eq!(
        typecheck_(&parse(
            r#"PUSH nat 5; PUSH (map nat nat) {}; MAP {DROP;PUSH int 1;}"#
        ))
        .unwrap(),
        parse_stack("map nat int; nat")
    );

    assert_eq!(
        typecheck_(&parse(
            r#"PUSH nat 5; PUSH (option nat) (Some 2); MAP {DROP;PUSH int 1;}"#
        ))
        .unwrap(),
        parse_stack("option int; nat")
    );

    assert_eq!(
        typecheck_(&parse(
            r#"PUSH nat 5; PUSH (list nat) ({}); MAP {DROP;PUSH int 1;}"#
        ))
        .unwrap(),
        parse_stack("list int; nat")
    );
}
