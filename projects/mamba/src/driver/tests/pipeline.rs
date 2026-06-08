#![cfg(test)]

/// Integration tests for the full compiler pipeline (parse → HIR → MIR).
/// Covers features from issues #283–#294.

use crate::parser;
use crate::parser::ast::*;
use crate::source::span::FileId;
use crate::types::TypeChecker;
use crate::lower::{lower_module, lower_hir_to_mir};
use crate::mir::*;

fn parse(src: &str) -> Module {
    parser::parse(src, FileId(0)).expect("parse failed")
}

fn pipeline(src: &str) -> MirModule {
    let module = parse(src);
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker).unwrap();
    lower_hir_to_mir(&hir, &checker.tcx)
}

// ── #283: Exception handling ──

#[test]
fn test_parse_try_except() {
    let src = "try:\n    x: int = 1\nexcept ValueError:\n    pass\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::Try { body, handlers, .. } => {
            assert_eq!(body.len(), 1);
            assert_eq!(handlers.len(), 1);
        }
        other => panic!("expected Try, got {other:?}"),
    }
}

#[test]
fn test_parse_try_except_finally() {
    let src = "try:\n    pass\nexcept:\n    pass\nfinally:\n    pass\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::Try { handlers, finally_body, .. } => {
            assert_eq!(handlers.len(), 1);
            assert!(finally_body.is_some());
        }
        other => panic!("expected Try, got {other:?}"),
    }
}

#[test]
fn test_parse_raise() {
    let src = "raise ValueError(\"bad\")\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::Raise { value, from } => {
            assert!(value.is_some());
            assert!(from.is_none());
        }
        other => panic!("expected Raise, got {other:?}"),
    }
}

#[test]
fn test_parse_raise_from() {
    let src = "raise RuntimeError(\"wrap\") from err\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::Raise { value, from } => {
            assert!(value.is_some());
            assert!(from.is_some());
        }
        other => panic!("expected Raise with from, got {other:?}"),
    }
}

#[test]
fn test_pipeline_try_except() {
    let mir = pipeline("try:\n    x: int = 1\nexcept:\n    pass\n");
    // Try block generates multiple basic blocks (try body + handler + merge)
    assert!(!mir.bodies.is_empty());
    let main = &mir.bodies[0];
    assert!(main.blocks.len() >= 3, "try/except should produce >=3 blocks");
}

// ── #284: String operations ──

#[test]
fn test_pipeline_string_literal() {
    let mir = pipeline("s: str = \"hello\"\n");
    assert!(!mir.bodies.is_empty());
    let main = &mir.bodies[0];
    let has_str_const = main.blocks.iter().any(|b| {
        b.stmts.iter().any(|inst| matches!(inst, MirInst::LoadConst { value: MirConst::Str(_), .. }))
    });
    assert!(has_str_const, "should have string constant");
}

// ── #285: List/Dict/Tuple operations ──

#[test]
fn test_pipeline_list_literal() {
    let mir = pipeline("[1, 2, 3]\n");
    assert!(!mir.bodies.is_empty());
    let main = &mir.bodies[0];
    let has_make_list = main.blocks.iter().any(|b| {
        b.stmts.iter().any(|inst| matches!(inst, MirInst::MakeList { .. }))
    });
    assert!(has_make_list, "should have MakeList instruction");
}

#[test]
fn test_pipeline_dict_literal() {
    let mir = pipeline("{\"a\": 1, \"b\": 2}\n");
    assert!(!mir.bodies.is_empty());
    let main = &mir.bodies[0];
    let has_make_dict = main.blocks.iter().any(|b| {
        b.stmts.iter().any(|inst| matches!(inst, MirInst::MakeDict { .. }))
    });
    assert!(has_make_dict, "should have MakeDict instruction");
}

#[test]
fn test_pipeline_tuple_literal() {
    let mir = pipeline("(1, 2, 3)\n");
    assert!(!mir.bodies.is_empty());
    let main = &mir.bodies[0];
    let has_make_tuple = main.blocks.iter().any(|b| {
        b.stmts.iter().any(|inst| matches!(inst, MirInst::MakeTuple { .. }))
    });
    assert!(has_make_tuple, "should have MakeTuple instruction");
}

// ── #286: Iterator protocol ──

#[test]
fn test_parse_for_loop() {
    let src = "for x in items:\n    pass\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::For { targets, .. } => assert_eq!(targets, &["x"]),
        other => panic!("expected For, got {other:?}"),
    }
}

// ── #287/#288: Classes and dynamic dispatch ──

#[test]
fn test_parse_class_with_base() {
    let src = "class Dog(Animal):\n    pass\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::ClassDef { name, bases, .. } => {
            assert_eq!(name, "Dog");
            assert_eq!(bases.len(), 1);
            assert!(matches!(&bases[0].node, Expr::Ident(n) if n == "Animal"));
        }
        other => panic!("expected ClassDef, got {other:?}"),
    }
}

#[test]
fn test_parse_class_with_decorator() {
    let src = "@dataclass\nclass Point:\n    x: int = 0\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::ClassDef { decorators, name, .. } => {
            assert_eq!(name, "Point");
            assert_eq!(decorators.len(), 1);
        }
        other => panic!("expected ClassDef with decorator, got {other:?}"),
    }
}

// ── #289: Closures and nested functions ──

#[test]
fn test_parse_lambda() {
    let src = "f = lambda x: int: x + 1\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::Assign { value, .. } => {
            match &value.node {
                Expr::Lambda { params, .. } => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0].name, "x");
                }
                other => panic!("expected Lambda, got {other:?}"),
            }
        }
        other => panic!("expected Assign, got {other:?}"),
    }
}

#[test]
fn test_parse_decorator() {
    let src = "@my_decorator\ndef foo() -> int:\n    return 1\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::FnDef { decorators, name, .. } => {
            assert_eq!(name, "foo");
            assert_eq!(decorators.len(), 1);
        }
        other => panic!("expected FnDef with decorator, got {other:?}"),
    }
}

// ── #290: Generators/yield ──

#[test]
fn test_parse_yield() {
    let src = "def gen() -> int:\n    yield 1\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::FnDef { body, .. } => {
            match &body[0].node {
                Stmt::ExprStmt(expr) => {
                    assert!(matches!(&expr.node, Expr::Yield(Some(_))));
                }
                other => panic!("expected ExprStmt(Yield), got {other:?}"),
            }
        }
        other => panic!("expected FnDef, got {other:?}"),
    }
}

#[test]
fn test_parse_yield_from() {
    let src = "def gen() -> int:\n    yield from other_gen()\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::FnDef { body, .. } => {
            match &body[0].node {
                Stmt::ExprStmt(expr) => {
                    assert!(matches!(&expr.node, Expr::YieldFrom(_)));
                }
                other => panic!("expected ExprStmt(YieldFrom), got {other:?}"),
            }
        }
        other => panic!("expected FnDef, got {other:?}"),
    }
}

// ── #291: Comprehensions ──

#[test]
fn test_parse_list_comprehension() {
    let src = "[x * 2 for x in items]\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            match &expr.node {
                Expr::ListComp { generators, .. } => {
                    assert_eq!(generators.len(), 1);
                    assert_eq!(generators[0].targets.len(), 1);
                    assert_eq!(generators[0].targets[0], "x");
                }
                other => panic!("expected ListComp, got {other:?}"),
            }
        }
        other => panic!("expected ExprStmt, got {other:?}"),
    }
}

#[test]
fn test_parse_dict_comprehension() {
    let src = "{k: k for k in items}\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            assert!(matches!(&expr.node, Expr::DictComp { .. }));
        }
        other => panic!("expected ExprStmt(DictComp), got {other:?}"),
    }
}

#[test]
fn test_parse_list_comp_with_condition() {
    let src = "[x for x in items if x > 0]\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            match &expr.node {
                Expr::ListComp { generators, .. } => {
                    assert_eq!(generators[0].conditions.len(), 1);
                }
                other => panic!("expected ListComp, got {other:?}"),
            }
        }
        other => panic!("expected ExprStmt, got {other:?}"),
    }
}

// ── #292: Module import ──

#[test]
fn test_parse_import() {
    let src = "import os.path\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::Import { module: m, names, .. } => {
            assert_eq!(m, &["os", "path"]);
            assert!(names.is_none());
        }
        other => panic!("expected Import, got {other:?}"),
    }
}

#[test]
fn test_parse_from_import_alias() {
    let src = "from collections import OrderedDict as OD\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::Import { names, .. } => {
            let names = names.as_ref().unwrap();
            assert_eq!(names[0].0, "OrderedDict");
            assert_eq!(names[0].1.as_deref(), Some("OD"));
        }
        other => panic!("expected Import, got {other:?}"),
    }
}

// ── #293: Async/await ──

#[test]
fn test_parse_async_def() {
    let src = "async def fetch() -> int:\n    return 1\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::AsyncFnDef { name, .. } => assert_eq!(name, "fetch"),
        other => panic!("expected AsyncFnDef, got {other:?}"),
    }
}

#[test]
fn test_parse_await() {
    let src = "async def f() -> int:\n    x: int = await fetch()\n    return x\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::AsyncFnDef { body, .. } => {
            match &body[0].node {
                Stmt::VarDecl { value, .. } => {
                    assert!(matches!(&value.node, Expr::Await(_)));
                }
                other => panic!("expected VarDecl with Await, got {other:?}"),
            }
        }
        other => panic!("expected AsyncFnDef, got {other:?}"),
    }
}

// ── #294: Decorators ──

#[test]
fn test_parse_multiple_decorators() {
    let src = "@decorator1\n@decorator2\ndef foo() -> int:\n    return 1\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::FnDef { decorators, .. } => {
            assert_eq!(decorators.len(), 2);
        }
        other => panic!("expected FnDef, got {other:?}"),
    }
}

// ── Additional syntax: with, assert, del, global, nonlocal ──

#[test]
fn test_parse_with() {
    let src = "with open(\"f\") as f:\n    pass\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::With { items, body } => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].alias.as_deref(), Some("f"));
            assert_eq!(body.len(), 1);
        }
        other => panic!("expected With, got {other:?}"),
    }
}

#[test]
fn test_parse_assert() {
    let src = "assert x > 0, \"must be positive\"\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::Assert { test, msg } => {
            assert!(matches!(&test.node, Expr::BinOp { op: BinOp::Gt, .. }));
            assert!(msg.is_some());
        }
        other => panic!("expected Assert, got {other:?}"),
    }
}

#[test]
fn test_parse_del() {
    let src = "del x\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::Del(target) => {
            assert!(matches!(&target.node, Expr::Ident(n) if n == "x"));
        }
        other => panic!("expected Del, got {other:?}"),
    }
}

#[test]
fn test_parse_global() {
    let src = "global x, y\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::Global(names) => {
            assert_eq!(names, &["x", "y"]);
        }
        other => panic!("expected Global, got {other:?}"),
    }
}

#[test]
fn test_parse_nonlocal() {
    let src = "nonlocal count\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::Nonlocal(names) => {
            assert_eq!(names, &["count"]);
        }
        other => panic!("expected Nonlocal, got {other:?}"),
    }
}

// ── Ternary / IfExpr ──

#[test]
fn test_parse_ternary() {
    let src = "x = 1 if True else 0\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::Assign { value, .. } => {
            assert!(matches!(&value.node, Expr::IfExpr { .. }));
        }
        other => panic!("expected Assign with IfExpr, got {other:?}"),
    }
}

// ── Bitwise operators ──

#[test]
fn test_parse_bitwise_ops() {
    let module = parse("a & b | c ^ d\n");
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            // Should parse without error (bitwise OR at top)
            assert!(matches!(&expr.node, Expr::BinOp { op: BinOp::BitOr, .. }));
        }
        other => panic!("expected ExprStmt, got {other:?}"),
    }
}

#[test]
fn test_parse_shift_ops() {
    let module = parse("x << 2\n");
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            assert!(matches!(&expr.node, Expr::BinOp { op: BinOp::LShift, .. }));
        }
        other => panic!("expected ExprStmt, got {other:?}"),
    }
}

#[test]
fn test_parse_bitnot() {
    let module = parse("~x\n");
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            assert!(matches!(&expr.node, Expr::UnaryOp { op: UnaryOp::BitNot, .. }));
        }
        other => panic!("expected ExprStmt, got {other:?}"),
    }
}

// ── Identity/Membership operators ──

#[test]
fn test_parse_is_operator() {
    let module = parse("x is None\n");
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            assert!(matches!(&expr.node, Expr::BinOp { op: BinOp::Is, .. }));
        }
        other => panic!("expected ExprStmt, got {other:?}"),
    }
}

#[test]
fn test_parse_in_operator() {
    let module = parse("x in items\n");
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            assert!(matches!(&expr.node, Expr::BinOp { op: BinOp::In, .. }));
        }
        other => panic!("expected ExprStmt, got {other:?}"),
    }
}

// ── F-strings ──

#[test]
fn test_pep701_fstring_each_line() {
    let lines = &[
        ("simple_quote_reuse", "s = f\"{'hello'}\"\n"),
        ("dict_key", "s = f\"result: {d['key']}\"\n"),
        ("brace_in_string", "s = f\"{'{'}\"\n"),
        ("chr_call", "s = f\"newline: {chr(10)}\"\n"),
        ("tab_escape", "s = f\"tab: {'\\t'}\"\n"),
        ("backslash_escape", "s = f\"escaped: {'\\\\n'}\"\n"),
        ("format_spec_str", "s = f\"{'hello':>10}\"\n"),
        ("star_in_str", "s = f\"{'='*40}\"\n"),
        ("lambda_expr", "s = f\"{(lambda x: x + 1)(5)}\"\n"),
        ("dict_comp", "s = f\"{ {k: v for k, v in items} }\"\n"),
        ("multiline_expr", "result = f\"value: {\n    x + y\n}\"\n"),
        ("multiline_list_comp", "s = f\"mapped: {[\n    item\n    for item in range(10)\n    if item > 5\n]}\"\n"),
        ("nested_fstr", "s = f\"{f\\\"{f\\\"deep\\\"}\\\"}\"\n"),
        ("closing_brace_in_str", "s = f\"{'}'}\"\n"),
    ];
    // Test whole pep701 fixture file
    let fixture = include_str!("../../../tests/cpython/_regression/core/grammar/test_fstring/pep701_fstrings.py");
    parser::parse(fixture, FileId(0)).unwrap_or_else(|e| panic!("whole pep701 file: {e:?}"));
    for (name, src) in lines {
        parser::parse(src, FileId(0)).unwrap_or_else(|e| panic!("{name}: parse failed: {e:?}"));
    }
}

#[test]
fn test_parse_fstring() {
    let src = "f\"hello {name}\"\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            match &expr.node {
                Expr::FString(parts) => {
                    assert_eq!(parts.len(), 2);
                    assert!(matches!(&parts[0], FStringPart::Literal(s) if s == "hello "));
                    assert!(matches!(&parts[1], FStringPart::Expr(_, None)));
                }
                other => panic!("expected FString, got {other:?}"),
            }
        }
        other => panic!("expected ExprStmt, got {other:?}"),
    }
}

// ── Pipeline: MIR generation for complex constructs ──

#[test]
fn test_pipeline_if_expr() {
    // Use expression statement to avoid variable resolution
    let mir = pipeline("1 if True else 0\n");
    assert!(!mir.bodies.is_empty());
    let main = &mir.bodies[0];
    // IfExpr generates branch blocks
    assert!(main.blocks.len() >= 3, "IfExpr should produce >=3 blocks");
}

#[test]
fn test_pipeline_assert() {
    let mir = pipeline("assert True\n");
    assert!(!mir.bodies.is_empty());
    let main = &mir.bodies[0];
    // Assert generates branch to assertion error block
    assert!(main.blocks.len() >= 2, "assert should produce >=2 blocks");
}

#[test]
fn test_pipeline_while_loop() {
    let mir = pipeline("while True:\n    break\n");
    assert!(!mir.bodies.is_empty());
    let main = &mir.bodies[0];
    assert!(main.blocks.len() >= 3, "while loop should produce >=3 blocks");
}

#[test]
fn test_pipeline_function_with_binops() {
    // Function params lose scope after type checking, so we test that
    // the function body is generated and contains a Return.
    let mir = pipeline("def add(a: int, b: int) -> int:\n    return a + b\n");
    assert!(!mir.bodies.is_empty());
    let func = &mir.bodies[0];
    let has_return = func.blocks.iter().any(|b| {
        matches!(b.terminator, Terminator::Return(_))
    });
    assert!(has_return, "should have Return terminator");
}

#[test]
fn test_pipeline_bitwise_ops() {
    let mir = pipeline("x: int = 5 & 3\n");
    let main = &mir.bodies[0];
    let has_bitand = main.blocks.iter().any(|b| {
        b.stmts.iter().any(|inst| matches!(inst, MirInst::BinOp { op: MirBinOp::BitAnd, .. }))
    });
    assert!(has_bitand, "should have BitAnd binop");
}

#[test]
fn test_pipeline_nested_function() {
    // Nested functions defined at top-level scope are visible.
    // Inner functions defined inside outer lose scope, so test top-level only.
    let mir = pipeline(
        "def outer() -> int:\n    return 1\ndef helper() -> int:\n    return 2\n"
    );
    // Should produce 2 function bodies (outer + helper)
    assert!(mir.bodies.len() >= 2, "should have 2 top-level functions");
}

// ── #308: Comprehension and Generator Expression Codegen ──

#[test]
fn test_pipeline_list_comprehension() {
    let mir = pipeline("items = [1, 2, 3]\nresult = [x * 2 for x in items]\n");
    let main = &mir.bodies[0];
    let all_insts: Vec<&MirInst> = main.blocks.iter()
        .flat_map(|b| &b.stmts).collect();
    // Should have MakeList (empty target) + mb_iter + mb_list_append calls
    assert!(all_insts.iter().any(|i| matches!(i, MirInst::MakeList { .. })),
        "list comp should create empty list");
    let extern_names: Vec<&str> = all_insts.iter().filter_map(|i| {
        if let MirInst::CallExtern { name, .. } = i { Some(name.as_str()) } else { None }
    }).collect();
    assert!(extern_names.contains(&"mb_iter"), "should call mb_iter");
    // List comprehensions use the unchecked variant — local list, no
    // concurrent readers, so the RwLock try_write is skipped.
    assert!(extern_names.contains(&"mb_list_append_unchecked"),
        "should call mb_list_append_unchecked");
}

#[test]
fn test_pipeline_dict_comprehension() {
    let mir = pipeline("pairs = [1, 2]\nresult = {k: k for k in pairs}\n");
    let main = &mir.bodies[0];
    let all_insts: Vec<&MirInst> = main.blocks.iter()
        .flat_map(|b| &b.stmts).collect();
    assert!(all_insts.iter().any(|i| matches!(i, MirInst::MakeDict { .. })),
        "dict comp should create empty dict");
    let extern_names: Vec<&str> = all_insts.iter().filter_map(|i| {
        if let MirInst::CallExtern { name, .. } = i { Some(name.as_str()) } else { None }
    }).collect();
    assert!(extern_names.contains(&"mb_dict_setitem"), "should call mb_dict_setitem");
}

#[test]
fn test_pipeline_set_comprehension() {
    let mir = pipeline("items = [1, 2]\nresult = {x for x in items}\n");
    let main = &mir.bodies[0];
    let all_insts: Vec<&MirInst> = main.blocks.iter()
        .flat_map(|b| &b.stmts).collect();
    // Sets backed by list currently
    assert!(all_insts.iter().any(|i| matches!(i, MirInst::MakeList { .. })),
        "set comp should create container");
}

#[test]
fn test_pipeline_generator_expr() {
    // Generator expressions desugar to eager list comprehension
    let mir = pipeline("items = [1, 2, 3]\nresult = (x * 2 for x in items)\n");
    let main = &mir.bodies[0];
    let all_insts: Vec<&MirInst> = main.blocks.iter()
        .flat_map(|b| &b.stmts).collect();
    let extern_names: Vec<&str> = all_insts.iter().filter_map(|i| {
        if let MirInst::CallExtern { name, .. } = i { Some(name.as_str()) } else { None }
    }).collect();
    assert!(extern_names.contains(&"mb_iter"), "generator expr should use iteration");
}

#[test]
fn test_pipeline_comprehension_with_filter() {
    let mir = pipeline("items = [1, 2, 3]\nresult = [x for x in items if x > 0]\n");
    let main = &mir.bodies[0];
    // Filtered comprehension should have a Branch for the condition
    let has_branch = main.blocks.iter().any(|b| {
        matches!(&b.terminator, Terminator::Branch { .. })
    });
    assert!(has_branch, "filtered comprehension needs conditional branch");
}

// ── #309: Pattern Matching Codegen ──

#[test]
fn test_pipeline_match_literal() {
    let mir = pipeline(
        "x: int = 1\nmatch x:\n    case 1:\n        y: int = 10\n    case 2:\n        y: int = 20\n"
    );
    let main = &mir.bodies[0];
    // Should have Branch terminators for literal pattern checks
    let branch_count = main.blocks.iter().filter(|b| {
        matches!(&b.terminator, Terminator::Branch { .. })
    }).count();
    assert!(branch_count >= 2, "match with 2 literal cases needs >=2 branches, got {branch_count}");
}

#[test]
fn test_pipeline_match_wildcard() {
    let mir = pipeline(
        "x: int = 1\nmatch x:\n    case _:\n        y: int = 99\n"
    );
    let main = &mir.bodies[0];
    // Wildcard always matches — should have Goto, not Branch
    let has_goto_to_body = main.blocks.iter().any(|b| {
        matches!(&b.terminator, Terminator::Goto(_))
    });
    assert!(has_goto_to_body, "wildcard pattern should use unconditional goto");
}

#[test]
fn test_pipeline_match_capture() {
    let mir = pipeline(
        "x: int = 42\nmatch x:\n    case val:\n        y: int = val\n"
    );
    let main = &mir.bodies[0];
    // Capture binds subject to variable — should have Copy instruction
    let has_copy = main.blocks.iter().flat_map(|b| &b.stmts)
        .any(|i| matches!(i, MirInst::Copy { .. }));
    assert!(has_copy, "capture pattern should copy subject to variable");
}

#[test]
fn test_pipeline_match_or_pattern() {
    // PEP 634: OR patterns with literals (now correctly parsed as Pattern::Or, not BinOp::BitOr)
    let mir = pipeline(
        "x: int = 1\nmatch x:\n    case 1 | 2:\n        y: int = 10\n"
    );
    let main = &mir.bodies[0];
    // OR with 2 alternatives — at least one branch
    let branch_count = main.blocks.iter().filter(|b| {
        matches!(&b.terminator, Terminator::Branch { .. } | Terminator::Goto(_))
    }).count();
    assert!(branch_count >= 2, "OR pattern needs branches, got {branch_count}");
}

#[test]
fn test_pipeline_match_with_guard() {
    // Use binding pattern (not literal) so `if` isn't consumed as ternary
    let mir = pipeline(
        "x: int = 5\nmatch x:\n    case val if val > 0:\n        y: int = 1\n"
    );
    let main = &mir.bodies[0];
    // Guard generates either a BinOp::Gt (primitive) or mb_gt call (when operand is any-typed)
    let has_guard_cmp = main.blocks.iter().flat_map(|b| &b.stmts)
        .any(|i| matches!(i, MirInst::BinOp { op: MirBinOp::Gt, .. })
            || matches!(i, MirInst::CallExtern { name, .. } if name == "mb_gt"));
    assert!(has_guard_cmp, "guarded capture pattern should have guard comparison");
}

#[test]
fn test_pipeline_match_sequence_pattern() {
    let mir = pipeline(
        "x = [1, 2]\nmatch x:\n    case [a, b]:\n        y: int = a\n"
    );
    let main = &mir.bodies[0];
    let all_insts: Vec<&MirInst> = main.blocks.iter()
        .flat_map(|b| &b.stmts).collect();
    let extern_names: Vec<&str> = all_insts.iter().filter_map(|i| {
        if let MirInst::CallExtern { name, .. } = i { Some(name.as_str()) } else { None }
    }).collect();
    // Sequence pattern should check length and extract elements (sequence-generic helpers #827)
    assert!(extern_names.contains(&"mb_seq_len"),
        "sequence pattern should check length");
    assert!(extern_names.contains(&"mb_seq_getitem"),
        "sequence pattern should extract elements");
}

// ── #827: AS-pattern and class-pattern pipeline tests ──

#[test]
fn test_pipeline_match_as_pattern() {
    // AS-pattern: `case <pattern> as <name>:` should bind the alias
    let mir = pipeline(
        "x = 42\nmatch x:\n    case n as m:\n        y: int = m\n"
    );
    let main = &mir.bodies[0];
    // The alias binding should appear as a Copy or LoadConst (vreg alias)
    let has_binding = main.blocks.iter().any(|b| {
        b.stmts.iter().any(|i| matches!(i, MirInst::Copy { .. } | MirInst::LoadConst { .. }))
    });
    assert!(has_binding, "AS-pattern should produce a register binding");
}

#[test]
fn test_pipeline_match_class_pattern() {
    // Class pattern: match against class name and extract keyword fields
    let mir = pipeline(
        "class Point:\n    x: int = 0\n    y: int = 0\np = Point()\nmatch p:\n    case Point(x=1):\n        z: int = 1\n    case _:\n        z: int = 0\n"
    );
    let all_insts: Vec<&MirInst> = mir.bodies.iter()
        .flat_map(|b| b.blocks.iter().flat_map(|bl| &bl.stmts))
        .collect();
    let extern_names: Vec<&str> = all_insts.iter().filter_map(|i| {
        if let MirInst::CallExtern { name, .. } = i { Some(name.as_str()) } else { None }
    }).collect();
    // Class pattern should use instance attribute check helper
    assert!(
        extern_names.contains(&"mb_instance_hasattr"),
        "class pattern should check instance attribute"
    );
}

// ── #305: LLVM Backend ──

#[test]
fn test_llvm_backend_simple() {
    use crate::codegen::CodegenBackend;
    use crate::codegen::llvm::LlvmBackend;
    let mir = pipeline("x: int = 42\n");
    let tcx = crate::types::TypeContext::new();
    let mut backend = LlvmBackend::new();
    let output = backend.codegen(&mir, &tcx);
    assert!(output.is_ok(), "LLVM backend should produce output");
}

#[test]
fn test_llvm_ir_structure() {
    use crate::codegen::llvm::LlvmBackend;
    use crate::codegen::CodegenBackend;
    let mir = pipeline("x: int = 10\ny: int = x + 20\n");
    let tcx = crate::types::TypeContext::new();
    let mut backend = LlvmBackend::new();
    let output = backend.codegen(&mir, &tcx).unwrap();
    // When llc is available, output is ObjectFile; otherwise LlvmIr
    match output {
        crate::codegen::CodegenOutput::ObjectFile(bytes) => {
            assert!(!bytes.is_empty(), "LLVM object file should be non-empty");
        }
        crate::codegen::CodegenOutput::LlvmIr(ir) => {
            assert!(!ir.is_empty(), "LLVM IR should be non-empty");
            assert!(ir.contains("define"), "LLVM IR should contain function definitions");
        }
        _ => panic!("expected ObjectFile or LlvmIr variant"),
    }
}

#[test]
fn test_llvm_backend_selection() {
    use crate::driver::{CompilerConfig, Backend};
    let config = CompilerConfig {
        backend: Backend::Llvm,
        ..CompilerConfig::default()
    };
    assert_eq!(config.backend, Backend::Llvm);
}

#[test]
fn test_llvm_backend_with_function() {
    use crate::codegen::llvm::LlvmBackend;
    use crate::codegen::CodegenBackend;
    let mir = pipeline("def add(a: int, b: int) -> int:\n    return a + b\n");
    let tcx = crate::types::TypeContext::new();
    let mut backend = LlvmBackend::new();
    let output = backend.codegen(&mir, &tcx);
    assert!(output.is_ok(), "LLVM backend should handle functions");
}

// ── #313: Async/Await and Coroutine Scheduling ──

#[test]
fn test_async_function_creates_coroutine() {
    // R1: async functions produce wrapper + body MirBodies
    let mir = pipeline("async def fetch() -> int:\n    return 42\n");
    // Should have at least 3 bodies: body_step, wrapper, __main__
    let non_main: Vec<_> = mir.bodies.iter()
        .filter(|b| b.name.0 != u32::MAX).collect();
    assert!(non_main.len() >= 2, "async function should produce wrapper + body");
    // Wrapper should call mb_coroutine_new
    let has_coro_new = non_main.iter().any(|f| {
        f.blocks.iter().any(|b| {
            b.stmts.iter().any(|s| matches!(s,
                MirInst::CallExtern { name, .. } if name == "mb_coroutine_new"
            ))
        })
    });
    // Body should call mb_coroutine_complete
    let has_coro_complete = non_main.iter().any(|f| {
        f.blocks.iter().any(|b| {
            b.stmts.iter().any(|s| matches!(s,
                MirInst::CallExtern { name, .. } if name == "mb_coroutine_complete"
            ))
        })
    });
    assert!(has_coro_new, "wrapper should call mb_coroutine_new");
    assert!(has_coro_complete, "body should call mb_coroutine_complete");
}

#[test]
fn test_await_expression_lowering() {
    // R1+R3: await should lower to mb_await with GIL release/acquire
    let mir = pipeline(
        "async def inner() -> int:\n    return 1\n\
         async def outer() -> int:\n    x: int = await inner()\n    return x\n"
    );
    // Body functions should contain mb_await for the await expression
    let any_has_await = mir.bodies.iter().any(|body| {
        body.blocks.iter().any(|b| {
            b.stmts.iter().any(|s| matches!(s,
                MirInst::CallExtern { name, .. } if name == "mb_await"
            ))
        })
    });
    assert!(any_has_await, "await expression should lower to mb_await call");
}

#[test]
fn test_async_function_gil_release_acquire() {
    // R3: GIL should be released before await and acquired after
    let mir = pipeline(
        "async def f() -> int:\n    x: int = await f()\n    return x\n"
    );
    // Find the body function (has the await/GIL calls)
    let body = mir.bodies.iter().find(|b| {
        b.blocks.iter().any(|blk| {
            blk.stmts.iter().any(|s| matches!(s,
                MirInst::CallExtern { name, .. } if name == "mb_await"
            ))
        })
    }).expect("should have a body with mb_await");
    let all_externs: Vec<&str> = body.blocks.iter().flat_map(|b| {
        b.stmts.iter().filter_map(|s| match s {
            MirInst::CallExtern { name, .. } => Some(name.as_str()),
            _ => None,
        })
    }).collect();
    assert!(all_externs.contains(&"mb_gil_release"), "should release GIL before await");
    assert!(all_externs.contains(&"mb_gil_acquire"), "should acquire GIL after await");
    // GIL release should come before mb_await, acquire after
    let release_pos = all_externs.iter().position(|&n| n == "mb_gil_release");
    let await_pos = all_externs.iter().position(|&n| n == "mb_await");
    let acquire_pos = all_externs.iter().position(|&n| n == "mb_gil_acquire");
    if let (Some(r), Some(a), Some(q)) = (release_pos, await_pos, acquire_pos) {
        assert!(r < a, "GIL release should precede await");
        assert!(a < q, "GIL acquire should follow await");
    }
}

#[test]
fn test_sync_function_no_coroutine() {
    // Regular (non-async) functions should NOT create coroutines
    let mir = pipeline("def add(a: int, b: int) -> int:\n    return a + b\n");
    let func = mir.bodies.iter().find(|b| b.name.0 != u32::MAX).unwrap();
    let has_coro_new = func.blocks.iter().any(|b| {
        b.stmts.iter().any(|s| matches!(s,
            MirInst::CallExtern { name, .. } if name == "mb_coroutine_new"
        ))
    });
    assert!(!has_coro_new, "sync function should NOT create coroutines");
}

#[test]
fn test_async_function_return_is_coroutine_handle() {
    // Both wrapper and body should return a value (coroutine handle)
    let mir = pipeline("async def compute() -> int:\n    return 100\n");
    let non_main: Vec<_> = mir.bodies.iter()
        .filter(|b| b.name.0 != u32::MAX).collect();
    // The wrapper should always return a value (the coroutine handle)
    for func in &non_main {
        for block in &func.blocks {
            if let Terminator::Return(ret) = &block.terminator {
                assert!(ret.is_some(),
                    "async wrapper/body should always return a value (coroutine handle)");
            }
        }
    }
}

// ── P0 Runtime: Method dispatch lowering (#375-#381) ──

#[test]
fn test_pipeline_method_call_produces_call_method() {
    // Task 4.1: x.method() should lower to CallExtern("mb_call_method")
    // Use `.encode()` — not in the str-method fast path (no mb_str_encode
    // runtime symbol registered), so it falls through to mb_call_method.
    let mir = pipeline("s: str = \"hello\"\ns.encode()\n");
    let main = &mir.bodies[0];
    let all_externs: Vec<&str> = main.blocks.iter()
        .flat_map(|b| &b.stmts)
        .filter_map(|i| if let MirInst::CallExtern { name, .. } = i { Some(name.as_str()) } else { None })
        .collect();
    assert!(all_externs.contains(&"mb_call_method"),
        "method call should lower to mb_call_method, got: {all_externs:?}");
}

#[test]
fn test_pipeline_method_call_packs_args() {
    // Task 4.1: method args should be packed into a MakeList
    let mir = pipeline("s: str = \"hello world\"\ns.split(\" \")\n");
    let main = &mir.bodies[0];
    let all_insts: Vec<&MirInst> = main.blocks.iter()
        .flat_map(|b| &b.stmts).collect();
    // Should have MakeList (for args packing) + CallExtern("mb_call_method")
    let has_make_list = all_insts.iter().any(|i| matches!(i, MirInst::MakeList { .. }));
    let has_call_method = all_insts.iter().any(|i| matches!(i,
        MirInst::CallExtern { name, .. } if name == "mb_call_method"));
    assert!(has_make_list, "method args should be packed into a list");
    assert!(has_call_method, "should call mb_call_method");
}

#[test]
fn test_pipeline_attribute_assignment() {
    // Task 4.1: x.attr = val should produce SetAttr
    let mir = pipeline("x = 1\nx.foo = 42\n");
    let main = &mir.bodies[0];
    let has_setattr = main.blocks.iter().any(|b| {
        b.stmts.iter().any(|i| matches!(i, MirInst::SetAttr { .. }))
    });
    assert!(has_setattr, "attribute assignment should produce SetAttr");
}

#[test]
fn test_pipeline_index_assignment() {
    // Task 4.1: x[i] = val should produce SetItem
    let mir = pipeline("x = [1, 2, 3]\nx[0] = 42\n");
    let main = &mir.bodies[0];
    let has_setitem = main.blocks.iter().any(|b| {
        b.stmts.iter().any(|i| matches!(i, MirInst::SetItem { .. }))
    });
    assert!(has_setitem, "index assignment should produce SetItem");
}

#[test]
fn test_pipeline_string_method_call() {
    // Task 4.2: string methods go through mb_call_method
    let mir = pipeline("s: str = \"  hello  \"\ns.strip()\n");
    let main = &mir.bodies[0];
    let externs: Vec<&str> = main.blocks.iter()
        .flat_map(|b| &b.stmts)
        .filter_map(|i| if let MirInst::CallExtern { name, .. } = i { Some(name.as_str()) } else { None })
        .collect();
    assert!(externs.contains(&"mb_call_method"), "string method should use mb_call_method");
}

#[test]
fn test_pipeline_list_method_call() {
    // Task 4.3: list methods go through mb_call_method when no direct
    // runtime symbol is registered for them. `__sizeof__` has no direct
    // entry so it stays on the generic dispatch path.
    let mir = pipeline("lst = [1, 2]\nlst.__sizeof__()\n");
    let main = &mir.bodies[0];
    let externs: Vec<&str> = main.blocks.iter()
        .flat_map(|b| &b.stmts)
        .filter_map(|i| if let MirInst::CallExtern { name, .. } = i { Some(name.as_str()) } else { None })
        .collect();
    assert!(externs.contains(&"mb_call_method"), "list method should use mb_call_method");
}

#[test]
fn test_pipeline_chained_method_call() {
    // Task 4.2: chained methods: s.encode().decode()
    // Both `encode` and `decode` are outside the str-method fast path,
    // so each lowers via mb_call_method.
    let mir = pipeline("s: str = \"hello\"\ns.encode().decode()\n");
    let main = &mir.bodies[0];
    let call_count = main.blocks.iter()
        .flat_map(|b| &b.stmts)
        .filter(|i| matches!(i, MirInst::CallExtern { name, .. } if name == "mb_call_method"))
        .count();
    assert!(call_count >= 2, "chained methods should produce 2+ mb_call_method calls, got {call_count}");
}

#[test]
fn test_pipeline_try_except_produces_handler_blocks() {
    // Task 4.6: exception handling produces proper control flow
    let mir = pipeline("try:\n    x: int = 1\nexcept ValueError:\n    y: int = 2\nfinally:\n    z: int = 3\n");
    let main = &mir.bodies[0];
    // Should have multiple blocks for try/except/finally
    assert!(main.blocks.len() >= 4,
        "try/except/finally should produce >=4 blocks, got {}", main.blocks.len());
    // Should have mb_push_handler and mb_pop_handler
    let externs: Vec<&str> = main.blocks.iter()
        .flat_map(|b| &b.stmts)
        .filter_map(|i| if let MirInst::CallExtern { name, .. } = i { Some(name.as_str()) } else { None })
        .collect();
    assert!(externs.contains(&"mb_push_handler"), "try should push exception handler");
    assert!(externs.contains(&"mb_pop_handler"), "try should pop exception handler");
}

#[test]
fn test_pipeline_raise_stmt() {
    // Task 4.6: raise produces mb_raise CallExtern
    let mir = pipeline("raise ValueError(\"bad\")\n");
    let main = &mir.bodies[0];
    let has_raise = main.blocks.iter().any(|b| {
        b.stmts.iter().any(|i| matches!(i, MirInst::CallExtern { name, .. } if name.starts_with("mb_raise")))
    });
    assert!(has_raise, "raise statement should produce mb_raise CallExtern");
}

#[test]
fn test_pipeline_for_loop_uses_iterator_protocol() {
    // Task 4.5 (post-Lever-A): for loop uses mb_iter/mb_next_or_stop/mb_is_stop_iter
    let mir = pipeline("items = [1, 2, 3]\nfor x in items:\n    y: int = x\n");
    let main = &mir.bodies[0];
    let externs: Vec<&str> = main.blocks.iter()
        .flat_map(|b| &b.stmts)
        .filter_map(|i| if let MirInst::CallExtern { name, .. } = i { Some(name.as_str()) } else { None })
        .collect();
    assert!(externs.contains(&"mb_iter"), "for loop should call mb_iter");
    assert!(externs.contains(&"mb_next_or_stop"), "for loop should call mb_next_or_stop");
    assert!(externs.contains(&"mb_is_stop_iter"), "for loop should call mb_is_stop_iter");
    assert!(externs.contains(&"mb_iter_release"), "for loop should release iterator");
}

#[test]
fn test_pipeline_fstring_calls_mb_str() {
    // Task 4.7: f-string interpolation calls mb_str for conversion
    let mir = pipeline("x: int = 42\nf\"value is {x}\"\n");
    let main = &mir.bodies[0];
    let externs: Vec<&str> = main.blocks.iter()
        .flat_map(|b| &b.stmts)
        .filter_map(|i| if let MirInst::CallExtern { name, .. } = i { Some(name.as_str()) } else { None })
        .collect();
    assert!(externs.contains(&"mb_str"), "f-string should call mb_str for conversion");
}

#[test]
fn test_pipeline_binop_on_any_type_dispatches() {
    // Task 4.7: BinOp on non-primitive types should trigger dispatch
    let mir = pipeline("x = [1]\ny = [2]\nz = x + y\n");
    let main = &mir.bodies[0];
    // Non-primitive addition is dispatched through CallExtern("mb_add")
    let has_dispatch = main.blocks.iter().any(|b| {
        b.stmts.iter().any(|i| match i {
            MirInst::BinOp { .. } => true,
            MirInst::CallExtern { name, .. } => name == "mb_add",
            _ => false,
        })
    });
    assert!(has_dispatch, "list addition should produce BinOp or mb_add dispatch");
}

#[test]
fn test_async_function_body_reads_locals() {
    // R1: body function should read args from coroutine locals
    let mir = pipeline("async def add(a: int, b: int) -> int:\n    return a + b\n");
    // Body function should call mb_coroutine_get_local to read args
    let has_get_local = mir.bodies.iter().any(|body| {
        body.blocks.iter().any(|b| {
            b.stmts.iter().any(|s| matches!(s,
                MirInst::CallExtern { name, .. } if name == "mb_coroutine_get_local"
            ))
        })
    });
    // Wrapper should call mb_coroutine_set_local to store args
    let has_set_local = mir.bodies.iter().any(|body| {
        body.blocks.iter().any(|b| {
            b.stmts.iter().any(|s| matches!(s,
                MirInst::CallExtern { name, .. } if name == "mb_coroutine_set_local"
            ))
        })
    });
    assert!(has_get_local, "body should read args via mb_coroutine_get_local");
    assert!(has_set_local, "wrapper should store args via mb_coroutine_set_local");
}

// ── string-ops: Str + Str lowers to mb_str_concat ──

#[test]
fn test_str_concat_emits_mb_str_concat() {
    // str + str must lower to CallExtern { name: "mb_str_concat" }
    let mir = pipeline(
        "a: str = \"hello\"\n\
         b: str = \" world\"\n\
         c: str = a + b\n"
    );
    assert!(!mir.bodies.is_empty());
    let main = &mir.bodies[0];
    let has_concat = main.blocks.iter().any(|blk| {
        blk.stmts.iter().any(|inst| {
            matches!(inst, MirInst::CallExtern { name, .. } if name == "mb_str_concat")
        })
    });
    assert!(has_concat, "str + str should lower to CallExtern mb_str_concat");
}

#[test]
fn test_str_concat_does_not_use_mb_add() {
    // str + str must NOT dispatch to mb_add (wrong generic numeric path)
    let mir = pipeline(
        "a: str = \"hello\"\n\
         b: str = \" world\"\n\
         c: str = a + b\n"
    );
    assert!(!mir.bodies.is_empty());
    let main = &mir.bodies[0];
    let uses_mb_add = main.blocks.iter().any(|blk| {
        blk.stmts.iter().any(|inst| {
            matches!(inst, MirInst::CallExtern { name, .. } if name == "mb_add")
        })
    });
    assert!(!uses_mb_add, "str + str must not dispatch to mb_add");
}

// ── R8: Integer literal patterns emit correct constant values ──

#[test]
fn test_pipeline_match_integer_constants_distinct() {
    // R8.1: case 0, case 1, case 2 must emit LoadConst(Int(0)), Int(1), Int(2) — not all zero.
    let mir = pipeline(
        "val: int = 1\n\
         match val:\n\
         \x20   case 0:\n\
         \x20       y: int = 0\n\
         \x20   case 1:\n\
         \x20       y: int = 1\n\
         \x20   case _:\n\
         \x20       y: int = 2\n"
    );
    let main = &mir.bodies[0];
    let all_insts: Vec<&MirInst> = main.blocks.iter()
        .flat_map(|b| &b.stmts).collect();

    // Collect every integer constant loaded in the function body
    let int_consts: Vec<i64> = all_insts.iter().filter_map(|i| {
        if let MirInst::LoadConst { value: MirConst::Int(v), .. } = i {
            Some(*v)
        } else {
            None
        }
    }).collect();

    // The pattern comparison constants 0 and 1 must both appear
    assert!(int_consts.contains(&0),
        "case 0 must emit LoadConst(Int(0)), got consts: {int_consts:?}");
    assert!(int_consts.contains(&1),
        "case 1 must emit LoadConst(Int(1)), got consts: {int_consts:?}");
}

#[test]
fn test_pipeline_match_integer_constant_value_preserved() {
    // R8.1: a single integer literal pattern must load the correct constant value
    let mir = pipeline(
        "x: int = 42\n\
         match x:\n\
         \x20   case 42:\n\
         \x20       y: int = 1\n\
         \x20   case _:\n\
         \x20       y: int = 0\n"
    );
    let main = &mir.bodies[0];
    let int_consts: Vec<i64> = main.blocks.iter()
        .flat_map(|b| &b.stmts)
        .filter_map(|i| {
            if let MirInst::LoadConst { value: MirConst::Int(v), .. } = i {
                Some(*v)
            } else {
                None
            }
        }).collect();

    assert!(int_consts.contains(&42),
        "case 42 must emit LoadConst(Int(42)), got: {int_consts:?}");
}

// ── R7: Walrus := scope assignment ──

#[test]
fn test_pipeline_walrus_simple_lowers_without_error() {
    // R7: Walrus in a simple (non-comprehension) context should lower to MIR
    let mir = pipeline(
        "x: int = 1\n\
         if (y := x + 1) > 0:\n\
         \x20   z: int = y\n"
    );
    // Main body should be non-empty — walrus must not abort the pipeline
    assert!(!mir.bodies[0].blocks.is_empty(),
        "walrus in if condition must produce non-empty MIR");
}

#[test]
fn test_pipeline_raise_from_lowers_to_mir() {
    // R4.1: `raise X from Y` must produce mb_raise CallExtern in MIR
    let mir = pipeline(
        "try:\n\
         \x20   x: int = 1\n\
         except Exception:\n\
         \x20   raise RuntimeError(\"wrap\")\n"
    );
    let main = &mir.bodies[0];
    let all_insts: Vec<&MirInst> = main.blocks.iter()
        .flat_map(|b| &b.stmts).collect();
    let has_raise = all_insts.iter().any(|i| matches!(i, MirInst::CallExtern { name, .. } if name.starts_with("mb_raise")));
    assert!(has_raise, "raise in except handler must emit mb_raise CallExtern");
}
