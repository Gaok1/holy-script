mod common;
use common::*;

// ── Aritmética ───────────────────────────────────────────────────

#[test]
fn arithmetic_basic() {
    let i = run("let there x of atom be 2 plus 3 times 4\namen\n");
    assert_eq!(get_int(&i, "x"), 14); // precedência: 2 + (3 * 4)
}

#[test]
fn arithmetic_remainder() {
    let i = run("let there x of atom be 10 remainder 3\namen\n");
    assert_eq!(get_int(&i, "x"), 1);
}

#[test]
fn arithmetic_negate() {
    let i = run("let there x of atom be negate 7\namen\n");
    assert_eq!(get_int(&i, "x"), -7);
}

#[test]
fn string_concat_with_plus() {
    let i = run(r#"let there s of word be "hello" plus " world"
amen
"#);
    assert_eq!(get_str(&i, "s"), "hello world");
}

#[test]
fn division_by_zero_becomes_builtin_sin() {
    let msg = run_err("let there x of atom be 1 over 0\namen\n");
    assert!(msg.contains("DivisionByZero"), "got: {msg}");
}

#[test]
fn confess_catches_builtin_division_by_zero() {
    let i = run(r#"let there caught of word be "no"

confess
    let there x of atom be 1 over 0
answer for DivisionByZero as err
    caught become message from err
amen
"#);
    assert_eq!(get_str(&i, "caught"), "division by zero");
}

// ── Condicional ──────────────────────────────────────────────────

#[test]
fn whether_executes_true_branch() {
    let i = run(r#"let there r of word be "no"
whether blessed
    r become "yes"
amen
"#);
    assert_eq!(get_str(&i, "r"), "yes");
}

#[test]
fn whether_skips_false_branch() {
    let i = run(r#"let there r of word be "no"
whether forsaken
    r become "yes"
amen
"#);
    assert_eq!(get_str(&i, "r"), "no");
}

#[test]
fn otherwise_so_chain() {
    let i = run(r#"let there x of atom be 2
let there r of word be ""
whether x is 1
    r become "one"
otherwise so x is 2
    r become "two"
otherwise
    r become "other"
amen
"#);
    assert_eq!(get_str(&i, "r"), "two");
}

// ── Litany / Forsake / Ascend ────────────────────────────────────

#[test]
fn litany_counts_to_five() {
    let i = run(r#"let there i of atom be 0
litany for i no greater than 5
    i become i plus 1
amen
"#);
    assert_eq!(get_int(&i, "i"), 6);
}

#[test]
fn forsake_exits_loop() {
    let i = run(r#"let there i of atom be 0
litany for i no greater than 100
    i become i plus 1
    whether i is 5
        forsake
amen
"#);
    assert_eq!(get_int(&i, "i"), 5);
}

#[test]
fn ascend_skips_body() {
    // soma só os ímpares de 1 a 9
    let i = run(r#"let there i of atom be 0
let there sum of atom be 0
litany for i no greater than 9
    i become i plus 1
    whether i remainder 2 is 0
        ascend
    sum become sum plus i
amen
"#);
    assert_eq!(get_int(&i, "sum"), 25); // 1+3+5+7+9
}

// ── Scripture ────────────────────────────────────────────────────

#[test]
fn scripture_manifest_and_field_access() {
    let i = run(r#"scripture Point
    x of atom
    y of atom

let there p of Point be manifest Point praying 3, 4
let there px of atom be x from p
amen
"#);
    assert_eq!(get_int(&i, "px"), 3);
}

#[test]
fn undefined_custom_field_type_becomes_builtin_sin() {
    let msg = run_err(r#"scripture id
    a of parse

hail proclaim praying "passei"

let there var of id be manifest id praying "number"

hail proclaim praying a from var

amen
"#);
    assert!(msg.contains("UndefinedType"), "got: {msg}");
    assert!(msg.contains("parse"), "got: {msg}");
}

#[test]
fn manifest_checks_field_type() {
    let msg = run_err(r#"scripture Id
    a of atom

let there var of Id be manifest Id praying "number"
amen
"#);
    assert!(msg.contains("TypeError"), "got: {msg}");
    assert!(msg.contains("Id.a"), "got: {msg}");
}

// ── Salm ─────────────────────────────────────────────────────────

#[test]
fn salm_add_returns_sum() {
    let i = run(r#"salm add receiving a of atom, b of atom reveals atom
    reveal a plus b

let there r of atom be hail add praying 10, 32
amen
"#);
    assert_eq!(get_int(&i, "r"), 42);
}

#[test]
fn salm_params_and_args_accept_final_and() {
    let i = run(r#"salm add receiving a of atom and b of atom reveals atom
    reveal a plus b

let there r of atom be hail add praying 10 and 32
amen
"#);
    assert_eq!(get_int(&i, "r"), 42);
}

#[test]
fn method_salm_accesses_its() {
    let i = run(r#"scripture Box
    value of atom

salm doubled upon Box reveals atom
    reveal value from its times 2

let there b of Box be manifest Box praying 21
let there r of atom be hail doubled upon b
amen
"#);
    assert_eq!(get_int(&i, "r"), 42);
}

#[test]
fn salm_argument_type_is_checked() {
    let msg = run_err(r#"salm add_one receiving a of atom reveals atom
    reveal a plus 1

let there r of atom be hail add_one praying "oops"
amen
"#);
    assert!(msg.contains("TypeError"), "got: {msg}");
    assert!(msg.contains("parameter 'a'"), "got: {msg}");
}

// ── Sin / Confess ─────────────────────────────────────────────────

#[test]
fn confess_catches_sin() {
    let i = run(r#"sin Boom
    msg of word

let there caught of word be "no"

confess
    transgress Boom praying "fire!"
answer for Boom as e
    caught become msg from e
amen
"#);
    assert_eq!(get_str(&i, "caught"), "fire!");
}

#[test]
fn transgress_accepts_final_and() {
    let i = run(r#"sin PairErr
    left of word
    right of word

let there caught of word be ""

confess
    transgress PairErr praying "alpha" and "beta"
answer for PairErr as e
    caught become left from e plus ":" plus right from e
amen
"#);
    assert_eq!(get_str(&i, "caught"), "alpha:beta");
}

#[test]
fn absolve_runs_after_sin() {
    let i = run(r#"sin Err

let there done of dogma be forsaken

confess
    transgress Err
answer for Err
    hail proclaim praying "caught"
absolve
    done become blessed
amen
"#);
    assert!(get_bool(&i, "done"));
}

#[test]
fn unhandled_sin_propagates() {
    let msg = run_err(r#"sin A
sin B

confess
    transgress A
answer for B
    hail proclaim praying "b"
amen
"#);
    assert!(msg.contains("A"), "got: {msg}");
}

#[test]
fn confess_catches_builtin_undefined_variable() {
    let i = run(r#"let there caught of word be "no"

confess
    let there x of atom be missing plus 1
answer for UndefinedVariable as err
    caught become message from err
amen
"#);
    assert!(get_str(&i, "caught").contains("missing"));
}

// ── Covenant / Discern ───────────────────────────────────────────

#[test]
fn discern_matches_variant() {
    let i = run(r#"covenant Color
    Red
    Blue

let there c of Color be Blue
let there r of word be ""

discern c
    as Red
        r become "red"
    as Blue
        r become "blue"
amen
"#);
    assert_eq!(get_str(&i, "r"), "blue");
}

#[test]
fn discern_falls_through_to_otherwise() {
    let i = run(r#"covenant Dir
    North
    South

let there d of Dir be South
let there r of word be ""

discern d
    as North
        r become "north"
    otherwise
        r become "other"
amen
"#);
    assert_eq!(get_str(&i, "r"), "other");
}

#[test]
fn discern_no_match_no_otherwise_becomes_builtin_sin() {
    let msg = run_err(r#"covenant X
    A
    B

let there v of X be B

discern v
    as A
        hail proclaim praying "a"
amen
"#);
    assert!(msg.contains("InvalidDiscern"), "got: {msg}");
}

// ── Built-in generics: grace and verdict ─────────────────────────

#[test]
fn grace_granted_holds_value() {
    let i = run(r#"
let there g of grace of atom be manifest granted of grace of atom praying 42
let there out of atom be 0

discern g
    as granted bearing v
        out become v
    as absent
        out become -1
amen
"#);
    assert_eq!(get_int(&i, "out"), 42);
}

#[test]
fn grace_absent_is_default() {
    let i = run(r#"
let there be g of grace of atom
let there out of word be ""

discern g
    as granted bearing v
        out become "has value"
    as absent
        out become "empty"
amen
"#);
    assert_eq!(get_str(&i, "out"), "empty");
}

#[test]
fn verdict_righteous_holds_value() {
    let i = run(r#"
let there r of verdict of atom, word be manifest righteous of verdict of atom, word praying 99
let there out of atom be 0

discern r
    as righteous bearing v
        out become v
    as condemned bearing msg
        out become 0
amen
"#);
    assert_eq!(get_int(&i, "out"), 99);
}

#[test]
fn verdict_condemned_holds_reason() {
    let i = run(r#"
let there r of verdict of atom, word be manifest condemned of verdict of atom, word praying "bad"
let there out of word be ""

discern r
    as righteous bearing v
        out become "ok"
    as condemned bearing msg
        out become msg
amen
"#);
    assert_eq!(get_str(&i, "out"), "bad");
}

#[test]
fn grace_granted_type_mismatch_is_error() {
    let msg = run_err(r#"
let there g of grace of atom be manifest granted of grace of atom praying "wrong"
amen
"#);
    assert!(msg.contains("TypeError"), "got: {msg}");
}

#[test]
fn verdict_condemned_type_mismatch_is_error() {
    let msg = run_err(r#"
let there r of verdict of atom, word be manifest condemned of verdict of atom, word praying 42
amen
"#);
    assert!(msg.contains("TypeError"), "got: {msg}");
}

// ── Covenant data variants ────────────────────────────────────────

#[test]
fn covenant_data_variant_manifest_and_bearing() {
    let i = run(r#"covenant Result
    Ok
        value of atom
    Err
        message of word

let there r of Result be manifest Ok praying 42
let there out of atom be 0

discern r
    as Ok bearing v
        out become v
    as Err bearing msg
        out become 0
amen
"#);
    assert_eq!(get_int(&i, "out"), 42);
}

#[test]
fn generic_types_accept_final_and() {
    let i = run(r#"scripture Pair of A and B
    first of A
    second of B

let there p of Pair of atom and word be manifest Pair praying 7 and "ok"
let there out of word be second from p
amen
"#);
    assert_eq!(get_str(&i, "out"), "ok");
}

#[test]
fn covenant_data_variant_err_branch() {
    let i = run(r#"covenant Result
    Ok
        value of atom
    Err
        message of word

let there r of Result be manifest Err praying "fail"
let there out of word be ""

discern r
    as Ok bearing v
        out become "ok"
    as Err bearing msg
        out become msg
amen
"#);
    assert_eq!(get_str(&i, "out"), "fail");
}

#[test]
fn covenant_unit_variant_still_works_alongside_data_variant() {
    let i = run(r#"covenant Shape
    Circle
        radius of fractional
    Dot

let there s of Shape be Dot
let there out of word be ""

discern s
    as Circle bearing r
        out become "circle"
    as Dot
        out become "dot"
amen
"#);
    assert_eq!(get_str(&i, "out"), "dot");
}

#[test]
fn covenant_data_variant_type_error_becomes_builtin_sin() {
    let msg = run_err(r#"covenant Box
    Wrap
        value of atom

let there b of Box be manifest Wrap praying "wrong"
amen
"#);
    assert!(msg.contains("TypeError"), "got: {msg}");
}

#[test]
fn covenant_unit_variant_via_manifest_is_error() {
    let msg = run_err(r#"covenant Tag
    Plain
    Rich
        label of word

let there t of Tag be manifest Plain praying "oops"
amen
"#);
    assert!(msg.contains("InvalidArgumentCount"), "got: {msg}");
}

#[test]
fn covenant_data_variant_without_manifest_is_error() {
    let msg = run_err(r#"covenant Option
    Some
        value of atom
    None

let there o of Option be Some
amen
"#);
    assert!(msg.contains("InvalidArgumentCount"), "got: {msg}");
}
