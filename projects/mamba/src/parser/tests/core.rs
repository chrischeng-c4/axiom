#![cfg(test)]

use crate::parser;
use crate::parser::ast::*;
use crate::source::span::FileId;

fn parse(src: &str) -> Module {
    parser::parse(src, FileId(0)).expect("parse failed")
}

#[test]
fn test_var_decl() {
    let module = parse("x: int = 42\n");
    assert_eq!(module.stmts.len(), 1);
    match &module.stmts[0].node {
        Stmt::VarDecl { name, ty, value } => {
            assert_eq!(name, "x");
            assert!(matches!(&ty.node, TypeExpr::Named(n) if n == "int"));
            assert!(matches!(&value.node, Expr::IntLit(42)));
        }
        _ => panic!("expected VarDecl"),
    }
}

#[test]
fn test_function_def() {
    let module = parse("def add(a: int, b: int) -> int:\n    return a + b\n");
    assert_eq!(module.stmts.len(), 1);
    match &module.stmts[0].node {
        Stmt::FnDef { name, params, return_ty, body, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name, "a");
            assert_eq!(params[1].name, "b");
            assert!(return_ty.is_some());
            assert_eq!(body.len(), 1);
            assert!(matches!(&body[0].node, Stmt::Return(Some(_))));
        }
        _ => panic!("expected FnDef"),
    }
}

#[test]
fn test_if_elif_else() {
    let src = "if x > 0:\n    pass\nelif x == 0:\n    pass\nelse:\n    pass\n";
    let module = parse(src);
    assert_eq!(module.stmts.len(), 1);
    match &module.stmts[0].node {
        Stmt::If { elif_clauses, else_body, .. } => {
            assert_eq!(elif_clauses.len(), 1);
            assert!(else_body.is_some());
        }
        _ => panic!("expected If"),
    }
}

#[test]
fn test_while_loop() {
    let src = "while i < 10:\n    i = i + 1\n";
    let module = parse(src);
    assert_eq!(module.stmts.len(), 1);
    match &module.stmts[0].node {
        Stmt::While { body, .. } => {
            assert_eq!(body.len(), 1);
        }
        _ => panic!("expected While"),
    }
}

#[test]
fn test_class_def() {
    let src = "class Point:\n    x: int = 0\n    y: int = 0\n";
    let module = parse(src);
    assert_eq!(module.stmts.len(), 1);
    match &module.stmts[0].node {
        Stmt::ClassDef { name, body, .. } => {
            assert_eq!(name, "Point");
            assert_eq!(body.len(), 2);
        }
        _ => panic!("expected ClassDef"),
    }
}

#[test]
fn test_enum_def() {
    let src = "enum Shape:\n    Circle(radius: float)\n    Point\n";
    let module = parse(src);
    assert_eq!(module.stmts.len(), 1);
    match &module.stmts[0].node {
        Stmt::EnumDef { name, variants, .. } => {
            assert_eq!(name, "Shape");
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "Circle");
            assert_eq!(variants[0].fields.len(), 1);
            assert_eq!(variants[1].name, "Point");
            assert!(variants[1].fields.is_empty());
        }
        _ => panic!("expected EnumDef"),
    }
}

#[test]
fn test_match_stmt() {
    let src = "match x:\n    case 1:\n        pass\n    case 2:\n        pass\n";
    let module = parse(src);
    assert_eq!(module.stmts.len(), 1);
    match &module.stmts[0].node {
        Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);
        }
        _ => panic!("expected Match"),
    }
}

// #1580: `case` is a PEP 634 soft keyword — outside `match` arms it
// must parse as an ordinary identifier (attribute, variable, parameter
// name). CPython tests use `self.case = "empty set"` as a common idiom.
#[test]
fn test_case_as_attribute_name() {
    let module = parse(
        "class C:\n\
         \x20   def setUp(self):\n\
         \x20       self.case = \"empty set\"\n\
         \x20       self.match = \"ok\"\n"
    );
    assert_eq!(module.stmts.len(), 1, "class def should parse");
}

// #1582: `lambda x: x, R(...)` inside a call-arg list must parse with
// body=`x` and the comma terminating the surrounding call-arg list — NOT
// with `x` (the body) confused for a type annotation.
#[test]
fn test_lambda_in_call_arg_list_terminated_by_comma() {
    let module = parse(
        "def chain(*args): return None\n\
         def map(fn, x): return None\n\
         def R(x): return x\n\
         result = chain(map(lambda x:x, R([1,2,3])))\n"
    );
    assert_eq!(module.stmts.len(), 4);
}

// #1580: `case` must also be usable as a bare identifier in expression
// position (e.g. `expected = case in self.cases`).
#[test]
fn test_case_as_expression_ident() {
    let module = parse(
        "def f(case):\n\
         \x20   return case\n"
    );
    assert_eq!(module.stmts.len(), 1, "function with case parameter should parse");
}

#[test]
fn test_binary_operators() {
    let module = parse("1 + 2 * 3\n");
    assert_eq!(module.stmts.len(), 1);
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            // Should be Add(1, Mul(2, 3)) due to precedence
            match &expr.node {
                Expr::BinOp { op: BinOp::Add, lhs, rhs } => {
                    assert!(matches!(&lhs.node, Expr::IntLit(1)));
                    assert!(matches!(&rhs.node, Expr::BinOp { op: BinOp::Mul, .. }));
                }
                _ => panic!("expected BinOp::Add"),
            }
        }
        _ => panic!("expected ExprStmt"),
    }
}

#[test]
fn test_function_call() {
    let module = parse("print(42)\n");
    assert_eq!(module.stmts.len(), 1);
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            match &expr.node {
                Expr::Call { func, args } => {
                    assert!(matches!(&func.node, Expr::Ident(n) if n == "print"));
                    assert_eq!(args.len(), 1);
                }
                _ => panic!("expected Call"),
            }
        }
        _ => panic!("expected ExprStmt"),
    }
}

#[test]
fn test_list_literal() {
    let module = parse("[1, 2, 3]\n");
    assert_eq!(module.stmts.len(), 1);
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            match &expr.node {
                Expr::ListLit(elems) => assert_eq!(elems.len(), 3),
                _ => panic!("expected ListLit"),
            }
        }
        _ => panic!("expected ExprStmt"),
    }
}

#[test]
fn test_optional_type() {
    let module = parse("x: int? = None\n");
    assert_eq!(module.stmts.len(), 1);
    match &module.stmts[0].node {
        Stmt::VarDecl { ty, .. } => {
            assert!(matches!(&ty.node, TypeExpr::Optional(_)));
        }
        _ => panic!("expected VarDecl with optional type"),
    }
}

#[test]
fn test_generic_type() {
    let module = parse("x: list[int] = [1]\n");
    assert_eq!(module.stmts.len(), 1);
    match &module.stmts[0].node {
        Stmt::VarDecl { ty, .. } => {
            match &ty.node {
                TypeExpr::Generic { name, args } => {
                    assert_eq!(name, "list");
                    assert_eq!(args.len(), 1);
                }
                _ => panic!("expected Generic type"),
            }
        }
        _ => panic!("expected VarDecl"),
    }
}

#[test]
fn test_import() {
    let module = parse("import math\n");
    match &module.stmts[0].node {
        Stmt::Import { module: m, names, .. } => {
            assert_eq!(m, &["math"]);
            assert!(names.is_none());
        }
        _ => panic!("expected Import"),
    }
}

#[test]
fn test_from_import() {
    let module = parse("from math import sqrt, pi\n");
    match &module.stmts[0].node {
        Stmt::Import { module: m, names, .. } => {
            assert_eq!(m, &["math"]);
            let names = names.as_ref().unwrap();
            assert_eq!(names.len(), 2);
            assert_eq!(names[0].0, "sqrt");
            assert_eq!(names[1].0, "pi");
        }
        _ => panic!("expected Import"),
    }
}

#[test]
fn test_type_params() {
    let src = "def first[T](items: list[int]) -> int:\n    return 0\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::FnDef { type_params, .. } => {
            let names: Vec<&str> = type_params.iter().map(|p| p.name.as_str()).collect();
            assert_eq!(names, vec!["T"]);
        }
        _ => panic!("expected FnDef with type params"),
    }
}

#[test]
fn test_for_loop() {
    let src = "for i in items:\n    pass\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::For { targets, .. } => {
            assert_eq!(targets, &["i"]);
        }
        _ => panic!("expected For"),
    }
}

#[test]
fn test_unary_neg() {
    let module = parse("-42\n");
    match &module.stmts[0].node {
        Stmt::ExprStmt(expr) => {
            assert!(matches!(&expr.node, Expr::UnaryOp { op: UnaryOp::Neg, .. }));
        }
        _ => panic!("expected UnaryOp"),
    }
}

// ── R6: metaclass= keyword in class declaration ──

#[test]
fn test_class_def_with_metaclass_keyword() {
    // R6.1: class Foo(object, metaclass=Meta) must parse without error.
    // metaclass= is a keyword arg: it should NOT appear in the bases list.
    let src = "class Meta(type):\n    pass\nclass Foo(object, metaclass=Meta):\n    pass\n";
    let module = parse(src);
    assert_eq!(module.stmts.len(), 2);
    match &module.stmts[1].node {
        Stmt::ClassDef { name, bases, .. } => {
            assert_eq!(name, "Foo");
            // Only positional base 'object' should appear; metaclass= must be filtered out
            assert_eq!(bases.len(), 1,
                "metaclass= keyword arg must not appear in bases, got {bases:?}");
            assert!(matches!(&bases[0].node, Expr::Ident(n) if n == "object"));
        }
        other => panic!("expected ClassDef for Foo, got {other:?}"),
    }
}

#[test]
fn test_class_def_metaclass_only() {
    // R6.1: class with ONLY a metaclass= keyword arg — bases list must be empty
    let src = "class Foo(metaclass=ABCMeta):\n    pass\n";
    let module = parse(src);
    match &module.stmts[0].node {
        Stmt::ClassDef { name, bases, .. } => {
            assert_eq!(name, "Foo");
            assert!(bases.is_empty(),
                "metaclass= should not appear in bases, got {bases:?}");
        }
        other => panic!("expected ClassDef, got {other:?}"),
    }
}

// ── R5: f-string parsing ──

#[test]
fn test_fstring_simple_expression() {
    // Verify that f"{x}" produces an FString node with one Expr part
    let src = "x: int = 1\ny: str = f\"{x}\"\n";
    let module = parse(src);
    match &module.stmts[1].node {
        Stmt::VarDecl { value: val, .. } => {
            assert!(matches!(&val.node, Expr::FString(_)),
                "expected FString, got {:?}", val.node);
            if let Expr::FString(parts) = &val.node {
                assert_eq!(parts.len(), 1, "should have one Expr part");
                assert!(matches!(&parts[0], FStringPart::Expr(_, None)));
            }
        }
        other => panic!("expected VarDecl with fstring, got {other:?}"),
    }
}

#[test]
fn test_fstring_with_literal_and_expr() {
    // f"hello {name}" produces [Literal("hello "), Expr(name)]
    let src = "name: str = \"world\"\ns: str = f\"hello {name}\"\n";
    let module = parse(src);
    match &module.stmts[1].node {
        Stmt::VarDecl { value: val, .. } => {
            if let Expr::FString(parts) = &val.node {
                assert_eq!(parts.len(), 2);
                assert!(matches!(&parts[0], FStringPart::Literal(s) if s == "hello "));
                assert!(matches!(&parts[1], FStringPart::Expr(_, None)));
            } else {
                panic!("expected FString, got {:?}", val.node);
            }
        }
        other => panic!("expected VarDecl, got {other:?}"),
    }
}

// #1584: chained assignment inside a class body (or any block whose terminator
// is `Dedent`) used to leak desugared continuation stmts (`b = __chained_N__;
// c = __chained_N__; ...`) out of the block. They were popped at module level
// where `__chained_N__` was undefined. Fix: parse_block must drain pending
// desugared stmts into the current block before consuming the Dedent.
#[test]
fn test_chained_assign_in_class_body_does_not_leak() {
    let src = "class C:\n    a = 1\n    b = c = d = a\n";
    let module = parse(src);
    // Exactly one top-level stmt — the class def. No leaked `b = ...` etc.
    assert_eq!(module.stmts.len(), 1, "chained-assign tail must not leak out of class body");
    match &module.stmts[0].node {
        Stmt::ClassDef { body, .. } => {
            // body holds: a=1, __chained=a, b=__chained, c=__chained, d=__chained
            assert!(body.len() >= 4,
                "class body should hold the chained-assign continuations, got {} stmts", body.len());
        }
        other => panic!("expected ClassDef, got {other:?}"),
    }
}

// #1590: parenthesized for-target `for (a, b) in ...` — outer parens
// are sugar around the tuple-target. Should parse identically to
// `for a, b in ...`.
#[test]
fn test_for_paren_tuple_target() {
    let module = parse("for (a, b) in pairs:\n    pass\n");
    match &module.stmts[0].node {
        Stmt::For { targets, .. } => {
            assert_eq!(targets, &["a", "b"]);
        }
        _ => panic!("expected For"),
    }
}

#[test]
fn test_for_paren_single_target_with_trailing_comma() {
    let module = parse("for (a,) in items:\n    pass\n");
    match &module.stmts[0].node {
        Stmt::For { targets, .. } => {
            assert_eq!(targets, &["a"]);
        }
        _ => panic!("expected For"),
    }
}

// #1592: parenthesized tuple-target in `with X as (a, b):` — desugars to
// a fresh `__with_target_N__` alias plus a tuple-unpack assign prepended
// to the body.
#[test]
fn test_with_paren_tuple_target() {
    let module = parse(
        "def make(): return (1, 2)\n\
         with make() as (a, b):\n\
         \x20   pass\n"
    );
    match &module.stmts[1].node {
        Stmt::With { items, body } => {
            assert_eq!(items.len(), 1);
            let alias = items[0].alias.as_deref().expect("alias must be set");
            assert!(alias.starts_with("__with_target_"),
                "expected synthetic alias, got {alias:?}");
            // First body stmt must be the unpack assign.
            assert!(!body.is_empty());
            assert!(matches!(&body[0].node, Stmt::Assign { .. }),
                "expected prepended unpack assign as first body stmt");
        }
        other => panic!("expected With, got {other:?}"),
    }
}

// #1594: nested-paren for-target — `for (a, b), c in ...` and
// `for (a, b), (c, d) in ...` desugar to fresh `__for_target_N_K__`
// flat names plus prepended unpack assigns at the start of the body.
#[test]
fn test_for_nested_paren_target() {
    let module = parse(
        "for (a, b), c in items:\n\
         \x20   pass\n"
    );
    match &module.stmts[0].node {
        Stmt::For { targets, body, .. } => {
            assert_eq!(targets.len(), 2);
            assert!(targets[0].starts_with("__for_target_"),
                "first target must be synthetic, got {:?}", targets[0]);
            assert_eq!(targets[1], "c");
            assert!(matches!(&body[0].node, Stmt::Assign { .. }),
                "expected prepended unpack assign as first body stmt");
        }
        _ => panic!("expected For"),
    }
}

#[test]
fn test_for_doubly_nested_paren_target() {
    let module = parse(
        "for (a, b), (c, d) in items:\n\
         \x20   pass\n"
    );
    match &module.stmts[0].node {
        Stmt::For { targets, body, .. } => {
            assert_eq!(targets.len(), 2);
            assert!(targets[0].starts_with("__for_target_"));
            assert!(targets[1].starts_with("__for_target_"));
            assert_eq!(body.len(), 3,
                "expected 2 prepended unpacks + pass body stmt, got {}", body.len());
        }
        _ => panic!("expected For"),
    }
}

#[test]
fn test_subscript_with_comma_tuple_index() {
    // `m[0, 1]` is shorthand for `m[(0, 1)]` — the comma builds an implicit
    // tuple inside the subscript. Common with dict-with-tuple-key access.
    let module = parse(
        "m = {}\n\
         x = m[0, 1]\n"
    );
    match &module.stmts[1].node {
        Stmt::Assign { value, .. } => match &value.node {
            Expr::Index { index, .. } => match &index.node {
                Expr::TupleLit(elems) => {
                    assert_eq!(elems.len(), 2,
                        "expected 2-element tuple index, got {} elems", elems.len());
                }
                other => panic!("expected TupleLit subscript index, got {other:?}"),
            },
            other => panic!("expected Index expr, got {other:?}"),
        },
        other => panic!("expected Assign, got {other:?}"),
    }
}

#[test]
fn test_subscript_with_three_comma_indices() {
    // `arr[a, b, c]` — generalize past 2-element case.
    let module = parse(
        "m = {}\n\
         x = m[1, 2, 3]\n"
    );
    match &module.stmts[1].node {
        Stmt::Assign { value, .. } => match &value.node {
            Expr::Index { index, .. } => match &index.node {
                Expr::TupleLit(elems) => assert_eq!(elems.len(), 3),
                other => panic!("expected TupleLit, got {other:?}"),
            },
            other => panic!("expected Index, got {other:?}"),
        },
        other => panic!("expected Assign, got {other:?}"),
    }
}

#[test]
fn test_chained_tuple_unpack_assign() {
    // `ka, va = ta = expr` — chain a tuple-unpack target with a simple target.
    // Both share the same RHS via a `__chained_N__` temp; mainline emits the
    // temp-assign stmt and queues the per-target assignments via pending_stmts.
    let module = parse(
        "def f():\n\
         \x20   ka, va = ta = (1, 2)\n"
    );
    match &module.stmts[0].node {
        Stmt::FnDef { body, .. } => {
            // Expect 3 stmts in the desugared body: __chained = (1,2);
            // (ka, va) = __chained; ta = __chained.
            assert_eq!(body.len(), 3,
                "expected 3 desugared stmts, got {}: {:?}",
                body.len(), body);
            // First stmt: __chained_N__ = (1, 2)
            match &body[0].node {
                Stmt::Assign { target, .. } => {
                    if let Expr::Ident(n) = &target.node {
                        assert!(n.starts_with("__chained_"), "expected chained temp, got {n}");
                    } else { panic!("expected ident target on temp assign, got {target:?}"); }
                }
                other => panic!("expected Assign, got {other:?}"),
            }
            // Second stmt: (ka, va) = __chained
            match &body[1].node {
                Stmt::Assign { target, .. } => {
                    assert!(matches!(&target.node, Expr::TupleLit(_)),
                        "expected TupleLit target, got {target:?}");
                }
                other => panic!("expected Assign, got {other:?}"),
            }
            // Third stmt: ta = __chained
            match &body[2].node {
                Stmt::Assign { target, .. } => {
                    assert!(matches!(&target.node, Expr::Ident(_)),
                        "expected Ident target, got {target:?}");
                }
                other => panic!("expected Assign, got {other:?}"),
            }
        }
        other => panic!("expected FnDef, got {other:?}"),
    }
}

#[test]
fn test_aug_assign_trailing_comma_tuple_rhs() {
    // `x += (a, b),` parses as `x += ((a, b),)` — single-element tuple via
    // trailing comma. Symmetric with plain `x = (a, b),` which already works.
    let module = parse(
        "success_cases = []\n\
         success_cases += (1, 2),\n"
    );
    match &module.stmts[1].node {
        Stmt::AugAssign { value, .. } => match &value.node {
            Expr::TupleLit(elems) => {
                assert_eq!(elems.len(), 1,
                    "expected single-element wrapper tuple, got {} elems", elems.len());
                assert!(matches!(&elems[0].node, Expr::TupleLit(_)),
                    "expected inner (1, 2) tuple, got {:?}", elems[0].node);
            }
            other => panic!("expected TupleLit RHS, got {other:?}"),
        },
        other => panic!("expected AugAssign, got {other:?}"),
    }
}

#[test]
fn test_aug_assign_bare_tuple_rhs() {
    // `x += a, b` — bare-tuple form (no parens) on aug-assign RHS.
    let module = parse(
        "x = ()\n\
         x += 1, 2\n"
    );
    match &module.stmts[1].node {
        Stmt::AugAssign { value, .. } => match &value.node {
            Expr::TupleLit(elems) => assert_eq!(elems.len(), 2),
            other => panic!("expected TupleLit RHS, got {other:?}"),
        },
        other => panic!("expected AugAssign, got {other:?}"),
    }
}
