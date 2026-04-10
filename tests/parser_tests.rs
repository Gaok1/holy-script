mod common;
use holy_script::ast::*;
use holy_script::lexer::tokenize;
use holy_script::parser::Parser;

fn parse(source: &str) -> Program {
    Parser::new(tokenize(source))
        .parse_program()
        .unwrap_or_else(|e| panic!("parse error: {}", e))
}

fn parse_err(source: &str) -> String {
    Parser::new(tokenize(source))
        .parse_program()
        .expect_err("expected parse error")
        .to_string()
}

// ── Declarações ──────────────────────────────────────────────────

#[test]
fn parses_decl_with_value() {
    let p = parse("let there x of atom be 42\namen\n");
    assert!(matches!(
        p.stmts.first(),
        Some(Stmt::DeclVal { name, .. }) if name == "x"
    ));
}

#[test]
fn parses_decl_no_value() {
    let p = parse("let there be x of word\namen\n");
    assert!(matches!(
        p.stmts.first(),
        Some(Stmt::DeclNoVal { name, ty: Some(HolyType::Word) }) if name == "x"
    ));
}

// ── Expressões ───────────────────────────────────────────────────

#[test]
fn parses_negate() {
    let p = parse("let there x of atom be negate 5\namen\n");
    match p.stmts.first() {
        Some(Stmt::DeclVal { val, .. }) => {
            assert!(matches!(val, Expr::Negate(_)));
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn parses_remainder() {
    let p = parse("let there x of atom be 10 remainder 3\namen\n");
    match p.stmts.first() {
        Some(Stmt::DeclVal { val, .. }) => {
            assert!(matches!(val, Expr::BinOp { op: BinOp::Rem, .. }));
        }
        other => panic!("unexpected: {:?}", other),
    }
}

#[test]
fn parses_negate_remainder_precedence() {
    // negate 10 remainder 3  →  BinOp(Rem, Negate(10), 3)  — negate binds tighter
    let p = parse("let there x of atom be negate 10 remainder 3\namen\n");
    match p.stmts.first() {
        Some(Stmt::DeclVal { val, .. }) => assert!(matches!(
            val,
            Expr::BinOp { op: BinOp::Rem, left, .. }
            if matches!(&**left, Expr::Negate(_))
        )),
        other => panic!("unexpected: {:?}", other),
    }
}

// ── Covenant / Discern ───────────────────────────────────────────

#[test]
fn parses_covenant() {
    let p = parse("covenant Dir\n    Left\n    Right\namen\n");
    assert!(matches!(
        p.top_decls.first(),
        Some(TopDecl::Covenant { name, variants, .. })
            if name == "Dir"
            && variants.len() == 2
            && variants[0].name == "Left"
            && variants[1].name == "Right"
    ));
}

#[test]
fn parses_discern_with_otherwise() {
    let p = parse(
        "covenant D\n    A\n    B\n\
         discern x\n    as A\n        hail proclaim\n    otherwise\n        hail proclaim\namen\n",
    );
    assert!(matches!(
        p.stmts.first(),
        Some(Stmt::Discern { branches, otherwise, .. })
            if branches.len() == 1 && otherwise.is_some()
    ));
}

#[test]
fn discern_requires_at_least_one_branch() {
    let msg = parse_err("discern x\n    otherwise\n        hail proclaim\namen\n");
    assert!(msg.contains("discern"), "got: {msg}");
}

// ── Litany / Forsake / Ascend ────────────────────────────────────

#[test]
fn parses_forsake_and_ascend() {
    let p = parse(
        "litany for blessed\n    forsake\n\
         litany for blessed\n    ascend\namen\n",
    );
    assert!(matches!(p.stmts[0], Stmt::Litany { .. }));
    assert!(matches!(p.stmts[1], Stmt::Litany { .. }));
}

// ── Sin / Confess ─────────────────────────────────────────────────

#[test]
fn parses_sin_and_confess() {
    let p = parse(
        "sin Boom\n    msg of word\n\
         confess\n    transgress Boom praying \"x\"\nanswer for Boom as e\n    hail proclaim\namen\n",
    );
    assert!(matches!(p.top_decls.first(), Some(TopDecl::SinDecl { name, .. }) if name == "Boom"));
    assert!(matches!(p.stmts.first(), Some(Stmt::Confess { .. })));
}

// ── Parâmetros ───────────────────────────────────────────────────

#[test]
fn param_list_accepts_final_and() {
    let p = parse("salm f receiving x of atom and y of atom reveals void\n    reveal x\namen\n");
    assert!(matches!(
        p.top_decls.first(),
        Some(TopDecl::Salm { params, .. }) if params.len() == 2
    ));
}

#[test]
fn revealing_list_accepts_final_and() {
    let p = parse("testament Math revealing add and sub\namen\n");
    assert!(matches!(
        p.testaments.first(),
        Some(Testament { revealing: Some(items), .. }) if items == &vec!["add".to_string(), "sub".to_string()]
    ));
}

#[test]
fn bearing_list_accepts_final_and() {
    let p = parse(
        "covenant Pair\n    Both\n\
         discern value\n    as Both bearing left and right\n        hail proclaim\namen\n",
    );
    assert!(matches!(
        p.stmts.first(),
        Some(Stmt::Discern { branches, .. }) if branches[0].bindings == vec!["left".to_string(), "right".to_string()]
    ));
}

#[test]
fn type_param_list_accepts_final_and() {
    let p = parse("scripture Pair of A and B\n    first of A\n    second of B\namen\n");
    assert!(matches!(
        p.top_decls.first(),
        Some(TopDecl::Scripture { type_params, .. }) if type_params == &vec!["A".to_string(), "B".to_string()]
    ));
}

#[test]
fn type_arg_list_accepts_final_and() {
    let p = parse("let there x of Pair of atom and word be v\namen\n");
    assert!(matches!(
        p.stmts.first(),
        Some(Stmt::DeclVal { ty: HolyType::Generic(name, args), .. })
            if name == "Pair" && args.len() == 2
    ));
}

#[test]
fn invalid_and_then_comma_in_arg_list_is_rejected() {
    let msg = parse_err("hail add praying 1 and 2, 3\namen\n");
    assert!(!msg.is_empty(), "expected parse error");
}

#[test]
fn invalid_and_then_comma_in_type_args_is_rejected() {
    let msg = parse_err("let there x of Pair of atom and word, dogma be v\namen\n");
    assert!(!msg.is_empty(), "expected parse error");
}

#[test]
fn method_call_upon_accepts_expression_target() {
    let p = parse(
        "scripture List\n    buff of atom\n\
         salm show upon List reveals atom\n    reveal hail length upon buff from its\n\
         amen\n",
    );
    match p.top_decls.get(1) {
        Some(TopDecl::MethodSalm { body, .. }) => match body.first() {
            Some(Stmt::Reveal(Expr::MethodCall { target, .. })) => {
                assert!(matches!(&**target, Expr::FieldAccess { .. } | Expr::SelfFieldAccess { .. }));
            }
            other => panic!("unexpected body: {:?}", other),
        },
        other => panic!("unexpected decls: {:?}", other),
    }
}

// ── Erros esperados ──────────────────────────────────────────────

#[test]
fn error_on_missing_amen() {
    // tokenize não produz Amen → parse_program falha no expect(Amen)
    let err = Parser::new(tokenize("let there x of atom be 1\n"))
        .parse_program()
        .expect_err("should fail without amen");
    assert!(err.to_string().contains("'amen'"), "got: {}", err);
}

#[test]
fn error_on_empty_scripture() {
    // Sem campos e sem bloco indentado → erro de indentação esperada
    let msg = parse_err("scripture Empty\namen\n");
    assert!(msg.contains("indented") || msg.contains("block"), "got: {msg}");
}
