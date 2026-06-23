use crate::error::MambaError;
use crate::hir::*;
use crate::parser::ast;
use crate::resolve::SymbolId;
use crate::source::span::{Span, Spanned};
use crate::types::{TypeChecker, TypeId};
/// AST → HIR lowering (#277).
///
/// Converts a parsed + type-checked AST into HIR, resolving all names to
/// SymbolIds and all types to TypeIds. Desugars compound constructs
/// (elif chains, augmented assignments).
use std::collections::HashMap;

/// Check whether a function body contains `yield` or `yield from` expressions.
///
/// This does NOT recurse into nested function/class/lambda definitions, since a
/// yield inside a nested function does not make the enclosing function a generator.
fn contains_yield(body: &[Spanned<ast::Stmt>]) -> bool {
    body.iter().any(|s| stmt_has_yield(&s.node))
}

fn stmt_has_yield(stmt: &ast::Stmt) -> bool {
    match stmt {
        // Do NOT recurse into nested defs / classes — their yields are their own.
        ast::Stmt::FnDef { .. }
        | ast::Stmt::AsyncFnDef { .. }
        | ast::Stmt::ClassDef { .. }
        | ast::Stmt::EnumDef { .. }
        | ast::Stmt::TypeAlias { .. }
        | ast::Stmt::Import { .. }
        | ast::Stmt::Pass
        | ast::Stmt::Break
        | ast::Stmt::Continue
        | ast::Stmt::Global(_)
        | ast::Stmt::Nonlocal(_) => false,

        // BareAnnotation has no value expression, so it never yields.
        ast::Stmt::BareAnnotation { .. } => false,
        ast::Stmt::ExprStmt(e) => expr_has_yield(&e.node),
        ast::Stmt::Return(opt) => opt.as_ref().map_or(false, |e| expr_has_yield(&e.node)),
        ast::Stmt::VarDecl { value, .. } => expr_has_yield(&value.node),
        ast::Stmt::Assign { target, value } => {
            expr_has_yield(&target.node) || expr_has_yield(&value.node)
        }
        ast::Stmt::AugAssign { target, value, .. } => {
            expr_has_yield(&target.node) || expr_has_yield(&value.node)
        }
        ast::Stmt::If {
            condition,
            body,
            elif_clauses,
            else_body,
        } => {
            expr_has_yield(&condition.node)
                || body.iter().any(|s| stmt_has_yield(&s.node))
                || elif_clauses.iter().any(|(c, b)| {
                    expr_has_yield(&c.node) || b.iter().any(|s| stmt_has_yield(&s.node))
                })
                || else_body
                    .as_ref()
                    .map_or(false, |b| b.iter().any(|s| stmt_has_yield(&s.node)))
        }
        ast::Stmt::While {
            condition,
            body,
            else_body,
        } => {
            expr_has_yield(&condition.node)
                || body.iter().any(|s| stmt_has_yield(&s.node))
                || else_body
                    .as_ref()
                    .map_or(false, |b| b.iter().any(|s| stmt_has_yield(&s.node)))
        }
        ast::Stmt::For {
            iter,
            body,
            else_body,
            ..
        }
        | ast::Stmt::AsyncFor {
            iter,
            body,
            else_body,
            ..
        } => {
            expr_has_yield(&iter.node)
                || body.iter().any(|s| stmt_has_yield(&s.node))
                || else_body
                    .as_ref()
                    .map_or(false, |b| b.iter().any(|s| stmt_has_yield(&s.node)))
        }
        ast::Stmt::Match { expr, arms } => {
            expr_has_yield(&expr.node)
                || arms
                    .iter()
                    .any(|a| a.body.iter().any(|s| stmt_has_yield(&s.node)))
        }
        ast::Stmt::Try {
            body,
            handlers,
            else_body,
            finally_body,
        } => {
            body.iter().any(|s| stmt_has_yield(&s.node))
                || handlers
                    .iter()
                    .any(|h| h.body.iter().any(|s| stmt_has_yield(&s.node)))
                || else_body
                    .as_ref()
                    .map_or(false, |b| b.iter().any(|s| stmt_has_yield(&s.node)))
                || finally_body
                    .as_ref()
                    .map_or(false, |b| b.iter().any(|s| stmt_has_yield(&s.node)))
        }
        ast::Stmt::Raise { value, from } => {
            value.as_ref().map_or(false, |e| expr_has_yield(&e.node))
                || from.as_ref().map_or(false, |e| expr_has_yield(&e.node))
        }
        ast::Stmt::With { items, body } | ast::Stmt::AsyncWith { items, body } => {
            items.iter().any(|w| expr_has_yield(&w.context.node))
                || body.iter().any(|s| stmt_has_yield(&s.node))
        }
        ast::Stmt::Assert { test, msg } => {
            expr_has_yield(&test.node) || msg.as_ref().map_or(false, |e| expr_has_yield(&e.node))
        }
        ast::Stmt::Del(e) => expr_has_yield(&e.node),
    }
}

fn expr_has_yield(expr: &ast::Expr) -> bool {
    match expr {
        ast::Expr::Yield(_) | ast::Expr::YieldFrom(_) => true,

        // Do NOT recurse into lambda — its yields are its own.
        ast::Expr::Lambda { .. } => false,

        // Leaf nodes
        ast::Expr::IntLit(_)
        | ast::Expr::FloatLit(_)
        | ast::Expr::ComplexLit(_)
        | ast::Expr::StrLit(_)
        | ast::Expr::BytesLit(_)
        | ast::Expr::BoolLit(_)
        | ast::Expr::NoneLit
        | ast::Expr::Ellipsis
        | ast::Expr::Ident(_) => false,

        ast::Expr::BinOp { lhs, rhs, .. } => expr_has_yield(&lhs.node) || expr_has_yield(&rhs.node),
        ast::Expr::UnaryOp { operand, .. } => expr_has_yield(&operand.node),
        ast::Expr::Call { func, args } => {
            expr_has_yield(&func.node)
                || args.iter().any(|a| match a {
                    ast::CallArg::Positional(e) => expr_has_yield(&e.node),
                    ast::CallArg::Keyword { value, .. } => expr_has_yield(&value.node),
                    ast::CallArg::StarArg(e) => expr_has_yield(&e.node),
                    ast::CallArg::DoubleStarArg(e) => expr_has_yield(&e.node),
                })
        }
        ast::Expr::Attr { object, .. } => expr_has_yield(&object.node),
        ast::Expr::Index { object, index } => {
            expr_has_yield(&object.node) || expr_has_yield(&index.node)
        }
        ast::Expr::Slice { start, stop, step } => {
            start.as_ref().map_or(false, |e| expr_has_yield(&e.node))
                || stop.as_ref().map_or(false, |e| expr_has_yield(&e.node))
                || step.as_ref().map_or(false, |e| expr_has_yield(&e.node))
        }
        ast::Expr::ListLit(es) | ast::Expr::SetLit(es) | ast::Expr::TupleLit(es) => {
            es.iter().any(|e| expr_has_yield(&e.node))
        }
        ast::Expr::DictLit(pairs) => pairs.iter().any(|(k, v)| {
            k.as_ref().map_or(false, |e| expr_has_yield(&e.node)) || expr_has_yield(&v.node)
        }),
        ast::Expr::IfExpr {
            body,
            condition,
            else_body,
        } => {
            expr_has_yield(&body.node)
                || expr_has_yield(&condition.node)
                || expr_has_yield(&else_body.node)
        }
        ast::Expr::ListComp {
            element,
            generators,
        }
        | ast::Expr::SetComp {
            element,
            generators,
        }
        | ast::Expr::GeneratorExpr {
            element,
            generators,
        } => {
            expr_has_yield(&element.node)
                || generators.iter().any(|g| {
                    expr_has_yield(&g.iter.node)
                        || g.conditions.iter().any(|c| expr_has_yield(&c.node))
                })
        }
        ast::Expr::DictComp {
            key,
            value,
            generators,
        } => {
            expr_has_yield(&key.node)
                || expr_has_yield(&value.node)
                || generators.iter().any(|g| {
                    expr_has_yield(&g.iter.node)
                        || g.conditions.iter().any(|c| expr_has_yield(&c.node))
                })
        }
        ast::Expr::FString(parts) => {
            fn part_has_yield(p: &ast::FStringPart) -> bool {
                match p {
                    ast::FStringPart::Literal(_) => false,
                    ast::FStringPart::Expr(e, spec) => {
                        expr_has_yield(&e.node) || spec.iter().flatten().any(part_has_yield)
                    }
                }
            }
            parts.iter().any(part_has_yield)
        }
        ast::Expr::Await(e) => expr_has_yield(&e.node),
        ast::Expr::Walrus { value, .. } => expr_has_yield(&value.node),
        ast::Expr::ChainedCompare { operands, .. } => {
            operands.iter().any(|e| expr_has_yield(&e.node))
        }
        ast::Expr::Starred(e) => expr_has_yield(&e.node),
        ast::Expr::UnpackTarget(es) => es.iter().any(|e| expr_has_yield(&e.node)),
    }
}

/// Coarse float/int classification of an AST expression used purely to soundly
/// infer an unannotated function's return type. `Unknown` means "could be either"
/// and is treated conservatively (the caller does not force a primitive type).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FloatHint {
    Float,
    Int,
    Unknown,
    /// Call sites pass a float at this position AND a non-float at another
    /// (or float mixed with an unknown). The param cannot use the raw-int or
    /// raw-float convention — a raw f64 in an int slot reinterprets its bits as
    /// an int — so it must be `any` (NaN-boxed). Only ever produced by the
    /// call-site hint merge, never by `ast_expr_float_hint`.
    Boxed,
}

/// Names of `math.*` (and `cmath.*`) functions that always return a Python
/// `float`. Used so `return math.sqrt(x)` is typed `float` even though the
/// argument param is unannotated.
fn math_attr_returns_float(attr: &str) -> bool {
    matches!(
        attr,
        "sqrt"
            | "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "atan2"
            | "sinh"
            | "cosh"
            | "tanh"
            | "asinh"
            | "acosh"
            | "atanh"
            | "exp"
            | "expm1"
            | "log"
            | "log2"
            | "log10"
            | "log1p"
            | "pow"
            | "hypot"
            | "fabs"
            | "fmod"
            | "remainder"
            | "dist"
            | "degrees"
            | "radians"
            | "gamma"
            | "lgamma"
            | "erf"
            | "erfc"
            | "copysign"
            | "nextafter"
            | "ulp"
            | "cbrt"
    )
}

/// `math.<name>` float-valued module constants (`math.pi`, `math.tau`, …).
/// Accessed as a bare `Attr` (not a call), so they need their own float hint;
/// without it `def f(): return math.pi` infers an int return and the caller
/// reboxes the float's raw bits as an integer.
fn math_attr_is_float_const(attr: &str) -> bool {
    matches!(attr, "pi" | "e" | "tau" | "inf" | "nan")
}

/// Classify an AST expression as float / int / unknown given an environment of
/// already-known local/param float hints. Conservative: anything not provably a
/// float stays `Unknown` (or `Int` for integer-only forms), so we never widen an
/// integer-returning function's type by mistake.
fn ast_expr_float_hint(
    expr: &ast::Expr,
    env: &HashMap<String, FloatHint>,
    func_ret_float: &HashMap<String, FloatHint>,
) -> FloatHint {
    use ast::{BinOp, UnaryOp};
    match expr {
        ast::Expr::FloatLit(_) => FloatHint::Float,
        ast::Expr::IntLit(_) => FloatHint::Int,
        ast::Expr::BoolLit(_) => FloatHint::Int,
        ast::Expr::Ident(name) => env.get(name).copied().unwrap_or(FloatHint::Unknown),
        ast::Expr::UnaryOp { op, operand } => match op {
            UnaryOp::Neg | UnaryOp::Pos => ast_expr_float_hint(&operand.node, env, func_ret_float),
            _ => FloatHint::Unknown,
        },
        ast::Expr::BinOp { op, lhs, rhs } => {
            let l = ast_expr_float_hint(&lhs.node, env, func_ret_float);
            let r = ast_expr_float_hint(&rhs.node, env, func_ret_float);
            match op {
                // True division always produces a float in Python 3.
                BinOp::Div => FloatHint::Float,
                BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Mod
                | BinOp::FloorDiv
                | BinOp::Pow => {
                    if l == FloatHint::Float || r == FloatHint::Float {
                        FloatHint::Float
                    } else if l == FloatHint::Int && r == FloatHint::Int {
                        // int Pow with a negative exponent can be float, but the
                        // common case (positive) is int; stay Unknown for Pow to
                        // avoid wrongly forcing int. Other ops on two ints are int.
                        if matches!(op, BinOp::Pow) {
                            FloatHint::Unknown
                        } else {
                            FloatHint::Int
                        }
                    } else {
                        FloatHint::Unknown
                    }
                }
                _ => FloatHint::Unknown,
            }
        }
        ast::Expr::Call { func, args } => {
            // abs(x) preserves the float-ness of its argument.
            if let ast::Expr::Ident(name) = &func.node {
                if name == "abs" && args.len() == 1 {
                    if let ast::CallArg::Positional(a) = &args[0] {
                        return ast_expr_float_hint(&a.node, env, func_ret_float);
                    }
                }
                if name == "float" {
                    return FloatHint::Float;
                }
                if name == "int" || name == "len" || name == "ord" || name == "hash" {
                    return FloatHint::Int;
                }
                // User function with a known float return type.
                if let Some(&h) = func_ret_float.get(name) {
                    return h;
                }
            }
            // math.sqrt(...) / math.sin(...) → float.
            if let ast::Expr::Attr { attr, .. } = &func.node {
                if math_attr_returns_float(attr) {
                    return FloatHint::Float;
                }
            }
            FloatHint::Unknown
        }
        // `math.pi` / `math.tau` / `math.inf` — bare float module constants.
        ast::Expr::Attr { object, attr } => {
            if let ast::Expr::Ident(m) = &object.node {
                if m == "math" && math_attr_is_float_const(attr) {
                    return FloatHint::Float;
                }
            }
            FloatHint::Unknown
        }
        // Subscripting a known float-element container (`xs[i]` where `xs` was
        // assigned a list/tuple of all-float elements) yields a float. The
        // container's element-floatness is recorded in `env` under the
        // `float_container_key` sentinel so the scalar Ident namespace is
        // untouched. Lets `echo(xs[0])` monomorphize `echo`'s param as float.
        ast::Expr::Index { object, .. } => {
            if let ast::Expr::Ident(name) = &object.node {
                if env.get(&float_container_key(name)).copied() == Some(FloatHint::Float) {
                    return FloatHint::Float;
                }
            }
            FloatHint::Unknown
        }
        _ => FloatHint::Unknown,
    }
}

/// Sentinel env key marking that local/global `name` holds a float-element
/// container (list/tuple/set of all-float values). The NUL prefix can never
/// collide with a real Python identifier, so these markers are inert for the
/// scalar Ident lookups that share the same `FloatHint` env.
fn float_container_key(name: &str) -> String {
    format!("\0[]{name}")
}

/// True when a list/tuple/set literal's elements are all statically float
/// (so subscripting the bound container yields a float).
fn ast_container_is_all_float(
    items: &[Spanned<ast::Expr>],
    env: &HashMap<String, FloatHint>,
    func_ret_float: &HashMap<String, FloatHint>,
) -> bool {
    !items.is_empty()
        && items
            .iter()
            .all(|e| ast_expr_float_hint(&e.node, env, func_ret_float) == FloatHint::Float)
}

/// Detect an unambiguous static call-binding violation of a known function's
/// signature, returning the CPython-faithful `TypeError` message (or `None`
/// when the call is valid or anything is uncertain — then the call lowers
/// normally). `sig` entries are `(name, has_default, kw_only, is_star,
/// is_double_star, pos_only)`. The caller guarantees `args` is splat-free.
///
/// Each check fires only where CPython unconditionally raises, so a correct
/// call never trips it. Checked in CPython's surfacing order so a single-fault
/// call yields the right message: duplicate value, positional-only-as-keyword,
/// Recursively collect names `f` that have `f.__defaults__` or
/// `f.__kwdefaults__` assigned anywhere in the statement tree. Such a runtime
/// mutation changes the effective default arguments, so static arg-count
/// validation must not fire for those functions.
fn collect_mutated_defaults(
    stmts: &[Spanned<ast::Stmt>],
    out: &mut std::collections::HashSet<String>,
) {
    for s in stmts {
        if let ast::Stmt::Assign { target, .. } = &s.node {
            if let ast::Expr::Attr { object, attr } = &target.node {
                if (attr == "__defaults__" || attr == "__kwdefaults__")
                    && matches!(&object.node, ast::Expr::Ident(_))
                {
                    if let ast::Expr::Ident(n) = &object.node {
                        out.insert(n.clone());
                    }
                }
            }
        }
        // Recurse into compound-statement bodies so a mutation nested in a
        // loop / conditional / function still disables the check.
        for body in stmt_child_bodies(&s.node) {
            collect_mutated_defaults(body, out);
        }
    }
}

/// Child statement-bodies of a compound statement, for recursive walks.
fn stmt_child_bodies(stmt: &ast::Stmt) -> Vec<&[Spanned<ast::Stmt>]> {
    let mut bodies: Vec<&[Spanned<ast::Stmt>]> = Vec::new();
    match stmt {
        ast::Stmt::If { body, else_body, .. }
        | ast::Stmt::While { body, else_body, .. }
        | ast::Stmt::For { body, else_body, .. } => {
            bodies.push(body);
            if let Some(e) = else_body { bodies.push(e); }
        }
        ast::Stmt::FnDef { body, .. } | ast::Stmt::AsyncFnDef { body, .. }
        | ast::Stmt::ClassDef { body, .. } => bodies.push(body),
        ast::Stmt::With { body, .. } | ast::Stmt::AsyncWith { body, .. } => bodies.push(body),
        ast::Stmt::Try { body, handlers, else_body, finally_body } => {
            bodies.push(body);
            for h in handlers { bodies.push(&h.body); }
            if let Some(e) = else_body { bodies.push(e); }
            if let Some(f) = finally_body { bodies.push(f); }
        }
        _ => {}
    }
    bodies
}

/// unexpected keyword, too many positional, missing required keyword-only.
fn arg_bind_violation(
    fname: &str,
    sig: &[(String, bool, bool, bool, bool, bool)],
    args: &[ast::CallArg],
    defaults_mutated: bool,
) -> Option<String> {
    let has_star = sig.iter().any(|p| p.3);
    let has_dstar = sig.iter().any(|p| p.4);
    // Params fillable by position, in declared order (not *,**, not kw-only).
    // Positional-only params are included — they bind by position.
    let pos_params: Vec<&(String, bool, bool, bool, bool, bool)> =
        sig.iter().filter(|p| !p.2 && !p.3 && !p.4).collect();
    let n_pos = args
        .iter()
        .filter(|a| matches!(a, ast::CallArg::Positional(_)))
        .count();
    let kw_names: Vec<&str> = args
        .iter()
        .filter_map(|a| match a {
            ast::CallArg::Keyword { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    let param_named = |n: &str| sig.iter().any(|p| !p.3 && !p.4 && p.0 == n);
    let is_pos_only = |n: &str| sig.iter().any(|p| p.5 && p.0 == n);

    // (1) Duplicate value: a keyword names a NON-positional-only param already
    // bound by position. (A positional-only name as keyword never binds to the
    // positional slot — it lands in **kwargs or is rejected below.)
    for (i, p) in pos_params.iter().enumerate() {
        if i < n_pos && !p.5 && kw_names.contains(&p.0.as_str()) {
            return Some(format!(
                "{fname}() got multiple values for argument '{}'",
                p.0
            ));
        }
    }

    // (2) Positional-only parameters passed by keyword (only an error without
    // `**kwargs` to absorb them). CPython lists all such names in one group.
    if !has_dstar {
        let posonly_kw: Vec<&str> = kw_names
            .iter()
            .copied()
            .filter(|n| is_pos_only(n))
            .collect();
        if !posonly_kw.is_empty() {
            return Some(format!(
                "{fname}() got some positional-only arguments passed as \
                 keyword arguments: '{}'",
                posonly_kw.join(", ")
            ));
        }
    }

    // (3) Unexpected keyword: names no parameter and there is no `**kwargs`.
    // Reports the first offending name, as CPython does.
    if !has_dstar {
        if let Some(bad) = kw_names.iter().find(|n| !param_named(n)) {
            return Some(format!(
                "{fname}() got an unexpected keyword argument '{bad}'"
            ));
        }
    }

    // (4) Too many positional, with no `*args` to absorb the overflow and no
    // positional default (a default switches CPython to "from X to Y" wording).
    // When keyword-only args are also supplied the count is reported in two
    // segments; otherwise the plain form. Only emit when every supplied keyword
    // binds a keyword-only param, so the two-segment count is exact.
    if !defaults_mutated && !has_star && n_pos > pos_params.len() && pos_params.iter().all(|p| !p.1) {
        let take = pos_params.len();
        let kwonly_supplied = kw_names
            .iter()
            .filter(|n| sig.iter().any(|p| p.2 && p.0 == **n))
            .count();
        let take_word = if take == 1 { "argument" } else { "arguments" };
        if kw_names.is_empty() {
            return Some(format!(
                "{fname}() takes {take} positional {take_word} but {n_pos} were given"
            ));
        } else if kwonly_supplied == kw_names.len() {
            let pos_word = if n_pos == 1 { "argument" } else { "arguments" };
            let k_word = if kwonly_supplied == 1 {
                "argument"
            } else {
                "arguments"
            };
            return Some(format!(
                "{fname}() takes {take} positional {take_word} but {n_pos} \
                 positional {pos_word} (and {kwonly_supplied} keyword-only \
                 {k_word}) were given"
            ));
        }
        // Mixed keyword shape — leave to the normal call path.
    }

    // (5) Missing required keyword-only argument(s): kw-only, no default, not
    // supplied by name.
    let missing: Vec<&str> = sig
        .iter()
        .filter(|p| p.2 && !p.3 && !p.4 && !p.1)
        .map(|p| p.0.as_str())
        .filter(|n| !kw_names.contains(n))
        .collect();
    if !missing.is_empty() {
        let n = missing.len();
        let word = if n == 1 { "argument" } else { "arguments" };
        let names = missing
            .iter()
            .map(|m| format!("'{m}'"))
            .collect::<Vec<_>>()
            .join(if n == 2 { " and " } else { ", " });
        return Some(format!(
            "{fname}() missing {n} required keyword-only {word}: {names}"
        ));
    }

    // (6) Missing required positional argument(s): a positional param with no
    // default, not filled by position and not supplied by name. Only fires when
    // there is no `*args` (which never supplies a *named* param anyway) — a
    // required positional left unbound is always a TypeError.
    let missing_pos: Vec<&str> = if defaults_mutated {
        Vec::new()
    } else {
        pos_params.iter().enumerate()
            .filter(|(i, p)| *i >= n_pos && !p.1 && !kw_names.contains(&p.0.as_str()))
            .map(|(_, p)| p.0.as_str())
            .collect()
    };
    if !missing_pos.is_empty() {
        let n = missing_pos.len();
        let word = if n == 1 { "argument" } else { "arguments" };
        let names = missing_pos.iter()
            .map(|m| format!("'{m}'"))
            .collect::<Vec<_>>()
            .join(if n == 2 { " and " } else { ", " });
        return Some(format!(
            "{fname}() missing {n} required positional {word}: {names}"
        ));
    }

    None
}

/// Walk a function body collecting a name→FloatHint environment for locals that
/// are assigned a provably-float (or provably-int) value. Seeded with parameter
/// hints. Lets `total = a / b; return total` infer a float return.
fn collect_local_float_env(
    body: &[Spanned<ast::Stmt>],
    env: &mut HashMap<String, FloatHint>,
    func_ret_float: &HashMap<String, FloatHint>,
) {
    for stmt in body {
        match &stmt.node {
            ast::Stmt::Assign { target, value } => {
                if let ast::Expr::Ident(name) = &target.node {
                    let h = ast_expr_float_hint(&value.node, env, func_ret_float);
                    match env.get(name).copied() {
                        // Promote toward float; once float, a later int/unknown
                        // assignment makes it unknown (could be either at return).
                        Some(prev) if prev != h => {
                            env.insert(name.clone(), FloatHint::Unknown);
                        }
                        _ => {
                            env.insert(name.clone(), h);
                        }
                    }
                }
            }
            ast::Stmt::VarDecl { name, value, .. } => {
                let h = ast_expr_float_hint(&value.node, env, func_ret_float);
                env.insert(name.clone(), h);
            }
            ast::Stmt::AugAssign { target, value, .. } => {
                if let ast::Expr::Ident(name) = &target.node {
                    let cur = env.get(name).copied().unwrap_or(FloatHint::Unknown);
                    let vh = ast_expr_float_hint(&value.node, env, func_ret_float);
                    let merged = if cur == FloatHint::Float || vh == FloatHint::Float {
                        FloatHint::Float
                    } else if cur == FloatHint::Int && vh == FloatHint::Int {
                        FloatHint::Int
                    } else {
                        FloatHint::Unknown
                    };
                    env.insert(name.clone(), merged);
                }
            }
            // Recurse into control flow so locals assigned in branches are seen.
            ast::Stmt::If {
                body: b, else_body, ..
            } => {
                collect_local_float_env(b, env, func_ret_float);
                if let Some(els) = else_body {
                    collect_local_float_env(els, env, func_ret_float);
                }
            }
            ast::Stmt::While { body: b, .. } | ast::Stmt::For { body: b, .. } => {
                collect_local_float_env(b, env, func_ret_float);
            }
            _ => {}
        }
    }
}

/// Gather every positional call-argument float hint for user functions across a
/// statement tree (recurses into bodies and nested expressions). Hints are merged
/// per position: a position stays Float only if every observed argument is Float;
/// a single Int/Unknown demotes it.
fn collect_call_arg_hints(
    stmts: &[Spanned<ast::Stmt>],
    env: &HashMap<String, FloatHint>,
    func_ret: &HashMap<String, FloatHint>,
    out: &mut HashMap<String, Vec<FloatHint>>,
    seen: &mut std::collections::HashSet<String>,
) {
    fn merge(out: &mut HashMap<String, Vec<FloatHint>>, name: &str, hs: Vec<FloatHint>) {
        let entry = out.entry(name.to_string()).or_default();
        for (i, h) in hs.into_iter().enumerate() {
            if i >= entry.len() {
                entry.push(h);
            } else {
                let cur = entry[i];
                entry[i] = if cur == h {
                    cur
                } else if cur == FloatHint::Boxed
                    || h == FloatHint::Boxed
                    || cur == FloatHint::Float
                    || h == FloatHint::Float
                {
                    // A float at one site mixed with anything else at another:
                    // the param must be NaN-boxed (`any`), never raw-int.
                    FloatHint::Boxed
                } else {
                    FloatHint::Unknown
                };
            }
        }
    }
    fn walk_expr(
        e: &ast::Expr,
        env: &HashMap<String, FloatHint>,
        func_ret: &HashMap<String, FloatHint>,
        out: &mut HashMap<String, Vec<FloatHint>>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        if let ast::Expr::Call { func, args } = e {
            if let ast::Expr::Ident(name) = &func.node {
                // Only record when all args are simple positional.
                if args
                    .iter()
                    .all(|a| matches!(a, ast::CallArg::Positional(_)))
                {
                    seen.insert(name.clone());
                    let hs: Vec<FloatHint> = args
                        .iter()
                        .map(|a| match a {
                            ast::CallArg::Positional(x) => {
                                ast_expr_float_hint(&x.node, env, func_ret)
                            }
                            _ => FloatHint::Unknown,
                        })
                        .collect();
                    merge(out, name, hs);
                }
            }
            // Recurse into call's func + args.
            walk_expr(&func.node, env, func_ret, out, seen);
            for a in args {
                if let ast::CallArg::Positional(x)
                | ast::CallArg::Keyword { value: x, .. }
                | ast::CallArg::StarArg(x)
                | ast::CallArg::DoubleStarArg(x) = a
                {
                    walk_expr(&x.node, env, func_ret, out, seen);
                }
            }
            return;
        }
        // Generic structural recursion for other expr kinds we care about.
        match e {
            ast::Expr::BinOp { lhs, rhs, .. } => {
                walk_expr(&lhs.node, env, func_ret, out, seen);
                walk_expr(&rhs.node, env, func_ret, out, seen);
            }
            ast::Expr::UnaryOp { operand, .. } => {
                walk_expr(&operand.node, env, func_ret, out, seen)
            }
            ast::Expr::Attr { object, .. } => walk_expr(&object.node, env, func_ret, out, seen),
            ast::Expr::Index { object, index } => {
                walk_expr(&object.node, env, func_ret, out, seen);
                walk_expr(&index.node, env, func_ret, out, seen);
            }
            ast::Expr::ListLit(items) | ast::Expr::TupleLit(items) | ast::Expr::SetLit(items) => {
                for it in items {
                    walk_expr(&it.node, env, func_ret, out, seen);
                }
            }
            _ => {}
        }
    }
    fn walk_stmts(
        stmts: &[Spanned<ast::Stmt>],
        env: &HashMap<String, FloatHint>,
        func_ret: &HashMap<String, FloatHint>,
        out: &mut HashMap<String, Vec<FloatHint>>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        for s in stmts {
            match &s.node {
                ast::Stmt::ExprStmt(e) => walk_expr(&e.node, env, func_ret, out, seen),
                ast::Stmt::Assign { value, .. } => walk_expr(&value.node, env, func_ret, out, seen),
                ast::Stmt::VarDecl { value, .. } => {
                    walk_expr(&value.node, env, func_ret, out, seen)
                }
                ast::Stmt::AugAssign { value, .. } => {
                    walk_expr(&value.node, env, func_ret, out, seen)
                }
                ast::Stmt::Return(Some(e)) => walk_expr(&e.node, env, func_ret, out, seen),
                ast::Stmt::If {
                    condition,
                    body,
                    else_body,
                    ..
                } => {
                    walk_expr(&condition.node, env, func_ret, out, seen);
                    walk_stmts(body, env, func_ret, out, seen);
                    if let Some(els) = else_body {
                        walk_stmts(els, env, func_ret, out, seen);
                    }
                }
                ast::Stmt::While {
                    condition,
                    body,
                    else_body,
                    ..
                } => {
                    walk_expr(&condition.node, env, func_ret, out, seen);
                    walk_stmts(body, env, func_ret, out, seen);
                    if let Some(els) = else_body {
                        walk_stmts(els, env, func_ret, out, seen);
                    }
                }
                ast::Stmt::For {
                    iter,
                    body,
                    else_body,
                    ..
                } => {
                    walk_expr(&iter.node, env, func_ret, out, seen);
                    walk_stmts(body, env, func_ret, out, seen);
                    if let Some(els) = else_body {
                        walk_stmts(els, env, func_ret, out, seen);
                    }
                }
                ast::Stmt::FnDef { body, .. } | ast::Stmt::AsyncFnDef { body, .. } => {
                    walk_stmts(body, env, func_ret, out, seen);
                }
                _ => {}
            }
        }
    }
    walk_stmts(stmts, env, func_ret, out, seen);
}

/// Merge the float hints of all `return` statements in a body into a single
/// function-level return hint. `Float` only when at least one return is provably
/// float and none is provably int.
fn infer_return_float_hint(
    body: &[Spanned<ast::Stmt>],
    env: &HashMap<String, FloatHint>,
    func_ret: &HashMap<String, FloatHint>,
) -> FloatHint {
    let mut any_float = false;
    let mut any_int = false;
    fn walk(
        body: &[Spanned<ast::Stmt>],
        env: &HashMap<String, FloatHint>,
        func_ret: &HashMap<String, FloatHint>,
        any_float: &mut bool,
        any_int: &mut bool,
    ) {
        for s in body {
            match &s.node {
                ast::Stmt::Return(Some(e)) => {
                    match ast_expr_float_hint(&e.node, env, func_ret) {
                        FloatHint::Float => *any_float = true,
                        FloatHint::Int => *any_int = true,
                        // `Boxed` is a call-site-merge-only state; ast_expr_float_hint
                        // never yields it. Treat like Unknown for return inference.
                        FloatHint::Unknown | FloatHint::Boxed => {}
                    }
                }
                ast::Stmt::If {
                    body: b, else_body, ..
                }
                | ast::Stmt::While {
                    body: b, else_body, ..
                } => {
                    walk(b, env, func_ret, any_float, any_int);
                    if let Some(els) = else_body {
                        walk(els, env, func_ret, any_float, any_int);
                    }
                }
                ast::Stmt::For {
                    body: b, else_body, ..
                } => {
                    walk(b, env, func_ret, any_float, any_int);
                    if let Some(els) = else_body {
                        walk(els, env, func_ret, any_float, any_int);
                    }
                }
                _ => {}
            }
        }
    }
    walk(body, env, func_ret, &mut any_float, &mut any_int);
    if any_float && !any_int {
        FloatHint::Float
    } else {
        FloatHint::Unknown
    }
}

fn add_expr_contains_string_literal(expr: &ast::Expr) -> bool {
    match expr {
        ast::Expr::StrLit(_) => true,
        ast::Expr::BinOp { op: ast::BinOp::Add, lhs, rhs } => {
            add_expr_contains_string_literal(&lhs.node)
                || add_expr_contains_string_literal(&rhs.node)
        }
        ast::Expr::IfExpr { body, else_body, .. } => {
            add_expr_contains_string_literal(&body.node)
                || add_expr_contains_string_literal(&else_body.node)
        }
        _ => false,
    }
}

/// Infer a function's return type from its `return` statements.
///
/// Recognizes literal returns (Float, Bool, Str, None) and, using the supplied
/// param/local float-hint `env`, also typed float arithmetic / float params /
/// float locals / float-returning calls (`math.sqrt`, `abs` of a float, …).
/// Returns `None` (caller falls back to `int_ty`) only when the return value is
/// not provably float and not a recognized non-int literal — so integer-returning
/// functions are never widened.
fn infer_return_type_from_ast(
    body: &[Spanned<ast::Stmt>],
    tc: &TypeChecker,
    env: &HashMap<String, FloatHint>,
    func_ret_float: &HashMap<String, FloatHint>,
) -> Option<TypeId> {
    for stmt in body {
        match &stmt.node {
            ast::Stmt::Return(Some(expr)) => {
                return match &expr.node {
                    ast::Expr::FloatLit(_) => Some(tc.tcx.float()),
                    ast::Expr::BoolLit(_) => Some(tc.tcx.bool()),
                    ast::Expr::StrLit(_) => Some(tc.tcx.str()),
                    ast::Expr::NoneLit => Some(tc.tcx.none()),
                    // Container returns are provably heap values — falling
                    // through to int_ty would make binops on the call result
                    // take the raw-i64 fast path (e.g. `f() | g()` compiles
                    // to a bitwise `bor` of two NaN-boxed POINTERS → UB).
                    // Widen to Any so operators dispatch through the runtime.
                    ast::Expr::ListLit(_)
                    | ast::Expr::SetLit(_)
                    | ast::Expr::DictLit(_)
                    | ast::Expr::TupleLit(_) => Some(tc.tcx.any()),
                    ast::Expr::Call { func, .. }
                        if matches!(
                            &func.node,
                            ast::Expr::Ident(n) if matches!(
                                n.as_str(),
                                "set" | "frozenset" | "dict" | "list" | "tuple" | "sorted"
                            )
                        ) =>
                    {
                        Some(tc.tcx.any())
                    }
                    ast::Expr::BinOp { op: ast::BinOp::Add, lhs, rhs }
                        if add_expr_contains_string_literal(&lhs.node)
                            || add_expr_contains_string_literal(&rhs.node) =>
                    {
                        // A string-concat-shaped return is a boxed runtime
                        // value even when part of the expression is Any. Do
                        // not let the raw-int fallback make callers compile
                        // `f() + g()` as primitive iadd over string pointers.
                        Some(tc.tcx.any())
                    }
                    _ => {
                        // Consult the float-hint analysis for non-literal returns.
                        // Only force `float` when provably float; otherwise fall
                        // through (None → int_ty) to preserve integer fast paths.
                        match ast_expr_float_hint(&expr.node, env, func_ret_float) {
                            FloatHint::Float => Some(tc.tcx.float()),
                            // Returning a NaN-boxed (`any`) value — e.g. an
                            // unannotated param that mixed-typed call sites
                            // boxed. Widen to Any so the caller treats the
                            // result as a boxed MbValue, not raw i64.
                            FloatHint::Boxed => Some(tc.tcx.any()),
                            _ => None,
                        }
                    }
                };
            }
            // Recurse into every compound statement that can hold a `return`,
            // returning the first inferred type. Without this, a function whose
            // only returns live inside `try`/`while`/`for`/`with`/`match`
            // bodies had no inferred return type and defaulted to the raw-int
            // (`int_ty`) convention, so `return True`/`return 1.5` from inside
            // a `try` block surfaced as `0`/`1` or raw f64 bits at the call
            // site. Mirrors the existing `if` handling.
            ast::Stmt::If {
                body: if_body,
                elif_clauses,
                else_body,
                ..
            } => {
                if let Some(ty) = infer_return_type_from_ast(if_body, tc, env, func_ret_float) {
                    return Some(ty);
                }
                for (_, elif_body) in elif_clauses {
                    if let Some(ty) =
                        infer_return_type_from_ast(elif_body, tc, env, func_ret_float)
                    {
                        return Some(ty);
                    }
                }
                if let Some(els) = else_body {
                    if let Some(ty) = infer_return_type_from_ast(els, tc, env, func_ret_float) {
                        return Some(ty);
                    }
                }
            }
            ast::Stmt::While { body, else_body, .. }
            | ast::Stmt::For { body, else_body, .. }
            | ast::Stmt::AsyncFor { body, else_body, .. } => {
                if let Some(ty) = infer_return_type_from_ast(body, tc, env, func_ret_float) {
                    return Some(ty);
                }
                if let Some(els) = else_body {
                    if let Some(ty) = infer_return_type_from_ast(els, tc, env, func_ret_float) {
                        return Some(ty);
                    }
                }
            }
            ast::Stmt::With { body, .. } | ast::Stmt::AsyncWith { body, .. } => {
                if let Some(ty) = infer_return_type_from_ast(body, tc, env, func_ret_float) {
                    return Some(ty);
                }
            }
            ast::Stmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
            } => {
                if let Some(ty) = infer_return_type_from_ast(body, tc, env, func_ret_float) {
                    return Some(ty);
                }
                for handler in handlers {
                    if let Some(ty) =
                        infer_return_type_from_ast(&handler.body, tc, env, func_ret_float)
                    {
                        return Some(ty);
                    }
                }
                if let Some(els) = else_body {
                    if let Some(ty) = infer_return_type_from_ast(els, tc, env, func_ret_float) {
                        return Some(ty);
                    }
                }
                if let Some(fin) = finally_body {
                    if let Some(ty) = infer_return_type_from_ast(fin, tc, env, func_ret_float) {
                        return Some(ty);
                    }
                }
            }
            ast::Stmt::Match { arms, .. } => {
                for arm in arms {
                    if let Some(ty) =
                        infer_return_type_from_ast(&arm.body, tc, env, func_ret_float)
                    {
                        return Some(ty);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Collect the names of unannotated params that must keep boxed value semantics:
/// direct operands of `==`, `!=`, `in`, or `not in` (including chained
/// comparisons), and direct arguments to runtime type checks such as
/// `isinstance` / `issubclass`.
///
/// Unannotated params default to the raw-int (`int_ty`) calling convention so
/// genuine integer params keep the fast native ABI. But when a param is the
/// direct operand of an equality/membership comparison, the int-typed lowering
/// emits a pointer-identity `icmp` instead of the value-comparing runtime
/// dispatch (`mb_eq`/`mb_list_contains`). For heap operands (list/str/tuple/
/// dict/set/bytes) that silently compares by identity, which is wrong. Promoting
/// just those params to `any` routes their `==`/`in` through the NaN-aware
/// runtime so value comparison is correct.
///
/// This is intentionally narrow: only equality/membership operand positions
/// and runtime type-check arguments trigger promotion. Params used only in
/// arithmetic, indexing, or as kwargs to native calls (e.g.
/// `datetime(..., hour=h)`, `int(x, base=16)`, `round(x, ndigits=2)`) keep
/// `int_ty`, preserving both the raw-int fast path and the native kwargs ABI.
fn collect_value_compared_params(
    body: &[Spanned<ast::Stmt>],
    param_names: &std::collections::HashSet<String>,
    out: &mut std::collections::HashSet<String>,
) {
    for stmt in body {
        stmt_collect_value_compared_params(&stmt.node, param_names, out);
    }
}

fn stmt_collect_value_compared_params(
    stmt: &ast::Stmt,
    params: &std::collections::HashSet<String>,
    out: &mut std::collections::HashSet<String>,
) {
    use ast::Stmt::*;
    let mut scan = |e: &Spanned<ast::Expr>, out: &mut std::collections::HashSet<String>| {
        expr_collect_value_compared_params(&e.node, params, out);
    };
    match stmt {
        ExprStmt(e) | Del(e) => scan(e, out),
        Return(opt) => {
            if let Some(e) = opt {
                scan(e, out);
            }
        }
        VarDecl { value, .. } => scan(value, out),
        Assign { target, value } => {
            scan(target, out);
            scan(value, out);
        }
        AugAssign { target, value, .. } => {
            scan(target, out);
            scan(value, out);
        }
        If {
            condition,
            body,
            elif_clauses,
            else_body,
        } => {
            scan(condition, out);
            collect_value_compared_params(body, params, out);
            for (c, b) in elif_clauses {
                scan(c, out);
                collect_value_compared_params(b, params, out);
            }
            if let Some(b) = else_body {
                collect_value_compared_params(b, params, out);
            }
        }
        While {
            condition,
            body,
            else_body,
        } => {
            scan(condition, out);
            collect_value_compared_params(body, params, out);
            if let Some(b) = else_body {
                collect_value_compared_params(b, params, out);
            }
        }
        For {
            iter,
            body,
            else_body,
            ..
        }
        | AsyncFor {
            iter,
            body,
            else_body,
            ..
        } => {
            scan(iter, out);
            collect_value_compared_params(body, params, out);
            if let Some(b) = else_body {
                collect_value_compared_params(b, params, out);
            }
        }
        Match { expr, arms } => {
            scan(expr, out);
            for a in arms {
                if let Some(g) = &a.guard {
                    scan(g, out);
                }
                collect_value_compared_params(&a.body, params, out);
            }
        }
        Try {
            body,
            handlers,
            else_body,
            finally_body,
        } => {
            collect_value_compared_params(body, params, out);
            for h in handlers {
                collect_value_compared_params(&h.body, params, out);
            }
            if let Some(b) = else_body {
                collect_value_compared_params(b, params, out);
            }
            if let Some(b) = finally_body {
                collect_value_compared_params(b, params, out);
            }
        }
        Raise { value, from } => {
            if let Some(e) = value {
                scan(e, out);
            }
            if let Some(e) = from {
                scan(e, out);
            }
        }
        With { items, body } | AsyncWith { items, body } => {
            for w in items {
                scan(&w.context, out);
            }
            collect_value_compared_params(body, params, out);
        }
        Assert { test, msg } => {
            scan(test, out);
            if let Some(e) = msg {
                scan(e, out);
            }
        }
        // Nested defs/classes have their own scope; their params shadow ours.
        // Do not descend — a same-named inner param is a different binding.
        FnDef { .. } | AsyncFnDef { .. } | ClassDef { .. } | EnumDef { .. } => {}
        _ => {}
    }
}

fn expr_collect_value_compared_params(
    expr: &ast::Expr,
    params: &std::collections::HashSet<String>,
    out: &mut std::collections::HashSet<String>,
) {
    use ast::Expr::*;
    // If this is an equality/membership comparison, record any direct
    // Ident operand that names a param.
    let mark = |e: &Spanned<ast::Expr>, out: &mut std::collections::HashSet<String>| {
        if let ast::Expr::Ident(n) = &e.node {
            if params.contains(n) {
                out.insert(n.clone());
            }
        }
    };
    if let BinOp { op, lhs, rhs } = expr {
        if matches!(
            op,
            ast::BinOp::Eq | ast::BinOp::NotEq | ast::BinOp::In | ast::BinOp::NotIn
        ) {
            mark(lhs, out);
            mark(rhs, out);
        }
    }
    if let ChainedCompare { operands, ops } = expr {
        for (i, op) in ops.iter().enumerate() {
            if matches!(
                op,
                ast::BinOp::Eq | ast::BinOp::NotEq | ast::BinOp::In | ast::BinOp::NotIn
            ) {
                if let Some(l) = operands.get(i) {
                    mark(l, out);
                }
                if let Some(r) = operands.get(i + 1) {
                    mark(r, out);
                }
            }
        }
    }
    if let Call { func, args } = expr {
        let is_runtime_type_check = matches!(
            &func.node,
            ast::Expr::Ident(name) if name == "isinstance" || name == "issubclass"
        );
        if is_runtime_type_check {
            for arg in args.iter().take(2) {
                if let ast::CallArg::Positional(e) = arg {
                    mark(e, out);
                }
            }
        }
    }
    // Recurse into all sub-expressions so nested comparisons are seen.
    let mut rec = |e: &Spanned<ast::Expr>, out: &mut std::collections::HashSet<String>| {
        expr_collect_value_compared_params(&e.node, params, out);
    };
    match expr {
        BinOp { lhs, rhs, .. } => {
            rec(lhs, out);
            rec(rhs, out);
        }
        UnaryOp { operand, .. } => rec(operand, out),
        Call { func, args } => {
            rec(func, out);
            for a in args {
                match a {
                    ast::CallArg::Positional(e)
                    | ast::CallArg::StarArg(e)
                    | ast::CallArg::DoubleStarArg(e) => rec(e, out),
                    ast::CallArg::Keyword { value, .. } => rec(value, out),
                }
            }
        }
        Attr { object, .. } => rec(object, out),
        Index { object, index } => {
            rec(object, out);
            rec(index, out);
        }
        Slice { start, stop, step } => {
            if let Some(e) = start {
                rec(e, out);
            }
            if let Some(e) = stop {
                rec(e, out);
            }
            if let Some(e) = step {
                rec(e, out);
            }
        }
        ListLit(es) | SetLit(es) | TupleLit(es) | UnpackTarget(es) => {
            for e in es {
                rec(e, out);
            }
        }
        DictLit(pairs) => {
            for (k, v) in pairs {
                if let Some(k) = k {
                    rec(k, out);
                }
                rec(v, out);
            }
        }
        IfExpr {
            body,
            condition,
            else_body,
        } => {
            rec(body, out);
            rec(condition, out);
            rec(else_body, out);
        }
        ListComp {
            element,
            generators,
        }
        | SetComp {
            element,
            generators,
        }
        | GeneratorExpr {
            element,
            generators,
        } => {
            rec(element, out);
            for g in generators {
                rec(&g.iter, out);
                for c in &g.conditions {
                    rec(c, out);
                }
            }
        }
        DictComp {
            key,
            value,
            generators,
        } => {
            rec(key, out);
            rec(value, out);
            for g in generators {
                rec(&g.iter, out);
                for c in &g.conditions {
                    rec(c, out);
                }
            }
        }
        FString(parts) => {
            fn walk_parts(
                parts: &[ast::FStringPart],
                out: &mut std::collections::HashSet<String>,
                rec: &impl Fn(&Spanned<ast::Expr>, &mut std::collections::HashSet<String>),
            ) {
                for p in parts {
                    if let ast::FStringPart::Expr(e, spec) = p {
                        rec(e, out);
                        if let Some(sp) = spec {
                            walk_parts(sp, out, rec);
                        }
                    }
                }
            }
            walk_parts(parts, out, &rec);
        }
        Yield(opt) => {
            if let Some(e) = opt {
                rec(e, out);
            }
        }
        YieldFrom(e) | Await(e) | Starred(e) => rec(e, out),
        Walrus { value, .. } => rec(value, out),
        ChainedCompare { operands, .. } => {
            for e in operands {
                rec(e, out);
            }
        }
        // Lambda has its own param scope; same-named params shadow ours.
        Lambda { .. } => {}
        _ => {}
    }
}

/// Collect the names of params used in a NUMERIC position — as an operand of
/// arithmetic / bitwise / ordering ops, as a unary-arith operand, as the target
/// of an augmented arithmetic assignment, or as an argument to a numeric builtin
/// (`abs`, `round`, `pow`, `divmod`, `min`, `max`, `sum`, `math.*`, `int`,
/// `float`, etc.).
///
/// Such params must keep the raw-int (`int_ty`) calling convention so the native
/// numeric fast path stays intact: a float param is passed as NaN-boxed bits in
/// an i64 slot, and promoting it to `any` would change how downstream `abs`,
/// `math.isnan`, and arithmetic interpret it. A param that is BOTH value-compared
/// (`x == y`) and numeric (`abs(x - y)`) is therefore NOT promoted — it stays
/// `int`, which is correct for numerics and the original (pre-fix) behavior for
/// its equality.
fn collect_numeric_used_params(
    body: &[Spanned<ast::Stmt>],
    param_names: &std::collections::HashSet<String>,
    out: &mut std::collections::HashSet<String>,
) {
    for stmt in body {
        stmt_collect_numeric_used_params(&stmt.node, param_names, out);
    }
}

/// Builtin / math function names whose params are treated as numeric.
fn is_numeric_callee(func: &ast::Expr) -> bool {
    match func {
        ast::Expr::Ident(n) => matches!(
            n.as_str(),
            "abs"
                | "round"
                | "pow"
                | "divmod"
                | "min"
                | "max"
                | "sum"
                | "int"
                | "float"
                | "complex"
                | "bin"
                | "hex"
                | "oct"
                | "bool"
        ),
        // math.sqrt, math.isnan, math.copysign, math.floor, ... — any `math.*`.
        ast::Expr::Attr { object, .. } => matches!(
            &object.node, ast::Expr::Ident(m) if m == "math" || m == "cmath"
        ),
        _ => false,
    }
}

fn stmt_collect_numeric_used_params(
    stmt: &ast::Stmt,
    params: &std::collections::HashSet<String>,
    out: &mut std::collections::HashSet<String>,
) {
    use ast::Stmt::*;
    let scan = |e: &Spanned<ast::Expr>, out: &mut std::collections::HashSet<String>| {
        expr_collect_numeric_used_params(&e.node, params, out);
    };
    let mark = |e: &Spanned<ast::Expr>, out: &mut std::collections::HashSet<String>| {
        if let ast::Expr::Ident(n) = &e.node {
            if params.contains(n) {
                out.insert(n.clone());
            }
        }
    };
    match stmt {
        ExprStmt(e) | Del(e) => scan(e, out),
        Return(opt) => {
            if let Some(e) = opt {
                scan(e, out);
            }
        }
        VarDecl { value, .. } => scan(value, out),
        Assign { target, value } => {
            scan(target, out);
            scan(value, out);
        }
        AugAssign { target, value, op } => {
            // `x += ...` arithmetic ops treat the target as numeric.
            if matches!(
                op,
                ast::AugOp::Add
                    | ast::AugOp::Sub
                    | ast::AugOp::Mul
                    | ast::AugOp::Div
                    | ast::AugOp::FloorDiv
                    | ast::AugOp::Mod
                    | ast::AugOp::Pow
                    | ast::AugOp::BitAnd
                    | ast::AugOp::BitOr
                    | ast::AugOp::BitXor
                    | ast::AugOp::LShift
                    | ast::AugOp::RShift
            ) {
                mark(target, out);
            }
            scan(target, out);
            scan(value, out);
        }
        If {
            condition,
            body,
            elif_clauses,
            else_body,
        } => {
            scan(condition, out);
            collect_numeric_used_params(body, params, out);
            for (c, b) in elif_clauses {
                scan(c, out);
                collect_numeric_used_params(b, params, out);
            }
            if let Some(b) = else_body {
                collect_numeric_used_params(b, params, out);
            }
        }
        While {
            condition,
            body,
            else_body,
        } => {
            scan(condition, out);
            collect_numeric_used_params(body, params, out);
            if let Some(b) = else_body {
                collect_numeric_used_params(b, params, out);
            }
        }
        For {
            iter,
            body,
            else_body,
            ..
        }
        | AsyncFor {
            iter,
            body,
            else_body,
            ..
        } => {
            scan(iter, out);
            collect_numeric_used_params(body, params, out);
            if let Some(b) = else_body {
                collect_numeric_used_params(b, params, out);
            }
        }
        Match { expr, arms } => {
            scan(expr, out);
            for a in arms {
                if let Some(g) = &a.guard {
                    scan(g, out);
                }
                collect_numeric_used_params(&a.body, params, out);
            }
        }
        Try {
            body,
            handlers,
            else_body,
            finally_body,
        } => {
            collect_numeric_used_params(body, params, out);
            for h in handlers {
                collect_numeric_used_params(&h.body, params, out);
            }
            if let Some(b) = else_body {
                collect_numeric_used_params(b, params, out);
            }
            if let Some(b) = finally_body {
                collect_numeric_used_params(b, params, out);
            }
        }
        Raise { value, from } => {
            if let Some(e) = value {
                scan(e, out);
            }
            if let Some(e) = from {
                scan(e, out);
            }
        }
        With { items, body } | AsyncWith { items, body } => {
            for w in items {
                scan(&w.context, out);
            }
            collect_numeric_used_params(body, params, out);
        }
        Assert { test, msg } => {
            scan(test, out);
            if let Some(e) = msg {
                scan(e, out);
            }
        }
        FnDef { .. } | AsyncFnDef { .. } | ClassDef { .. } | EnumDef { .. } => {}
        _ => {}
    }
}

fn expr_collect_numeric_used_params(
    expr: &ast::Expr,
    params: &std::collections::HashSet<String>,
    out: &mut std::collections::HashSet<String>,
) {
    use ast::Expr::*;
    let mark = |e: &Spanned<ast::Expr>, out: &mut std::collections::HashSet<String>| {
        if let ast::Expr::Ident(n) = &e.node {
            if params.contains(n) {
                out.insert(n.clone());
            }
        }
    };
    // Arithmetic / bitwise / ordering binops mark direct Ident operands numeric.
    if let BinOp { op, lhs, rhs } = expr {
        if matches!(
            op,
            ast::BinOp::Add
                | ast::BinOp::Sub
                | ast::BinOp::Mul
                | ast::BinOp::Div
                | ast::BinOp::FloorDiv
                | ast::BinOp::Mod
                | ast::BinOp::Pow
                | ast::BinOp::MatMul
                | ast::BinOp::BitAnd
                | ast::BinOp::BitOr
                | ast::BinOp::BitXor
                | ast::BinOp::LShift
                | ast::BinOp::RShift
                | ast::BinOp::Lt
                | ast::BinOp::Gt
                | ast::BinOp::LtEq
                | ast::BinOp::GtEq
        ) {
            mark(lhs, out);
            mark(rhs, out);
        }
    }
    if let ChainedCompare { operands, ops } = expr {
        for (i, op) in ops.iter().enumerate() {
            if matches!(
                op,
                ast::BinOp::Lt | ast::BinOp::Gt | ast::BinOp::LtEq | ast::BinOp::GtEq
            ) {
                if let Some(l) = operands.get(i) {
                    mark(l, out);
                }
                if let Some(r) = operands.get(i + 1) {
                    mark(r, out);
                }
            }
        }
    }
    // Unary +/-/~ operand is numeric.
    if let UnaryOp { op, operand } = expr {
        if matches!(
            op,
            ast::UnaryOp::Pos | ast::UnaryOp::Neg | ast::UnaryOp::BitNot
        ) {
            mark(operand, out);
        }
    }
    // Numeric-builtin call arguments are numeric.
    if let Call { func, args } = expr {
        if is_numeric_callee(&func.node) {
            for a in args {
                match a {
                    ast::CallArg::Positional(e) | ast::CallArg::StarArg(e) => mark(e, out),
                    ast::CallArg::Keyword { value, .. } => mark(value, out),
                    _ => {}
                }
            }
        }
    }
    // Recurse into all sub-expressions.
    let rec = |e: &Spanned<ast::Expr>, out: &mut std::collections::HashSet<String>| {
        expr_collect_numeric_used_params(&e.node, params, out);
    };
    match expr {
        BinOp { lhs, rhs, .. } => {
            rec(lhs, out);
            rec(rhs, out);
        }
        UnaryOp { operand, .. } => rec(operand, out),
        Call { func, args } => {
            rec(func, out);
            for a in args {
                match a {
                    ast::CallArg::Positional(e)
                    | ast::CallArg::StarArg(e)
                    | ast::CallArg::DoubleStarArg(e) => rec(e, out),
                    ast::CallArg::Keyword { value, .. } => rec(value, out),
                }
            }
        }
        Attr { object, .. } => rec(object, out),
        Index { object, index } => {
            rec(object, out);
            rec(index, out);
        }
        Slice { start, stop, step } => {
            if let Some(e) = start {
                rec(e, out);
            }
            if let Some(e) = stop {
                rec(e, out);
            }
            if let Some(e) = step {
                rec(e, out);
            }
        }
        ListLit(es) | SetLit(es) | TupleLit(es) | UnpackTarget(es) => {
            for e in es {
                rec(e, out);
            }
        }
        DictLit(pairs) => {
            for (k, v) in pairs {
                if let Some(k) = k {
                    rec(k, out);
                }
                rec(v, out);
            }
        }
        IfExpr {
            body,
            condition,
            else_body,
        } => {
            rec(body, out);
            rec(condition, out);
            rec(else_body, out);
        }
        ListComp {
            element,
            generators,
        }
        | SetComp {
            element,
            generators,
        }
        | GeneratorExpr {
            element,
            generators,
        } => {
            rec(element, out);
            for g in generators {
                rec(&g.iter, out);
                for c in &g.conditions {
                    rec(c, out);
                }
            }
        }
        DictComp {
            key,
            value,
            generators,
        } => {
            rec(key, out);
            rec(value, out);
            for g in generators {
                rec(&g.iter, out);
                for c in &g.conditions {
                    rec(c, out);
                }
            }
        }
        FString(parts) => {
            fn walk_parts(
                parts: &[ast::FStringPart],
                out: &mut std::collections::HashSet<String>,
                rec: &impl Fn(&Spanned<ast::Expr>, &mut std::collections::HashSet<String>),
            ) {
                for p in parts {
                    if let ast::FStringPart::Expr(e, spec) = p {
                        rec(e, out);
                        if let Some(sp) = spec {
                            walk_parts(sp, out, rec);
                        }
                    }
                }
            }
            walk_parts(parts, out, &rec);
        }
        Yield(opt) => {
            if let Some(e) = opt {
                rec(e, out);
            }
        }
        YieldFrom(e) | Await(e) | Starred(e) => rec(e, out),
        Walrus { value, .. } => rec(value, out),
        ChainedCompare { operands, .. } => {
            for e in operands {
                rec(e, out);
            }
        }
        Lambda { .. } => {}
        _ => {}
    }
}

/// Collect all binding names introduced by an AST pattern (used for OR-pattern merge).
fn collect_ast_pattern_bindings(pat: &Spanned<ast::Pattern>) -> std::collections::BTreeSet<String> {
    let mut names = std::collections::BTreeSet::new();
    collect_ast_bindings_inner(&pat.node, &mut names);
    names
}

fn collect_ast_bindings_inner(pat: &ast::Pattern, names: &mut std::collections::BTreeSet<String>) {
    match pat {
        ast::Pattern::Binding(name) => {
            names.insert(name.clone());
        }
        ast::Pattern::Or(alts) => {
            for alt in alts {
                collect_ast_bindings_inner(&alt.node, names);
            }
        }
        ast::Pattern::Sequence(pats) => {
            for p in pats {
                collect_ast_bindings_inner(&p.node, names);
            }
        }
        ast::Pattern::As { pattern, name } => {
            collect_ast_bindings_inner(&pattern.node, names);
            names.insert(name.clone());
        }
        ast::Pattern::ClassPattern { patterns, .. } => {
            for (_, p) in patterns {
                collect_ast_bindings_inner(&p.node, names);
            }
        }
        ast::Pattern::Mapping { pairs, rest } => {
            for (_, p) in pairs {
                collect_ast_bindings_inner(&p.node, names);
            }
            if let Some(r) = rest {
                names.insert(r.clone());
            }
        }
        ast::Pattern::Star(Some(name)) => {
            names.insert(name.clone());
        }
        ast::Pattern::Constructor { fields, .. } => {
            for f in fields {
                names.insert(f.clone());
            }
        }
        _ => {}
    }
}

/// Lower a parsed module to HIR.
/// Render an annotation `TypeExpr` back to a compact textual form for the
/// module `__annotations__` dict (e.g. "int", "list[int]", "int | str"). Only
/// the key presence is load-bearing for current fixtures; the repr is a stable
/// human-readable value.
/// Build introspection signature metadata (the runtime FUNC_PARAMS payload)
/// from the AST parameter list of a `def`. Kinds follow CPython's
/// `inspect.Parameter` ordinals; defaults are captured only when they are
/// simple literals (everything else records "has a default" with a None
/// placeholder). `self` receivers are skipped — CPython bound-method
/// signatures exclude them.
fn func_sig_meta(
    params: &[ast::Param],
    return_ty: &Option<Spanned<ast::TypeExpr>>,
) -> crate::hir::HirFuncSig {
    use crate::hir::{HirFuncSig, HirParamSig, HirSigDefault};
    let mut out = Vec::new();
    for p in params {
        if p.name == "self" {
            continue;
        }
        let kind = match p.kind {
            ast::ParamKind::Star => 2u8,
            ast::ParamKind::DoubleStar => 4u8,
            ast::ParamKind::Regular if p.kw_only => 3u8,
            ast::ParamKind::Regular if p.pos_only => 0u8,
            ast::ParamKind::Regular => 1u8,
        };
        let (default, default_opaque) = match &p.default {
            Option::None => (Option::None, false),
            Some(expr) => match &expr.node {
                ast::Expr::IntLit(v) => (Some(HirSigDefault::Int(*v)), false),
                ast::Expr::FloatLit(v) => (Some(HirSigDefault::Float(*v)), false),
                ast::Expr::StrLit(s) => (Some(HirSigDefault::Str(s.clone())), false),
                ast::Expr::BoolLit(b) => (Some(HirSigDefault::Bool(*b)), false),
                ast::Expr::NoneLit => (Some(HirSigDefault::None), false),
                ast::Expr::UnaryOp {
                    op: ast::UnaryOp::Neg,
                    operand,
                } => match &operand.node {
                    ast::Expr::IntLit(v) => (Some(HirSigDefault::Int(v.wrapping_neg())), false),
                    ast::Expr::FloatLit(v) => (Some(HirSigDefault::Float(-*v)), false),
                    _ => (Option::None, true),
                },
                _ => (Option::None, true),
            },
        };
        out.push(HirParamSig {
            name: p.name.clone(),
            kind,
            default,
            default_opaque,
            annotation: annotation_repr_opt(&p.ty.node),
        });
    }
    HirFuncSig {
        params: out,
        return_annotation: return_ty
            .as_ref()
            .and_then(|t| annotation_repr_opt(&t.node)),
    }
}

/// Textual annotation for introspection, or None for the parser's implicit
/// `Any` filler (an un-annotated param/return parses as `Named("Any")`).
fn annotation_repr_opt(ty: &ast::TypeExpr) -> Option<String> {
    let repr = type_expr_repr(ty);
    if repr == "Any" {
        None
    } else {
        Some(repr)
    }
}

fn type_expr_repr(ty: &ast::TypeExpr) -> String {
    match ty {
        ast::TypeExpr::Named(n) => ast::strip_forward_ref_name(n).unwrap_or(n).to_string(),
        ast::TypeExpr::Generic { name, args } => {
            let inner: Vec<String> = args.iter().map(|a| type_expr_repr(&a.node)).collect();
            format!("{}[{}]", name, inner.join(", "))
        }
        ast::TypeExpr::Optional(inner) => {
            format!("{} | None", type_expr_repr(&inner.node))
        }
        ast::TypeExpr::Union(parts) => {
            let inner: Vec<String> = parts.iter().map(|p| type_expr_repr(&p.node)).collect();
            inner.join(" | ")
        }
        ast::TypeExpr::Fn { params, ret } => {
            let p: Vec<String> = params.iter().map(|x| type_expr_repr(&x.node)).collect();
            format!("({}) -> {}", p.join(", "), type_expr_repr(&ret.node))
        }
        ast::TypeExpr::Tuple(parts) => {
            let inner: Vec<String> = parts.iter().map(|p| type_expr_repr(&p.node)).collect();
            format!("tuple[{}]", inner.join(", "))
        }
    }
}

/// Leaf name of a (possibly dotted) annotation type name: `typing.ClassVar`
/// → `ClassVar`, `ClassVar` → `ClassVar`.
fn type_name_leaf(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// Does the annotation name this PEP 557 marker (`ClassVar`, `InitVar`,
/// `KW_ONLY`), bare or generic (`ClassVar[str]`), plain or dotted
/// (`typing.ClassVar[int]`, `dataclasses.KW_ONLY`)?
fn type_expr_is_marker(ty: &ast::TypeExpr, marker: &str) -> bool {
    match ty {
        ast::TypeExpr::Named(n) => type_name_leaf(n) == marker,
        ast::TypeExpr::Generic { name, .. } => type_name_leaf(name) == marker,
        _ => false,
    }
}

/// Does this decorator expression denote `@dataclass` (PEP 557)? Matches the
/// bare name (`@dataclass`), attribute form (`@dataclasses.dataclass`), and
/// the called forms of both (`@dataclass(frozen=True)`).
fn decorator_is_dataclass(expr: &ast::Expr) -> bool {
    match expr {
        ast::Expr::Ident(n) => n == "dataclass",
        ast::Expr::Attr { attr, .. } => attr == "dataclass",
        ast::Expr::Call { func, .. } => decorator_is_dataclass(&func.node),
        _ => false,
    }
}

fn decorator_is_typing_overload(expr: &ast::Expr) -> bool {
    match expr {
        ast::Expr::Ident(n) => n == "overload",
        ast::Expr::Attr { attr, .. } => attr == "overload",
        ast::Expr::Call { func, .. } => decorator_is_typing_overload(&func.node),
        _ => false,
    }
}

fn erase_param_annotations(params: &[ast::Param]) -> Vec<ast::Param> {
    params
        .iter()
        .cloned()
        .map(|mut p| {
            p.ty = Spanned::new(ast::TypeExpr::Named("Any".to_string()), p.ty.span);
            p
        })
        .collect()
}

fn decorator_preserves_call_signature(expr: &ast::Expr) -> bool {
    match expr {
        ast::Expr::Ident(n) => matches!(n.as_str(), "contextmanager" | "asynccontextmanager"),
        ast::Expr::Attr { attr, .. } => matches!(
            attr.as_str(),
            "contextmanager" | "asynccontextmanager"
        ),
        ast::Expr::Call { func, .. } => decorator_preserves_call_signature(&func.node),
        _ => false,
    }
}

/// Is this class-body default value a `field(...)` / `dataclasses.field(...)`
/// call carrying `init=False`? Such fields are excluded from the synthesized
/// `__init__` parameter list (PEP 557).
fn field_call_has_init_false(expr: &ast::Expr) -> bool {
    let ast::Expr::Call { func, args } = expr else {
        return false;
    };
    let is_field = match &func.node {
        ast::Expr::Ident(n) => n == "field",
        ast::Expr::Attr { attr, .. } => attr == "field",
        _ => false,
    };
    if !is_field {
        return false;
    }
    args.iter().any(|a| {
        matches!(a, ast::CallArg::Keyword { name, value }
            if name == "init" && matches!(value.node, ast::Expr::BoolLit(false)))
    })
}

pub fn lower_module(
    module: &ast::Module,
    checker: &TypeChecker,
) -> Result<HirModule, Vec<MambaError>> {
    let mut lowerer = AstLowerer::new(checker);
    lowerer.lower(module);
    if lowerer.errors.is_empty() {
        // Merge local_names into result.sym_names instead of overwriting.
        // result.sym_names may already contain method names stored during lower_class
        // (via direct result.sym_names.insert); those must survive scope clears (#827).
        for (name, &id) in &lowerer.local_names {
            lowerer
                .result
                .sym_names
                .entry(id)
                .or_insert_with(|| name.clone());
        }
        // Flush remaining scope types (enter_local_scope already flushed earlier scopes).
        for (&k, &v) in &lowerer.local_types {
            lowerer.result.sym_types.entry(k).or_insert(v);
        }
        Ok(lowerer.result)
    } else {
        Err(lowerer.errors)
    }
}

/// REPL accumulated symbol info: SymbolId → (name, type).
pub type ReplSymInfo =
    std::collections::HashMap<crate::resolve::SymbolId, (String, crate::types::TypeId)>;

/// REPL-aware AST→HIR lowering: pre-seeds known global names and types from
/// previous iterations so that references to persisted variables resolve correctly.
pub fn lower_module_repl(
    module: &ast::Module,
    checker: &TypeChecker,
    prev_syms: &ReplSymInfo,
) -> Result<HirModule, Vec<MambaError>> {
    let mut lowerer = AstLowerer::new(checker);
    // Pre-seed local_names and local_types with accumulated info
    for (&sym_id, (name, ty)) in prev_syms {
        lowerer.local_names.entry(name.clone()).or_insert(sym_id);
        lowerer.local_types.entry(sym_id).or_insert(*ty);
    }
    // Advance next_local_sym past any pre-seeded IDs to avoid collisions
    let max_pre = prev_syms.keys().map(|s| s.0).max().unwrap_or(999_999);
    if lowerer.next_local_sym <= max_pre {
        lowerer.next_local_sym = max_pre + 1;
    }
    lowerer.lower(module);
    if lowerer.errors.is_empty() {
        // Merge local_names into result.sym_names instead of overwriting.
        // result.sym_names may already contain method names stored during lower_class
        // (via direct result.sym_names.insert); those must survive scope clears (#827).
        for (name, &id) in &lowerer.local_names {
            lowerer
                .result
                .sym_names
                .entry(id)
                .or_insert_with(|| name.clone());
        }
        // Flush remaining scope types (enter_local_scope already flushed earlier scopes).
        for (&k, &v) in &lowerer.local_types {
            lowerer.result.sym_types.entry(k).or_insert(v);
        }
        Ok(lowerer.result)
    } else {
        Err(lowerer.errors)
    }
}

struct AstLowerer<'a> {
    checker: &'a TypeChecker,
    result: HirModule,
    errors: Vec<MambaError>,
    /// Function-local name→SymbolId (checker scopes are popped after checking)
    local_names: HashMap<String, SymbolId>,
    /// Type info for locally-defined symbols
    local_types: HashMap<SymbolId, TypeId>,
    /// Counter for locally-allocated SymbolIds (offset to avoid collisions)
    next_local_sym: u32,
    /// Subject type of the enclosing `match` statement for capture binding lowering (#827).
    current_match_subject_ty: Option<TypeId>,
    /// Snapshot of outer function's local names for nonlocal resolution in nested functions.
    outer_scope_names: HashMap<String, SymbolId>,
    /// Synthetic SymbolIds that are shared via nonlocal (Cell variables in outer, Free in inner).
    /// These must be stored/loaded via global storage so both functions share the same slot.
    cell_override_syms: std::collections::HashSet<SymbolId>,
    /// Function parameter info for kwargs resolution at call sites.
    /// Maps function name → vec of (param_name, default_expr_option).
    func_param_info: HashMap<String, Vec<(String, Option<Spanned<ast::Expr>>, ast::ParamKind)>>,
    /// Static arg-binding validation: top-level function name → param shape
    /// `(name, has_default, kw_only, is_star, is_double_star, pos_only)`,
    /// captured at the def so the call-site validator can raise a
    /// CPython-faithful TypeError for too-many-positional / duplicate-argument /
    /// missing-required-keyword-only / unexpected-keyword / positional-only-as-
    /// keyword calls. Only top-level defs are recorded and only bare-Ident,
    /// splat-free calls are checked, so a violation is unambiguous before we
    /// raise.
    arg_bind_sigs: HashMap<String, Vec<(String, bool, bool, bool, bool, bool)>>,
    /// Function names whose `__defaults__` / `__kwdefaults__` is assigned at
    /// runtime. Static arg-count validation (missing/too-many positional) is
    /// skipped for these — the mutation changes the effective defaults in a way
    /// the source signature can't see (`def f(x): f.__defaults__=(None,); f()`).
    funcs_with_mutated_defaults: std::collections::HashSet<String>,
    /// PEP 557: per-dataclass synthesized __init__ parameter shapes, kept
    /// separately from `func_param_info` so subclasses can prepend their base
    /// dataclass's params (Derived(Counted) accepts Counted's fields first).
    dataclass_init_params:
        HashMap<String, Vec<(String, Option<Spanned<ast::Expr>>, ast::ParamKind)>>,
    /// PEP 557: local names bound to `dataclasses.dataclass` / `field` /
    /// `replace` by a `from dataclasses import ...` statement. Bare-Ident
    /// calls to these names pack keyword args into a trailing dict (the
    /// native-dispatcher kwargs convention) instead of flattening them to
    /// positionals — `dataclass(frozen=True)` / `field(default_factory=list)`
    /// / `replace(obj, a=99)` all need their keyword names at runtime.
    dataclasses_kwarg_idents: std::collections::HashSet<String>,
    /// Local module aliases that refer to `functools` (`functools`, or
    /// `import functools as ft`). Used to recognize `ft.partial(...)`.
    functools_module_idents: std::collections::HashSet<String>,
    /// Local names that refer to the `functools.partial` factory, for example
    /// `from functools import partial as p`.
    functools_partial_factory_idents: std::collections::HashSet<String>,
    /// Local names bound to a `functools.partial(...)` instance. Calls through
    /// these names must keep explicit keyword arguments structural so call-time
    /// kwargs can override the partial's stored kwargs.
    functools_partial_kwarg_idents: std::collections::HashSet<String>,
    /// Class names whose MRO passes through `types.SimpleNamespace`. Their
    /// inherited native initializer needs keyword names preserved as a trailing
    /// kwargs dict; flattening keywords to values builds an empty namespace.
    simple_namespace_subclass_idents: std::collections::HashSet<String>,
    /// Function-name SymbolId → declared return type. Populated *before* a
    /// function's body is lowered so recursive calls can read the callee's
    /// return type (without this, the call falls through to `any_ty`,
    /// hir_to_mir's `(Ty::Int, Ty::Int)` BinOp gate fails, and CheckedMul
    /// is never emitted on the recursive integer call — see fire 49).
    func_return_tys: HashMap<SymbolId, TypeId>,
    /// Names that Python scoping rules treat as local to the enclosing function
    /// (any name assigned in the body, minus global/nonlocal declarations).
    /// Populated at function-body entry and consumed by the Assign arm so
    /// `x = val` defines a local even if an outer-scope `x` exists.
    local_assigned_names: Vec<String>,
    /// Names declared `global` / `nonlocal` in the enclosing function — kept
    /// separate so Assign arms can skip the local-definition promotion.
    local_declared_names: Vec<String>,
    /// Per-function call-site argument float hints: function name → vec of
    /// per-positional-param FloatHint, merged over every call seen in the module.
    /// Used to soundly monomorphize an unannotated param as `float` when it is
    /// only ever called with float arguments (e.g. `add(a, b)` called with
    /// `add(1.25, 2.5)` → both params float). Populated by a pre-pass before any
    /// function body is lowered.
    func_param_float_hint: HashMap<String, Vec<FloatHint>>,
    /// User function name → return-value FloatHint (Float when the function is
    /// inferred to return a float). Lets `return helper(x)` propagate float-ness
    /// of a callee through to the caller's own return-type inference.
    func_ret_float_hint: HashMap<String, FloatHint>,
    /// Module-level global name → FloatHint, so a function reading a global float
    /// (e.g. `scale = 0.25; def ff(j): return j * scale`) infers a float return.
    /// Seeded into each function's return-inference env below the params.
    module_float_globals: HashMap<String, FloatHint>,
    /// Module-scope names that have only a bare annotation so far (`x: int`)
    /// and no runtime binding. CPython records them in `__annotations__`, but
    /// reading them before a later assignment raises NameError.
    module_unbound_annotation_names: std::collections::HashSet<String>,
    /// True while lowering a function body. Module annotation-only fast
    /// NameError lowering must not be baked into function bodies because a
    /// later module assignment may bind the global before the function runs.
    in_function_body: bool,
    /// PEP 695 type-parameter names of the function currently being lowered
    /// (`def f[T](x: T) -> T`). A `T`-annotated param/return is a boxed
    /// MbValue at runtime (the TypeVar erases to `any`), so the int-default
    /// fallback for unresolved annotations must not fire for these names.
    active_type_params: std::collections::HashSet<String>,
}

impl<'a> AstLowerer<'a> {
    fn new(checker: &'a TypeChecker) -> Self {
        Self {
            checker,
            result: HirModule {
                functions: Vec::new(),
                classes: Vec::new(),
                top_level: Vec::new(),
                imports: Vec::new(),
                sym_names: std::collections::HashMap::new(),
                sym_types: std::collections::HashMap::new(),
                module_annotations: Vec::new(),
                func_sigs: std::collections::HashMap::new(),
            },
            errors: Vec::new(),
            local_assigned_names: Vec::new(),
            local_declared_names: Vec::new(),
            active_type_params: std::collections::HashSet::new(),
            local_names: HashMap::new(),
            local_types: HashMap::new(),
            next_local_sym: 1_000_000,
            current_match_subject_ty: None,
            outer_scope_names: HashMap::new(),
            cell_override_syms: std::collections::HashSet::new(),
            func_param_info: HashMap::new(),
            arg_bind_sigs: HashMap::new(),
            funcs_with_mutated_defaults: std::collections::HashSet::new(),
            dataclass_init_params: HashMap::new(),
            dataclasses_kwarg_idents: std::collections::HashSet::new(),
            functools_module_idents: std::iter::once("functools".to_string()).collect(),
            functools_partial_factory_idents: std::collections::HashSet::new(),
            functools_partial_kwarg_idents: std::collections::HashSet::new(),
            simple_namespace_subclass_idents: std::collections::HashSet::new(),
            func_return_tys: HashMap::new(),
            func_param_float_hint: HashMap::new(),
            func_ret_float_hint: HashMap::new(),
            module_float_globals: HashMap::new(),
            module_unbound_annotation_names: std::collections::HashSet::new(),
            in_function_body: false,
        }
    }

    /// Reset local scope (call before lowering each function body).
    fn enter_local_scope(&mut self) {
        // Flush current scope's type bindings into the module-level sym_types map before
        // clearing. This ensures captures in any function (not just the last one) retain
        // their primitive TypeId so MIR lowering can emit correct mb_unbox_* calls (#827).
        for (&k, &v) in &self.local_types {
            self.result.sym_types.entry(k).or_insert(v);
        }
        self.local_names.clear();
        self.local_types.clear();
    }

    /// Define a local name, returning its SymbolId.
    fn define_local(&mut self, name: &str, ty: TypeId) -> SymbolId {
        if let Some(&existing) = self.local_names.get(name) {
            self.local_types.insert(existing, ty);
            return existing;
        }
        let id = SymbolId(self.next_local_sym);
        self.next_local_sym += 1;
        self.local_names.insert(name.to_string(), id);
        self.local_types.insert(id, ty);
        id
    }

    /// Get the type of a symbol (local first, then checker).
    fn get_type(&self, sym: SymbolId) -> TypeId {
        if let Some(&ty) = self.local_types.get(&sym) {
            return ty;
        }
        self.checker.get_sym_type(sym.0)
    }

    /// Infer for-loop element type from the iterable expression.
    /// range() → int, otherwise → any.
    fn infer_for_elem_ty(&self, iter: &Spanned<ast::Expr>) -> TypeId {
        if let ast::Expr::Call { func, .. } = &iter.node {
            if let ast::Expr::Ident(name) = &func.node {
                if name == "range" {
                    return self.checker.tcx.int();
                }
            }
        }
        self.checker.tcx.any()
    }

    /// Resolve a type expression to a TypeId using only immutable access.
    /// Handles common annotation types (int, str, float, bool, None, Any, list[T], dict[K,V],
    /// class names). Unknown or complex types fall back to any().
    fn resolve_type_expr_ro(&self, ty: &Spanned<ast::TypeExpr>) -> TypeId {
        use ast::TypeExpr;
        match &ty.node {
            TypeExpr::Named(name) => match name.as_str() {
                "int" => self.checker.tcx.int(),
                "float" => self.checker.tcx.float(),
                "bool" => self.checker.tcx.bool(),
                "str" => self.checker.tcx.str(),
                "None" => self.checker.tcx.none(),
                "Any" => self.checker.tcx.any(),
                n if ast::strip_forward_ref_name(n).is_some() => self.checker.tcx.any(),
                _ => {
                    // Try type alias, then class symbol in checker
                    if let Some(id) = self.checker.tcx.resolve_alias(name) {
                        return id;
                    }
                    if let Some(sym) = self.checker.symbols.lookup(name) {
                        let t = self.checker.get_sym_type(sym.0);
                        if t != self.checker.tcx.error() {
                            return t;
                        }
                    }
                    self.checker.tcx.any()
                }
            },
            TypeExpr::Generic { name, args } => match name.as_str() {
                "list" | "List" if args.len() == 1 => {
                    let elem = self.resolve_type_expr_ro(&args[0]);
                    self.checker
                        .tcx
                        .find(&crate::types::Ty::List(elem))
                        .unwrap_or_else(|| self.checker.tcx.any())
                }
                "dict" | "Dict" if args.len() == 2 => {
                    let k = self.resolve_type_expr_ro(&args[0]);
                    let v = self.resolve_type_expr_ro(&args[1]);
                    self.checker
                        .tcx
                        .find(&crate::types::Ty::Dict(k, v))
                        .unwrap_or_else(|| self.checker.tcx.any())
                }
                _ => self.checker.tcx.any(),
            },
            TypeExpr::Optional(inner) => {
                // Optional[T] — for now, use any() to avoid Union interning
                let _ = inner;
                self.checker.tcx.any()
            }
            _ => self.checker.tcx.any(),
        }
    }

    /// Pre-pass for float-return-inference: build `func_param_float_hint`
    /// (per-position argument float-ness over every call) and `func_ret_float_hint`
    /// (each function's inferred return float-ness, via a small fixpoint).
    fn collect_float_hints(&mut self, module: &ast::Module) {
        // Names of functions that take any explicit param annotation — those keep
        // their annotated convention; do not override them from call sites.
        let mut unannotated_params: HashMap<String, Vec<bool>> = HashMap::new();
        let mut fn_bodies: Vec<(String, &[Spanned<ast::Stmt>])> = Vec::new();
        for stmt in &module.stmts {
            if let ast::Stmt::FnDef {
                name,
                params,
                return_ty,
                body,
                decorators,
                ..
            }
            | ast::Stmt::AsyncFnDef {
                name,
                params,
                return_ty,
                body,
                decorators,
                ..
            } = &stmt.node
            {
                // Only consider plain, undecorated functions with no return
                // annotation and only regular positional params — these are the
                // ones whose convention we may refine from call sites.
                let simple = decorators.is_empty()
                    && return_ty.is_none()
                    && params.iter().all(|p| p.kind == ast::ParamKind::Regular);
                if simple {
                    let unann: Vec<bool> = params
                        .iter()
                        .map(|p| matches!(&p.ty.node, ast::TypeExpr::Named(n) if n == "Any"))
                        .collect();
                    unannotated_params.insert(name.clone(), unann);
                    fn_bodies.push((name.clone(), body.as_slice()));
                }
            }
        }

        // Module-level float globals (e.g. `scale = 0.25`) so a function reading a
        // global float infers a float return. Classified with the globals seen so
        // far (sequential) and no func-return map; only definite hints are kept.
        let no_ret0: HashMap<String, FloatHint> = HashMap::new();
        // Record a float-element container global under the sentinel key so
        // `name[i]` infers float (see `ast_expr_float_hint`'s Index arm).
        let mut note_container =
            |globals: &mut HashMap<String, FloatHint>, name: &str, value: &ast::Expr| {
                if let ast::Expr::ListLit(items)
                | ast::Expr::TupleLit(items)
                | ast::Expr::SetLit(items) = value
                {
                    if ast_container_is_all_float(items, globals, &no_ret0) {
                        globals.insert(float_container_key(name), FloatHint::Float);
                    }
                }
            };
        for stmt in &module.stmts {
            match &stmt.node {
                ast::Stmt::Assign { target, value } => {
                    if let ast::Expr::Ident(name) = &target.node {
                        let h =
                            ast_expr_float_hint(&value.node, &self.module_float_globals, &no_ret0);
                        if h != FloatHint::Unknown {
                            self.module_float_globals.insert(name.clone(), h);
                        }
                        note_container(&mut self.module_float_globals, name, &value.node);
                    }
                }
                ast::Stmt::VarDecl { name, value, .. } => {
                    let h = ast_expr_float_hint(&value.node, &self.module_float_globals, &no_ret0);
                    if h != FloatHint::Unknown {
                        self.module_float_globals.insert(name.clone(), h);
                    }
                    note_container(&mut self.module_float_globals, name, &value.node);
                }
                _ => {}
            }
        }

        // Collect per-position argument float hints over all calls in the
        // module. Seed with `module_float_globals` (scalar float globals +
        // float-container sentinels) so `echo(scale)` and `echo(xs[0])` — where
        // `scale`/`xs` are module-level floats/float-lists — hint the param as
        // float and monomorphize the callee accordingly.
        let no_ret: HashMap<String, FloatHint> = HashMap::new();
        let mut call_hints: HashMap<String, Vec<FloatHint>> = HashMap::new();
        let mut have_call: std::collections::HashSet<String> = std::collections::HashSet::new();
        collect_call_arg_hints(
            &module.stmts,
            &self.module_float_globals,
            &no_ret,
            &mut call_hints,
            &mut have_call,
        );

        for (fname, hints) in call_hints {
            // Only keep param hints for functions whose params are unannotated;
            // intersect positions with the unannotated mask so annotated params
            // are never overridden.
            if let Some(mask) = unannotated_params.get(&fname) {
                let merged: Vec<FloatHint> = hints
                    .iter()
                    .enumerate()
                    .map(|(i, &h)| {
                        if mask.get(i).copied().unwrap_or(false) {
                            h
                        } else {
                            FloatHint::Unknown
                        }
                    })
                    .collect();
                self.func_param_float_hint.insert(fname.clone(), merged);
            }
        }
        // Functions never called in this module: no param hint (stay int).
        let _ = have_call;

        // Fixpoint over function return float-ness. Seed all unknown, then
        // repeatedly recompute each function's return hint using current param
        // hints + other functions' return hints, until stable.
        for _ in 0..(fn_bodies.len() + 1) {
            let mut changed = false;
            for (fname, body) in &fn_bodies {
                // Build env from param hints.
                let mut env: HashMap<String, FloatHint> = HashMap::new();
                // Seed module-level float globals first; params/locals shadow them.
                env.extend(
                    self.module_float_globals
                        .iter()
                        .map(|(k, &v)| (k.clone(), v)),
                );
                if let (Some(param_names), Some(hints)) = (
                    module.stmts.iter().find_map(|s| match &s.node {
                        ast::Stmt::FnDef { name, params, .. }
                        | ast::Stmt::AsyncFnDef { name, params, .. }
                            if name == fname =>
                        {
                            Some(params.iter().map(|p| p.name.clone()).collect::<Vec<_>>())
                        }
                        _ => None,
                    }),
                    Some(self.func_param_float_hint.get(fname)),
                ) {
                    if let Some(hints) = hints {
                        for (pn, &h) in param_names.iter().zip(hints.iter()) {
                            env.insert(pn.clone(), h);
                        }
                    }
                }
                collect_local_float_env(body, &mut env, &self.func_ret_float_hint);
                let ret = infer_return_float_hint(body, &env, &self.func_ret_float_hint);
                let prev = self
                    .func_ret_float_hint
                    .get(fname)
                    .copied()
                    .unwrap_or(FloatHint::Unknown);
                if prev != ret {
                    self.func_ret_float_hint.insert(fname.clone(), ret);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    fn lower(&mut self, module: &ast::Module) {
        // Pre-pass: collect per-function call-site argument float hints and a
        // fixpoint of each function's return float-ness, so unannotated params
        // that only ever receive floats are monomorphized as `float` and float
        // returns are typed correctly (the float-return-inference soundness wall).
        self.collect_float_hints(module);
        collect_mutated_defaults(&module.stmts, &mut self.funcs_with_mutated_defaults);
        for stmt in &module.stmts {
            match &stmt.node {
                ast::Stmt::FnDef {
                    name,
                    type_params,
                    params,
                    return_ty,
                    body,
                    decorators,
                    ..
                } => {
                    // Register param info for kwargs resolution at call sites.
                    self.func_param_info.insert(
                        name.clone(),
                        params
                            .iter()
                            .map(|p| (p.name.clone(), p.default.clone(), p.kind))
                            .collect(),
                    );
                    // Param shape for static call-site arg-binding validation.
                    // Most decorators can replace the callable with an arbitrary
                    // wrapper whose signature differs. A small allowlist preserves
                    // the original call signature, so static CPython-style
                    // argument binding can still reject malformed calls.
                    let preserves_declared_signature = decorators.is_empty()
                        || decorators
                            .iter()
                            .all(|d| decorator_preserves_call_signature(&d.node));
                    if preserves_declared_signature {
                        self.arg_bind_sigs.insert(
                            name.clone(),
                            params
                                .iter()
                                .map(|p| {
                                    (
                                        p.name.clone(),
                                        p.default.is_some(),
                                        p.kw_only,
                                        p.kind == ast::ParamKind::Star,
                                        p.kind == ast::ParamKind::DoubleStar,
                                        p.pos_only,
                                    )
                                })
                                .collect(),
                        );
                    } else {
                        // A redefinition that is decorated must not keep a stale
                        // bare-signature entry from an earlier plain def.
                        self.arg_bind_sigs.remove(name);
                    }
                    let is_decorated = !decorators.is_empty();
                    let overload_decorated = decorators
                        .iter()
                        .any(|d| decorator_is_typing_overload(&d.node));
                    let erased_params;
                    let params_for_lower: &[ast::Param] = if overload_decorated {
                        erased_params = erase_param_annotations(params);
                        &erased_params
                    } else {
                        params
                    };
                    let erased_return: Option<Spanned<ast::TypeExpr>> = None;
                    let return_for_lower = if overload_decorated {
                        &erased_return
                    } else {
                        return_ty
                    };
                    // PEP 695: make this def's type-param names visible to the
                    // param/return type lowering (TypeVar-annotated values are
                    // boxed `any`, never raw ints).
                    let saved_tps = std::mem::replace(
                        &mut self.active_type_params,
                        type_params.iter().map(|p| p.name.clone()).collect(),
                    );
                    let lowered = if is_decorated {
                        self.lower_decorated_fn(name, params_for_lower, return_for_lower, body, stmt.span)
                    } else {
                        self.lower_fn(name, params, return_ty, body, stmt.span)
                    };
                    self.active_type_params = saved_tps;
                    if let Some(mut func) = lowered {
                        // Introspection: record the declared signature shape so
                        // module init can prime the runtime FUNC_PARAMS registry
                        // (inspect.signature / getfullargspec).
                        self.result
                            .func_sigs
                            .insert(func.name.0, func_sig_meta(params, return_ty));
                        func.is_generator = contains_yield(body);
                        func.decorators = decorators
                            .iter()
                            .filter_map(|d| self.lower_expr(d))
                            .collect();
                        if !func.decorators.is_empty() {
                            // Params were already lowered with any_ty (body expressions
                            // route through NaN-aware runtime dispatch). Force the return
                            // type to any_ty as well so call sites treat the dispatch
                            // result uniformly.
                            let any_ty = self.checker.tcx.any();
                            func.return_ty = any_ty;
                            // Update func_return_tys so call-site lookup at
                            // ast_to_hir.rs:2053 reads `any` instead of the body-inferred
                            // primitive type — otherwise `add_one(5) == 6` (decorated)
                            // routes through native int compare on NaN-boxed bits and
                            // returns False even though both sides are 6.
                            self.func_return_tys.insert(func.name, any_ty);
                            // Emit a placeholder so decorator application happens at the
                            // correct position in the module execution order (#decorder).
                            self.result.top_level.push(HirStmt::FuncDefPlaceholder {
                                name: func.name,
                                span: stmt.span,
                            });
                        }
                        self.result.functions.push(func);
                    }
                    self.module_unbound_annotation_names.remove(name);
                }
                ast::Stmt::AsyncFnDef {
                    name,
                    type_params,
                    params,
                    return_ty,
                    body,
                    decorators,
                    ..
                } => {
                    let is_decorated = !decorators.is_empty();
                    let overload_decorated = decorators
                        .iter()
                        .any(|d| decorator_is_typing_overload(&d.node));
                    let erased_params;
                    let params_for_lower: &[ast::Param] = if overload_decorated {
                        erased_params = erase_param_annotations(params);
                        &erased_params
                    } else {
                        params
                    };
                    let erased_return: Option<Spanned<ast::TypeExpr>> = None;
                    let return_for_lower = if overload_decorated {
                        &erased_return
                    } else {
                        return_ty
                    };
                    // PEP 695: make this def's type-param names visible to the
                    // param/return type lowering (TypeVar-annotated values are
                    // boxed `any`, never raw ints).
                    let saved_tps = std::mem::replace(
                        &mut self.active_type_params,
                        type_params.iter().map(|p| p.name.clone()).collect(),
                    );
                    let lowered = if is_decorated {
                        self.lower_decorated_fn(name, params_for_lower, return_for_lower, body, stmt.span)
                    } else {
                        self.lower_fn(name, params, return_ty, body, stmt.span)
                    };
                    self.active_type_params = saved_tps;
                    if let Some(mut func) = lowered {
                        // Introspection: same FUNC_PARAMS priming as sync defs.
                        self.result
                            .func_sigs
                            .insert(func.name.0, func_sig_meta(params, return_ty));
                        let has_yield = contains_yield(body);
                        // `async def f(): yield` is an async generator — CPython
                        // returns an async-generator object, not a coroutine.
                        // mamba's runtime does not yet model async-generator vs
                        // coroutine separately, so route the body through the
                        // sync generator lowering when a yield is present. This
                        // gives us a working `next()` / `for x in agen()` and a
                        // suspended call site (the wrapper returns the generator
                        // handle without running the body), at the cost of true
                        // async semantics inside the generator. AsyncFor is also
                        // lowered as For, so iteration consumes the generator
                        // synchronously. Real `await` between yields is tracked
                        // under epic-py3-12 #850.
                        if has_yield {
                            func.is_generator = true;
                            // Leave is_async=false so lower_function picks the
                            // generator path. The async/await machinery of the
                            // body still expands as written.
                        } else {
                            func.is_async = true;
                        }
                        func.decorators = decorators
                            .iter()
                            .filter_map(|d| self.lower_expr(d))
                            .collect();
                        if !func.decorators.is_empty() {
                            let any_ty = self.checker.tcx.any();
                            func.return_ty = any_ty;
                            self.func_return_tys.insert(func.name, any_ty);
                            self.result.top_level.push(HirStmt::FuncDefPlaceholder {
                                name: func.name,
                                span: stmt.span,
                            });
                        }
                        self.result.functions.push(func);
                    }
                    self.module_unbound_annotation_names.remove(name);
                }
                ast::Stmt::ClassDef {
                    name,
                    body,
                    bases,
                    decorators,
                    keyword_args,
                    ..
                } => {
                    self.collect_class_stmt(
                        name,
                        body,
                        bases,
                        decorators,
                        keyword_args,
                        stmt.span,
                        true,
                    );
                    self.module_unbound_annotation_names.remove(name);
                }
                _ => {
                    // Module-scope variable annotations record their name in the
                    // module __annotations__ dict (CPython: PEP 526 semantics).
                    // `x: int` (BareAnnotation) and `x: int = v` (VarDecl) both
                    // qualify; the dict is auto-created at module init.
                    match &stmt.node {
                        ast::Stmt::BareAnnotation { name, ty } => {
                            self.result
                                .module_annotations
                                .push((name.clone(), type_expr_repr(&ty.node)));
                            self.module_unbound_annotation_names.insert(name.clone());
                        }
                        ast::Stmt::VarDecl { name, ty, .. } => {
                            self.result
                                .module_annotations
                                .push((name.clone(), type_expr_repr(&ty.node)));
                        }
                        _ => {}
                    }
                    if let Some(s) = self.lower_stmt(stmt) {
                        self.result.top_level.push(s);
                    }
                    match &stmt.node {
                        ast::Stmt::VarDecl { name, .. } => {
                            self.module_unbound_annotation_names.remove(name);
                        }
                        ast::Stmt::Assign { target, .. } => {
                            if let ast::Expr::Ident(name) = &target.node {
                                self.module_unbound_annotation_names.remove(name);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn lower_fn(
        &mut self,
        name: &str,
        params: &[ast::Param],
        _return_ty: &Option<Spanned<ast::TypeExpr>>,
        body: &[Spanned<ast::Stmt>],
        span: Span,
    ) -> Option<HirFunction> {
        self.lower_fn_inner(name, params, _return_ty, body, span, false, false)
    }

    /// Lower a function whose definition is decorated. Decorated functions
    /// receive NaN-boxed MbValues at the dynamic dispatch boundary
    /// (`mb_call1_val` / `mb_call_spread`), so body expressions must be
    /// lowered with `any_ty` for unannotated params — otherwise primitive
    /// Cranelift ops (`icmp`, `iadd`) would operate on NaN-boxed bits and
    /// produce garbage.
    fn lower_decorated_fn(
        &mut self,
        name: &str,
        params: &[ast::Param],
        _return_ty: &Option<Spanned<ast::TypeExpr>>,
        body: &[Spanned<ast::Stmt>],
        span: Span,
    ) -> Option<HirFunction> {
        self.lower_fn_inner(name, params, _return_ty, body, span, false, true)
    }

    fn lower_fn_inner(
        &mut self,
        name: &str,
        params: &[ast::Param],
        _return_ty: &Option<Spanned<ast::TypeExpr>>,
        body: &[Spanned<ast::Stmt>],
        span: Span,
        is_method: bool,
        is_decorated: bool,
    ) -> Option<HirFunction> {
        let name_id = self.resolve_name(name, span)?;
        // Save entire outer scope state so nested function lowering doesn't corrupt it.
        // enter_local_scope() clears local_names/local_types; after nested lowering we
        // must restore the outer function's local bindings so its remaining body stmts
        // can still resolve names correctly.
        let saved_local_names = self.local_names.clone();
        let saved_local_types = self.local_types.clone();
        let saved_outer_scope = self.outer_scope_names.clone();
        // Merge current outer_scope_names into local_names so that grandparent
        // (and deeper ancestor) variables are visible to multiply-nested inner
        // functions.  Without this, a variable from 2+ levels up is invisible
        // because outer_scope_names only contained the immediate parent's locals.
        let mut new_outer = self.outer_scope_names.clone();
        // Child locals shadow grandparent names via extend overwrite semantics.
        new_outer.extend(self.local_names.iter().map(|(k, v)| (k.clone(), *v)));
        self.outer_scope_names = new_outer;
        // Save cell_override_syms so each nesting level starts clean.
        let saved_cell_syms = self.cell_override_syms.clone();
        self.cell_override_syms = std::collections::HashSet::new();
        self.enter_local_scope();
        let saved_in_function_body = self.in_function_body;
        self.in_function_body = true;

        // Define params in local scope with their actual annotation types (#827 R5).
        // Method params use `any` because they receive NaN-boxed MbValues via mb_call_method.
        // Non-method params use the resolved annotation type so match patterns see the
        // correct subject type (e.g. Class{...} for `p: Point`, List(int) for `xs: list[int]`).
        // EXCEPTION: truly unannotated params (annotation is a bare `Any` name) keep `int` to
        // preserve the raw-i64 calling convention used by arithmetic and generators.
        // Complex annotations (Generic, Union, Tuple) that fail interning stay `any` so they
        // are treated as boxed NaN-values (needed for `xs: list[int]` sequence patterns) (#827).
        let any_ty = self.checker.tcx.any();
        let int_ty = self.checker.tcx.int();
        let float_ty = self.checker.tcx.float();
        // Scan the body to find unannotated params used as a direct operand of
        // `==`/`!=`/`in`/`not in`. Those must compare by VALUE, so they cannot
        // keep the raw-int (pointer-identity) convention — promote them to
        // `any` so the comparison routes through the NaN-aware runtime dispatch
        // (mb_eq / mb_list_contains). Genuine int / kwargs-passthrough params
        // are untouched, preserving the fast native ABI. (value_equality_inference)
        let value_compared_params: std::collections::HashSet<String> = if is_method {
            std::collections::HashSet::new()
        } else {
            let param_name_set: std::collections::HashSet<String> = params
                .iter()
                .filter(|p| p.kind == ast::ParamKind::Regular)
                .map(|p| p.name.clone())
                .collect();
            let mut found = std::collections::HashSet::new();
            if !param_name_set.is_empty() {
                collect_value_compared_params(body, &param_name_set, &mut found);
                // Exclude params also used numerically (arithmetic / ordering /
                // numeric builtins). A float/int param passed as NaN-boxed bits
                // must keep the raw-int ABI for the native numeric fast path;
                // promoting it to `any` breaks downstream abs/math/arithmetic.
                // Heap params in the check(result, expect) pattern are never
                // numeric, so they remain promoted.
                if !found.is_empty() {
                    let mut numeric = std::collections::HashSet::new();
                    collect_numeric_used_params(body, &param_name_set, &mut numeric);
                    found.retain(|n| !numeric.contains(n));
                }
            }
            found
        };
        // Generator params arrive through `mb_generator_store_arg` / `call_body_fn`,
        // which always carry a NaN-boxed MbValue (i64). The "raw-int convention"
        // (defaulting an unannotated param to `int`) is unsound for generators:
        // `frange(0.0, 2.0, 0.5)` passes floats into unannotated params, and the
        // int default makes the body unbox/re-box them as ints, leaking raw
        // IEEE-754 bits or looping forever. Treat unannotated generator params as
        // `any` so body expressions route through NaN-aware runtime dispatch and
        // the float value survives the i64 trampoline ABI intact.
        let is_gen_fn = contains_yield(body);
        // Call-site float hints for this function's unannotated positional params:
        // a param only ever called with float arguments is monomorphized as
        // `float` (not the default raw-int) so float NaN-box bits don't leak as
        // ints. Annotated/decorated params are untouched.
        let param_float_hints = if is_method || is_decorated {
            None
        } else {
            self.func_param_float_hint.get(name).cloned()
        };
        let hir_params: Vec<(SymbolId, TypeId)> = params
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                let param_ty = if is_method {
                    any_ty
                } else if p.kind == ast::ParamKind::Star || p.kind == ast::ParamKind::DoubleStar {
                    // *args receives a NaN-boxed MbList, **kwargs receives a NaN-boxed MbDict.
                    // Both must use any_ty so codegen treats them as MbValue (not raw i64).
                    any_ty
                } else if is_decorated {
                    // Decorated functions are called through mb_call1_val /
                    // mb_call_spread, which pass NaN-boxed MbValues regardless of
                    // annotation. Use any_ty for all params so body expressions
                    // route through the NaN-aware runtime dispatch (`mb_lt`,
                    // `mb_sub`, `mb_str_concat`, …). Exception: if the caller
                    // annotated the type explicitly, honor the annotation — the
                    // wrapper passes boxed values but primitive annotations let
                    // hir_to_mir insert an unbox at entry.
                    let resolved = self.resolve_type_expr_ro(&p.ty);
                    if resolved == any_ty {
                        any_ty
                    } else {
                        resolved
                    }
                } else {
                    let resolved = self.resolve_type_expr_ro(&p.ty);
                    if resolved == any_ty {
                        // Only fall back to raw-int convention for bare/unannotated params.
                        // Generic/Union/Tuple annotations that failed interning must stay `any`
                        // so the param is treated as a boxed NaN-value at pattern-match time.
                        match &p.ty.node {
                            ast::TypeExpr::Generic { .. }
                            | ast::TypeExpr::Union(_)
                            | ast::TypeExpr::Tuple(_)
                            | ast::TypeExpr::Optional(_) => any_ty,
                            // PEP 695: a `T`-annotated param of a generic function
                            // is a boxed MbValue at runtime (call sites box every
                            // primitive destined for a TypeVar param) — the
                            // raw-int default would reinterpret boxed bits.
                            ast::TypeExpr::Named(n)
                                if self.active_type_params.contains(n.as_str()) =>
                            {
                                any_ty
                            }
                            // An unannotated param used in `==`/`!=`/`in` must compare
                            // by value, so it cannot keep the raw-int identity path.
                            _ if value_compared_params.contains(&p.name) => any_ty,
                            // Unannotated generator params stay `any` (NaN-boxed) — the
                            // int default is unsound for float args crossing the i64
                            // generator trampoline ABI.
                            _ if is_gen_fn => any_ty,
                            _ => {
                                // Unannotated param: default int, but promote to float
                                // when every call site passes a float at this position,
                                // or to `any` (boxed) when float is mixed with a
                                // non-float — a raw f64 in an int slot leaks its bits
                                // as an int (e.g. `isinstance(0.5, int)` → True).
                                match param_float_hints.as_ref().and_then(|h| h.get(idx)).copied() {
                                    Some(FloatHint::Float) => float_ty,
                                    Some(FloatHint::Boxed) => any_ty,
                                    _ => int_ty,
                                }
                            }
                        }
                    } else {
                        resolved
                    }
                };
                let pid = self.define_local(&p.name, param_ty);
                (pid, param_ty)
            })
            .collect();

        // Return type: use annotation if available; otherwise infer from body.
        let ret_ty = if is_method {
            any_ty
        } else {
            let resolved = _return_ty
                .as_ref()
                .map(|rt| self.resolve_type_expr_ro(rt))
                .unwrap_or(any_ty);
            // PEP 695: `-> T` returns a boxed MbValue (TypeVar erases to
            // any); skip the float/int return inference entirely.
            let ret_is_type_param = matches!(
                _return_ty.as_ref().map(|rt| &rt.node),
                Some(ast::TypeExpr::Named(n))
                    if self.active_type_params.contains(n.as_str())
            );
            if ret_is_type_param {
                any_ty
            } else if resolved == any_ty {
                // Build a name→FloatHint environment from the (possibly float-
                // promoted) param types plus float-typed locals, then infer the
                // return type. Provably-float returns become `float`; everything
                // else falls through to int (preserving integer fast paths).
                let mut env: HashMap<String, FloatHint> = HashMap::new();
                // Seed module-level float globals first; params/locals shadow them.
                env.extend(
                    self.module_float_globals
                        .iter()
                        .map(|(k, &v)| (k.clone(), v)),
                );
                for (p, &(_, pty)) in params.iter().zip(hir_params.iter()) {
                    let h = if pty == float_ty {
                        FloatHint::Float
                    } else if pty == int_ty {
                        FloatHint::Int
                    } else if pty == any_ty {
                        // A NaN-boxed param (e.g. mixed float/non-float call
                        // sites): returning it directly yields an `any` result,
                        // not raw int.
                        FloatHint::Boxed
                    } else {
                        FloatHint::Unknown
                    };
                    env.insert(p.name.clone(), h);
                }
                collect_local_float_env(body, &mut env, &self.func_ret_float_hint);
                infer_return_type_from_ast(body, self.checker, &env, &self.func_ret_float_hint)
                    .unwrap_or(int_ty)
            } else {
                resolved
            }
        };
        // Collect names assigned anywhere in the body so the Assign arm of
        // lower_stmt can tell whether `x = val` should define a local (if
        // outer `x` exists but the function also assigns to x, Python makes
        // the assignment local). Stored in `local_assigned_names` for the
        // duration of this function's lowering.
        let mut pre_assigned: Vec<String> = Vec::new();
        let mut pre_declared: Vec<String> = Vec::new();
        crate::resolve::pass::collect_assignment_targets(
            body,
            &mut pre_assigned,
            &mut pre_declared,
        );
        crate::resolve::pass::collect_walrus_targets_in_stmts(body, &mut pre_assigned);
        let saved_local_assigned = std::mem::take(&mut self.local_assigned_names);
        let saved_declared = std::mem::take(&mut self.local_declared_names);
        self.local_assigned_names = pre_assigned;
        self.local_declared_names = pre_declared;

        // Pre-register this function's return type *before* lowering the body
        // so recursive calls inside the body can resolve their own callee's
        // return type. Without this, `def fact(n:int)->int: return n*fact(n-1)`
        // lowers `fact(n-1)` with `ty = any_ty`, hir_to_mir's CheckedMul gate
        // (matching `(Ty::Int, Ty::Int)`) rejects the multiplication, and the
        // raw-int fast path is bypassed entirely (factorial 0.68× — fire 49).
        // Methods (is_method=true) intentionally use any_ty; skip them so we
        // don't accidentally promote a method call's result to a primitive.
        if !is_method {
            self.func_return_tys.insert(name_id, ret_ty);
        }

        let hir_body: Vec<HirStmt> = body.iter().filter_map(|s| self.lower_stmt(s)).collect();

        self.local_assigned_names = saved_local_assigned;
        self.local_declared_names = saved_declared;

        // Collect cell symbols discovered while lowering this function body.
        let captures: Vec<SymbolId> = self.cell_override_syms.iter().copied().collect();

        // Flush the local_types accumulated during this function's body into sym_types
        // BEFORE restoring. Without this, pattern capture bindings (e.g. `case [x]`)
        // recorded in local_types would be lost when we restore the outer scope state,
        // causing hir_to_mir to fall back to any_ty for those captures (#827).
        for (&k, &v) in &self.local_types {
            self.result.sym_types.entry(k).or_insert(v);
        }
        // HANDWRITE-BEGIN gap="standardize:projects-mamba-src-lower-ast-to-hir-rs" tracker="standardize-gap-projects-mamba-src-lower-ast-to-hir-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
        // Mirror the same flush for local_names so this function's local
        // SymbolId → name pairs survive into hir.sym_names. Without this,
        // saved_local_names is restored next, dropping the inner function's
        // names — `locals()` snapshots inside that function would then be
        // unable to resolve names.
        for (name, &sym_id) in &self.local_names {
            self.result
                .sym_names
                .entry(sym_id)
                .or_insert_with(|| name.clone());
        }
        // HANDWRITE-END

        // Restore outer scope state so the outer function's remaining body stmts
        // can still resolve names from the outer scope.
        self.local_names = saved_local_names;
        self.local_types = saved_local_types;
        self.outer_scope_names = saved_outer_scope;
        self.cell_override_syms = saved_cell_syms;
        self.in_function_body = saved_in_function_body;

        let star_param_pos = params.iter().position(|p| p.kind == ast::ParamKind::Star);
        let has_star_args = star_param_pos.is_some();
        let has_kwargs = params.iter().any(|p| p.kind == ast::ParamKind::DoubleStar);
        Some(HirFunction {
            name: name_id,
            params: hir_params,
            return_ty: ret_ty,
            body: hir_body,
            span,
            captures,
            is_async: false,
            is_generator: false,
            decorators: Vec::new(),
            has_star_args,
            star_param_pos,
            has_kwargs,
        })
    }

    /// Collect a `class` statement into the module class set: lower the body,
    /// register dataclass init params, record decorators / metaclass /
    /// class-attr assigns, and emit a ClassDefPlaceholder marker when needed.
    /// Returns the class symbol when the class needs a ClassDefPlaceholder
    /// (decorators or class-attr assigns). When `placeholder_to_top` the
    /// marker is pushed to top_level (module top-level path); otherwise the
    /// CALLER must emit it into its own statement stream (nested classes in
    /// try/if/for bodies — previously silently dropped there).
    fn collect_class_stmt(
        &mut self,
        name: &str,
        body: &[Spanned<ast::Stmt>],
        bases: &[Spanned<ast::Expr>],
        decorators: &[Spanned<ast::Expr>],
        keyword_args: &[(String, Spanned<ast::Expr>)],
        span: crate::source::span::Span,
        placeholder_to_top: bool,
    ) -> Option<SymbolId> {
        let placeholder_sym: std::cell::Cell<Option<SymbolId>> = std::cell::Cell::new(None);
        let stmt_span = span;
        let dataclass_decorated = decorators.iter().any(|d| decorator_is_dataclass(&d.node));
        if let Some(mut cls) = self.lower_class(name, body, stmt_span, dataclass_decorated) {
            // PEP 557: register the synthesized __init__'s parameter
            // shape (declaration order; base dataclass fields first;
            // ClassVar / KW_ONLY sentinel / field(init=False) fields
            // excluded; InitVars included) so call sites resolve
            // keyword args and fill defaults exactly like calls to
            // classes with an explicit __init__. Skipped when the
            // class defines its own __init__ (pre-scan already
            // registered it).
            if dataclass_decorated && !self.func_param_info.contains_key(name) {
                let mut params: Vec<(String, Option<Spanned<ast::Expr>>, ast::ParamKind)> =
                    Vec::new();
                // Inherited dataclass init params first (single
                // inheritance chains; names overridden by the
                // subclass are replaced in place below).
                for b in bases {
                    let base_name = match &b.node {
                        ast::Expr::Ident(n) => Some(n.clone()),
                        ast::Expr::Attr { attr, .. } => Some(attr.clone()),
                        _ => None,
                    };
                    if let Some(bn) = base_name {
                        if let Some(binfo) = self.dataclass_init_params.get(&bn) {
                            params.extend(binfo.iter().cloned());
                        }
                    }
                }
                for s in body.iter() {
                    let (fname, ann, default) = match &s.node {
                        ast::Stmt::VarDecl {
                            name: fname,
                            ty,
                            value,
                        } => (fname, ty, Some(value.clone())),
                        ast::Stmt::BareAnnotation { name: fname, ty } => (fname, ty, None),
                        _ => continue,
                    };
                    if fname == "__match_args__" || fname == "__slots__" {
                        continue;
                    }
                    // ClassVar fields and the KW_ONLY sentinel are
                    // not __init__ params; field(init=False) opts out.
                    if type_expr_is_marker(&ann.node, "ClassVar")
                        || type_expr_is_marker(&ann.node, "KW_ONLY")
                    {
                        continue;
                    }
                    if default
                        .as_ref()
                        .is_some_and(|v| field_call_has_init_false(&v.node))
                    {
                        continue;
                    }
                    let entry = (fname.clone(), default, ast::ParamKind::Regular);
                    if let Some(pos) = params.iter().position(|(n, _, _)| n == fname) {
                        params[pos] = entry;
                    } else {
                        params.push(entry);
                    }
                }
                self.dataclass_init_params
                    .insert(name.to_string(), params.clone());
                self.func_param_info.insert(name.to_string(), params);
            }
            // Resolve all base classes for multiple inheritance (P1 OOP)
            cls.all_bases = bases
                .iter()
                .filter_map(|b| {
                    if let ast::Expr::Ident(name) = &b.node {
                        self.resolve_name(name, stmt_span)
                    } else if let ast::Expr::Attr { attr, .. } = &b.node {
                        // `class X(unittest.TestCase):` — treat the
                        // attribute's bare name as the base class id
                        // so MRO walks find the runtime-registered
                        // class. Fall back to declaring the symbol
                        // when it isn't in scope yet so the
                        // resolver doesn't drop the base.
                        self.resolve_name(attr, stmt_span)
                            .or_else(|| Some(self.define_local(attr, self.checker.tcx.any())))
                    } else {
                        None
                    }
                })
                .collect();
            // Keep first base for backward compatibility
            cls.base = cls.all_bases.first().copied();
            cls.decorators = decorators
                .iter()
                .filter_map(|d| self.lower_expr(d))
                .collect();
            // Extract metaclass keyword arg if present. The value
            // may be a bare name (`metaclass=Meta`) or an attribute
            // access (`metaclass=abc.ABCMeta`); in both cases the
            // metaclass identity is the leaf name.
            cls.metaclass = keyword_args.iter().find_map(|(k, v)| {
                if k == "metaclass" {
                    match &v.node {
                        ast::Expr::Ident(meta_name) => Some(meta_name.clone()),
                        ast::Expr::Attr { attr, .. } => Some(attr.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            });
            // R10: Extract non-metaclass keyword arguments for __init_subclass__.
            cls.class_kwargs = keyword_args
                .iter()
                .filter(|(k, _)| k != "metaclass")
                .filter_map(|(k, v)| self.lower_expr(v).map(|expr| (k.clone(), expr)))
                .collect();
            let inherits_simple_namespace = bases.iter().any(|b| {
                let leaf = match &b.node {
                    ast::Expr::Ident(base_name) => Some(base_name.as_str()),
                    ast::Expr::Attr { attr, .. } => Some(attr.as_str()),
                    _ => None,
                };
                leaf.is_some_and(|base_name| {
                    base_name == "SimpleNamespace"
                        || self.simple_namespace_subclass_idents.contains(base_name)
                })
            });
            if inherits_simple_namespace {
                self.simple_namespace_subclass_idents.insert(name.to_string());
            }
            // Emit a ClassDefPlaceholder so decorator application
            // happens at the textual position (#1690). Without
            // a placeholder, decorators were applied at module-end
            // (the #1686 stop-gap), which broke patterns like
            // `@deco class C; obj = C()` where the post-class
            // statement expects the decorated class.
            // Classes with class-level attribute assignments also
            // need one: initializer expressions like
            // `X = enum.auto()` must evaluate at the class's
            // textual position, after preceding imports/bindings
            // have run (P2-R3 ordering, #1686 motivation).
            if !cls.decorators.is_empty() || !cls.class_attr_assigns.is_empty() {
                if placeholder_to_top {
                    self.result.top_level.push(HirStmt::ClassDefPlaceholder {
                        name: cls.name,
                        span: stmt_span,
                    });
                }
                placeholder_sym.set(Some(cls.name));
            }
            self.result.classes.push(cls);
        }
        placeholder_sym.get()
    }

    fn lower_class(
        &mut self,
        name: &str,
        body: &[Spanned<ast::Stmt>],
        span: Span,
        dataclass_decorated: bool,
    ) -> Option<HirClass> {
        let name_id = self.resolve_name(name, span)?;
        // PEP 557: ordered (field_name, annotation_repr, default_expr) facts
        // from class-body annotations, recorded only for @dataclass classes so
        // the runtime synthesizer can build __init__/__repr__/__eq__/etc.
        let mut dataclass_fields: Vec<(String, String, Option<HirExpr>)> = Vec::new();
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        // Track all method name→SymbolId mappings so they survive scope clears
        let mut method_name_map: Vec<(String, crate::resolve::SymbolId)> = Vec::new();
        // Scan for explicit `__match_args__ = ("x", "y")` in the class body (#827).
        let mut explicit_match_args: Option<Vec<String>> = None;
        // P2-R3: Class-level attribute assignments (e.g., `attr = Verbose()` in class body).
        let mut class_attr_assigns: Vec<(String, HirExpr)> = Vec::new();
        // R14: __slots__ declaration from class body.
        let mut slots: Option<Vec<String>> = None;
        // PEP 526: ordered (name, annotation_repr) for EVERY annotated class-body
        // name (`a: int = 1` and bare `x: float`), recorded for all classes so
        // `C.__annotations__` exposes the mapping (CPython semantics). Values are
        // the textual annotation, matching mamba's module-level `__annotations__`.
        let mut class_annotations: Vec<(String, String)> = Vec::new();

        // PRE-SCAN: Extract __init__ param names BEFORE any method lowering (#827).
        // lower_fn_inner calls enter_local_scope() which clears local_names, so we must
        // capture __init__ params early — before subsequent method processing erases them.
        // If no explicit __match_args__ assignment is found, we'll use these as match args.
        let mut init_derived_match_args: Option<Vec<String>> = None;
        for stmt in body {
            if let ast::Stmt::FnDef {
                name: mname,
                params,
                ..
            } = &stmt.node
            {
                if *mname == "__init__" {
                    // Collect param names, skip "self"
                    let names: Vec<String> = params
                        .iter()
                        .filter(|p| p.name != "self")
                        .map(|p| p.name.clone())
                        .collect();
                    if !names.is_empty() {
                        init_derived_match_args = Some(names);
                    }
                    // Register __init__ params (minus self) under the class name so
                    // `ClassName(arg)` call sites can fill defaults via the same
                    // resolution path used for free functions.
                    let init_param_info: Vec<(String, Option<Spanned<ast::Expr>>, ast::ParamKind)> =
                        params
                            .iter()
                            .filter(|p| p.name != "self")
                            .map(|p| (p.name.clone(), p.default.clone(), p.kind))
                            .collect();
                    self.func_param_info
                        .insert(name.to_string(), init_param_info);
                    break;
                }
            }
        }

        for stmt in body {
            match &stmt.node {
                ast::Stmt::VarDecl {
                    name: fname,
                    ty,
                    value,
                } => {
                    // `__match_args__: tuple = ("x", "y")` — typed var declaration (#827)
                    if fname == "__match_args__" {
                        if let ast::Expr::TupleLit(elems) = &value.node {
                            let names: Vec<String> = elems
                                .iter()
                                .filter_map(|e| {
                                    if let ast::Expr::StrLit(s) = &e.node {
                                        Some(s.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            explicit_match_args = Some(names);
                        }
                    } else {
                        class_annotations.push((fname.clone(), type_expr_repr(&ty.node)));
                        if let Some(fid) = self.resolve_name(fname, stmt.span) {
                            fields.push((fid, self.checker.tcx.int()));
                        }
                        // PEP 557: annotated assignment with default value.
                        if dataclass_decorated && fname != "__slots__" {
                            let default = self.lower_expr(value);
                            dataclass_fields.push((
                                fname.clone(),
                                type_expr_repr(&ty.node),
                                default,
                            ));
                        } else if fname != "__slots__" {
                            // `y: str = "hi"` in a non-dataclass class body binds a
                            // real class attribute (CPython), unlike a bare `x: int`.
                            // Mirror the plain-assignment path.
                            if let Some(val_expr) = self.lower_expr(value) {
                                class_attr_assigns.push((fname.clone(), val_expr));
                            }
                        }
                    }
                }
                // PEP 557: bare annotation `x: float` — an ordered dataclass
                // field fact with no default. (Outside dataclasses these are
                // type-info-only and remain dropped.)
                ast::Stmt::BareAnnotation { name: fname, ty } => {
                    class_annotations.push((fname.clone(), type_expr_repr(&ty.node)));
                    if dataclass_decorated {
                        dataclass_fields.push((fname.clone(), type_expr_repr(&ty.node), None));
                    }
                }
                ast::Stmt::FnDef {
                    name: mname,
                    params,
                    return_ty,
                    body: mbody,
                    decorators,
                    ..
                }
                | ast::Stmt::AsyncFnDef {
                    name: mname,
                    params,
                    return_ty,
                    body: mbody,
                    decorators,
                    ..
                } => {
                    let is_async_method = matches!(stmt.node, ast::Stmt::AsyncFnDef { .. });
                    // Always allocate a fresh SymbolId for each class method.
                    // Using define_local would reuse the same SymbolId when multiple classes
                    // define methods with the same name (e.g. two `__enter__` methods), causing
                    // duplicate MIR body names and Cranelift "Duplicate definition" errors.
                    let method_sym = {
                        let id = SymbolId(self.next_local_sym);
                        self.next_local_sym += 1;
                        self.local_names.insert(mname.to_string(), id);
                        self.local_types.insert(id, self.checker.tcx.int());
                        id
                    };
                    method_name_map.push((mname.to_string(), method_sym));
                    let method_is_decorated = !decorators.is_empty();
                    if let Some(mut m) = self.lower_fn_inner(
                        mname,
                        params,
                        return_ty,
                        mbody,
                        stmt.span,
                        true,
                        method_is_decorated,
                    ) {
                        self.result.func_sigs
                            .insert(m.name.0, func_sig_meta(params, return_ty));
                        let has_yield = contains_yield(mbody);
                        if is_async_method {
                            // `async def` method: route same way the top-level
                            // AsyncFnDef arm does — yield → sync generator,
                            // otherwise coroutine. Without this, async dunders
                            // like `__aenter__` / `__aexit__` were silently
                            // dropped at class registration time.
                            if has_yield {
                                m.is_generator = true;
                            } else {
                                m.is_async = true;
                            }
                        } else {
                            m.is_generator = has_yield;
                        }
                        // Capture method decorators so hir_to_mir can wrap the method
                        // in a property/classmethod/staticmethod descriptor at class
                        // registration time.
                        m.decorators = decorators
                            .iter()
                            .filter_map(|d| self.lower_expr(d))
                            .collect();
                        methods.push(m);
                    }
                }
                // `__match_args__ = ("x", "y")` — explicit tuple assignment (#827)
                // `__slots__ = ['x', 'y']` or `__slots__ = ('x', 'y')` — R14
                // Other assignments → class-level attribute init (P2-R3)
                ast::Stmt::Assign { target, value } => {
                    if let ast::Expr::Ident(aname) = &target.node {
                        if aname == "__match_args__" {
                            if let ast::Expr::TupleLit(elems) = &value.node {
                                let names: Vec<String> = elems
                                    .iter()
                                    .filter_map(|e| {
                                        if let ast::Expr::StrLit(s) = &e.node {
                                            Some(s.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                explicit_match_args = Some(names);
                            }
                        } else if aname == "__slots__" {
                            // R14: Extract __slots__ from list literal or tuple literal.
                            let elems_opt = match &value.node {
                                ast::Expr::ListLit(elems) => Some(elems),
                                ast::Expr::TupleLit(elems) => Some(elems),
                                _ => None,
                            };
                            if let Some(elems) = elems_opt {
                                let slot_names: Vec<String> = elems
                                    .iter()
                                    .filter_map(|e| {
                                        if let ast::Expr::StrLit(s) = &e.node {
                                            Some(s.clone())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                slots = Some(slot_names);
                            } else {
                                // __slots__ = () — empty tuple, no instance attributes allowed
                                slots = Some(Vec::new());
                            }
                        } else {
                            // P2-R3: Class-level attribute assignment (e.g., `attr = Verbose()`).
                            // Lower the value expression and store for emission after class registration.
                            if let Some(val_expr) = self.lower_expr(value) {
                                class_attr_assigns.push((aname.clone(), val_expr));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Re-insert all method name→SymbolId mappings into local_names AND result.sym_names.
        // local_names alone is not enough: subsequent lower_fn calls invoke enter_local_scope()
        // which clears local_names, so method names would be lost before lower_module builds
        // sym_names at the end. Storing in result.sym_names immediately guarantees they
        // survive scope clears and are available for sym_name_lookup in hir_to_mir (#827).
        for (mname, msym) in method_name_map {
            self.local_names.insert(mname.clone(), msym);
            self.result.sym_names.insert(msym, mname);
        }

        // If no explicit __match_args__ was found, use __init__ params extracted in the
        // pre-scan above. This avoids relying on local_names surviving through multiple
        // method lowerings (each of which calls enter_local_scope, clearing local_names).
        // If neither is found, fall back to field declaration order (matches type checker
        // behavior so that `case Point(1, 2):` works at runtime too — #827).
        let resolved_match_args = explicit_match_args.or(init_derived_match_args).or_else(|| {
            // PEP 557: for @dataclass classes the runtime decorator computes
            // `__match_args__` from the processed field list (kw_only and
            // ClassVar excluded) — suppress the raw field-order fallback so
            // the decorator's or_insert is not pre-empted by a wrong tuple.
            if dataclass_decorated || fields.is_empty() {
                None
            } else {
                let field_names: Vec<String> = body
                    .iter()
                    .filter_map(|s| {
                        if let ast::Stmt::VarDecl { name: fname, .. } = &s.node {
                            if fname != "__match_args__" {
                                Some(fname.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                if field_names.is_empty() {
                    None
                } else {
                    Some(field_names)
                }
            }
        });

        // PEP 526: expose `C.__annotations__` as a class attribute mapping every
        // annotated class-body name to its textual annotation. Built for all
        // classes (not just dataclasses) so introspection matches CPython.
        if !class_annotations.is_empty() {
            let str_ty = self.checker.tcx.str();
            let any_ty = self.checker.tcx.any();
            let entries: Vec<(HirExpr, HirExpr)> = class_annotations
                .iter()
                .map(|(n, r)| {
                    (
                        HirExpr::StrLit(n.clone(), str_ty),
                        HirExpr::StrLit(r.clone(), str_ty),
                    )
                })
                .collect();
            class_attr_assigns.push((
                "__annotations__".to_string(),
                HirExpr::Dict { entries, ty: any_ty },
            ));
        }

        // Class-body docstring: first bare string statement (inspect.getdoc).
        let class_doc = body.first().and_then(|s| {
            if let ast::Stmt::ExprStmt(e) = &s.node {
                if let ast::Expr::StrLit(d) = &e.node {
                    Some(d.clone())
                } else {
                    None
                }
            } else {
                None
            }
        });
        Some(HirClass {
            name: name_id,
            base: None,
            all_bases: Vec::new(),
            fields,
            methods,
            span,
            decorators: Vec::new(),
            explicit_match_args: resolved_match_args,
            metaclass: None,
            class_attr_assigns,
            slots,
            class_kwargs: Vec::new(),
            dataclass_fields,
            doc: class_doc,
        })
    }

    fn lower_stmt(&mut self, stmt: &Spanned<ast::Stmt>) -> Option<HirStmt> {
        match &stmt.node {
            // Classes nested inside try/if/for bodies: collect like a
            // top-level class (registration is hoisted), and leave a
            // ClassDefPlaceholder IN this statement stream so decorators and
            // class-attr initializers run at the textual position — inside
            // the enclosing try's handler scope.
            ast::Stmt::ClassDef {
                name,
                body,
                bases,
                decorators,
                keyword_args,
                ..
            } => {
                let sym = self.collect_class_stmt(
                    name,
                    body,
                    bases,
                    decorators,
                    keyword_args,
                    stmt.span,
                    false,
                );
                return sym.map(|name| HirStmt::ClassDefPlaceholder {
                    name,
                    span: stmt.span,
                });
            }
            ast::Stmt::VarDecl { name, value, .. } => {
                let val = self.lower_expr(value)?;
                // Mirror the Assign first-definition path: when the resolve
                // pass already recorded a SymbolId for this name in an
                // accessible scope (always the case at module scope, since
                // resolve_pass.rs:80 defines VarDecl names), bind the Let
                // to THAT id rather than allocating a fresh local. Without
                // this, function bodies that read the same name resolve
                // via name_map to the resolve-pass id and LoadGlobal at a
                // slot we never StoreGlobal'd — annotated module globals
                // appeared as None inside any function reading them.
                //
                // Inside a function body, `x: int = …` shadows the outer
                // binding per Python scoping. Detect that the body assigns
                // to the name (mirrors Assign's `is_function_local_target`)
                // and prefer a fresh local in that case.
                let is_function_local_target = self.local_assigned_names.iter().any(|n| n == name)
                    && !self.local_declared_names.iter().any(|n| n == name)
                    && !self.local_names.contains_key(name);
                let sym = if is_function_local_target {
                    self.define_local(name, val.ty())
                } else if let Some(id) = self.resolve_name(name, stmt.span) {
                    self.local_types.insert(id, val.ty());
                    // Mirror the binding into local_names so it flows into
                    // hir.sym_names (built from local_names at end of module
                    // lowering). When VarDecl resolves to a pre-defined checker
                    // symbol (resolve_pass defines module-level annotated names
                    // but NOT plain assignments), the name was otherwise never
                    // recorded in local_names — so an annotated module global
                    // `x: int = v` was absent from sym_names. The REPL collects
                    // new_globals by filtering sym_to_vreg through sym_names, so
                    // the missing name meant `x` was neither tracked in
                    // known_globals nor persisted via a boxed StoreGlobal,
                    // unlike plain `x = v` (which takes the define_local path).
                    // This makes annotated module globals persist identically.
                    self.local_names.entry(name.clone()).or_insert(id);
                    id
                } else {
                    self.define_local(name, val.ty())
                };
                Some(HirStmt::Let {
                    target: sym,
                    ty: val.ty(),
                    value: val,
                    span: stmt.span,
                })
            }
            ast::Stmt::Assign { target, value } => {
                // Ident assignments: define locally if new, reassign if exists
                if let ast::Expr::Ident(name) = &target.node {
                    // Alias tracking for the kwargs-dict call convention:
                    // `quantiles = statistics.quantiles` must keep keyword
                    // names at bare-ident call sites, like the import form.
                    if let ast::Expr::Attr { object, attr } = &value.node {
                        if attr == "quantiles" {
                            if let ast::Expr::Ident(m) = &object.node {
                                if m == "statistics" {
                                    self.dataclasses_kwarg_idents.insert(name.clone());
                                }
                            }
                        }
                    }
                    let is_functools_partial_instance = match &value.node {
                        ast::Expr::Call { func: call_func, .. } => match &call_func.node {
                            ast::Expr::Attr { object, attr } if attr == "partial" => {
                                matches!(
                                    &object.node,
                                    ast::Expr::Ident(module_name)
                                        if self.functools_module_idents.contains(module_name.as_str())
                                )
                            }
                            ast::Expr::Ident(factory_name) => self
                                .functools_partial_factory_idents
                                .contains(factory_name.as_str()),
                            _ => false,
                        },
                        _ => false,
                    };
                    if is_functools_partial_instance {
                        self.functools_partial_kwarg_idents.insert(name.clone());
                    } else {
                        self.functools_partial_kwarg_idents.remove(name.as_str());
                    }
                    let val = self.lower_expr(value)?;
                    // Python scoping: inside a function body, an assignment
                    // to a name that's NOT already local defines a new local
                    // — even if an outer-scope symbol with the same name
                    // exists. Only defer to the outer binding when explicitly
                    // declared via `global` / `nonlocal`.
                    let is_function_local_target =
                        self.local_assigned_names.iter().any(|n| n == name)
                            && !self.local_declared_names.iter().any(|n| n == name)
                            && !self.local_names.contains_key(name);
                    if is_function_local_target || self.resolve_name(name, stmt.span).is_none() {
                        // Implicit declaration — creates a fresh local.
                        let sym = self.define_local(name, val.ty());
                        return Some(HirStmt::Let {
                            target: sym,
                            ty: val.ty(),
                            value: val,
                            span: stmt.span,
                        });
                    }
                    let sym = self.resolve_name(name, stmt.span)?;
                    // If the HIR-inferred val type is a primitive but the
                    // checker tagged the symbol Any (typically because
                    // check_fn_body records non-annotated returns as Any
                    // while HIR infers int from the literal), coerce the
                    // value back to Any before the Assign so downstream
                    // reads don't get a stray primitive-typed Var that
                    // skips NaN-boxing at runtime dispatch sites.
                    let val_boxed = if self.get_type(sym) == self.checker.tcx.any()
                        && val.ty() != self.checker.tcx.any()
                    {
                        // Wrap in a cast expression that advertises Any but
                        // carries the underlying value. HIR has no dedicated
                        // cast node, so we pass the value through: the boxing
                        // happens at MIR emit via box_operand when the Assign
                        // lowering sees value.ty() == Any.
                        match val {
                            HirExpr::Var(s, _) => HirExpr::Var(s, self.checker.tcx.any()),
                            HirExpr::Call { func, args, .. } => HirExpr::Call {
                                func,
                                args,
                                ty: self.checker.tcx.any(),
                            },
                            other => other,
                        }
                    } else {
                        val
                    };
                    return Some(HirStmt::Assign {
                        target: HirLValue::Var(sym),
                        value: val_boxed,
                        span: stmt.span,
                    });
                }
                let lv = self.lower_lvalue(target)?;
                let val = self.lower_expr(value)?;
                Some(HirStmt::Assign {
                    target: lv,
                    value: val,
                    span: stmt.span,
                })
            }
            ast::Stmt::AugAssign { target, op, value } => {
                // Desugar: x += e → x = x + e
                //
                // CPython evaluates the target's receiver (and index) exactly
                // ONCE in `a[i] += v` / `a.b += v`. The naive desugar lowers
                // `target` twice (lvalue setitem + rvalue getitem), re-running
                // any side effects — `faces[rng.randint(1,6)-1] += 1` drew TWO
                // indices (read one slot, write another, corrupting the tally).
                // For non-atomic receiver/index sub-exprs, bind them to fresh
                // temps via Walrus inside the RVALUE (Assign lowers the value
                // before the target, so the temps are live when the setitem
                // path reads them as plain Vars).
                fn hir_atomic(e: &HirExpr) -> bool {
                    matches!(
                        e,
                        HirExpr::Var(..) | HirExpr::IntLit(..) | HirExpr::FloatLit(..)
                            | HirExpr::StrLit(..) | HirExpr::BoolLit(..) | HirExpr::NoneLit(..)
                            // Slice nodes are special-cased at Index sites
                            // (packed to (start, stop, step)); hoisting one
                            // into a Walrus temp would bypass that packing.
                            // Cloning keeps the pre-fix behavior for slices.
                            | HirExpr::Slice { .. }
                    )
                }
                let is_index_or_attr = matches!(
                    target.node,
                    ast::Expr::Index { .. } | ast::Expr::Attr { .. }
                );
                let (lv, lhs) = if is_index_or_attr {
                    let mut lhs = self.lower_expr(target)?;
                    let mut hoist = |slot: &mut Box<HirExpr>, tag: &str, this: &mut Self| {
                        if hir_atomic(slot) {
                            return (**slot).clone();
                        }
                        let ty = slot.ty();
                        let name = format!("__aug_{tag}_{}", this.next_local_sym);
                        let sym = this.define_local(&name, ty);
                        let orig = std::mem::replace(&mut **slot, HirExpr::NoneLit(ty));
                        **slot = HirExpr::Walrus {
                            target: sym,
                            value: Box::new(orig),
                            ty,
                        };
                        HirExpr::Var(sym, ty)
                    };
                    let lv = match &mut lhs {
                        HirExpr::Index { object, index, .. } => {
                            let lv_obj = hoist(object, "obj", self);
                            let lv_idx = hoist(index, "idx", self);
                            HirLValue::Index {
                                object: Box::new(lv_obj),
                                index: Box::new(lv_idx),
                            }
                        }
                        HirExpr::Attr { object, attr, .. } => {
                            let lv_obj = hoist(object, "obj", self);
                            HirLValue::Attr {
                                object: Box::new(lv_obj),
                                attr: attr.clone(),
                            }
                        }
                        _ => self.lower_lvalue(target)?,
                    };
                    (lv, lhs)
                } else {
                    // Var/unpack targets: keep the original order — lower_lvalue
                    // first so an unbound name is defined before the read.
                    let lv = self.lower_lvalue(target)?;
                    let lhs = self.lower_expr(target)?;
                    (lv, lhs)
                };
                let rhs = self.lower_expr(value)?;
                let hir_op = lower_aug_op(*op);
                // In-place dunder dispatch (CPython tries `__iadd__` etc. before
                // `__add__`): for a non-primitive (instance) target, route
                // `a <op>= b` through a runtime helper that calls the in-place
                // dunder when present and otherwise falls back to the normal
                // binary op. Primitive targets (int/float/bool) keep the fast
                // BinOp path below. Only the six ops with an in-place helper
                // are rerouted; others fall through.
                {
                    use crate::types::Ty;
                    let lhs_primitive = matches!(
                        self.checker.tcx.get(lhs.ty()),
                        Ty::Int | Ty::Float | Ty::Bool
                    );
                    let helper = if lhs_primitive {
                        None
                    } else {
                        match op {
                            ast::AugOp::Add => Some("mb_iadd"),
                            ast::AugOp::Sub => Some("mb_isub"),
                            ast::AugOp::Mul => Some("mb_imul"),
                            ast::AugOp::Pow => Some("mb_ipow"),
                            ast::AugOp::BitAnd => Some("mb_iand"),
                            ast::AugOp::BitOr => Some("mb_ior"),
                            ast::AugOp::BitXor => Some("mb_ixor"),
                            _ => None,
                        }
                    };
                    if let Some(helper) = helper {
                        let any_ty = self.checker.tcx.any();
                        let call = HirExpr::Call {
                            func: Box::new(HirExpr::StrLit(helper.to_string(), any_ty)),
                            args: vec![lhs, rhs],
                            ty: any_ty,
                        };
                        if let HirLValue::Var(sym) = &lv {
                            self.local_types.insert(*sym, any_ty);
                        }
                        return Some(HirStmt::Assign {
                            target: lv,
                            value: call,
                            span: stmt.span,
                        });
                    }
                }
                // Python truediv always returns float: x /= 2 → float even if both are int.
                // Widen variable type to Any so subsequent reads don't double-box.
                let is_truediv = matches!(op, ast::AugOp::Div);
                // A mixed int/bool-with-float augmented op (e.g. `x = 0; x += 0.5`)
                // produces a float; widen the variable to `any` so the float
                // NaN-box isn't stored/read back as a raw int (the variable's old
                // int type would leak the IEEE-754 bits as a huge integer).
                let mixed_float = {
                    use crate::types::Ty;
                    matches!(
                        (
                            self.checker.tcx.get(lhs.ty()),
                            self.checker.tcx.get(rhs.ty())
                        ),
                        (Ty::Int | Ty::Bool, Ty::Float) | (Ty::Float, Ty::Int | Ty::Bool)
                    )
                };
                let widen = is_truediv || mixed_float;
                let ty = if widen {
                    self.checker.tcx.any()
                } else {
                    lhs.ty()
                };
                let binop = HirExpr::BinOp {
                    op: hir_op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    ty,
                };
                // Update variable type when we widened so subsequent reads
                // (print(x), x == 0.5) don't treat the float result as a raw int.
                if widen {
                    if let HirLValue::Var(sym) = &lv {
                        self.local_types.insert(*sym, ty);
                    }
                }
                Some(HirStmt::Assign {
                    target: lv,
                    value: binop,
                    span: stmt.span,
                })
            }
            ast::Stmt::Return(expr) => {
                let val = expr.as_ref().and_then(|e| self.lower_expr(e));
                Some(HirStmt::Return {
                    value: val,
                    span: stmt.span,
                })
            }
            ast::Stmt::If {
                condition,
                body,
                elif_clauses,
                else_body,
            } => {
                let cond = self.lower_expr(condition)?;
                let then_b: Vec<HirStmt> = body.iter().filter_map(|s| self.lower_stmt(s)).collect();
                // Desugar elif chains into nested if/else
                let else_b = self.lower_elif_chain(elif_clauses, else_body);
                Some(HirStmt::If {
                    cond,
                    then_body: then_b,
                    else_body: else_b,
                    span: stmt.span,
                })
            }
            ast::Stmt::While {
                condition,
                body,
                else_body,
            } => {
                let cond = self.lower_expr(condition)?;
                let b: Vec<HirStmt> = body.iter().filter_map(|s| self.lower_stmt(s)).collect();
                let eb: Vec<HirStmt> = else_body
                    .iter()
                    .flat_map(|stmts| stmts.iter())
                    .filter_map(|s| self.lower_stmt(s))
                    .collect();
                Some(HirStmt::While {
                    cond,
                    body: b,
                    else_body: eb,
                    span: stmt.span,
                })
            }
            ast::Stmt::For {
                targets,
                var_ty,
                iter,
                body,
                else_body,
            }
            | ast::Stmt::AsyncFor {
                targets,
                var_ty,
                iter,
                body,
                else_body,
            } => {
                let it = self.lower_expr(iter)?;
                let any_ty = self.checker.tcx.any();
                let span = stmt.span;

                // Infer element type: explicit annotation > range() → int > any
                let elem_ty = var_ty
                    .as_ref()
                    .map(|t| self.resolve_type_expr_ro(t))
                    .unwrap_or_else(|| self.infer_for_elem_ty(iter));

                if targets.len() > 1 {
                    // Desugar `for a, b in iter:` into:
                    //   for __for_tmp in iter:
                    //       a, b = __for_tmp   ← unpack each iteration value
                    //       <original body>
                    let tmp_name = format!("__for_tmp_{}", self.next_local_sym);
                    let tmp_sym = self.define_local(&tmp_name, any_ty);
                    // Define all named targets so the body can reference them.
                    let target_syms: Vec<SymbolId> = targets
                        .iter()
                        .map(|t| self.define_local(t, any_ty))
                        .collect();
                    // Build unpack assignment: (a, b, ...) = __for_tmp
                    let unpack_targets: Vec<HirLValue> =
                        target_syms.iter().map(|&sym| HirLValue::Var(sym)).collect();
                    let unpack_stmt = HirStmt::Assign {
                        target: HirLValue::Unpack {
                            targets: unpack_targets,
                            star_index: None,
                        },
                        value: HirExpr::Var(tmp_sym, any_ty),
                        span,
                    };
                    let mut b: Vec<HirStmt> = vec![unpack_stmt];
                    b.extend(body.iter().filter_map(|s| self.lower_stmt(s)));
                    let eb: Vec<HirStmt> = else_body
                        .iter()
                        .flat_map(|stmts| stmts.iter())
                        .filter_map(|s| self.lower_stmt(s))
                        .collect();
                    Some(HirStmt::For {
                        var: tmp_sym,
                        iter: it,
                        body: b,
                        else_body: eb,
                        span,
                    })
                } else {
                    let var_id = self.define_local(&targets[0], elem_ty);
                    let b: Vec<HirStmt> = body.iter().filter_map(|s| self.lower_stmt(s)).collect();
                    let eb: Vec<HirStmt> = else_body
                        .iter()
                        .flat_map(|stmts| stmts.iter())
                        .filter_map(|s| self.lower_stmt(s))
                        .collect();
                    Some(HirStmt::For {
                        var: var_id,
                        iter: it,
                        body: b,
                        else_body: eb,
                        span,
                    })
                }
            }
            ast::Stmt::Break => Some(HirStmt::Break { span: stmt.span }),
            ast::Stmt::Continue => Some(HirStmt::Continue { span: stmt.span }),
            ast::Stmt::ExprStmt(expr) => {
                // A bare statement that is a reference to a name resolving to
                // nothing (no local, global, builtin, or outer-scope capture)
                // is a runtime `NameError` in CPython. The checker's
                // compile-time "undefined name" diagnostic is suppressed in
                // such fixtures via `# type: ignore`, so lowering reaches here
                // with an unresolvable Ident; emit the raise rather than
                // silently dropping the statement (which printed "no_raise").
                if let ast::Expr::Ident(name) = &expr.node {
                    if !self.in_function_body
                        && self.module_unbound_annotation_names.contains(name)
                    {
                        if let Some(raise) = self.name_error_raise(name, stmt.span) {
                            return Some(raise);
                        }
                    }
                    if self.resolve_name(name, expr.span).is_none()
                        && !self.outer_scope_names.contains_key(name.as_str())
                    {
                        if let Some(raise) = self.name_error_raise(name, stmt.span) {
                            return Some(raise);
                        }
                    }
                }
                let e = self.lower_expr(expr)?;
                Some(HirStmt::Expr {
                    expr: e,
                    span: stmt.span,
                })
            }
            ast::Stmt::Pass => None,
            ast::Stmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
            } => {
                let hir_body: Vec<HirStmt> =
                    body.iter().filter_map(|s| self.lower_stmt(s)).collect();
                let hir_handlers: Vec<HirExceptHandler> = handlers
                    .iter()
                    .map(|h| {
                        // Define handler name locally so body can reference it
                        let name_sym = h
                            .name
                            .as_ref()
                            .map(|n| self.define_local(n, self.checker.tcx.any()));
                        HirExceptHandler {
                            exc_type: h.exc_type.as_ref().and_then(|e| self.lower_expr(e)),
                            name: name_sym,
                            body: h.body.iter().filter_map(|s| self.lower_stmt(s)).collect(),
                            is_star: h.is_star,
                            span: h.span,
                        }
                    })
                    .collect();
                let hir_else = else_body
                    .as_ref()
                    .map(|eb| eb.iter().filter_map(|s| self.lower_stmt(s)).collect())
                    .unwrap_or_default();
                let hir_finally = finally_body
                    .as_ref()
                    .map(|fb| fb.iter().filter_map(|s| self.lower_stmt(s)).collect())
                    .unwrap_or_default();
                Some(HirStmt::Try {
                    body: hir_body,
                    handlers: hir_handlers,
                    else_body: hir_else,
                    finally_body: hir_finally,
                    span: stmt.span,
                })
            }
            ast::Stmt::Raise { value, from } => {
                let v = value.as_ref().and_then(|e| self.lower_expr(e));
                let f = from.as_ref().and_then(|e| self.lower_expr(e));
                Some(HirStmt::Raise {
                    value: v,
                    from: f,
                    span: stmt.span,
                })
            }
            ast::Stmt::Import {
                module,
                names,
                module_alias,
            } => {
                // PEP 557: track local bindings of dataclasses.{dataclass,
                // field, replace} so bare-Ident calls to them keep keyword
                // names (trailing-kwargs-dict convention) at the call site.
                if module.len() == 1 && module[0] == "dataclasses" {
                    if let Some(names) = names {
                        for (orig, alias) in names {
                            if matches!(orig.as_str(), "dataclass" | "field" | "replace") {
                                self.dataclasses_kwarg_idents
                                    .insert(alias.clone().unwrap_or_else(|| orig.clone()));
                            }
                        }
                    }
                }
                // statistics.quantiles has a keyword-only signature
                // (`quantiles(data, *, n=4, method=...)`): its dispatcher must
                // distinguish `quantiles(data, n=4)` (fine) from
                // `quantiles(data, 4)` (TypeError), which requires keyword
                // names to survive lowering via the trailing-kwargs-dict
                // convention rather than flattening to positionals.
                if module.len() == 1 && module[0] == "statistics" {
                    if let Some(names) = names {
                        for (orig, alias) in names {
                            if orig == "quantiles" {
                                self.dataclasses_kwarg_idents
                                    .insert(alias.clone().unwrap_or_else(|| orig.clone()));
                            }
                        }
                    }
                }
                if module.len() == 1 && module[0] == "functools" {
                    if let Some(alias) = module_alias {
                        self.functools_module_idents.insert(alias.clone());
                    }
                    if let Some(names) = names {
                        for (orig, alias) in names {
                            if orig == "partial" {
                                self.functools_partial_factory_idents
                                    .insert(alias.clone().unwrap_or_else(|| orig.clone()));
                            }
                        }
                    }
                }
                Some(HirStmt::Import {
                    import: HirImport {
                        module: module.clone(),
                        names: names.clone(),
                        module_alias: module_alias.clone(),
                        span: stmt.span,
                    },
                    span: stmt.span,
                })
            }
            // Bare annotation `name: type` — emit a no-op (type info only, no runtime effect).
            ast::Stmt::BareAnnotation { .. } => None,
            ast::Stmt::With { items, body } | ast::Stmt::AsyncWith { items, body } => {
                let is_async = matches!(stmt.node, ast::Stmt::AsyncWith { .. });
                let hir_items: Vec<(HirExpr, Option<SymbolId>)> = items
                    .iter()
                    .filter_map(|item| {
                        let ctx = self.lower_expr(&item.context)?;
                        // `with expr as name` always binds `name` — define it if new,
                        // reuse existing SymbolId if already in scope.
                        let alias = item.alias.as_ref().map(|n| {
                            self.resolve_name(n, stmt.span)
                                .unwrap_or_else(|| self.define_local(n, self.checker.tcx.any()))
                        });
                        Some((ctx, alias))
                    })
                    .collect();
                let b: Vec<HirStmt> = body.iter().filter_map(|s| self.lower_stmt(s)).collect();
                Some(HirStmt::With {
                    items: hir_items,
                    body: b,
                    is_async,
                    span: stmt.span,
                })
            }
            ast::Stmt::Assert { test, msg } => {
                let t = self.lower_expr(test)?;
                let m = msg.as_ref().and_then(|e| self.lower_expr(e));
                Some(HirStmt::Assert {
                    test: t,
                    msg: m,
                    span: stmt.span,
                })
            }
            ast::Stmt::Del(target) => {
                let lv = self.lower_lvalue(target)?;
                Some(HirStmt::Del {
                    target: lv,
                    span: stmt.span,
                })
            }
            ast::Stmt::Global(names) => {
                let syms: Vec<SymbolId> = names
                    .iter()
                    .filter_map(|n| self.resolve_name(n, stmt.span))
                    .collect();
                Some(HirStmt::Global {
                    names: syms,
                    span: stmt.span,
                })
            }
            ast::Stmt::Nonlocal(names) => {
                let syms: Vec<SymbolId> = names
                    .iter()
                    .filter_map(|n| {
                        // Look up in outer scope first to get the same synthetic SymbolId.
                        // This ensures inner-function references use the same slot as the outer.
                        if let Some(&outer_sym) = self.outer_scope_names.get(n.as_str()) {
                            // Bind the name in the current (inner) scope to the outer SymbolId.
                            self.local_names.insert(n.clone(), outer_sym);
                            // Mark this SymbolId as a Cell (shared via global storage).
                            self.cell_override_syms.insert(outer_sym);
                            return Some(outer_sym);
                        }
                        self.resolve_name(n, stmt.span)
                    })
                    .collect();
                Some(HirStmt::Nonlocal {
                    names: syms,
                    span: stmt.span,
                })
            }
            ast::Stmt::Match { expr, arms } => {
                let subject = self.lower_expr(expr)?;
                // Stash subject type so lower_pattern can use it for capture bindings (#827).
                let subject_ty = subject.ty();
                let prev_subj_ty = self.current_match_subject_ty.replace(subject_ty);
                let cases: Vec<HirMatchCase> = arms
                    .iter()
                    .filter_map(|arm| {
                        let pattern = self.lower_pattern(&arm.pattern)?;
                        let guard = arm.guard.as_ref().and_then(|g| self.lower_expr(g));
                        let body: Vec<HirStmt> =
                            arm.body.iter().filter_map(|s| self.lower_stmt(s)).collect();
                        Some(HirMatchCase {
                            pattern,
                            guard,
                            body,
                            span: arm.span,
                        })
                    })
                    .collect();
                self.current_match_subject_ty = prev_subj_ty;
                Some(HirStmt::Match {
                    subject,
                    cases,
                    span: stmt.span,
                })
            }
            ast::Stmt::FnDef {
                name,
                type_params,
                params,
                return_ty,
                body,
                decorators,
                ..
            } => {
                // Register param info for kwargs resolution.
                self.func_param_info.insert(
                    name.clone(),
                    params
                        .iter()
                        .map(|p| (p.name.clone(), p.default.clone(), p.kind))
                        .collect(),
                );
                // Nested function definition inside a function body.
                // Define the function name in the current (outer) local scope first so the
                // outer function body can call it, and so resolve_name works inside lower_fn.
                let fn_sym = self.define_local(name, self.checker.tcx.any());
                let is_decorated = !decorators.is_empty();
                let overload_decorated = decorators
                    .iter()
                    .any(|d| decorator_is_typing_overload(&d.node));
                let erased_params;
                let params_for_lower: &[ast::Param] = if overload_decorated {
                    erased_params = erase_param_annotations(params);
                    &erased_params
                } else {
                    params
                };
                let erased_return: Option<Spanned<ast::TypeExpr>> = None;
                let return_for_lower = if overload_decorated {
                    &erased_return
                } else {
                    return_ty
                };
                // PEP 695: see the module-level FnDef arm — type-param names
                // must reach the param/return type lowering.
                let saved_tps = std::mem::replace(
                    &mut self.active_type_params,
                    type_params.iter().map(|p| p.name.clone()).collect(),
                );
                let lowered = if is_decorated {
                    self.lower_decorated_fn(name, params_for_lower, return_for_lower, body, stmt.span)
                } else {
                    self.lower_fn(name, params, return_ty, body, stmt.span)
                };
                self.active_type_params = saved_tps;
                if let Some(mut func) = lowered {
                    func.is_generator = contains_yield(body);
                    func.decorators = decorators
                        .iter()
                        .filter_map(|d| self.lower_expr(d))
                        .collect();
                    // Propagate the cell symbols discovered in this nested function back
                    // to the outer scope so the outer function also uses global storage
                    // for those shared variables.
                    for sym in &func.captures {
                        self.cell_override_syms.insert(*sym);
                    }
                    self.result.functions.push(func);
                }
                // Return a placeholder that will be resolved at call-site (the function is
                // now in result.functions). The local sym binding allows callers to emit
                // a Call MirInst against fn_sym.
                let _ = fn_sym;
                Some(HirStmt::FuncDefPlaceholder {
                    name: fn_sym,
                    span: stmt.span,
                })
            }
            ast::Stmt::AsyncFnDef {
                name,
                type_params,
                params,
                return_ty,
                body,
                decorators,
                ..
            } => {
                // Nested async function inside a function body — same as FnDef above.
                let fn_sym = self.define_local(name, self.checker.tcx.any());
                let is_decorated = !decorators.is_empty();
                // PEP 695: see the module-level FnDef arm — type-param names
                // must reach the param/return type lowering.
                let saved_tps = std::mem::replace(
                    &mut self.active_type_params,
                    type_params.iter().map(|p| p.name.clone()).collect(),
                );
                let lowered = if is_decorated {
                    self.lower_decorated_fn(name, params, return_ty, body, stmt.span)
                } else {
                    self.lower_fn(name, params, return_ty, body, stmt.span)
                };
                self.active_type_params = saved_tps;
                if let Some(mut func) = lowered {
                    if contains_yield(body) {
                        // Same async-generator routing as the top-level
                        // AsyncFnDef arm: route through the sync generator
                        // lowering so iteration works (cf. epic-py3-12 #850).
                        func.is_generator = true;
                    } else {
                        func.is_async = true;
                    }
                    func.decorators = decorators
                        .iter()
                        .filter_map(|d| self.lower_expr(d))
                        .collect();
                    for sym in &func.captures {
                        self.cell_override_syms.insert(*sym);
                    }
                    self.result.functions.push(func);
                }
                let _ = fn_sym;
                Some(HirStmt::FuncDefPlaceholder {
                    name: fn_sym,
                    span: stmt.span,
                })
            }
            _ => None,
        }
    }

    fn lower_elif_chain(
        &mut self,
        elifs: &[(Spanned<ast::Expr>, Vec<Spanned<ast::Stmt>>)],
        else_body: &Option<Vec<Spanned<ast::Stmt>>>,
    ) -> Vec<HirStmt> {
        if let Some((cond, body)) = elifs.first() {
            let hir_cond = match self.lower_expr(cond) {
                Some(c) => c,
                None => return Vec::new(),
            };
            let then_b: Vec<HirStmt> = body.iter().filter_map(|s| self.lower_stmt(s)).collect();
            let else_b = self.lower_elif_chain(&elifs[1..], else_body);
            vec![HirStmt::If {
                cond: hir_cond,
                then_body: then_b,
                else_body: else_b,
                span: cond.span,
            }]
        } else if let Some(eb) = else_body {
            eb.iter().filter_map(|s| self.lower_stmt(s)).collect()
        } else {
            Vec::new()
        }
    }

    /// Lower f-string parts, recursing into structured format specs so
    /// nested replacement fields ({value:{width}}) evaluate at runtime.
    fn lower_fstring_parts(&mut self, parts: &[ast::FStringPart]) -> Vec<HirFStringPart> {
        parts
            .iter()
            .filter_map(|p| match p {
                ast::FStringPart::Literal(s) => Some(HirFStringPart::Literal(s.clone())),
                ast::FStringPart::Expr(e, spec) => {
                    let he = self.lower_expr(e)?;
                    let hir_spec = spec.as_ref().map(|sp| self.lower_fstring_parts(sp));
                    Some(HirFStringPart::Expr(he, hir_spec))
                }
            })
            .collect()
    }

    fn lower_expr(&mut self, expr: &Spanned<ast::Expr>) -> Option<HirExpr> {
        match &expr.node {
            ast::Expr::IntLit(i) => Some(HirExpr::IntLit(*i, self.checker.tcx.int())),
            ast::Expr::FloatLit(f) => Some(HirExpr::FloatLit(*f, self.checker.tcx.float())),
            ast::Expr::ComplexLit(f) => {
                // Lower `Nj` to `complex(0.0, N)` so the value is an
                // `ObjData::Complex` at runtime, not a collapsed float.
                // (#1885 — without this, `print(2j)` prints `2.0`, and
                // `(1+2j).real`/`.imag` give wrong answers.)
                let any_ty = self.checker.tcx.any();
                let float_ty = self.checker.tcx.float();
                Some(HirExpr::Call {
                    func: Box::new(HirExpr::StrLit("mb_complex".to_string(), any_ty)),
                    args: vec![
                        HirExpr::FloatLit(0.0, float_ty),
                        HirExpr::FloatLit(*f, float_ty),
                    ],
                    ty: any_ty,
                })
            }
            ast::Expr::StrLit(s) => Some(HirExpr::StrLit(s.clone(), self.checker.tcx.str())),
            ast::Expr::BytesLit(bytes) => {
                let ty = self.checker.tcx.any();
                Some(HirExpr::BytesLit(bytes.clone(), ty))
            }
            ast::Expr::BoolLit(b) => Some(HirExpr::BoolLit(*b, self.checker.tcx.bool())),
            ast::Expr::NoneLit => Some(HirExpr::NoneLit(self.checker.tcx.none())),
            // `...` lowers as a read of the builtin `Ellipsis` symbol, which
            // hir_to_mir folds to MirConst::Ellipsis — same singleton as the
            // name `Ellipsis`, so `... is Ellipsis` holds.
            ast::Expr::Ellipsis => {
                let sym = self.resolve_name("Ellipsis", expr.span)?;
                Some(HirExpr::Var(sym, self.checker.tcx.any()))
            }
            ast::Expr::Ident(name) => {
                let sym = if let Some(id) = self.resolve_name(name, expr.span) {
                    id
                } else if let Some(&outer_id) = self.outer_scope_names.get(name.as_str()) {
                    // Implicit capture: inner function reads outer function's variable
                    // without an explicit `nonlocal` declaration (read-only capture).
                    // Register in local_names so future lookups find it directly,
                    // and mark as cell so the outer function stores it to global storage.
                    self.local_names.insert(name.to_string(), outer_id);
                    self.cell_override_syms.insert(outer_id);
                    outer_id
                } else {
                    return None;
                };
                let ty = self.get_type(sym);
                Some(HirExpr::Var(sym, ty))
            }
            ast::Expr::BinOp { op, lhs, rhs } => {
                let l = self.lower_expr(lhs)?;
                let r = self.lower_expr(rhs)?;
                let hir_op = lower_bin_op(*op)?;
                let ty = match hir_op {
                    // `is` / `is not` always use primitive identity comparison
                    // (icmp equal on raw bits), so result is always raw bool.
                    HirBinOp::Is | HirBinOp::IsNot => self.checker.tcx.bool(),
                    HirBinOp::Eq
                    | HirBinOp::NotEq
                    | HirBinOp::Lt
                    | HirBinOp::Gt
                    | HirBinOp::LtEq
                    | HirBinOp::GtEq
                    | HirBinOp::In
                    | HirBinOp::NotIn => {
                        // If operands are non-primitive, runtime dispatch returns
                        // NaN-boxed result → use Any to prevent double-boxing.
                        let lt = self.checker.tcx.get(l.ty());
                        let rt = self.checker.tcx.get(r.ty());
                        let needs_runtime = !matches!(
                            lt,
                            crate::types::Ty::Int
                                | crate::types::Ty::Float
                                | crate::types::Ty::Bool
                        ) || !matches!(
                            rt,
                            crate::types::Ty::Int
                                | crate::types::Ty::Float
                                | crate::types::Ty::Bool
                        );
                        if needs_runtime {
                            self.checker.tcx.any()
                        } else {
                            self.checker.tcx.bool()
                        }
                    }
                    _ => {
                        // Python and/or return operand values (already boxed in MIR),
                        // so result type is Any to prevent double-boxing.
                        if matches!(hir_op, HirBinOp::And | HirBinOp::Or) {
                            return Some(HirExpr::BinOp {
                                op: hir_op,
                                lhs: Box::new(l),
                                rhs: Box::new(r),
                                ty: self.checker.tcx.any(),
                            });
                        }
                        let lt = self.checker.tcx.get(l.ty());
                        let rt = self.checker.tcx.get(r.ty());
                        // Mixed numeric promotion: int+float or bool+float → Any (runtime dispatch)
                        let is_mixed = matches!(
                            (lt, rt),
                            (crate::types::Ty::Int, crate::types::Ty::Float)
                                | (crate::types::Ty::Float, crate::types::Ty::Int)
                                | (crate::types::Ty::Bool, crate::types::Ty::Float)
                                | (crate::types::Ty::Float, crate::types::Ty::Bool)
                        );
                        // Python true division always returns float
                        let is_true_div = matches!(hir_op, HirBinOp::Div)
                            && matches!(lt, crate::types::Ty::Int)
                            && matches!(rt, crate::types::Ty::Int);
                        // Bool subtype promotion: bool+bool or bool+int → Int
                        // Exception: bool & bool, bool | bool, bool ^ bool → Bool (Python)
                        let both_bool = matches!(lt, crate::types::Ty::Bool)
                            && matches!(rt, crate::types::Ty::Bool);
                        let is_bitwise_bool = both_bool
                            && matches!(
                                hir_op,
                                HirBinOp::BitAnd | HirBinOp::BitOr | HirBinOp::BitXor
                            );
                        let is_bool_promotion = !is_bitwise_bool
                            && matches!(
                                (lt, rt),
                                (crate::types::Ty::Bool, crate::types::Ty::Bool)
                                    | (crate::types::Ty::Bool, crate::types::Ty::Int)
                                    | (crate::types::Ty::Int, crate::types::Ty::Bool)
                            );
                        // Class operands → runtime dunder dispatch returns NaN-boxed Any.
                        let has_class = matches!(lt, crate::types::Ty::Class { .. })
                            || matches!(rt, crate::types::Ty::Class { .. });
                        // #2104: when either operand is Any (e.g. an unannotated
                        // for-loop element from `for x in [a, b, c]:`), the binop
                        // will be lowered through the runtime dispatcher and the
                        // result vreg holds a NaN-boxed MbValue — *not* a raw
                        // scalar in the lhs type. Mark the static result as Any
                        // so downstream consumers (print, int(), comparisons)
                        // unbox before touching the bits. Without this, an int /
                        // any binop is typed `int` but actually carries the
                        // IEEE-754 bit pattern of the float that mb_div returns,
                        // which surfaces as garbage bits (12500.0 prints as
                        // 4_668_097_562_002_063_360 etc.).
                        let has_any = matches!(lt, crate::types::Ty::Any)
                            || matches!(rt, crate::types::Ty::Any);
                        // Mirror `binop_to_runtime` in hir_to_mir.rs: every op
                        // that has a `mb_*` runtime entry boxes its result.
                        let runtime_dispatched = matches!(
                            hir_op,
                            HirBinOp::Add
                                | HirBinOp::Sub
                                | HirBinOp::Mul
                                | HirBinOp::Div
                                | HirBinOp::FloorDiv
                                | HirBinOp::Mod
                                | HirBinOp::Pow
                                | HirBinOp::Eq
                                | HirBinOp::NotEq
                                | HirBinOp::Lt
                                | HirBinOp::Gt
                                | HirBinOp::LtEq
                                | HirBinOp::GtEq
                                | HirBinOp::BitOr
                                | HirBinOp::BitAnd
                                | HirBinOp::BitXor
                        );
                        if is_mixed || is_true_div || has_class || (has_any && runtime_dispatched) {
                            self.checker.tcx.any()
                        } else if is_bitwise_bool {
                            self.checker.tcx.bool()
                        } else if is_bool_promotion {
                            self.checker.tcx.int()
                        } else {
                            l.ty()
                        }
                    }
                };
                Some(HirExpr::BinOp {
                    op: hir_op,
                    lhs: Box::new(l),
                    rhs: Box::new(r),
                    ty,
                })
            }
            ast::Expr::UnaryOp { op, operand } => {
                let inner = self.lower_expr(operand)?;
                let hir_op = lower_unary_op(*op)?;
                // Result type depends on operator:
                // - `not` always returns Bool
                // - `~` on Bool returns Int (Python: ~True == -2)
                // - `-`/`+` preserves operand type
                let ty = match hir_op {
                    HirUnaryOp::Not => self.checker.tcx.bool(),
                    HirUnaryOp::BitNot
                        if matches!(self.checker.tcx.get(inner.ty()), crate::types::Ty::Bool) =>
                    {
                        self.checker.tcx.int()
                    }
                    _ => inner.ty(),
                };
                Some(HirExpr::UnaryOp {
                    op: hir_op,
                    operand: Box::new(inner),
                    ty,
                })
            }
            ast::Expr::Call { func, args } => {
                let any_ty = self.checker.tcx.any();
                let str_ty = self.checker.tcx.str();
                // Static arg-binding validation: a bare-Ident call to a known
                // top-level function, with no *args/**kwargs splat, whose
                // argument shape unambiguously violates the callee's signature,
                // lowers to a runtime TypeError matching CPython (the would-be
                // call is replaced). Conservative: skips on any splat or unknown
                // callee, so a valid call is never rejected.
                if let ast::Expr::Ident(fname) = &func.node {
                    if let Some(sig) = self.arg_bind_sigs.get(fname).cloned() {
                        let splat_free = args.iter().all(|a| {
                            matches!(
                                a,
                                ast::CallArg::Positional(_) | ast::CallArg::Keyword { .. }
                            )
                        });
                        if splat_free {
                            let defaults_mutated =
                                self.funcs_with_mutated_defaults.contains(fname);
                            if let Some(msg) = arg_bind_violation(fname, &sig, args, defaults_mutated) {
                                return Some(HirExpr::Call {
                                    func: Box::new(HirExpr::StrLit(
                                        "mb_arg_bind_error".to_string(),
                                        any_ty,
                                    )),
                                    args: vec![HirExpr::StrLit(msg, str_ty)],
                                    ty: any_ty,
                                });
                            }
                        }
                    }
                }
                // all(x for x in it) / any(x for x in it) must preserve
                // generator-expression laziness so the builtin can short-circuit.
                if let ast::Expr::Ident(name) = &func.node {
                    if (name == "all" || name == "any") && args.len() == 1 {
                        if let ast::CallArg::Positional(arg) = &args[0] {
                            if let ast::Expr::GeneratorExpr {
                                element,
                                generators,
                            } = &arg.node
                            {
                                let saved = self.save_comp_scope(generators);
                                let gens = self.lower_comprehensions(generators);
                                let elem = self.lower_expr(element)?;
                                self.restore_comp_scope(saved);
                                return Some(HirExpr::AnyAllComp {
                                    is_all: name == "all",
                                    element: Box::new(elem),
                                    generators: gens,
                                    ty: self.checker.tcx.bool(),
                                });
                            }
                        }
                    }
                }
                // Special case: dict(a=1, b=2) with all-keyword args → HirExpr::Dict
                if let ast::Expr::Ident(name) = &func.node {
                    if name == "dict"
                        && !args.is_empty()
                        && args
                            .iter()
                            .all(|a| matches!(a, ast::CallArg::Keyword { .. }))
                    {
                        let entries: Vec<(HirExpr, HirExpr)> = args
                            .iter()
                            .filter_map(|a| {
                                if let ast::CallArg::Keyword { name: k, value } = a {
                                    let key = HirExpr::StrLit(k.clone(), str_ty);
                                    let val = self.lower_expr(value)?;
                                    Some((key, val))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        return Some(HirExpr::Dict {
                            entries,
                            ty: any_ty,
                        });
                    }
                }
                // Kwargs-aware builtin dispatch: when a known builtin is called
                // with keyword arguments, route to the kwargs variant that preserves
                // keyword semantics. Without this, keyword names are lost during
                // HIR lowering and the flattened positional args cause either wrong
                // behavior or Cranelift verifier errors.
                let has_kwargs = args
                    .iter()
                    .any(|a| matches!(a, ast::CallArg::Keyword { .. }));
                // `"...".format(*seq)` / `.format(**mapping)` carry no inline
                // Keyword arg, so `has_kwargs` is false and the call would fall
                // through to the generic mb_call_spread path — which has no
                // template-substitution semantics and drops the splatted args.
                // Route any `.format(...)` with a positional/double-star splat
                // into this block so the dedicated `attr == "format"` case below
                // builds the proper pos_list / kwargs_dict via mb_args_concat /
                // mb_dict_merge. (The Ident-builtin cases are gated on
                // `func.node == Ident`, and the sibling Attr cases match
                // `attr == "sort"/"dumps"`, so entering here for a format-splat
                // call only reaches the format case.)
                let is_format_splat = matches!(&func.node, ast::Expr::Attr { attr, .. } if attr == "format")
                    && args.iter().any(|a| {
                        matches!(a, ast::CallArg::StarArg(_) | ast::CallArg::DoubleStarArg(_))
                    });
                if let ast::Expr::Ident(name) = &func.node {
                    if name == "open"
                        && args.iter().any(|a| matches!(a, ast::CallArg::DoubleStarArg(_)))
                    {
                        let none_hir = HirExpr::NoneLit(any_ty);
                        let pos: Vec<HirExpr> = args.iter().filter_map(|a| {
                            if let ast::CallArg::Positional(e) = a { self.lower_expr(e) } else { None }
                        }).collect();
                        let path = pos.first().cloned().unwrap_or_else(|| none_hir.clone());
                        let mode = pos.get(1).cloned()
                            .unwrap_or_else(|| HirExpr::StrLit("r".to_string(), str_ty));
                        let encoding = pos.get(3).cloned().unwrap_or_else(|| none_hir.clone());
                        let errors = pos.get(4).cloned().unwrap_or_else(|| none_hir.clone());
                        let closefd = pos.get(6).cloned().unwrap_or_else(|| none_hir.clone());
                        let kwargs = self.build_kwargs_dict(args, any_ty).unwrap_or(HirExpr::Dict {
                            entries: vec![],
                            ty: any_ty,
                        });
                        return Some(HirExpr::Call {
                            func: Box::new(HirExpr::StrLit("mb_open_kwargs".to_string(), any_ty)),
                            args: vec![path, mode, encoding, errors, closefd, kwargs],
                            ty: any_ty,
                        });
                    }
                }
                if has_kwargs || is_format_splat {
                    if let ast::Expr::Ident(name) = &func.node {
                        let none_hir = HirExpr::NoneLit(any_ty);

                        // print(*args, sep=' ', end='\n') → mb_print_kwargs(args_list, sep, end)
                        if name == "print" {
                            let hir_pos: Vec<HirExpr> = args
                                .iter()
                                .filter_map(|a| {
                                    if let ast::CallArg::Positional(e) = a {
                                        self.lower_expr(e)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            let args_list = HirExpr::List {
                                elements: hir_pos,
                                ty: any_ty,
                            };
                            let sep = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "sep" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            let end = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "end" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            let file = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "file" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            return Some(HirExpr::Call {
                                func: Box::new(HirExpr::StrLit(
                                    "mb_print_kwargs_file".to_string(),
                                    any_ty,
                                )),
                                args: vec![args_list, sep, end, file],
                                ty: any_ty,
                            });
                        }
                        // sorted(iterable, key=None, reverse=False) → mb_sorted_kwargs
                        if name == "sorted" {
                            let pos: Vec<HirExpr> = args
                                .iter()
                                .filter_map(|a| {
                                    if let ast::CallArg::Positional(e) = a {
                                        self.lower_expr(e)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            let iterable =
                                pos.into_iter().next().unwrap_or_else(|| none_hir.clone());
                            let key = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "key" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            let reverse = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "reverse" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            return Some(HirExpr::Call {
                                func: Box::new(HirExpr::StrLit(
                                    "mb_sorted_kwargs".to_string(),
                                    any_ty,
                                )),
                                args: vec![iterable, key, reverse],
                                ty: any_ty,
                            });
                        }
                        // min(iterable, key=None, default=None) → mb_min_kwargs
                        if name == "min" {
                            let pos: Vec<HirExpr> = args
                                .iter()
                                .filter_map(|a| {
                                    if let ast::CallArg::Positional(e) = a {
                                        self.lower_expr(e)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            let iterable = if pos.len() >= 2 {
                                HirExpr::List {
                                    elements: pos,
                                    ty: any_ty,
                                }
                            } else {
                                pos.into_iter().next().unwrap_or_else(|| none_hir.clone())
                            };
                            let key = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "key" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            let default = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "default" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            return Some(HirExpr::Call {
                                func: Box::new(HirExpr::StrLit(
                                    "mb_min_kwargs".to_string(),
                                    any_ty,
                                )),
                                args: vec![iterable, key, default],
                                ty: any_ty,
                            });
                        }
                        // max(iterable, key=None, default=None) → mb_max_kwargs
                        if name == "max" {
                            let pos: Vec<HirExpr> = args
                                .iter()
                                .filter_map(|a| {
                                    if let ast::CallArg::Positional(e) = a {
                                        self.lower_expr(e)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            let iterable = if pos.len() >= 2 {
                                HirExpr::List {
                                    elements: pos,
                                    ty: any_ty,
                                }
                            } else {
                                pos.into_iter().next().unwrap_or_else(|| none_hir.clone())
                            };
                            let key = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "key" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            let default = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "default" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            return Some(HirExpr::Call {
                                func: Box::new(HirExpr::StrLit(
                                    "mb_max_kwargs".to_string(),
                                    any_ty,
                                )),
                                args: vec![iterable, key, default],
                                ty: any_ty,
                            });
                        }
                        // zip(*iterables, strict=False) → mb_zip_strict(list, strict).
                        // Without this, the `strict` kwarg silently appends as a
                        // positional bool, blowing up Cranelift signature
                        // verification at the `mb_zip(arg, arg, arg)` call site.
                        if name == "zip" {
                            let pos: Vec<HirExpr> = args
                                .iter()
                                .filter_map(|a| {
                                    if let ast::CallArg::Positional(e) = a {
                                        self.lower_expr(e)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            let iterables = HirExpr::List {
                                elements: pos,
                                ty: any_ty,
                            };
                            let strict = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "strict" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| {
                                    HirExpr::BoolLit(false, self.checker.tcx.bool())
                                });
                            return Some(HirExpr::Call {
                                func: Box::new(HirExpr::StrLit(
                                    "mb_zip_strict".to_string(),
                                    any_ty,
                                )),
                                args: vec![iterables, strict],
                                ty: any_ty,
                            });
                        }
                        // sum(iterable, start=0) → mb_sum_with_start
                        if name == "sum" {
                            let has_start = args.iter().any(|a| {
                                matches!(a, ast::CallArg::Keyword { name: n, .. } if n == "start")
                            });
                            if has_start {
                                let pos: Vec<HirExpr> = args
                                    .iter()
                                    .filter_map(|a| {
                                        if let ast::CallArg::Positional(e) = a {
                                            self.lower_expr(e)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                let iterable =
                                    pos.into_iter().next().unwrap_or_else(|| none_hir.clone());
                                let start = args
                                    .iter()
                                    .find_map(|a| {
                                        if let ast::CallArg::Keyword { name: n, value } = a {
                                            if n == "start" {
                                                return self.lower_expr(value);
                                            }
                                        }
                                        None
                                    })
                                    .unwrap_or_else(|| none_hir.clone());
                                return Some(HirExpr::Call {
                                    func: Box::new(HirExpr::StrLit(
                                        "mb_sum_with_start".to_string(),
                                        any_ty,
                                    )),
                                    args: vec![iterable, start],
                                    ty: any_ty,
                                });
                            }
                        }
                        // open(file, mode='r', buffering=-1, encoding=None, ...) →
                        // mb_open_ex(path, mode, encoding, errors). Pull named
                        // kwargs explicitly so the generic path does not flatten
                        // keyword values positionally (e.g. misrouting
                        // `open(p, encoding='utf-8')` as mode='utf-8').
                        if name == "open" {
                            let pos: Vec<HirExpr> = args
                                .iter()
                                .filter_map(|a| {
                                    if let ast::CallArg::Positional(e) = a {
                                        self.lower_expr(e)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            let mut pos_iter = pos.into_iter();
                            let path = pos_iter.next().unwrap_or_else(|| none_hir.clone());
                            let mode = pos_iter
                                .next()
                                .or_else(|| {
                                    args.iter().find_map(|a| {
                                        if let ast::CallArg::Keyword { name: n, value } = a {
                                            if n == "mode" {
                                                return self.lower_expr(value);
                                            }
                                        }
                                        None
                                    })
                                })
                                .unwrap_or_else(|| HirExpr::StrLit("r".to_string(), str_ty));
                            drop(pos_iter);
                            // open(file, mode, buffering, encoding, errors, ...):
                            // `encoding=` is positional 3, `errors=` positional 4.
                            // Record both so `f.encoding` / `f.errors` reflect them.
                            let mut kw = |key: &str, pos_idx: usize| -> HirExpr {
                                args.iter()
                                    .enumerate()
                                    .find_map(|(i, a)| match a {
                                        ast::CallArg::Positional(e) if i == pos_idx => {
                                            self.lower_expr(e)
                                        }
                                        ast::CallArg::Keyword { name: n, value } if n == key => {
                                            self.lower_expr(value)
                                        }
                                        _ => None,
                                    })
                                    .unwrap_or_else(|| none_hir.clone())
                            };
                            let encoding = kw("encoding", 3);
                            let errors = kw("errors", 4);
                            // closefd is positional 6 (after newline at 5); None
                            // means the default True (no borrowed-fd guard).
                            let closefd = kw("closefd", 6);
                            let opener = kw("opener", 7);
                            if !matches!(opener, HirExpr::NoneLit(_)) {
                                return Some(HirExpr::Call {
                                    func: Box::new(HirExpr::StrLit("mb_open_with_opener".to_string(), any_ty)),
                                    args: vec![path, mode, encoding, errors, closefd, opener],
                                    ty: any_ty,
                                });
                            }
                            return Some(HirExpr::Call {
                                func: Box::new(HirExpr::StrLit("mb_open_ex".to_string(), any_ty)),
                                args: vec![path, mode, encoding, errors, closefd],
                                ty: any_ty,
                            });
                        }
                    }
                    // Method calls with kwargs: x.method(kwargs)
                    if let ast::Expr::Attr { object, attr } = &func.node {
                        let none_hir = HirExpr::NoneLit(any_ty);
                        // .sort(key=f, reverse=r) → mb_list_sort_kwargs(list, key, reverse)
                        if attr == "sort" {
                            let recv = self.lower_expr(object)?;
                            let key = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "key" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            let reverse = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "reverse" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            return Some(HirExpr::Call {
                                func: Box::new(HirExpr::StrLit(
                                    "mb_list_sort_kwargs".to_string(),
                                    any_ty,
                                )),
                                args: vec![recv, key, reverse],
                                ty: any_ty,
                            });
                        }
                        // json.dumps(obj, sort_keys=..., indent=..., separators=...):
                        // pass kwargs as a trailing dict so dispatch_dumps can read them.
                        // Without this, keyword names are dropped and sort_keys/separators
                        // become unreachable.
                        if attr == "dumps" {
                            if let ast::Expr::Ident(obj_name) = &object.node {
                                if obj_name == "json" {
                                    let val = args
                                        .iter()
                                        .find_map(|a| {
                                            if let ast::CallArg::Positional(e) = a {
                                                self.lower_expr(e)
                                            } else {
                                                None
                                            }
                                        })
                                        .unwrap_or_else(|| HirExpr::NoneLit(any_ty));
                                    let kwargs_entries: Vec<(HirExpr, HirExpr)> = args
                                        .iter()
                                        .filter_map(|a| {
                                            if let ast::CallArg::Keyword { name, value } = a {
                                                let key = HirExpr::StrLit(
                                                    name.clone(),
                                                    self.checker.tcx.str(),
                                                );
                                                let v = self.lower_expr(value)?;
                                                Some((key, v))
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                    if !kwargs_entries.is_empty() {
                                        let kwargs_dict = HirExpr::Dict {
                                            entries: kwargs_entries,
                                            ty: any_ty,
                                        };
                                        let f = self.lower_expr(func)?;
                                        return Some(HirExpr::Call {
                                            func: Box::new(f),
                                            args: vec![val, kwargs_dict],
                                            ty: any_ty,
                                        });
                                    }
                                }
                            }
                        }
                        // .format(*seq, name=x, **mapping) →
                        //   mb_str_format_kwargs(str, pos_args_list, kwargs_dict)
                        //
                        // Positional `*seq` splats and `**mapping` double-star
                        // splats must be honored: a `*seq` extends the positional
                        // list via mb_args_concat, and a `**mapping` merges into
                        // the kwargs dict via mb_dict_merge. Dropping either left
                        // `"{0}".format(*xs)` and `"{k}".format(**d)` producing an
                        // empty args set (positional → None, keyword → literal).
                        if attr == "format" {
                            let recv = self.lower_expr(object)?;
                            // Build the positional list, interleaving inline
                            // positionals into a running List literal and folding
                            // each `*seq` through mb_args_concat in source order.
                            let mut pos_acc: HirExpr = HirExpr::List {
                                elements: Vec::new(),
                                ty: any_ty,
                            };
                            let mut pending: Vec<HirExpr> = Vec::new();
                            let flush = |pending: &mut Vec<HirExpr>, acc: &mut HirExpr, any_ty| {
                                if pending.is_empty() {
                                    return;
                                }
                                let chunk = HirExpr::List {
                                    elements: std::mem::take(pending),
                                    ty: any_ty,
                                };
                                *acc = HirExpr::Call {
                                    func: Box::new(HirExpr::StrLit(
                                        "mb_args_concat".to_string(),
                                        any_ty,
                                    )),
                                    args: vec![
                                        std::mem::replace(acc, HirExpr::NoneLit(any_ty)),
                                        chunk,
                                    ],
                                    ty: any_ty,
                                };
                            };
                            for a in args {
                                match a {
                                    ast::CallArg::Positional(e) => {
                                        if let Some(he) = self.lower_expr(e) {
                                            pending.push(he);
                                        }
                                    }
                                    ast::CallArg::StarArg(e) => {
                                        flush(&mut pending, &mut pos_acc, any_ty);
                                        if let Some(he) = self.lower_expr(e) {
                                            pos_acc = HirExpr::Call {
                                                func: Box::new(HirExpr::StrLit(
                                                    "mb_args_concat".to_string(),
                                                    any_ty,
                                                )),
                                                args: vec![pos_acc, he],
                                                ty: any_ty,
                                            };
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            flush(&mut pending, &mut pos_acc, any_ty);
                            let pos_list = pos_acc;
                            // Build the kwargs dict, folding each `**mapping`
                            // through mb_dict_merge. CPython raises TypeError on a
                            // duplicate keyword key across splats; mamba currently
                            // last-wins instead (narrow strictness gap, shared with
                            // the general **kwargs path — not introduced here).
                            let kwargs_entries: Vec<(HirExpr, HirExpr)> = args
                                .iter()
                                .filter_map(|a| {
                                    if let ast::CallArg::Keyword { name, value } = a {
                                        let key = HirExpr::StrLit(name.clone(), str_ty);
                                        let val = self.lower_expr(value)?;
                                        Some((key, val))
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            let mut kwargs_dict = HirExpr::Dict {
                                entries: kwargs_entries,
                                ty: any_ty,
                            };
                            for a in args {
                                if let ast::CallArg::DoubleStarArg(e) = a {
                                    if let Some(he) = self.lower_expr(e) {
                                        kwargs_dict = HirExpr::Call {
                                            func: Box::new(HirExpr::StrLit(
                                                "mb_dict_merge".to_string(),
                                                any_ty,
                                            )),
                                            args: vec![kwargs_dict, he],
                                            ty: any_ty,
                                        };
                                    }
                                }
                            }
                            return Some(HirExpr::Call {
                                func: Box::new(HirExpr::StrLit(
                                    "mb_str_format_kwargs".to_string(),
                                    any_ty,
                                )),
                                args: vec![recv, pos_list, kwargs_dict],
                                ty: any_ty,
                            });
                        }
                    }
                }
                let f = self.lower_expr(func)?;
                // Check if any argument is a StarArg (splat: f(*args)).
                // If so, lower to mb_call_spread(func, args_list) where args_list
                // collects all pre-star positional args and the starred iterable.
                let has_star = args.iter().any(|a| matches!(a, ast::CallArg::StarArg(_)));
                if has_star {
                    // Build a combined args list: [pre_star..., *star_items..., post_star...]
                    // We pack all positional args into a list and use mb_call_spread.
                    // For `f(*lst)` with only a star arg, the list IS the spread list.
                    // For `f(a, *lst, b)`, we build a combined [a] + lst + [b] list.
                    // Build ordered segments preserving SOURCE ORDER:
                    //   f(a, *xs, c, *ys)  →  [a] ++ xs ++ [c] ++ ys
                    // Consecutive positionals coalesce into one List segment; each
                    // *star is its own iterable segment. The prior code collected
                    // ALL positionals into a single leading slab, so a positional
                    // AFTER a star (`f(*xs, c)`) was mis-ordered to the front
                    // (`[c] ++ xs`). Keyword args in a `*args` call (e.g.
                    // `heapq.merge(*streams, key=fn)`) must NOT be flattened into
                    // the positional spread — they are collected separately and
                    // appended as a trailing kwargs dict the native dispatcher
                    // recovers by convention.
                    let mut segments: Vec<HirExpr> = Vec::new();
                    let mut pending: Vec<HirExpr> = Vec::new();
                    let mut kw_entries: Vec<(HirExpr, HirExpr)> = Vec::new();
                    for a in args {
                        match a {
                            ast::CallArg::Positional(e) => {
                                if let Some(he) = self.lower_expr(e) {
                                    pending.push(he);
                                }
                            }
                            ast::CallArg::Keyword { name, value } => {
                                if let Some(he) = self.lower_expr(value) {
                                    let key = HirExpr::StrLit(name.clone(), self.checker.tcx.str());
                                    kw_entries.push((key, he));
                                }
                            }
                            ast::CallArg::StarArg(e) => {
                                if !pending.is_empty() {
                                    segments.push(HirExpr::List {
                                        elements: std::mem::take(&mut pending), ty: any_ty,
                                    });
                                }
                                if let Some(he) = self.lower_expr(e) {
                                    segments.push(he);
                                }
                            }
                            ast::CallArg::DoubleStarArg(_) => {}
                        }
                    }
                    if !pending.is_empty() {
                        segments.push(HirExpr::List {
                            elements: std::mem::take(&mut pending),
                            ty: any_ty,
                        });
                    }
                    // Fold segments left-to-right with mb_args_concat, which
                    // REQUIRES its prefix (first arg) to be a materialized List —
                    // a non-list prefix is treated as empty and its items lost. So
                    // when the first segment is a bare starred iterable (`f(*xs)`),
                    // seed the fold from an empty list; a List-valued first segment
                    // is used directly. A lone segment (`f(*xs)`) passes straight
                    // through to mb_call_spread.
                    let mut spread_list = if segments.len() == 1 {
                        segments.remove(0)
                    } else if segments.is_empty() {
                        HirExpr::List {
                            elements: Vec::new(),
                            ty: any_ty,
                        }
                    } else {
                        let mut seg_iter = segments.into_iter();
                        let first = seg_iter.next().unwrap();
                        let mut acc = if matches!(first, HirExpr::List { .. }) {
                            first
                        } else {
                            HirExpr::Call {
                                func: Box::new(HirExpr::StrLit(
                                    "mb_args_concat".to_string(),
                                    any_ty,
                                )),
                                args: vec![
                                    HirExpr::List {
                                        elements: Vec::new(),
                                        ty: any_ty,
                                    },
                                    first,
                                ],
                                ty: any_ty,
                            }
                        };
                        for seg in seg_iter {
                            acc = HirExpr::Call {
                                func: Box::new(HirExpr::StrLit(
                                    "mb_args_concat".to_string(),
                                    any_ty,
                                )),
                                args: vec![acc, seg],
                                ty: any_ty,
                            };
                        }
                        acc
                    };
                    // `recv.f(*args, key=fn)` — append the keyword bundle as a
                    // trailing dict positional so native dispatchers
                    // (heapq.merge/nlargest/nsmallest, etc.) recover it via the
                    // trailing-kwargs-dict convention, exactly as the non-splat
                    // kwargs path does. The keyword name would otherwise be lost
                    // in the spread. Scoped to attribute/method calls so the
                    // bare-Ident builtins above (print/zip/min/max/sum) — which
                    // consume their own kwargs and reuse `spread_list` for the
                    // positional slab — are unaffected.
                    let is_attr_call = matches!(func.node, ast::Expr::Attr { .. });
                    if is_attr_call && !kw_entries.is_empty() {
                        let kwargs_dict = HirExpr::Dict {
                            entries: kw_entries,
                            ty: any_ty,
                        };
                        spread_list = HirExpr::Call {
                            func: Box::new(HirExpr::StrLit("mb_args_concat".to_string(), any_ty)),
                            args: vec![
                                spread_list,
                                HirExpr::List {
                                    elements: vec![kwargs_dict],
                                    ty: any_ty,
                                },
                            ],
                            ty: any_ty,
                        };
                    }
                    // Variadic-shaped builtins are exposed as binary helpers
                    // (mb_zip, mb_print, mb_min, mb_max, mb_sum) and rely on a
                    // direct-call shortcut in hir_to_mir to fan out >2 args.
                    // mb_call_spread bypasses that shortcut and falls through
                    // to the JIT-cc fallback which drops args beyond the
                    // declared arity, so route these names directly to the
                    // variadic helper that takes a packed list.
                    if let ast::Expr::Ident(name) = &func.node {
                        let direct = match name.as_str() {
                            "zip" => Some("mb_zip_n"),
                            "min" => Some("mb_min"),
                            "max" => Some("mb_max"),
                            "sum" => Some("mb_sum"),
                            _ => None,
                        };
                        if let Some(rt_name) = direct {
                            return Some(HirExpr::Call {
                                func: Box::new(HirExpr::StrLit(rt_name.to_string(), any_ty)),
                                args: vec![spread_list],
                                ty: any_ty,
                            });
                        }
                        // print(*args, sep=..., end=...) → mb_print_kwargs(list, sep, end).
                        // print has kwargs in addition to *args; reuse the kwargs surface.
                        if name == "print" {
                            let none_hir = HirExpr::NoneLit(any_ty);
                            let sep = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "sep" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            let end = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "end" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            let file = args
                                .iter()
                                .find_map(|a| {
                                    if let ast::CallArg::Keyword { name: n, value } = a {
                                        if n == "file" {
                                            return self.lower_expr(value);
                                        }
                                    }
                                    None
                                })
                                .unwrap_or_else(|| none_hir.clone());
                            return Some(HirExpr::Call {
                                func: Box::new(HirExpr::StrLit(
                                    "mb_print_kwargs_file".to_string(),
                                    any_ty,
                                )),
                                args: vec![spread_list, sep, end, file],
                                ty: any_ty,
                            });
                        }
                    }
                    // A bare-Ident `*`-splat call that ALSO carries a `**mapping`
                    // (`collect(0, *[1,2], k=9, **{"m":8})`) must bind its keywords
                    // — the generic mb_call_spread below drops them. Route through
                    // mb_call_spread_kwargs with the already-expanded positional
                    // spread list plus the merged kwargs dict. Scoped to calls with
                    // a `**` splat (explicit-kwargs-only `*`-splat calls keep their
                    // existing handling); the builtin idents (zip/min/max/sum/print)
                    // returned earlier.
                    if matches!(func.node, ast::Expr::Ident(_))
                        && args.iter().any(|a| matches!(a, ast::CallArg::DoubleStarArg(_)))
                    {
                        if let Some(kwargs_dict) = self.build_kwargs_dict(args, any_ty) {
                            return Some(HirExpr::Call {
                                func: Box::new(HirExpr::StrLit(
                                    "mb_call_spread_kwargs".to_string(), any_ty,
                                )),
                                args: vec![f, spread_list, kwargs_dict],
                                ty: any_ty,
                            });
                        }
                    }
                    return Some(HirExpr::Call {
                        func: Box::new(HirExpr::StrLit("mb_call_spread".to_string(), any_ty)),
                        args: vec![f, spread_list],
                        ty: any_ty,
                    });
                }
                // Generic kwargs + default resolution: if the callee is a known
                // user function and the call has keyword args OR fewer positional
                // args than params (defaults should fill the gap), reorder and pad.
                let has_kwargs = args
                    .iter()
                    .any(|a| matches!(a, ast::CallArg::Keyword { .. }));
                let pos_count = args
                    .iter()
                    .filter(|a| matches!(a, ast::CallArg::Positional(_)))
                    .count();
                let needs_default_fill = {
                    let fname = if let ast::Expr::Ident(ref n) = func.node {
                        Some(n.clone())
                    } else {
                        None
                    };
                    fname
                        .as_ref()
                        .and_then(|n| self.func_param_info.get(n))
                        .map(|info| {
                            let regular_count = info
                                .iter()
                                .filter(|(_, _, k)| *k == ast::ParamKind::Regular)
                                .count();
                            pos_count < regular_count && info.iter().any(|(_, d, _)| d.is_some())
                        })
                        .unwrap_or(false)
                };
                // If the callee is a known variadic (*args) or **kwargs function, always
                // go through the full resolution path so MIR lowering sees a properly
                // constructed kwargs dict sentinel (otherwise a positional arg gets
                // mistaken for the kwargs dict at the variadic-packing step).
                let callee_is_variadic = {
                    let fname = if let ast::Expr::Ident(ref n) = func.node {
                        Some(n.clone())
                    } else {
                        None
                    };
                    fname
                        .as_ref()
                        .and_then(|n| self.func_param_info.get(n))
                        .map(|info| {
                            info.iter().any(|(_, _, k)| {
                                *k == ast::ParamKind::Star || *k == ast::ParamKind::DoubleStar
                            })
                        })
                        .unwrap_or(false)
                };
                if has_kwargs || needs_default_fill || callee_is_variadic {
                    let func_name = if let ast::Expr::Ident(ref n) = func.node {
                        Some(n.clone())
                    } else {
                        None
                    };
                    if let Some(ref fname) = func_name {
                        if let Some(param_info) = self.func_param_info.get(fname).cloned() {
                            // A `**mapping` splat has dynamic keys the static
                            // reorder below cannot bind (it only matches literal
                            // keyword names; the splat would be dropped). Route the
                            // whole call through mb_call_spread_kwargs, which binds
                            // the kwargs dict — including to a `**kwargs` receiver —
                            // at runtime.
                            if args.iter().any(|a| matches!(a, ast::CallArg::DoubleStarArg(_))) {
                                return Some(self.build_spread_kwargs_call(f, args, any_ty));
                            }
                            // Separate regular params from *args/**kwargs for ordered resolution.
                            let regular_params: Vec<(
                                usize,
                                &(String, Option<Spanned<ast::Expr>>, ast::ParamKind),
                            )> = param_info
                                .iter()
                                .enumerate()
                                .filter(|(_, (_, _, k))| *k == ast::ParamKind::Regular)
                                .collect();
                            let has_star = param_info
                                .iter()
                                .any(|(_, _, k)| *k == ast::ParamKind::Star);
                            let has_dstar = param_info
                                .iter()
                                .any(|(_, _, k)| *k == ast::ParamKind::DoubleStar);
                            // Positional-only param names: a keyword of the same
                            // name does NOT bind the param (CPython) — with
                            // **kwargs it lands in the dict. pos_only is only in
                            // arg_bind_sigs (top-level undecorated defs); empty
                            // elsewhere, preserving prior behavior.
                            let posonly_names: std::collections::HashSet<String> = if has_dstar {
                                self.arg_bind_sigs
                                    .get(fname)
                                    .map(|sig| {
                                        sig.iter().filter(|p| p.5).map(|p| p.0.clone()).collect()
                                    })
                                    .unwrap_or_default()
                            } else {
                                std::collections::HashSet::new()
                            };
                            let mut ordered: Vec<Option<HirExpr>> =
                                vec![None; regular_params.len()];
                            let mut excess_pos: Vec<HirExpr> = Vec::new();
                            let mut excess_kw: Vec<(String, HirExpr)> = Vec::new();
                            let mut pos_idx = 0;
                            for a in args {
                                match a {
                                    ast::CallArg::Positional(e) => {
                                        if pos_idx < regular_params.len() {
                                            ordered[pos_idx] = self.lower_expr(e);
                                            pos_idx += 1;
                                        } else if has_star {
                                            if let Some(expr) = self.lower_expr(e) {
                                                excess_pos.push(expr);
                                            }
                                        }
                                    }
                                    ast::CallArg::Keyword { name: kw, value } => {
                                        // Try to match keyword arg to a regular param name —
                                        // but never a positional-only param (it cannot be
                                        // filled by keyword; the keyword belongs in **kwargs).
                                        let matched = if posonly_names.contains(kw) {
                                            None
                                        } else {
                                            regular_params.iter().position(|(_, (n, _, _))| n == kw)
                                        };
                                        if let Some(idx) = matched {
                                            ordered[idx] = self.lower_expr(value);
                                        } else if has_dstar {
                                            // Unmatched (or positional-only) kwargs go to the
                                            // **kwargs dict.
                                            if let Some(expr) = self.lower_expr(value) {
                                                excess_kw.push((kw.clone(), expr));
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            // Fill defaults for missing regular args.
                            for (i, (orig_idx, _)) in regular_params.iter().enumerate() {
                                if ordered[i].is_none() {
                                    let (_, ref default, _) = param_info[*orig_idx];
                                    if let Some(ref def_expr) = default {
                                        ordered[i] = self.lower_expr(def_expr);
                                    }
                                }
                            }
                            // Build final HirExpr args: regular_args... [, *args_list] [, **kwargs_dict]
                            // Note: excess positional/keyword args are NOT appended here.
                            // They are handled at MIR lowering time via variadic packing.
                            // However, we pass them as trailing HirExpr elements so the MIR
                            // lowering can recognize them.
                            let mut hir_args: Vec<HirExpr> =
                                ordered.into_iter().flatten().collect();
                            // For *args: append excess positional args (MIR will pack them)
                            hir_args.extend(excess_pos);
                            // For **kwargs: if present, skip here — handled at MIR level
                            // We store excess kwargs as a dict literal in the call args.
                            if has_dstar && !excess_kw.is_empty() {
                                let dict_entries: Vec<(HirExpr, HirExpr)> = excess_kw
                                    .into_iter()
                                    .map(|(k, v)| {
                                        let key = HirExpr::StrLit(k, self.checker.tcx.str());
                                        (key, v)
                                    })
                                    .collect();
                                let dict = HirExpr::Dict {
                                    entries: dict_entries,
                                    ty: any_ty,
                                };
                                hir_args.push(dict);
                            } else if has_dstar {
                                // No excess kwargs — pass an empty dict
                                let dict = HirExpr::Dict {
                                    entries: vec![],
                                    ty: any_ty,
                                };
                                hir_args.push(dict);
                            }
                            return Some(HirExpr::Call {
                                func: Box::new(f),
                                args: hir_args,
                                ty: any_ty,
                            });
                        }
                    }
                }
                // Method calls with kwargs: `d.update(b=2, c=3)` — pack the
                // kwargs into a dict literal appended to the positional args
                // so mb_call_method's runtime dispatcher can unpack them.
                let is_method_call = matches!(func.node, ast::Expr::Attr { .. });
                let has_any_kwargs = args
                    .iter()
                    .any(|a| matches!(a, ast::CallArg::Keyword { .. }));
                let has_dstar = args
                    .iter()
                    .any(|a| matches!(a, ast::CallArg::DoubleStarArg(_)));
                // Bare-Ident calls to native/stdlib constructors use the SAME
                // trailing-kwargs-dict convention as method calls, so
                // `Counter(a=3)` / `deque(maxlen=2)` keep their keyword names
                // instead of flattening to positionals. Scoped to a known
                // native allowlist: user functions reached through dynamic
                // dispatch (mb_call0/mb_call1_val/mb_call_spread) bind args
                // positionally and would misread an appended dict — closing
                // that gap needs runtime kwargs binding for JIT-compiled
                // functions, tracked separately. (User functions/classes that
                // ARE statically known were already handled by the
                // func_param_info reorder path above and never reach here.)
                let is_native_kwargs_ident = matches!(
                    &func.node,
                    ast::Expr::Ident(name) if matches!(
                        name.as_str(),
                        "Counter" | "OrderedDict" | "deque" | "defaultdict" | "dict"
                            // namedtuple takes rename= / defaults= / module=;
                            // typing.NamedTuple's keyword field form / mixed-form
                            // TypeError check reads a trailing kwargs dict.
                            | "namedtuple" | "NamedTuple"
                            // UserDict seeds its payload from kwargs
                            | "UserDict"
                            // unittest.mock factories take config kwargs
                            // (return_value= / side_effect= / spec=).
                            | "MagicMock" | "Mock" | "AsyncMock" | "PropertyMock"
                            | "NonCallableMock" | "NonCallableMagicMock"
                            | "patch" | "mock_open" | "call"
                            // operator.methodcaller(name, *args, **kwargs)
                            // stores call-time kwargs for the later method call.
                            | "methodcaller"
                            // urllib.parse functions with behavioral kwargs
                            | "parse_qs" | "parse_qsl" | "urlencode"
                            // quote/unquote take safe= / encoding= / errors=;
                            // their native dispatchers read a trailing kwargs
                            // dict, so a bare from-import must keep the keyword
                            // names rather than flattening them to positional.
                            | "quote" | "unquote"
                            // property(fget=, fset=, fdel=, doc=) keyword form
                            | "property"
                            // decimal.localcontext(prec=, rounding=, Emin=, …)
                            // and Context(...) read a trailing kwargs dict.
                            | "localcontext" | "Context"
                            // urllib.request.Request(url, data=, headers=,
                            // method=) — its native dispatcher reads a trailing
                            // kwargs dict, so a bare `Request(...)` (from-import)
                            // must keep the keyword names instead of flattening.
                            | "Request"
                            // builtins.open(file, mode=..., encoding=..., closefd=...)
                            // reads a trailing kwargs dict in its native dispatcher.
                            | "open"
                            // xml.etree.ElementTree constructors use **extra
                            // keyword attributes as element attrib entries.
                            | "Element" | "SubElement"
                            // xml.etree.ElementTree.indent reads space=/level=
                            // from the trailing kwargs dict.
                            | "indent"
                    ) || self.dataclasses_kwarg_idents.contains(name.as_str())
                );
                let is_type_metaclass_kwargs = matches!(
                    &func.node,
                    ast::Expr::Ident(name) if name == "type"
                ) && (has_dstar
                    || args.iter().any(|a| matches!(
                        a,
                        ast::CallArg::Keyword { name, .. } if name == "metaclass"
                    )));
                let is_simple_namespace_subclass_kwargs = matches!(
                    &func.node,
                    ast::Expr::Ident(name)
                        if self.simple_namespace_subclass_idents.contains(name.as_str())
                );
                let pack_trailing_kwargs = (is_method_call && (has_any_kwargs || has_dstar))
                    || (is_native_kwargs_ident && (has_any_kwargs || has_dstar))
                    || (is_type_metaclass_kwargs && (has_any_kwargs || has_dstar))
                    || (is_simple_namespace_subclass_kwargs && (has_any_kwargs || has_dstar));

                let is_functools_partial_kwarg_ident = matches!(
                    &func.node,
                    ast::Expr::Ident(name)
                        if self.functools_partial_kwarg_idents.contains(name.as_str())
                );
                // `p(k=1)` where `p` is a local `functools.partial(...)`
                // instance must keep keyword names alive so call-time kwargs
                // can override stored kwargs. Do not generalize this to every
                // bare identifier: builtin constructors and imported classes
                // still rely on the legacy flattening/trailing-dict conventions.
                if (has_any_kwargs || has_dstar) && !pack_trailing_kwargs && !has_star
                    && is_functools_partial_kwarg_ident
                {
                    return Some(self.build_spread_kwargs_call(f, args, any_ty));
                }

                let hir_args: Vec<HirExpr> = if pack_trailing_kwargs {
                    let mut out: Vec<HirExpr> = Vec::new();
                    // Trailing kwargs accumulator: runs of explicit keywords
                    // become Dict literals; each `**mapping` splat is folded in
                    // source order through mb_dict_merge (later wins), instead
                    // of leaking into the positional slab where the dispatcher
                    // would misbind it as a leading argument.
                    let mut kw_acc: Option<HirExpr> = None;
                    let mut pending: Vec<(HirExpr, HirExpr)> = Vec::new();
                    fn merge_dicts(acc: HirExpr, next: HirExpr, any_ty: TypeId) -> HirExpr {
                        HirExpr::Call {
                            func: Box::new(HirExpr::StrLit("mb_dict_merge".to_string(), any_ty)),
                            args: vec![acc, next],
                            ty: any_ty,
                        }
                    }
                    fn flush_pending(
                        pending: &mut Vec<(HirExpr, HirExpr)>,
                        kw_acc: &mut Option<HirExpr>,
                        any_ty: TypeId,
                    ) {
                        if pending.is_empty() {
                            return;
                        }
                        let chunk = HirExpr::Dict {
                            entries: std::mem::take(pending),
                            ty: any_ty,
                        };
                        *kw_acc = Some(match kw_acc.take() {
                            None => chunk,
                            Some(prev) => merge_dicts(prev, chunk, any_ty),
                        });
                    }
                    for a in args {
                        match a {
                            ast::CallArg::Positional(e) => {
                                if let Some(he) = self.lower_expr(e) {
                                    out.push(he);
                                }
                            }
                            ast::CallArg::Keyword { name, value } => {
                                if let Some(he) = self.lower_expr(value) {
                                    let key = HirExpr::StrLit(name.clone(), self.checker.tcx.str());
                                    pending.push((key, he));
                                }
                            }
                            ast::CallArg::DoubleStarArg(e) => {
                                if let Some(he) = self.lower_expr(e) {
                                    flush_pending(&mut pending, &mut kw_acc, any_ty);
                                    kw_acc = Some(match kw_acc.take() {
                                        // Lone splat: pass the mapping through
                                        // unchanged (same shape the fallback
                                        // path produced for `dict(**m)`).
                                        None => he,
                                        Some(prev) => merge_dicts(prev, he, any_ty),
                                    });
                                }
                            }
                            // StarArg is unreachable here (the has_star branch
                            // above returns early); keep the bare push as a
                            // safety net.
                            ast::CallArg::StarArg(e) => {
                                if let Some(he) = self.lower_expr(e) {
                                    out.push(he);
                                }
                            }
                        }
                    }
                    flush_pending(&mut pending, &mut kw_acc, any_ty);
                    out.push(kw_acc.unwrap_or(HirExpr::Dict {
                        entries: vec![],
                        ty: any_ty,
                    }));
                    out
                } else {
                    args.iter()
                        .filter_map(|a| match a {
                            ast::CallArg::Positional(e) => self.lower_expr(e),
                            ast::CallArg::Keyword { value, .. } => self.lower_expr(value),
                            ast::CallArg::StarArg(e) => self.lower_expr(e),
                            ast::CallArg::DoubleStarArg(e) => self.lower_expr(e),
                        })
                        .collect()
                };
                // Use the declared return type only for user-defined functions so callers
                // don't incorrectly unbox the raw primitive they return (#827).
                // Builtin/extern functions (int(), float(), getattr(), …) return NaN-boxed
                // MbValues even when the type checker declares them as primitives, so we
                // keep any_ty for them to avoid Cranelift type mismatches.
                let ty = if let HirExpr::Var(sym, _) = &f {
                    // Use the HIR function's actual return_ty (which is consistent with
                    // the MIR body's return type) rather than the type checker's Fn type
                    // (which defaults to any_ty for unannotated functions). This ensures
                    // callers box the return value correctly when printing (#scope_modifiers).
                    // Check func_return_tys first — it's pre-populated before each body
                    // is lowered, so recursive self-calls find their own return type
                    // even though the HirFunction isn't pushed to result.functions until
                    // lowering finishes (fire 50: closes factorial's CheckedMul fast-path
                    // gap noted in fire 49).
                    let hir_return_ty = self.func_return_tys.get(sym).copied().or_else(|| {
                        self.result
                            .functions
                            .iter()
                            .find(|hf| hf.name == *sym)
                            .map(|hf| hf.return_ty)
                    });
                    if let Some(ret_ty) = hir_return_ty {
                        ret_ty
                    } else {
                        self.checker.tcx.any()
                    }
                } else {
                    self.checker.tcx.any()
                };
                // Method call carrying keyword args: route to mb_call_method_kwargs
                // so the runtime binds the keywords to the method's parameters by
                // name (mb_call_method only takes positional args). `hir_args` is
                // [pos..., kwargs_dict]; split it into a positional list + the dict.
                if is_method_call && pack_trailing_kwargs {
                    if let HirExpr::Attr { object, attr, .. } = f {
                        let mut parts = hir_args;
                        let kwargs_dict = parts.pop().unwrap_or(HirExpr::Dict {
                            entries: vec![],
                            ty: any_ty,
                        });
                        let pos_list = HirExpr::List {
                            elements: parts,
                            ty: any_ty,
                        };
                        let name_str = HirExpr::StrLit(attr, self.checker.tcx.str());
                        return Some(HirExpr::Call {
                            func: Box::new(HirExpr::StrLit(
                                "mb_call_method_kwargs".to_string(),
                                any_ty,
                            )),
                            args: vec![*object, name_str, pos_list, kwargs_dict],
                            ty: any_ty,
                        });
                    }
                }
                Some(HirExpr::Call {
                    func: Box::new(f),
                    args: hir_args,
                    ty,
                })
            }
            ast::Expr::Attr { object, attr } => {
                let obj = self.lower_expr(object)?;
                let ty = self.checker.tcx.any();
                Some(HirExpr::Attr {
                    object: Box::new(obj),
                    attr: attr.clone(),
                    ty,
                })
            }
            ast::Expr::Index { object, index } => {
                let obj = self.lower_expr(object)?;
                let idx = self.lower_expr(index)?;
                let ty = self.checker.tcx.any();
                Some(HirExpr::Index {
                    object: Box::new(obj),
                    index: Box::new(idx),
                    ty,
                })
            }
            ast::Expr::ListLit(elems) => {
                if elems
                    .iter()
                    .any(|e| matches!(e.node, ast::Expr::Starred(_)))
                {
                    return self.lower_starred_display(elems, None);
                }
                let hir_elems: Vec<HirExpr> =
                    elems.iter().filter_map(|e| self.lower_expr(e)).collect();
                // Infer element type from the first element to enable typed match patterns.
                // A typed list (e.g. list[int]) lets sequence patterns propagate the capture
                // type so nested bindings get the right primitive type (#827).
                // We use `find` (non-mutating) since the checker already interned this type.
                let list_ty = if let Some(first) = hir_elems.first() {
                    let elem_ty = first.ty();
                    // Only type the list if the element type is concrete (not any/error)
                    if elem_ty != self.checker.tcx.any() && elem_ty != self.checker.tcx.error() {
                        self.checker
                            .tcx
                            .find(&crate::types::Ty::List(elem_ty))
                            .unwrap_or(self.checker.tcx.any())
                    } else {
                        self.checker.tcx.any()
                    }
                } else {
                    self.checker.tcx.any()
                };
                Some(HirExpr::List {
                    elements: hir_elems,
                    ty: list_ty,
                })
            }
            ast::Expr::TupleLit(elems) => {
                if elems
                    .iter()
                    .any(|e| matches!(e.node, ast::Expr::Starred(_)))
                {
                    return self.lower_starred_display(elems, Some("mb_tuple_from_iterable"));
                }
                let hir_elems: Vec<HirExpr> =
                    elems.iter().filter_map(|e| self.lower_expr(e)).collect();
                // Build a proper Tuple type from element types so that sequence patterns
                // on tuple subjects can propagate per-slot types (#827).
                // The type checker has already interned Ty::Tuple([...]) during its pass,
                // so find() here is non-mutating and safe.
                let elem_tys: Vec<crate::types::TypeId> =
                    hir_elems.iter().map(|e| e.ty()).collect();
                let tup_ty = if !elem_tys.is_empty() {
                    self.checker
                        .tcx
                        .find(&crate::types::Ty::Tuple(elem_tys))
                        .unwrap_or_else(|| self.checker.tcx.any())
                } else {
                    self.checker.tcx.any()
                };
                Some(HirExpr::Tuple {
                    elements: hir_elems,
                    ty: tup_ty,
                })
            }
            ast::Expr::DictLit(entries) => {
                // For unpack entries `{**d, ...}`, lower the value (k=None → unpack expr).
                // We represent them as (unpack_sentinel_expr, value) where sentinel is `None`
                // lowered as a no-key marker.  For now, we flatten unpacks using `any` key.
                let hir_entries: Vec<(HirExpr, HirExpr)> = entries
                    .iter()
                    .filter_map(|(k, v)| {
                        let hir_val = self.lower_expr(v)?;
                        let hir_key = if let Some(key) = k {
                            self.lower_expr(key)?
                        } else {
                            // Dict unpack `**expr` — represent as a NoneLit sentinel key
                            // tagged with the `Never` type so it is distinguishable from a
                            // user-written `None` literal key (which carries `NoneType`).
                            // Downstream codegen recognizes `NoneLit(Never)` as the unpack
                            // sentinel; a real `None` key is preserved as a normal entry.
                            let ty = self.checker.tcx.never();
                            HirExpr::NoneLit(ty)
                        };
                        Some((hir_key, hir_val))
                    })
                    .collect();
                let dict_ty = self.checker.tcx.any();
                Some(HirExpr::Dict {
                    entries: hir_entries,
                    ty: dict_ty,
                })
            }
            ast::Expr::IfExpr {
                body,
                condition,
                else_body,
            } => {
                let then_val = self.lower_expr(body)?;
                let cond = self.lower_expr(condition)?;
                let else_val = self.lower_expr(else_body)?;
                let ty = then_val.ty();
                Some(HirExpr::IfExpr {
                    cond: Box::new(cond),
                    then_val: Box::new(then_val),
                    else_val: Box::new(else_val),
                    ty,
                })
            }
            ast::Expr::Lambda { params, body } => {
                let any_ty = self.checker.tcx.any();
                // Default-arg expressions are lowered in the *enclosing* scope
                // (Python semantics: evaluated at function-creation time, not
                // call time). We must lower them BEFORE binding the lambda
                // params into local_names, or a default like `x=i` would
                // incorrectly shadow the outer `i`.
                let hir_defaults: Vec<Option<Box<HirExpr>>> = params
                    .iter()
                    .map(|p| {
                        p.default
                            .as_ref()
                            .and_then(|d| self.lower_expr(d))
                            .map(Box::new)
                    })
                    .collect();
                // CPython inspect.Parameter kind ordinals (mirrors the `def`
                // signature lowering): 0 POSITIONAL_ONLY, 1 POSITIONAL_OR_KEYWORD,
                // 2 VAR_POSITIONAL, 3 KEYWORD_ONLY, 4 VAR_KEYWORD. Threaded so a
                // lambda's __code__.co_posonlyargcount / co_kwonlyargcount report
                // the `/` and `*` markers.
                let hir_param_kinds: Vec<u8> = params
                    .iter()
                    .map(|p| match p.kind {
                        ast::ParamKind::Star => 2u8,
                        ast::ParamKind::DoubleStar => 4u8,
                        ast::ParamKind::Regular if p.kw_only => 3u8,
                        ast::ParamKind::Regular if p.pos_only => 0u8,
                        ast::ParamKind::Regular => 1u8,
                    })
                    .collect();
                // Temporarily register lambda params in local_names so resolve_name()
                // finds them when lowering the body.  Type-checker scopes are already
                // popped at lowering time, so we inject the params manually and then
                // restore the previous mapping afterwards.
                let mut saved: Vec<(String, Option<SymbolId>)> = Vec::new();
                let hir_params: Vec<(SymbolId, TypeId)> = params
                    .iter()
                    .map(|p| {
                        let old = self.local_names.get(&p.name).copied();
                        saved.push((p.name.clone(), old));
                        // Always allocate a fresh ID so lambda params never alias outer vars.
                        let pid = SymbolId(self.next_local_sym);
                        self.next_local_sym += 1;
                        self.local_names.insert(p.name.clone(), pid);
                        self.local_types.insert(pid, any_ty);
                        // Record the real name in sym_names so introspection
                        // (inspect.signature, arity-error messages) reports `x`/`y`
                        // rather than the `arg0`/`arg1` placeholder — local_names is
                        // restored below, dropping the temporary binding otherwise.
                        self.result
                            .sym_names
                            .entry(pid)
                            .or_insert_with(|| p.name.clone());
                        (pid, any_ty)
                    })
                    .collect();

                let body_result = self.lower_expr(body);

                // Restore local_names to pre-lambda state.
                for (name, old) in saved {
                    match old {
                        Some(id) => {
                            self.local_names.insert(name, id);
                        }
                        None => {
                            self.local_names.remove(&name);
                        }
                    }
                }

                let body_expr = body_result?;
                let ty = any_ty;
                Some(HirExpr::Lambda {
                    params: hir_params,
                    param_kinds: hir_param_kinds,
                    defaults: hir_defaults,
                    body: Box::new(body_expr),
                    ty,
                    span: expr.span,
                })
            }
            ast::Expr::Slice { start, stop, step } => {
                let s = start
                    .as_ref()
                    .and_then(|e| self.lower_expr(e))
                    .map(Box::new);
                let e = stop.as_ref().and_then(|e| self.lower_expr(e)).map(Box::new);
                let st = step.as_ref().and_then(|e| self.lower_expr(e)).map(Box::new);
                let ty = self.checker.tcx.any();
                Some(HirExpr::Slice {
                    start: s,
                    stop: e,
                    step: st,
                    ty,
                })
            }
            ast::Expr::Yield(value) => {
                let v = value
                    .as_ref()
                    .and_then(|e| self.lower_expr(e))
                    .map(Box::new);
                let ty = self.checker.tcx.any();
                Some(HirExpr::Yield { value: v, ty })
            }
            ast::Expr::YieldFrom(iter) => {
                let it = self.lower_expr(iter)?;
                let ty = self.checker.tcx.any();
                Some(HirExpr::YieldFrom {
                    iter: Box::new(it),
                    ty,
                })
            }
            ast::Expr::Await(value) => {
                let v = self.lower_expr(value)?;
                let ty = self.checker.tcx.any();
                Some(HirExpr::Await {
                    value: Box::new(v),
                    ty,
                })
            }
            ast::Expr::ListComp {
                element,
                generators,
            } => {
                // P0-R5: Save outer names that comprehension variables will shadow.
                // Comprehension loop variables must not leak into the enclosing scope.
                let saved = self.save_comp_scope(generators);
                let gens = self.lower_comprehensions(generators);
                let elem = self.lower_expr(element)?;
                let ty = self.checker.tcx.any();
                self.restore_comp_scope(saved);
                Some(HirExpr::ListComp {
                    element: Box::new(elem),
                    generators: gens,
                    ty,
                })
            }
            ast::Expr::SetComp {
                element,
                generators,
            } => {
                let saved = self.save_comp_scope(generators);
                let gens = self.lower_comprehensions(generators);
                let elem = self.lower_expr(element)?;
                let ty = self.checker.tcx.any();
                self.restore_comp_scope(saved);
                Some(HirExpr::SetComp {
                    element: Box::new(elem),
                    generators: gens,
                    ty,
                })
            }
            ast::Expr::GeneratorExpr {
                element,
                generators,
            } => {
                // Desugar generator expression to an ITERATOR over an eager list
                // comprehension. A genexpr is a single-use iterator in CPython, so
                // `next(genexpr)` must work and the value must not be reusable/
                // subscriptable/len-able like a plain list. Wrapping the materialized
                // ListComp in mb_iter gives it iterator identity while keeping the
                // eager element evaluation. (Full lazy state-machine codegen deferred.)
                let saved = self.save_comp_scope(generators);
                let gens = self.lower_comprehensions(generators);
                let elem = self.lower_expr(element)?;
                let ty = self.checker.tcx.any();
                self.restore_comp_scope(saved);
                let list = HirExpr::ListComp {
                    element: Box::new(elem),
                    generators: gens,
                    ty,
                };
                Some(HirExpr::Call {
                    func: Box::new(HirExpr::StrLit("mb_iter".to_string(), ty)),
                    args: vec![list],
                    ty,
                })
            }
            ast::Expr::DictComp {
                key,
                value,
                generators,
            } => {
                let saved = self.save_comp_scope(generators);
                let gens = self.lower_comprehensions(generators);
                let k = self.lower_expr(key)?;
                let v = self.lower_expr(value)?;
                let ty = self.checker.tcx.any();
                self.restore_comp_scope(saved);
                Some(HirExpr::DictComp {
                    key: Box::new(k),
                    value: Box::new(v),
                    generators: gens,
                    ty,
                })
            }
            ast::Expr::FString(parts) => {
                let hir_parts = self.lower_fstring_parts(parts);
                let ty = self.checker.tcx.str();
                Some(HirExpr::FString {
                    parts: hir_parts,
                    ty,
                })
            }
            ast::Expr::SetLit(elems) => {
                if elems
                    .iter()
                    .any(|e| matches!(e.node, ast::Expr::Starred(_)))
                {
                    return self.lower_starred_display(elems, Some("mb_set_from_iterable"));
                }
                let hir_elems: Vec<HirExpr> =
                    elems.iter().filter_map(|e| self.lower_expr(e)).collect();
                let ty = self.checker.tcx.any();
                Some(HirExpr::Set {
                    elements: hir_elems,
                    ty,
                })
            }
            ast::Expr::Walrus { target, value } => {
                let val_expr = self.lower_expr(value)?;
                let ty = val_expr.ty();
                // Python scoping for walrus: the target is local to the
                // enclosing function (unless declared global/nonlocal). If
                // we're inside a function body and the name appears in the
                // function's assigned-names set without a global/nonlocal
                // declaration, define a fresh local regardless of whether
                // resolve_name finds an outer-scope symbol — otherwise the
                // walrus would rebind a module-scope variable of the same
                // name, matching the function-local-shadowing bug fixed for
                // regular assignment.
                let is_function_local_target =
                    self.local_assigned_names.iter().any(|n| n == target)
                        && !self.local_declared_names.iter().any(|n| n == target)
                        && !self.local_names.contains_key(target);
                let sym = if is_function_local_target {
                    let id = SymbolId(self.next_local_sym);
                    self.next_local_sym += 1;
                    self.local_names.insert(target.to_string(), id);
                    self.local_types.insert(id, ty);
                    id
                } else if let Some(id) = self.resolve_name(target, expr.span) {
                    id
                } else if let Some(&outer_id) = self.outer_scope_names.get(target.as_str()) {
                    self.local_names.insert(target.to_string(), outer_id);
                    outer_id
                } else {
                    let id = SymbolId(self.next_local_sym);
                    self.next_local_sym += 1;
                    self.local_names.insert(target.to_string(), id);
                    id
                };
                Some(HirExpr::Walrus {
                    target: sym,
                    value: Box::new(val_expr),
                    ty,
                })
            }
            ast::Expr::ChainedCompare { operands, ops } => {
                // Desugar `a < b < c` into `(a < tmp_b) and (tmp_b < c)`.
                // Middle operands are stored in temp variables to evaluate once.
                let bool_ty = self.checker.tcx.bool();
                let any_ty = self.checker.tcx.any();

                // Lower all operands; for middle ones, wrap in Walrus to create a temp.
                let n = operands.len();
                let mut lowered: Vec<HirExpr> = Vec::with_capacity(n);
                // Temp symbols for middle operands (indices 1..n-1).
                let mut temp_vars: Vec<Option<SymbolId>> = Vec::with_capacity(n);

                for (i, operand) in operands.iter().enumerate() {
                    let val = self.lower_expr(operand)?;
                    if i > 0 && i < n - 1 {
                        // Middle operand: create temp variable
                        let ty = val.ty();
                        let tmp_name = format!("__chain_tmp_{}", self.next_local_sym);
                        let sym = self.define_local(&tmp_name, ty);
                        // Wrap in Walrus so the value is assigned and the var is available
                        let walrus = HirExpr::Walrus {
                            target: sym,
                            value: Box::new(val),
                            ty,
                        };
                        lowered.push(walrus);
                        temp_vars.push(Some(sym));
                    } else {
                        lowered.push(val);
                        temp_vars.push(None);
                    }
                }

                // Build comparison pairs: ops[i] compares lowered[i] with lowered[i+1].
                // For middle operands that were walrus-wrapped, the LHS of the next
                // comparison uses a Var reference to the temp.
                let mut comparisons: Vec<HirExpr> = Vec::with_capacity(ops.len());
                for i in 0..ops.len() {
                    let hir_op = lower_bin_op(ops[i])?;
                    // LHS of comparison i: for i==0, use lowered[0]; for i>0, use the temp var of operand[i]
                    let lhs_expr = if i == 0 {
                        lowered[0].clone()
                    } else {
                        // Middle operand i was stored in a temp; use Var reference
                        let sym = temp_vars[i].unwrap();
                        let ty = self.get_type(sym);
                        HirExpr::Var(sym, ty)
                    };
                    // RHS: lowered[i+1] (may be a Walrus for middle operands, or plain for the last)
                    let rhs_expr = lowered[i + 1].clone();

                    // Determine result type for this comparison
                    let lt = self.checker.tcx.get(lhs_expr.ty());
                    let rt = self.checker.tcx.get(rhs_expr.ty());
                    let needs_runtime = !matches!(
                        lt,
                        crate::types::Ty::Int | crate::types::Ty::Float | crate::types::Ty::Bool
                    ) || !matches!(
                        rt,
                        crate::types::Ty::Int | crate::types::Ty::Float | crate::types::Ty::Bool
                    );
                    let cmp_ty = if needs_runtime { any_ty } else { bool_ty };

                    comparisons.push(HirExpr::BinOp {
                        op: hir_op,
                        lhs: Box::new(lhs_expr),
                        rhs: Box::new(rhs_expr),
                        ty: cmp_ty,
                    });
                }

                // Chain comparisons with And: comp[0] and comp[1] and ...
                // And/Or result type must be Any since the MIR lowering returns
                // already-boxed MbValues (short-circuit semantics).
                let mut result = comparisons.remove(0);
                for cmp in comparisons {
                    result = HirExpr::BinOp {
                        op: HirBinOp::And,
                        lhs: Box::new(result),
                        rhs: Box::new(cmp),
                        ty: any_ty,
                    };
                }
                Some(result)
            }
            _ => None,
        }
    }

    fn lower_lvalue(&mut self, expr: &Spanned<ast::Expr>) -> Option<HirLValue> {
        match &expr.node {
            ast::Expr::Ident(name) => {
                // Mirror the single-ident `Assign` path: if the name is unbound
                // but appears on the LHS of an assignment elsewhere in this
                // function (i.e. it's in `local_assigned_names`), or if it's
                // unbound full stop, define a local now. Without this, an
                // unpack target like `a, b = …` silently propagates `None`
                // through `?` and the entire statement gets dropped from HIR
                // — the bug that made tuple-unpack assignments inside
                // generator bodies vanish (`a, b = 0, 1; yield a` yielded
                // `None` because the assign never lowered).
                if let Some(id) = self.resolve_name(name, expr.span) {
                    return Some(HirLValue::Var(id));
                }
                let any_ty = self.checker.tcx.any();
                let sym = self.define_local(name, any_ty);
                Some(HirLValue::Var(sym))
            }
            ast::Expr::Attr { object, attr } => {
                let obj = self.lower_expr(object)?;
                Some(HirLValue::Attr {
                    object: Box::new(obj),
                    attr: attr.clone(),
                })
            }
            ast::Expr::Index { object, index } => {
                let obj = self.lower_expr(object)?;
                let idx = self.lower_expr(index)?;
                Some(HirLValue::Index {
                    object: Box::new(obj),
                    index: Box::new(idx),
                })
            }
            // Tuple/unpack targets: `a, b = ...` or `a, *rest, b = ...`
            ast::Expr::TupleLit(elems) | ast::Expr::UnpackTarget(elems) => {
                let mut targets = Vec::new();
                let mut star_index = None;
                for (i, elem) in elems.iter().enumerate() {
                    if let ast::Expr::Starred(inner) = &elem.node {
                        // CPython 3.12: at most one starred target is allowed in
                        // a single unpack target. A second `*` raises
                        // `SyntaxError: multiple starred expressions in assignment`
                        // (Python/compile.c). Report against the offending
                        // starred element so the diagnostic points at the second
                        // `*` rather than the whole tuple.
                        if star_index.is_some() {
                            self.errors.push(MambaError::syntax(
                                elem.span,
                                "multiple starred expressions in assignment",
                            ));
                            return None;
                        }
                        star_index = Some(i);
                        targets.push(self.lower_lvalue(inner)?);
                    } else {
                        targets.push(self.lower_lvalue(elem)?);
                    }
                }
                Some(HirLValue::Unpack {
                    targets,
                    star_index,
                })
            }
            _ => {
                self.errors
                    .push(MambaError::syntax(expr.span, "invalid assignment target"));
                None
            }
        }
    }

    /// Lower a sequence display that contains starred elements —
    /// `[a, *b, c]`, `(a, *b)`, `{a, *b}`. Plain elements accumulate into
    /// list segments; each segment and each unpacked iterable (materialized
    /// via mb_list_from_iterable) concatenates with `+` (mb_add), preserving
    /// order. `wrap` names the runtime conversion for tuple/set displays;
    /// None keeps the flat list.
    fn lower_starred_display(
        &mut self,
        elems: &[Spanned<ast::Expr>],
        wrap: Option<&'static str>,
    ) -> Option<HirExpr> {
        let any_ty = self.checker.tcx.any();
        let concat = |acc: Option<HirExpr>, rhs: HirExpr| -> HirExpr {
            match acc {
                None => rhs,
                Some(lhs) => HirExpr::BinOp {
                    op: HirBinOp::Add,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    ty: any_ty,
                },
            }
        };
        let mut acc: Option<HirExpr> = None;
        let mut pending: Vec<HirExpr> = Vec::new();
        for e in elems {
            if let ast::Expr::Starred(inner) = &e.node {
                if !pending.is_empty() {
                    let seg = HirExpr::List {
                        elements: std::mem::take(&mut pending),
                        ty: any_ty,
                    };
                    acc = Some(concat(acc, seg));
                }
                let star = HirExpr::Call {
                    func: Box::new(HirExpr::StrLit("mb_list_from_iterable".to_string(), any_ty)),
                    args: vec![self.lower_expr(inner)?],
                    ty: any_ty,
                };
                acc = Some(concat(acc, star));
            } else {
                pending.push(self.lower_expr(e)?);
            }
        }
        if !pending.is_empty() {
            let seg = HirExpr::List {
                elements: pending,
                ty: any_ty,
            };
            acc = Some(concat(acc, seg));
        }
        let flat = acc.unwrap_or(HirExpr::List {
            elements: Vec::new(),
            ty: any_ty,
        });
        Some(match wrap {
            Some(extern_name) => HirExpr::Call {
                func: Box::new(HirExpr::StrLit(extern_name.to_string(), any_ty)),
                args: vec![flat],
                ty: any_ty,
            },
            None => flat,
        })
    }

    /// P0-R5: Save outer `local_names` entries for comprehension variable names.
    /// Returns saved entries so `restore_comp_scope` can undo the shadowing.
    fn save_comp_scope(&self, gens: &[ast::Comprehension]) -> Vec<(String, Option<SymbolId>)> {
        gens.iter()
            .map(|g| {
                let name = g.targets.first().cloned().unwrap_or_default();
                let saved = self.local_names.get(&name).copied();
                (name, saved)
            })
            .collect()
    }

    /// P0-R5: Restore outer `local_names` after comprehension lowering so loop
    /// variables do not leak into the enclosing scope.
    fn restore_comp_scope(&mut self, saved: Vec<(String, Option<SymbolId>)>) {
        for (name, old_sym) in saved {
            match old_sym {
                Some(id) => {
                    self.local_names.insert(name, id);
                }
                None => {
                    self.local_names.remove(&name);
                }
            }
        }
    }

    fn lower_comprehensions(&mut self, gens: &[ast::Comprehension]) -> Vec<HirComprehension> {
        gens.iter()
            .filter_map(|g| {
                // Define all loop variables. For tuple targets `for k, v in ...`,
                // the first goes in `var` and the rest in `extra_vars` — the hir_to_mir
                // lowering unpacks the iterator's next value into these via tuple indexing.
                let var = self.define_local(
                    g.targets.first().map(String::as_str).unwrap_or("_"),
                    self.checker.tcx.any(),
                );
                let extra_vars: Vec<SymbolId> = g
                    .targets
                    .iter()
                    .skip(1)
                    .map(|name| self.define_local(name, self.checker.tcx.any()))
                    .collect();
                let iter = self.lower_expr(&g.iter)?;
                let conditions: Vec<HirExpr> = g
                    .conditions
                    .iter()
                    .filter_map(|c| self.lower_expr(c))
                    .collect();
                Some(HirComprehension {
                    var,
                    extra_vars,
                    unpack_target: g.unpack_target,
                    iter,
                    conditions,
                    is_async: g.is_async,
                })
            })
            .collect()
    }

    /// Lower an AST pattern to HIR pattern (#309).
    fn lower_pattern(&mut self, pat: &Spanned<ast::Pattern>) -> Option<HirPattern> {
        Some(match &pat.node {
            ast::Pattern::Wildcard => HirPattern::Wildcard,
            ast::Pattern::Binding(name) => {
                // Use subject type for the capture binding so primitive ops are emitted (#827).
                let ty = self
                    .current_match_subject_ty
                    .unwrap_or_else(|| self.checker.tcx.any());
                let sym = self.define_local(name, ty);
                HirPattern::Capture(sym)
            }
            ast::Pattern::Literal(expr) => {
                let spanned = Spanned {
                    node: expr.clone(),
                    span: pat.span,
                };
                HirPattern::Literal(self.lower_expr(&spanned)?)
            }
            ast::Pattern::Or(pats) => {
                // Collect binding names before lowering so we can merge types afterward.
                let binding_names = collect_ast_pattern_bindings(&pats[0]);
                // Track per-name types from each alternative (#827 OR-pattern merge).
                let mut per_name_first: HashMap<String, TypeId> = HashMap::new();
                let mut per_name_consistent: HashMap<String, bool> = HashMap::new();
                let mut hir_pats: Vec<HirPattern> = Vec::new();
                for p in pats {
                    // lower_pattern takes &mut self, so collect names AFTER each call.
                    if let Some(hp) = self.lower_pattern(p) {
                        hir_pats.push(hp);
                    }
                    // Read binding types from local scope after this alternative.
                    for name in &binding_names {
                        if let Some(&sym) = self.local_names.get(name.as_str()) {
                            if let Some(&ty) = self.local_types.get(&sym) {
                                let first_ty = *per_name_first.entry(name.clone()).or_insert(ty);
                                let consistent = first_ty == ty;
                                per_name_consistent
                                    .entry(name.clone())
                                    .and_modify(|c| *c = *c && consistent)
                                    .or_insert(consistent);
                            }
                        }
                    }
                }
                // For captures with inconsistent types across alternatives, fall back to
                // Any so MIR lowering doesn't attempt type-specific unboxing (#827).
                let any_ty = self.checker.tcx.any();
                for name in &binding_names {
                    let consistent = per_name_consistent.get(name).copied().unwrap_or(true);
                    if !consistent {
                        self.define_local(name, any_ty);
                    }
                }
                HirPattern::Or(hir_pats)
            }
            ast::Pattern::Sequence(pats) => {
                // Derive per-position element type from the current match subject (#827).
                // Cloning to avoid borrow conflicts when calling tcx.any() inside the loop.
                let subj = self.current_match_subject_ty;
                let subj_ty_clone = subj.map(|id| self.checker.tcx.get(id).clone());
                let mut hir_pats = Vec::new();
                for (i, p) in pats.iter().enumerate() {
                    let elem_ty = match &subj_ty_clone {
                        Some(crate::types::Ty::List(inner)) => *inner,
                        Some(crate::types::Ty::Tuple(ts)) => {
                            if i < ts.len() {
                                ts[i]
                            } else {
                                self.checker.tcx.any()
                            }
                        }
                        _ => self.checker.tcx.any(),
                    };
                    let saved = self.current_match_subject_ty;
                    self.current_match_subject_ty = Some(elem_ty);
                    if let Some(hp) = self.lower_pattern(p) {
                        hir_pats.push(hp);
                    }
                    self.current_match_subject_ty = saved;
                }
                HirPattern::Sequence(hir_pats)
            }
            ast::Pattern::ClassPattern { cls, patterns } => {
                let class_name_str = cls.last()?.clone(); // use last segment for dotted paths
                let class_sym = self.resolve_name(&class_name_str, pat.span)?;
                let args: Vec<(String, HirPattern)> = patterns
                    .iter()
                    .enumerate()
                    .filter_map(|(i, (name, p))| {
                        let hp = self.lower_pattern(p)?;
                        let field = name.clone().unwrap_or_else(|| format!("_{i}"));
                        Some((field, hp))
                    })
                    .collect();
                HirPattern::Class {
                    class: class_sym,
                    class_name: class_name_str,
                    args,
                }
            }
            ast::Pattern::Constructor { path, fields } => {
                // PEP 634: a dotted name WITHOUT parentheses is a VALUE
                // pattern (`case Suit.HEARTS:` compares subject == Suit.HEARTS),
                // never a class pattern. The parser emits these as Constructor
                // with empty fields; rebuild the attribute chain and lower as
                // a Literal equality test. Single-segment bare constructors
                // (`case int:`) keep the legacy class-pattern shape.
                if fields.is_empty() && path.len() >= 2 {
                    let mut expr = ast::Expr::Ident(path[0].clone());
                    for seg in &path[1..] {
                        expr = ast::Expr::Attr {
                            object: Box::new(Spanned {
                                node: expr,
                                span: pat.span,
                            }),
                            attr: seg.clone(),
                        };
                    }
                    let spanned = Spanned {
                        node: expr,
                        span: pat.span,
                    };
                    return Some(HirPattern::Literal(self.lower_expr(&spanned)?));
                }
                let class_name_str = path.last()?.clone(); // use last segment for dotted paths
                let class_sym = self.resolve_name(&class_name_str, pat.span)?;
                let args: Vec<(String, HirPattern)> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let sym = self.define_local(f, self.checker.tcx.any());
                        (format!("_{i}"), HirPattern::Capture(sym))
                    })
                    .collect();
                HirPattern::Class {
                    class: class_sym,
                    class_name: class_name_str,
                    args,
                }
            }
            ast::Pattern::Star(name) => {
                if let Some(n) = name {
                    // Star capture: use subject type directly (list element capture is Any-typed
                    // in the lowerer since we can't call intern on immutable checker; type
                    // checker already sets list[subject_ty] via check_pattern).
                    let sym = self.define_local(n, self.checker.tcx.any());
                    HirPattern::Star(Some(sym))
                } else {
                    HirPattern::Star(None)
                }
            }
            ast::Pattern::Mapping { pairs, rest } => {
                // Derive the value type from the current match subject (#827).
                let val_ty = if let Some(subj) = self.current_match_subject_ty {
                    match self.checker.tcx.get(subj).clone() {
                        crate::types::Ty::Dict(_, v) => v,
                        _ => self.checker.tcx.any(),
                    }
                } else {
                    self.checker.tcx.any()
                };
                // Lower each key-value pair: key expr + sub-pattern (#827)
                let mut hir_pairs = Vec::new();
                for (key, val_pat) in pairs {
                    let saved = self.current_match_subject_ty;
                    self.current_match_subject_ty = Some(val_ty);
                    let result = (|| -> Option<(HirExpr, HirPattern)> {
                        let hir_key = self.lower_expr(key)?;
                        let hir_pat = self.lower_pattern(val_pat)?;
                        Some((hir_key, hir_pat))
                    })();
                    self.current_match_subject_ty = saved;
                    if let Some(pair) = result {
                        hir_pairs.push(pair);
                    }
                }
                // Lower optional rest capture: `**rest` binds remaining entries (#827)
                let hir_rest = rest.as_ref().map(|n| {
                    let rest_ty = self
                        .current_match_subject_ty
                        .unwrap_or_else(|| self.checker.tcx.any());
                    self.define_local(n, rest_ty)
                });
                HirPattern::Mapping {
                    pairs: hir_pairs,
                    rest: hir_rest,
                }
            }
            ast::Pattern::As { pattern, name } => {
                // Lower the inner pattern, then bind the whole match to `name` (#827).
                // Use subject type for non-class AS bindings so primitive ops work.
                let inner = self.lower_pattern(pattern)?;
                let ty = self
                    .current_match_subject_ty
                    .unwrap_or_else(|| self.checker.tcx.any());
                let sym = self.define_local(name, ty);
                HirPattern::As {
                    pattern: Box::new(inner),
                    name: sym,
                }
            }
        })
    }

    fn resolve_name(&self, name: &str, _span: Span) -> Option<SymbolId> {
        // Check function-local scope first (survives checker scope pops)
        if let Some(&id) = self.local_names.get(name) {
            return Some(id);
        }
        // Fall back to global scope (functions, classes — still in checker)
        self.checker.symbols.lookup(name)
    }

    /// Build a `mb_call_spread_kwargs(f, pos_list, kwargs_dict)` call for a
    /// call carrying a `**mapping` splat. Positionals (and any `*` splats) form
    /// the ordered positional list; explicit keywords and each `**mapping` merge
    /// in source order via mb_dict_merge into the kwargs dict. The runtime helper
    /// binds the kwargs dict to the callee's declared params (or `**kwargs`
    /// receiver) at call time — the static reorder can't, since the keys are
    /// dynamic.
    fn build_spread_kwargs_call(
        &mut self,
        f: HirExpr,
        args: &[ast::CallArg],
        any_ty: TypeId,
    ) -> HirExpr {
        let merge = |acc: HirExpr, next: HirExpr| HirExpr::Call {
            func: Box::new(HirExpr::StrLit("mb_dict_merge".to_string(), any_ty)),
            args: vec![acc, next],
            ty: any_ty,
        };
        let mut pos_elems: Vec<HirExpr> = Vec::new();
        let mut kw_acc: Option<HirExpr> = None;
        let mut pend: Vec<(HirExpr, HirExpr)> = Vec::new();
        for a in args {
            match a {
                ast::CallArg::Positional(e) | ast::CallArg::StarArg(e) => {
                    if let Some(he) = self.lower_expr(e) { pos_elems.push(he); }
                }
                ast::CallArg::Keyword { name, value } => {
                    if let Some(he) = self.lower_expr(value) {
                        pend.push((
                            HirExpr::StrLit(name.clone(), self.checker.tcx.str()),
                            he,
                        ));
                    }
                }
                ast::CallArg::DoubleStarArg(e) => {
                    if let Some(he) = self.lower_expr(e) {
                        if !pend.is_empty() {
                            let chunk = HirExpr::Dict {
                                entries: std::mem::take(&mut pend), ty: any_ty,
                            };
                            kw_acc = Some(match kw_acc.take() {
                                None => chunk,
                                Some(prev) => merge(prev, chunk),
                            });
                        }
                        kw_acc = Some(match kw_acc.take() {
                            None => he,
                            Some(prev) => merge(prev, he),
                        });
                    }
                }
            }
        }
        if !pend.is_empty() {
            let chunk = HirExpr::Dict { entries: pend, ty: any_ty };
            kw_acc = Some(match kw_acc.take() {
                None => chunk,
                Some(prev) => merge(prev, chunk),
            });
        }
        let kwargs_dict = kw_acc.unwrap_or(HirExpr::Dict { entries: vec![], ty: any_ty });
        let pos_list = HirExpr::List { elements: pos_elems, ty: any_ty };
        HirExpr::Call {
            func: Box::new(HirExpr::StrLit("mb_call_spread_kwargs".to_string(), any_ty)),
            args: vec![f, pos_list, kwargs_dict],
            ty: any_ty,
        }
    }

    /// Build the merged kwargs dict for a call's explicit keywords and
    /// `**mapping` splats, in source order (later wins, via mb_dict_merge).
    /// Returns None when the call has no keyword/`**` arguments.
    fn build_kwargs_dict(
        &mut self,
        args: &[ast::CallArg],
        any_ty: TypeId,
    ) -> Option<HirExpr> {
        let merge = |acc: HirExpr, next: HirExpr| HirExpr::Call {
            func: Box::new(HirExpr::StrLit("mb_dict_merge".to_string(), any_ty)),
            args: vec![acc, next],
            ty: any_ty,
        };
        let mut kw_acc: Option<HirExpr> = None;
        let mut pend: Vec<(HirExpr, HirExpr)> = Vec::new();
        for a in args {
            match a {
                ast::CallArg::Keyword { name, value } => {
                    if let Some(he) = self.lower_expr(value) {
                        pend.push((
                            HirExpr::StrLit(name.clone(), self.checker.tcx.str()),
                            he,
                        ));
                    }
                }
                ast::CallArg::DoubleStarArg(e) => {
                    if let Some(he) = self.lower_expr(e) {
                        if !pend.is_empty() {
                            let chunk = HirExpr::Dict {
                                entries: std::mem::take(&mut pend), ty: any_ty,
                            };
                            kw_acc = Some(match kw_acc.take() {
                                None => chunk,
                                Some(prev) => merge(prev, chunk),
                            });
                        }
                        kw_acc = Some(match kw_acc.take() {
                            None => he,
                            Some(prev) => merge(prev, he),
                        });
                    }
                }
                _ => {}
            }
        }
        if !pend.is_empty() {
            let chunk = HirExpr::Dict { entries: pend, ty: any_ty };
            kw_acc = Some(match kw_acc.take() {
                None => chunk,
                Some(prev) => merge(prev, chunk),
            });
        }
        kw_acc
    }

    /// Build `raise NameError("name '<name>' is not defined")` as a HirStmt for
    /// a reference to an undefined name. Returns None when the `NameError`
    /// builtin itself can't be resolved (so the caller falls back to normal
    /// lowering rather than emitting a broken raise).
    fn name_error_raise(&self, name: &str, span: Span) -> Option<HirStmt> {
        let ne_sym = self.resolve_name("NameError", span)?;
        let any_ty = self.checker.tcx.any();
        let func = HirExpr::Var(ne_sym, any_ty);
        let msg = HirExpr::StrLit(
            format!("name '{name}' is not defined"),
            self.checker.tcx.str(),
        );
        let call = HirExpr::Call {
            func: Box::new(func),
            args: vec![msg],
            ty: any_ty,
        };
        Some(HirStmt::Raise { value: Some(call), from: None, span })
    }

    #[allow(dead_code)]
    fn resolve_type_from_checker(&self, _name: &str) -> TypeId {
        self.checker.tcx.any()
    }
}

fn lower_bin_op(op: ast::BinOp) -> Option<HirBinOp> {
    Some(match op {
        ast::BinOp::Add => HirBinOp::Add,
        ast::BinOp::Sub => HirBinOp::Sub,
        ast::BinOp::Mul => HirBinOp::Mul,
        ast::BinOp::Div => HirBinOp::Div,
        ast::BinOp::FloorDiv => HirBinOp::FloorDiv,
        ast::BinOp::Mod => HirBinOp::Mod,
        ast::BinOp::Pow => HirBinOp::Pow,
        ast::BinOp::Eq => HirBinOp::Eq,
        ast::BinOp::NotEq => HirBinOp::NotEq,
        ast::BinOp::Lt => HirBinOp::Lt,
        ast::BinOp::Gt => HirBinOp::Gt,
        ast::BinOp::LtEq => HirBinOp::LtEq,
        ast::BinOp::GtEq => HirBinOp::GtEq,
        ast::BinOp::And => HirBinOp::And,
        ast::BinOp::Or => HirBinOp::Or,
        ast::BinOp::BitAnd => HirBinOp::BitAnd,
        ast::BinOp::BitOr => HirBinOp::BitOr,
        ast::BinOp::BitXor => HirBinOp::BitXor,
        ast::BinOp::LShift => HirBinOp::LShift,
        ast::BinOp::RShift => HirBinOp::RShift,
        ast::BinOp::Is => HirBinOp::Is,
        ast::BinOp::IsNot => HirBinOp::IsNot,
        ast::BinOp::In => HirBinOp::In,
        ast::BinOp::NotIn => HirBinOp::NotIn,
        ast::BinOp::MatMul => return None,
    })
}

fn lower_unary_op(op: ast::UnaryOp) -> Option<HirUnaryOp> {
    Some(match op {
        ast::UnaryOp::Pos => HirUnaryOp::Pos,
        ast::UnaryOp::Neg => HirUnaryOp::Neg,
        ast::UnaryOp::Not => HirUnaryOp::Not,
        ast::UnaryOp::BitNot => HirUnaryOp::BitNot,
    })
}

fn lower_aug_op(op: ast::AugOp) -> HirBinOp {
    match op {
        ast::AugOp::Add => HirBinOp::Add,
        ast::AugOp::Sub => HirBinOp::Sub,
        ast::AugOp::Mul => HirBinOp::Mul,
        ast::AugOp::Div => HirBinOp::Div,
        ast::AugOp::FloorDiv => HirBinOp::FloorDiv,
        ast::AugOp::Mod => HirBinOp::Mod,
        ast::AugOp::Pow => HirBinOp::Pow,
        ast::AugOp::BitAnd => HirBinOp::BitAnd,
        ast::AugOp::BitOr => HirBinOp::BitOr,
        ast::AugOp::BitXor => HirBinOp::BitXor,
        ast::AugOp::LShift => HirBinOp::LShift,
        ast::AugOp::RShift => HirBinOp::RShift,
        ast::AugOp::MatMul => HirBinOp::Mul,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::*;

    fn sp<T>(node: T) -> Spanned<T> {
        Spanned::new(node, Span::dummy())
    }

    #[test]
    fn test_lower_int_literal() {
        let checker = TypeChecker::new();
        let module = Module {
            stmts: vec![sp(Stmt::ExprStmt(sp(Expr::IntLit(42))))],
        };
        let hir = lower_module(&module, &checker).unwrap();
        assert_eq!(hir.top_level.len(), 1);
        match &hir.top_level[0] {
            HirStmt::Expr {
                expr: HirExpr::IntLit(42, _),
                ..
            } => {}
            other => panic!("expected IntLit(42), got {other:?}"),
        }
    }

    #[test]
    fn test_lower_binop() {
        let checker = TypeChecker::new();
        let module = Module {
            stmts: vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
                op: BinOp::Add,
                lhs: Box::new(sp(Expr::IntLit(1))),
                rhs: Box::new(sp(Expr::IntLit(2))),
            })))],
        };
        let hir = lower_module(&module, &checker).unwrap();
        assert_eq!(hir.top_level.len(), 1);
    }

    #[test]
    fn test_lower_float_literal() {
        let checker = TypeChecker::new();
        let module = Module {
            stmts: vec![sp(Stmt::ExprStmt(sp(Expr::FloatLit(3.14))))],
        };
        let hir = lower_module(&module, &checker).unwrap();
        assert_eq!(hir.top_level.len(), 1);
        match &hir.top_level[0] {
            HirStmt::Expr {
                expr: HirExpr::FloatLit(f, _),
                ..
            } => {
                assert!((f - 3.14).abs() < 1e-10);
            }
            other => panic!("expected FloatLit, got {other:?}"),
        }
    }

    #[test]
    fn test_lower_bool_literal() {
        let checker = TypeChecker::new();
        let module = Module {
            stmts: vec![sp(Stmt::ExprStmt(sp(Expr::BoolLit(true))))],
        };
        let hir = lower_module(&module, &checker).unwrap();
        match &hir.top_level[0] {
            HirStmt::Expr {
                expr: HirExpr::BoolLit(true, _),
                ..
            } => {}
            other => panic!("expected BoolLit(true), got {other:?}"),
        }
    }

    #[test]
    fn test_lower_none_literal() {
        let checker = TypeChecker::new();
        let module = Module {
            stmts: vec![sp(Stmt::ExprStmt(sp(Expr::NoneLit)))],
        };
        let hir = lower_module(&module, &checker).unwrap();
        match &hir.top_level[0] {
            HirStmt::Expr {
                expr: HirExpr::NoneLit(_),
                ..
            } => {}
            other => panic!("expected NoneLit, got {other:?}"),
        }
    }

    #[test]
    fn test_lower_str_literal() {
        let checker = TypeChecker::new();
        let module = Module {
            stmts: vec![sp(Stmt::ExprStmt(sp(Expr::StrLit("hello".to_string()))))],
        };
        let hir = lower_module(&module, &checker).unwrap();
        match &hir.top_level[0] {
            HirStmt::Expr {
                expr: HirExpr::StrLit(s, _),
                ..
            } => {
                assert_eq!(s, "hello");
            }
            other => panic!("expected StrLit, got {other:?}"),
        }
    }

    #[test]
    fn test_lower_unary_neg() {
        let checker = TypeChecker::new();
        let module = Module {
            stmts: vec![sp(Stmt::ExprStmt(sp(Expr::UnaryOp {
                op: UnaryOp::Neg,
                operand: Box::new(sp(Expr::IntLit(5))),
            })))],
        };
        let hir = lower_module(&module, &checker).unwrap();
        match &hir.top_level[0] {
            HirStmt::Expr {
                expr: HirExpr::UnaryOp { op, .. },
                ..
            } => {
                assert_eq!(*op, crate::hir::HirUnaryOp::Neg);
            }
            other => panic!("expected UnaryOp, got {other:?}"),
        }
    }

    #[test]
    fn test_lower_return_stmt() {
        let checker = TypeChecker::new();
        let module = Module {
            stmts: vec![sp(Stmt::Return(Some(sp(Expr::IntLit(42)))))],
        };
        let hir = lower_module(&module, &checker).unwrap();
        match &hir.top_level[0] {
            HirStmt::Return {
                value: Some(HirExpr::IntLit(42, _)),
                ..
            } => {}
            other => panic!("expected Return(42), got {other:?}"),
        }
    }

    #[test]
    fn test_lower_break_continue() {
        let checker = TypeChecker::new();
        let module = Module {
            stmts: vec![sp(Stmt::Break), sp(Stmt::Continue)],
        };
        let hir = lower_module(&module, &checker).unwrap();
        assert_eq!(hir.top_level.len(), 2);
        assert!(matches!(hir.top_level[0], HirStmt::Break { .. }));
        assert!(matches!(hir.top_level[1], HirStmt::Continue { .. }));
    }

    #[test]
    fn test_lower_pass_is_skipped() {
        let checker = TypeChecker::new();
        let module = Module {
            stmts: vec![sp(Stmt::Pass)],
        };
        let hir = lower_module(&module, &checker).unwrap();
        assert!(hir.top_level.is_empty(), "pass should produce no HIR");
    }

    #[test]
    fn test_lower_empty_module() {
        let checker = TypeChecker::new();
        let module = Module { stmts: vec![] };
        let hir = lower_module(&module, &checker).unwrap();
        assert!(hir.functions.is_empty());
        assert!(hir.classes.is_empty());
        assert!(hir.top_level.is_empty());
    }

    #[test]
    fn test_lower_bin_op_mapping() {
        // Test all BinOp to HirBinOp mappings
        let mappings = [
            (BinOp::Add, HirBinOp::Add),
            (BinOp::Sub, HirBinOp::Sub),
            (BinOp::Mul, HirBinOp::Mul),
            (BinOp::Div, HirBinOp::Div),
            (BinOp::Mod, HirBinOp::Mod),
            (BinOp::Pow, HirBinOp::Pow),
            (BinOp::Eq, HirBinOp::Eq),
            (BinOp::NotEq, HirBinOp::NotEq),
            (BinOp::Lt, HirBinOp::Lt),
            (BinOp::Gt, HirBinOp::Gt),
            (BinOp::And, HirBinOp::And),
            (BinOp::Or, HirBinOp::Or),
            (BinOp::Is, HirBinOp::Is),
            (BinOp::IsNot, HirBinOp::IsNot),
            (BinOp::In, HirBinOp::In),
            (BinOp::NotIn, HirBinOp::NotIn),
        ];
        for (ast_op, expected) in mappings {
            assert_eq!(lower_bin_op(ast_op), Some(expected), "{ast_op:?}");
        }
    }

    #[test]
    fn test_lower_bin_op_matmul_returns_none() {
        assert_eq!(lower_bin_op(BinOp::MatMul), None);
    }

    #[test]
    fn test_lower_unary_op_mapping() {
        assert_eq!(lower_unary_op(UnaryOp::Pos), Some(HirUnaryOp::Pos));
        assert_eq!(lower_unary_op(UnaryOp::Neg), Some(HirUnaryOp::Neg));
        assert_eq!(lower_unary_op(UnaryOp::Not), Some(HirUnaryOp::Not));
        assert_eq!(lower_unary_op(UnaryOp::BitNot), Some(HirUnaryOp::BitNot));
    }

    #[test]
    fn test_lower_aug_op_mapping() {
        assert_eq!(lower_aug_op(AugOp::Add), HirBinOp::Add);
        assert_eq!(lower_aug_op(AugOp::Sub), HirBinOp::Sub);
        assert_eq!(lower_aug_op(AugOp::Mul), HirBinOp::Mul);
        assert_eq!(lower_aug_op(AugOp::Div), HirBinOp::Div);
        assert_eq!(lower_aug_op(AugOp::FloorDiv), HirBinOp::FloorDiv);
        assert_eq!(lower_aug_op(AugOp::Mod), HirBinOp::Mod);
        assert_eq!(lower_aug_op(AugOp::Pow), HirBinOp::Pow);
        assert_eq!(lower_aug_op(AugOp::BitAnd), HirBinOp::BitAnd);
        assert_eq!(lower_aug_op(AugOp::BitOr), HirBinOp::BitOr);
        assert_eq!(lower_aug_op(AugOp::BitXor), HirBinOp::BitXor);
        assert_eq!(lower_aug_op(AugOp::LShift), HirBinOp::LShift);
        assert_eq!(lower_aug_op(AugOp::RShift), HirBinOp::RShift);
        assert_eq!(lower_aug_op(AugOp::MatMul), HirBinOp::Mul);
    }

    // -------------------------------------------------------------------------
    // Helper
    // -------------------------------------------------------------------------

    fn helper_lower(stmts: Vec<Spanned<Stmt>>) -> HirModule {
        let checker = TypeChecker::new();
        let module = Module { stmts };
        lower_module(&module, &checker).expect("lower failed")
    }

    /// Lower a module after pre-registering the given names as Function symbols
    /// in the checker's symbol table so that `resolve_name` can find them.
    fn helper_lower_with_fns(stmts: Vec<Spanned<Stmt>>, fn_names: &[&str]) -> HirModule {
        let mut checker = TypeChecker::new();
        for &name in fn_names {
            checker
                .symbols
                .define(name.to_string(), crate::resolve::SymbolKind::Function);
        }
        let module = Module { stmts };
        lower_module(&module, &checker).expect("lower failed")
    }

    /// Lower a module after pre-registering names as Class symbols.
    fn helper_lower_with_classes(stmts: Vec<Spanned<Stmt>>, class_names: &[&str]) -> HirModule {
        let mut checker = TypeChecker::new();
        for &name in class_names {
            checker
                .symbols
                .define(name.to_string(), crate::resolve::SymbolKind::Class);
        }
        let module = Module { stmts };
        lower_module(&module, &checker).expect("lower failed")
    }

    fn make_param(name: &str) -> Param {
        Param {
            name: name.to_string(),
            ty: sp(TypeExpr::Named("Any".to_string())),
            default: None,
            kind: ParamKind::Regular,
            pos_only: false,
            kw_only: false,
            span: Span::dummy(),
        }
    }

    // -------------------------------------------------------------------------
    // 1. Literal lowering extras
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_bytes_lit() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BytesLit(vec![1, 2, 3]))))]);
        assert_eq!(hir.top_level.len(), 1);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::BytesLit(b, _), .. } if b == &[1u8, 2, 3]
        ));
    }

    #[test]
    fn test_lower_complex_lit() {
        // ComplexLit lowers to `complex(0.0, N)` so the value is an
        // ObjData::Complex at runtime, not a collapsed float.
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::ComplexLit(2.0))))]);
        assert_eq!(hir.top_level.len(), 1);
        let HirStmt::Expr { expr, .. } = &hir.top_level[0] else {
            panic!("expected expression statement");
        };
        let HirExpr::Call { func, args, .. } = expr else {
            panic!("expected Call, got {expr:?}");
        };
        assert!(matches!(func.as_ref(), HirExpr::StrLit(s, _) if s == "mb_complex"));
        assert_eq!(args.len(), 2);
        assert!(matches!(args[0], HirExpr::FloatLit(r, _) if r == 0.0));
        assert!(matches!(args[1], HirExpr::FloatLit(i, _) if i == 2.0));
    }

    #[test]
    fn test_lower_fstring_literal_parts() {
        let parts = vec![
            FStringPart::Literal("hello ".to_string()),
            FStringPart::Expr(sp(Expr::IntLit(42)), None),
        ];
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::FString(parts))))]);
        assert_eq!(hir.top_level.len(), 1);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::FString { parts, .. }, .. } if parts.len() == 2
        ));
    }

    #[test]
    fn test_lower_tuple_lit() {
        let elems = vec![sp(Expr::IntLit(1)), sp(Expr::IntLit(2))];
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::TupleLit(elems))))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::Tuple { elements, .. }, .. } if elements.len() == 2
        ));
    }

    #[test]
    fn test_lower_set_lit() {
        let elems = vec![sp(Expr::IntLit(1)), sp(Expr::IntLit(2))];
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::SetLit(elems))))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::Set { elements, .. }, .. } if elements.len() == 2
        ));
    }

    // -------------------------------------------------------------------------
    // 2. BinOp full coverage
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_binop_floordiv() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
            op: BinOp::FloorDiv,
            lhs: Box::new(sp(Expr::IntLit(10))),
            rhs: Box::new(sp(Expr::IntLit(3))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    op: HirBinOp::FloorDiv,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_binop_bitand() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
            op: BinOp::BitAnd,
            lhs: Box::new(sp(Expr::IntLit(0b1100))),
            rhs: Box::new(sp(Expr::IntLit(0b1010))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    op: HirBinOp::BitAnd,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_binop_bitor() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
            op: BinOp::BitOr,
            lhs: Box::new(sp(Expr::IntLit(1))),
            rhs: Box::new(sp(Expr::IntLit(2))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    op: HirBinOp::BitOr,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_binop_bitxor() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
            op: BinOp::BitXor,
            lhs: Box::new(sp(Expr::IntLit(5))),
            rhs: Box::new(sp(Expr::IntLit(3))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    op: HirBinOp::BitXor,
                    ..
                },
                ..
            }
        ));
    }

    // -------------------------------------------------------------------------
    // 3. Comparison ops
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_binop_lteq() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
            op: BinOp::LtEq,
            lhs: Box::new(sp(Expr::IntLit(1))),
            rhs: Box::new(sp(Expr::IntLit(2))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    op: HirBinOp::LtEq,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_binop_gteq() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
            op: BinOp::GtEq,
            lhs: Box::new(sp(Expr::IntLit(2))),
            rhs: Box::new(sp(Expr::IntLit(1))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    op: HirBinOp::GtEq,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_binop_is() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
            op: BinOp::Is,
            lhs: Box::new(sp(Expr::NoneLit)),
            rhs: Box::new(sp(Expr::NoneLit)),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    op: HirBinOp::Is,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_binop_isnot() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
            op: BinOp::IsNot,
            lhs: Box::new(sp(Expr::IntLit(1))),
            rhs: Box::new(sp(Expr::NoneLit)),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    op: HirBinOp::IsNot,
                    ..
                },
                ..
            }
        ));
    }

    // -------------------------------------------------------------------------
    // 4. UnaryOp extras
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_unary_pos() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::UnaryOp {
            op: UnaryOp::Pos,
            operand: Box::new(sp(Expr::IntLit(5))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::UnaryOp {
                    op: HirUnaryOp::Pos,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_unary_not() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::UnaryOp {
            op: UnaryOp::Not,
            operand: Box::new(sp(Expr::BoolLit(true))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::UnaryOp {
                    op: HirUnaryOp::Not,
                    ..
                },
                ..
            }
        ));
    }

    // -------------------------------------------------------------------------
    // 5. Assignment forms
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_vardecl() {
        let hir = helper_lower(vec![sp(Stmt::VarDecl {
            name: "x".to_string(),
            ty: sp(TypeExpr::Named("int".to_string())),
            value: sp(Expr::IntLit(99)),
        })]);
        assert_eq!(hir.top_level.len(), 1);
        assert!(matches!(&hir.top_level[0], HirStmt::Let { .. }));
    }

    #[test]
    fn test_lower_assign_to_subscript() {
        // x: Any = [1, 2]; x[0] = 5  (define x first, then assign)
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "x".to_string(),
                ty: sp(TypeExpr::Named("Any".to_string())),
                value: sp(Expr::ListLit(vec![sp(Expr::IntLit(1))])),
            }),
            sp(Stmt::Assign {
                target: sp(Expr::Index {
                    object: Box::new(sp(Expr::Ident("x".to_string()))),
                    index: Box::new(sp(Expr::IntLit(0))),
                }),
                value: sp(Expr::IntLit(5)),
            }),
        ]);
        assert_eq!(hir.top_level.len(), 2);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Assign {
                target: HirLValue::Index { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_augassign_add() {
        // x: int = 0; x += 1
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "x".to_string(),
                ty: sp(TypeExpr::Named("int".to_string())),
                value: sp(Expr::IntLit(0)),
            }),
            sp(Stmt::AugAssign {
                target: sp(Expr::Ident("x".to_string())),
                op: AugOp::Add,
                value: sp(Expr::IntLit(1)),
            }),
        ]);
        assert_eq!(hir.top_level.len(), 2);
        // AugAssign desugars to Assign with BinOp::Add
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Assign {
                value: HirExpr::BinOp {
                    op: HirBinOp::Add,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_augassign_sub() {
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "y".to_string(),
                ty: sp(TypeExpr::Named("int".to_string())),
                value: sp(Expr::IntLit(10)),
            }),
            sp(Stmt::AugAssign {
                target: sp(Expr::Ident("y".to_string())),
                op: AugOp::Sub,
                value: sp(Expr::IntLit(3)),
            }),
        ]);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Assign {
                value: HirExpr::BinOp {
                    op: HirBinOp::Sub,
                    ..
                },
                ..
            }
        ));
    }

    // -------------------------------------------------------------------------
    // 6. Function definitions
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_fn_no_params() {
        let hir = helper_lower_with_fns(
            vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "foo".to_string(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::Return(None))],
            })],
            &["foo"],
        );
        assert_eq!(hir.functions.len(), 1);
        assert_eq!(hir.functions[0].params.len(), 0);
        assert!(!hir.functions[0].is_async);
        assert!(!hir.functions[0].is_generator);
    }

    #[test]
    fn test_lower_fn_single_param() {
        let hir = helper_lower_with_fns(
            vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "bar".to_string(),
                type_params: vec![],
                params: vec![make_param("x")],
                return_ty: None,
                body: vec![sp(Stmt::Return(Some(sp(Expr::Ident("x".to_string())))))],
            })],
            &["bar"],
        );
        assert_eq!(hir.functions.len(), 1);
        assert_eq!(hir.functions[0].params.len(), 1);
    }

    #[test]
    fn test_lower_fn_multi_params() {
        let hir = helper_lower_with_fns(
            vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "add".to_string(),
                type_params: vec![],
                params: vec![make_param("a"), make_param("b")],
                return_ty: None,
                body: vec![sp(Stmt::Return(Some(sp(Expr::BinOp {
                    op: BinOp::Add,
                    lhs: Box::new(sp(Expr::Ident("a".to_string()))),
                    rhs: Box::new(sp(Expr::Ident("b".to_string()))),
                }))))],
            })],
            &["add"],
        );
        assert_eq!(hir.functions[0].params.len(), 2);
        assert_eq!(hir.functions[0].body.len(), 1);
    }

    #[test]
    fn test_lower_fn_with_return_type() {
        let mut checker = TypeChecker::new();
        checker
            .symbols
            .define("get_int".to_string(), crate::resolve::SymbolKind::Function);
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "get_int".to_string(),
                type_params: vec![],
                params: vec![],
                return_ty: Some(sp(TypeExpr::Named("int".to_string()))),
                body: vec![sp(Stmt::Return(Some(sp(Expr::IntLit(0)))))],
            })],
        };
        let hir = lower_module(&module, &checker).unwrap();
        assert_eq!(hir.functions.len(), 1);
        // return_ty should be the int TypeId, not fallback any
        let int_ty = checker.tcx.int();
        assert_eq!(hir.functions[0].return_ty, int_ty);
    }

    #[test]
    fn test_lower_fn_nested_def() {
        // Nested function should appear in hir.functions as well as outer
        let mut checker = TypeChecker::new();
        checker
            .symbols
            .define("outer".to_string(), crate::resolve::SymbolKind::Function);
        let inner_body = vec![sp(Stmt::Return(None))];
        let outer_body = vec![
            sp(Stmt::FnDef {
                decorators: vec![],
                name: "inner".to_string(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: inner_body,
            }),
            sp(Stmt::Return(None)),
        ];
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "outer".to_string(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: outer_body,
            })],
        };
        let hir = lower_module(&module, &checker).unwrap();
        // Both outer and inner should be in functions
        assert_eq!(hir.functions.len(), 2);
    }

    // -------------------------------------------------------------------------
    // 7. Async function definitions
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_async_fn_basic() {
        let hir = helper_lower_with_fns(
            vec![sp(Stmt::AsyncFnDef {
                decorators: vec![],
                name: "async_fn".to_string(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::Return(None))],
            })],
            &["async_fn"],
        );
        assert_eq!(hir.functions.len(), 1);
        assert!(hir.functions[0].is_async);
    }

    #[test]
    fn test_lower_async_fn_with_await() {
        let hir = helper_lower_with_fns(
            vec![sp(Stmt::AsyncFnDef {
                decorators: vec![],
                name: "fetch".to_string(),
                type_params: vec![],
                params: vec![make_param("coro")],
                return_ty: None,
                body: vec![
                    sp(Stmt::ExprStmt(sp(Expr::Await(Box::new(sp(Expr::Ident(
                        "coro".to_string(),
                    ))))))),
                    sp(Stmt::Return(None)),
                ],
            })],
            &["fetch"],
        );
        assert!(hir.functions[0].is_async);
        assert_eq!(hir.functions[0].body.len(), 2);
    }

    #[test]
    fn test_lower_async_fn_body_is_not_generator() {
        let hir = helper_lower_with_fns(
            vec![sp(Stmt::AsyncFnDef {
                decorators: vec![],
                name: "plain_async".to_string(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::Return(Some(sp(Expr::IntLit(1)))))],
            })],
            &["plain_async"],
        );
        assert!(hir.functions[0].is_async);
        assert!(!hir.functions[0].is_generator);
    }

    // -------------------------------------------------------------------------
    // 8. Generator functions
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_generator_with_yield() {
        let hir = helper_lower_with_fns(
            vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "gen".to_string(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::ExprStmt(sp(Expr::Yield(Some(Box::new(sp(
                    Expr::IntLit(1),
                )))))))],
            })],
            &["gen"],
        );
        assert!(hir.functions[0].is_generator);
    }

    #[test]
    fn test_lower_generator_yield_from() {
        let hir = helper_lower_with_fns(
            vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "gen_from".to_string(),
                type_params: vec![],
                params: vec![make_param("it")],
                return_ty: None,
                body: vec![sp(Stmt::ExprStmt(sp(Expr::YieldFrom(Box::new(sp(
                    Expr::Ident("it".to_string()),
                ))))))],
            })],
            &["gen_from"],
        );
        assert!(hir.functions[0].is_generator);
    }

    #[test]
    fn test_lower_yield_expr_in_top_level() {
        // Yield expression at top level lowers to HirExpr::Yield
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::Yield(None))))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::Yield { value: None, .. },
                ..
            }
        ));
    }

    // -------------------------------------------------------------------------
    // 9. Class definitions
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_empty_class() {
        let hir = helper_lower_with_classes(
            vec![sp(Stmt::ClassDef {
                decorators: vec![],
                name: "Empty".to_string(),
                type_params: vec![],
                bases: vec![],
                keyword_args: vec![],
                body: vec![sp(Stmt::Pass)],
            })],
            &["Empty"],
        );
        assert_eq!(hir.classes.len(), 1);
        assert_eq!(hir.classes[0].methods.len(), 0);
        assert_eq!(hir.classes[0].fields.len(), 0);
    }

    #[test]
    fn test_lower_class_with_method() {
        let hir = helper_lower_with_classes(
            vec![sp(Stmt::ClassDef {
                decorators: vec![],
                name: "Foo".to_string(),
                type_params: vec![],
                bases: vec![],
                keyword_args: vec![],
                body: vec![sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "method".to_string(),
                    type_params: vec![],
                    params: vec![make_param("self")],
                    return_ty: None,
                    body: vec![sp(Stmt::Return(None))],
                })],
            })],
            &["Foo"],
        );
        assert_eq!(hir.classes.len(), 1);
        assert_eq!(hir.classes[0].methods.len(), 1);
    }

    #[test]
    fn test_lower_class_with_field() {
        // Field name "myfield" must also be pre-registered so resolve_name can find it.
        let mut checker = TypeChecker::new();
        checker
            .symbols
            .define("Point".to_string(), crate::resolve::SymbolKind::Class);
        checker
            .symbols
            .define("myfield".to_string(), crate::resolve::SymbolKind::Variable);
        let module = Module {
            stmts: vec![sp(Stmt::ClassDef {
                decorators: vec![],
                name: "Point".to_string(),
                type_params: vec![],
                bases: vec![],
                keyword_args: vec![],
                body: vec![sp(Stmt::VarDecl {
                    name: "myfield".to_string(),
                    ty: sp(TypeExpr::Named("int".to_string())),
                    value: sp(Expr::IntLit(0)),
                })],
            })],
        };
        let hir = lower_module(&module, &checker).unwrap();
        assert_eq!(hir.classes.len(), 1);
        assert_eq!(hir.classes[0].fields.len(), 1);
    }

    #[test]
    fn test_lower_class_with_decorator() {
        // decorator ident not resolvable, so decorators vec will be empty
        let hir = helper_lower_with_classes(
            vec![sp(Stmt::ClassDef {
                decorators: vec![sp(Expr::Ident("dataclass".to_string()))],
                name: "DC".to_string(),
                type_params: vec![],
                bases: vec![],
                keyword_args: vec![],
                body: vec![sp(Stmt::Pass)],
            })],
            &["DC"],
        );
        assert_eq!(hir.classes.len(), 1);
        // Decorator ident "dataclass" is unresolvable → decorators is empty
        // (that's correct behavior — the test just verifies no crash)
        let _ = &hir.classes[0].decorators;
    }

    #[test]
    fn test_lower_class_top_level_is_empty() {
        // ClassDef should not produce a top_level HirStmt (no placeholder without decorators)
        let hir = helper_lower_with_classes(
            vec![sp(Stmt::ClassDef {
                decorators: vec![],
                name: "NoTop".to_string(),
                type_params: vec![],
                bases: vec![],
                keyword_args: vec![],
                body: vec![sp(Stmt::Pass)],
            })],
            &["NoTop"],
        );
        assert!(hir.top_level.is_empty());
    }

    #[test]
    fn test_lower_class_multiple_methods() {
        let hir = helper_lower_with_classes(
            vec![sp(Stmt::ClassDef {
                decorators: vec![],
                name: "Multi".to_string(),
                type_params: vec![],
                bases: vec![],
                keyword_args: vec![],
                body: vec![
                    sp(Stmt::FnDef {
                        decorators: vec![],
                        name: "a".to_string(),
                        type_params: vec![],
                        params: vec![make_param("self")],
                        return_ty: None,
                        body: vec![sp(Stmt::Return(None))],
                    }),
                    sp(Stmt::FnDef {
                        decorators: vec![],
                        name: "b".to_string(),
                        type_params: vec![],
                        params: vec![make_param("self")],
                        return_ty: None,
                        body: vec![sp(Stmt::Return(None))],
                    }),
                ],
            })],
            &["Multi"],
        );
        assert_eq!(hir.classes[0].methods.len(), 2);
    }

    // -------------------------------------------------------------------------
    // 10. Control flow
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_if_only() {
        let hir = helper_lower(vec![sp(Stmt::If {
            condition: sp(Expr::BoolLit(true)),
            body: vec![sp(Stmt::Pass)],
            elif_clauses: vec![],
            else_body: None,
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::If { then_body, else_body, .. }
                if then_body.is_empty() && else_body.is_empty()
        ));
    }

    #[test]
    fn test_lower_if_else() {
        let hir = helper_lower(vec![sp(Stmt::If {
            condition: sp(Expr::BoolLit(true)),
            body: vec![sp(Stmt::Break)],
            elif_clauses: vec![],
            else_body: Some(vec![sp(Stmt::Continue)]),
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::If { then_body, else_body, .. }
                if then_body.len() == 1 && else_body.len() == 1
        ));
    }

    #[test]
    fn test_lower_if_elif_else() {
        let hir = helper_lower(vec![sp(Stmt::If {
            condition: sp(Expr::BoolLit(false)),
            body: vec![sp(Stmt::Break)],
            elif_clauses: vec![(sp(Expr::BoolLit(true)), vec![sp(Stmt::Continue)])],
            else_body: Some(vec![sp(Stmt::Pass)]),
        })]);
        // The elif is desugared into a nested If in the else_body
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::If { else_body, .. } if else_body.len() == 1
        ));
        if let HirStmt::If { else_body, .. } = &hir.top_level[0] {
            assert!(matches!(&else_body[0], HirStmt::If { .. }));
        }
    }

    #[test]
    fn test_lower_while_loop() {
        let hir = helper_lower(vec![sp(Stmt::While {
            condition: sp(Expr::BoolLit(true)),
            body: vec![sp(Stmt::Break)],
            else_body: None,
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::While { body, else_body, .. }
                if body.len() == 1 && else_body.is_empty()
        ));
    }

    #[test]
    fn test_lower_for_loop() {
        let hir = helper_lower(vec![sp(Stmt::For {
            targets: vec!["i".to_string()],
            var_ty: None,
            iter: sp(Expr::ListLit(vec![sp(Expr::IntLit(1))])),
            body: vec![sp(Stmt::Pass)],
            else_body: None,
        })]);
        assert!(matches!(&hir.top_level[0], HirStmt::For { body, .. } if body.is_empty()));
    }

    #[test]
    fn test_lower_for_with_else() {
        let hir = helper_lower(vec![sp(Stmt::For {
            targets: vec!["x".to_string()],
            var_ty: None,
            iter: sp(Expr::ListLit(vec![])),
            body: vec![sp(Stmt::Pass)],
            else_body: Some(vec![sp(Stmt::Break)]),
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::For { else_body, .. } if else_body.len() == 1
        ));
    }

    #[test]
    fn test_lower_break_in_for() {
        let hir = helper_lower(vec![sp(Stmt::For {
            targets: vec!["n".to_string()],
            var_ty: None,
            iter: sp(Expr::ListLit(vec![])),
            body: vec![sp(Stmt::Break)],
            else_body: None,
        })]);
        if let HirStmt::For { body, .. } = &hir.top_level[0] {
            assert!(matches!(&body[0], HirStmt::Break { .. }));
        } else {
            panic!("expected For");
        }
    }

    #[test]
    fn test_lower_continue_in_for() {
        let hir = helper_lower(vec![sp(Stmt::For {
            targets: vec!["n".to_string()],
            var_ty: None,
            iter: sp(Expr::ListLit(vec![])),
            body: vec![sp(Stmt::Continue)],
            else_body: None,
        })]);
        if let HirStmt::For { body, .. } = &hir.top_level[0] {
            assert!(matches!(&body[0], HirStmt::Continue { .. }));
        } else {
            panic!("expected For");
        }
    }

    // -------------------------------------------------------------------------
    // 11. Import statements
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_simple_import() {
        let hir = helper_lower(vec![sp(Stmt::Import {
            module: vec!["os".to_string()],
            names: None,
            module_alias: None,
        })]);
        assert_eq!(hir.top_level.len(), 1);
        assert!(matches!(&hir.top_level[0], HirStmt::Import { .. }));
    }

    #[test]
    fn test_lower_import_with_alias() {
        let hir = helper_lower(vec![sp(Stmt::Import {
            module: vec!["numpy".to_string()],
            names: None,
            module_alias: Some("np".to_string()),
        })]);
        if let HirStmt::Import { import, .. } = &hir.top_level[0] {
            assert_eq!(import.module_alias, Some("np".to_string()));
        } else {
            panic!("expected Import");
        }
    }

    #[test]
    fn test_lower_from_import() {
        let hir = helper_lower(vec![sp(Stmt::Import {
            module: vec!["os".to_string(), "path".to_string()],
            names: Some(vec![("join".to_string(), None)]),
            module_alias: None,
        })]);
        if let HirStmt::Import { import, .. } = &hir.top_level[0] {
            assert!(import.names.is_some());
            let names = import.names.as_ref().unwrap();
            assert_eq!(names[0].0, "join");
        } else {
            panic!("expected Import");
        }
    }

    #[test]
    fn test_lower_from_import_with_alias() {
        let hir = helper_lower(vec![sp(Stmt::Import {
            module: vec!["os".to_string()],
            names: Some(vec![("path".to_string(), Some("p".to_string()))]),
            module_alias: None,
        })]);
        if let HirStmt::Import { import, .. } = &hir.top_level[0] {
            let names = import.names.as_ref().unwrap();
            assert_eq!(names[0].1, Some("p".to_string()));
        } else {
            panic!("expected Import");
        }
    }

    #[test]
    fn test_lower_dotted_module_import() {
        let hir = helper_lower(vec![sp(Stmt::Import {
            module: vec!["os".to_string(), "path".to_string()],
            names: None,
            module_alias: None,
        })]);
        if let HirStmt::Import { import, .. } = &hir.top_level[0] {
            assert_eq!(import.module, vec!["os", "path"]);
        } else {
            panic!("expected Import");
        }
    }

    // -------------------------------------------------------------------------
    // 12. Exception handling
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_try_except() {
        let hir = helper_lower(vec![sp(Stmt::Try {
            body: vec![sp(Stmt::Pass)],
            handlers: vec![ExceptHandler {
                exc_type: None,
                name: None,
                body: vec![sp(Stmt::Pass)],
                is_star: false,
                span: Span::dummy(),
            }],
            else_body: None,
            finally_body: None,
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Try { handlers, .. } if handlers.len() == 1
        ));
    }

    #[test]
    fn test_lower_try_finally() {
        let hir = helper_lower(vec![sp(Stmt::Try {
            body: vec![sp(Stmt::Pass)],
            handlers: vec![],
            else_body: None,
            finally_body: Some(vec![sp(Stmt::Pass)]),
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Try { finally_body, .. } if finally_body.is_empty()
            // Pass produces no HirStmt so finally_body is empty
        ));
    }

    #[test]
    fn test_lower_try_except_finally() {
        let hir = helper_lower(vec![sp(Stmt::Try {
            body: vec![sp(Stmt::Break)],
            handlers: vec![ExceptHandler {
                exc_type: Some(sp(Expr::Ident("ValueError".to_string()))),
                name: None,
                body: vec![sp(Stmt::Pass)],
                is_star: false,
                span: Span::dummy(),
            }],
            else_body: None,
            finally_body: Some(vec![sp(Stmt::Break)]),
        })]);
        if let HirStmt::Try {
            body,
            handlers,
            finally_body,
            ..
        } = &hir.top_level[0]
        {
            assert_eq!(body.len(), 1);
            assert_eq!(handlers.len(), 1);
            assert_eq!(finally_body.len(), 1);
        } else {
            panic!("expected Try");
        }
    }

    #[test]
    fn test_lower_raise_plain() {
        let hir = helper_lower(vec![sp(Stmt::Raise {
            value: None,
            from: None,
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Raise {
                value: None,
                from: None,
                ..
            }
        ));
    }

    #[test]
    fn test_lower_raise_from() {
        let hir = helper_lower(vec![sp(Stmt::Raise {
            value: Some(sp(Expr::Ident("RuntimeError".to_string()))),
            from: Some(sp(Expr::Ident("ValueError".to_string()))),
        })]);
        // Both 'value' and 'from' will be None in HIR since the idents are unresolved,
        // but the statement structure should be a Raise
        assert!(matches!(&hir.top_level[0], HirStmt::Raise { .. }));
    }

    // -------------------------------------------------------------------------
    // 13. Context managers
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_with_statement() {
        let hir = helper_lower(vec![sp(Stmt::With {
            items: vec![WithItem {
                context: sp(Expr::IntLit(1)),
                alias: None,
            }],
            body: vec![sp(Stmt::Pass)],
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::With { items, body, .. } if items.len() == 1 && body.is_empty()
        ));
    }

    #[test]
    fn test_lower_async_with() {
        let hir = helper_lower(vec![sp(Stmt::AsyncWith {
            items: vec![WithItem {
                context: sp(Expr::IntLit(42)),
                alias: None,
            }],
            body: vec![sp(Stmt::Pass)],
        })]);
        // AsyncWith is handled by the same arm as With
        assert!(matches!(&hir.top_level[0], HirStmt::With { .. }));
    }

    #[test]
    fn test_lower_with_binding() {
        // with ctx as f — alias resolution. Since 'f' is not pre-defined, alias will be None.
        let hir = helper_lower(vec![sp(Stmt::With {
            items: vec![WithItem {
                context: sp(Expr::IntLit(1)),
                alias: Some("f".to_string()),
            }],
            body: vec![sp(Stmt::Pass)],
        })]);
        // alias is None because "f" isn't in scope
        if let HirStmt::With { items, .. } = &hir.top_level[0] {
            assert_eq!(items.len(), 1);
        } else {
            panic!("expected With");
        }
    }

    // -------------------------------------------------------------------------
    // 14. Walrus operator
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_walrus_in_if_condition() {
        // `if (x := 5): pass` — Walrus lowers successfully (drops from expr if unresolved target)
        let hir = helper_lower(vec![sp(Stmt::If {
            condition: sp(Expr::Walrus {
                target: "w".to_string(),
                value: Box::new(sp(Expr::IntLit(5))),
            }),
            body: vec![sp(Stmt::Pass)],
            elif_clauses: vec![],
            else_body: None,
        })]);
        // The walrus expr is unrecognized (returns None), so If itself won't be lowered
        // OR it does lower if the expr is handled. Just check top_level has at most 1 entry.
        // Based on code, Walrus hits the `_ => None` arm, so the If condition returns None,
        // meaning the If statement itself produces None (condition is None → lower_stmt skips).
        let _ = hir; // don't panic — just verify we don't crash
    }

    #[test]
    fn test_lower_walrus_in_while_condition() {
        // Same — walrus at while condition is unrecognized, produces empty top_level
        let hir = helper_lower(vec![sp(Stmt::While {
            condition: sp(Expr::Walrus {
                target: "n".to_string(),
                value: Box::new(sp(Expr::IntLit(0))),
            }),
            body: vec![sp(Stmt::Break)],
            else_body: None,
        })]);
        let _ = hir; // no panic
    }

    // -------------------------------------------------------------------------
    // 15. Comprehensions
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_list_comp() {
        let gen = Comprehension {
            targets: vec!["x".to_string()],
            unpack_target: false,
            iter: sp(Expr::ListLit(vec![sp(Expr::IntLit(1))])),
            conditions: vec![],
            is_async: false,
        };
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::ListComp {
            element: Box::new(sp(Expr::Ident("x".to_string()))),
            generators: vec![gen],
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::ListComp { generators, .. }, .. }
                if generators.len() == 1
        ));
    }

    #[test]
    fn test_lower_dict_comp() {
        let gen = Comprehension {
            targets: vec!["k".to_string()],
            unpack_target: false,
            iter: sp(Expr::ListLit(vec![])),
            conditions: vec![],
            is_async: false,
        };
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::DictComp {
            key: Box::new(sp(Expr::Ident("k".to_string()))),
            value: Box::new(sp(Expr::IntLit(0))),
            generators: vec![gen],
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::DictComp { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_set_comp() {
        let gen = Comprehension {
            targets: vec!["s".to_string()],
            unpack_target: false,
            iter: sp(Expr::ListLit(vec![])),
            conditions: vec![],
            is_async: false,
        };
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::SetComp {
            element: Box::new(sp(Expr::IntLit(1))),
            generators: vec![gen],
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::SetComp { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_generator_expr_desugared_to_list_comp() {
        // GeneratorExpr is desugared to mb_iter(<ListComp>): the eager list
        // comprehension is wrapped in an iterator so the genexpr keeps
        // single-use iterator identity (next() works, no len()/indexing).
        let gen = Comprehension {
            targets: vec!["g".to_string()],
            unpack_target: false,
            iter: sp(Expr::ListLit(vec![])),
            conditions: vec![],
            is_async: false,
        };
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::GeneratorExpr {
            element: Box::new(sp(Expr::IntLit(0))),
            generators: vec![gen],
        })))]);
        match &hir.top_level[0] {
            HirStmt::Expr {
                expr: HirExpr::Call { func, args, .. },
                ..
            } => {
                assert!(
                    matches!(&**func, HirExpr::StrLit(name, _) if name == "mb_iter"),
                    "genexpr must lower to an mb_iter call"
                );
                assert!(matches!(args.first(), Some(HirExpr::ListComp { .. })));
            }
            other => panic!("expected mb_iter(ListComp) call, got {other:?}"),
        }
    }

    #[test]
    fn test_lower_list_comp_with_filter() {
        let gen = Comprehension {
            targets: vec!["x".to_string()],
            unpack_target: false,
            iter: sp(Expr::ListLit(vec![
                sp(Expr::IntLit(1)),
                sp(Expr::IntLit(2)),
            ])),
            conditions: vec![sp(Expr::BoolLit(true))],
            is_async: false,
        };
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::ListComp {
            element: Box::new(sp(Expr::Ident("x".to_string()))),
            generators: vec![gen],
        })))]);
        if let HirStmt::Expr {
            expr: HirExpr::ListComp { generators, .. },
            ..
        } = &hir.top_level[0]
        {
            assert_eq!(generators[0].conditions.len(), 1);
        } else {
            panic!("expected ListComp");
        }
    }

    #[test]
    fn test_lower_list_comp_async() {
        let gen = Comprehension {
            targets: vec!["x".to_string()],
            unpack_target: false,
            iter: sp(Expr::ListLit(vec![])),
            conditions: vec![],
            is_async: true,
        };
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::ListComp {
            element: Box::new(sp(Expr::IntLit(0))),
            generators: vec![gen],
        })))]);
        if let HirStmt::Expr {
            expr: HirExpr::ListComp { generators, .. },
            ..
        } = &hir.top_level[0]
        {
            assert!(generators[0].is_async);
        } else {
            panic!("expected ListComp");
        }
    }

    // -------------------------------------------------------------------------
    // 16. Match statement
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_match_literal_arm() {
        let hir = helper_lower(vec![sp(Stmt::Match {
            expr: sp(Expr::IntLit(1)),
            arms: vec![MatchArm {
                pattern: sp(Pattern::Literal(Expr::IntLit(1))),
                guard: None,
                body: vec![sp(Stmt::Pass)],
                span: Span::dummy(),
            }],
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Match { cases, .. } if cases.len() == 1
        ));
    }

    #[test]
    fn test_lower_match_wildcard_arm() {
        let hir = helper_lower(vec![sp(Stmt::Match {
            expr: sp(Expr::IntLit(0)),
            arms: vec![MatchArm {
                pattern: sp(Pattern::Wildcard),
                guard: None,
                body: vec![sp(Stmt::Pass)],
                span: Span::dummy(),
            }],
        })]);
        if let HirStmt::Match { cases, .. } = &hir.top_level[0] {
            assert!(matches!(&cases[0].pattern, HirPattern::Wildcard));
        } else {
            panic!("expected Match");
        }
    }

    #[test]
    fn test_lower_match_capture_arm() {
        let hir = helper_lower(vec![sp(Stmt::Match {
            expr: sp(Expr::IntLit(5)),
            arms: vec![MatchArm {
                pattern: sp(Pattern::Binding("x".to_string())),
                guard: None,
                body: vec![sp(Stmt::Pass)],
                span: Span::dummy(),
            }],
        })]);
        if let HirStmt::Match { cases, .. } = &hir.top_level[0] {
            assert!(matches!(&cases[0].pattern, HirPattern::Capture(_)));
        } else {
            panic!("expected Match");
        }
    }

    #[test]
    fn test_lower_match_multiple_arms() {
        let hir = helper_lower(vec![sp(Stmt::Match {
            expr: sp(Expr::IntLit(0)),
            arms: vec![
                MatchArm {
                    pattern: sp(Pattern::Literal(Expr::IntLit(0))),
                    guard: None,
                    body: vec![sp(Stmt::Break)],
                    span: Span::dummy(),
                },
                MatchArm {
                    pattern: sp(Pattern::Wildcard),
                    guard: None,
                    body: vec![sp(Stmt::Continue)],
                    span: Span::dummy(),
                },
            ],
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Match { cases, .. } if cases.len() == 2
        ));
    }

    // -------------------------------------------------------------------------
    // 17. Del statement
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_del_name() {
        // Define a variable first, then del it
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "d".to_string(),
                ty: sp(TypeExpr::Named("int".to_string())),
                value: sp(Expr::IntLit(0)),
            }),
            sp(Stmt::Del(sp(Expr::Ident("d".to_string())))),
        ]);
        assert_eq!(hir.top_level.len(), 2);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Del {
                target: HirLValue::Var(_),
                ..
            }
        ));
    }

    #[test]
    fn test_lower_del_subscript() {
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "arr".to_string(),
                ty: sp(TypeExpr::Named("Any".to_string())),
                value: sp(Expr::ListLit(vec![])),
            }),
            sp(Stmt::Del(sp(Expr::Index {
                object: Box::new(sp(Expr::Ident("arr".to_string()))),
                index: Box::new(sp(Expr::IntLit(0))),
            }))),
        ]);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Del {
                target: HirLValue::Index { .. },
                ..
            }
        ));
    }

    // -------------------------------------------------------------------------
    // 18. Assert statement
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_assert_with_message() {
        let hir = helper_lower(vec![sp(Stmt::Assert {
            test: sp(Expr::BoolLit(true)),
            msg: Some(sp(Expr::StrLit("fail".to_string()))),
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Assert { msg: Some(_), .. }
        ));
    }

    #[test]
    fn test_lower_assert_no_message() {
        let hir = helper_lower(vec![sp(Stmt::Assert {
            test: sp(Expr::BoolLit(true)),
            msg: None,
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Assert {
                test: HirExpr::BoolLit(true, _),
                msg: None,
                ..
            }
        ));
    }

    // -------------------------------------------------------------------------
    // 19. Global / Nonlocal
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_global_decl() {
        // `global x` where `x` is not resolvable → syms vec will be empty
        let hir = helper_lower(vec![sp(Stmt::Global(vec!["g".to_string()]))]);
        assert!(matches!(&hir.top_level[0], HirStmt::Global { .. }));
    }

    #[test]
    fn test_lower_nonlocal_decl() {
        // `nonlocal x` at top level — syms vec will be empty (no outer scope)
        let hir = helper_lower(vec![sp(Stmt::Nonlocal(vec!["n".to_string()]))]);
        assert!(matches!(&hir.top_level[0], HirStmt::Nonlocal { .. }));
    }

    // -------------------------------------------------------------------------
    // 20. Call expressions
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_call_no_args() {
        // Function must be pre-registered so resolve_name can find "f".
        let hir = helper_lower_with_fns(
            vec![
                sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "f".to_string(),
                    type_params: vec![],
                    params: vec![],
                    return_ty: None,
                    body: vec![sp(Stmt::Return(None))],
                }),
                sp(Stmt::ExprStmt(sp(Expr::Call {
                    func: Box::new(sp(Expr::Ident("f".to_string()))),
                    args: vec![],
                }))),
            ],
            &["f"],
        );
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::Call { args, .. }, .. } if args.is_empty()
        ));
    }

    #[test]
    fn test_lower_call_with_positional_args() {
        let hir = helper_lower_with_fns(
            vec![
                sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "g".to_string(),
                    type_params: vec![],
                    params: vec![make_param("a"), make_param("b")],
                    return_ty: None,
                    body: vec![sp(Stmt::Return(None))],
                }),
                sp(Stmt::ExprStmt(sp(Expr::Call {
                    func: Box::new(sp(Expr::Ident("g".to_string()))),
                    args: vec![
                        CallArg::Positional(sp(Expr::IntLit(1))),
                        CallArg::Positional(sp(Expr::IntLit(2))),
                    ],
                }))),
            ],
            &["g"],
        );
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::Call { args, .. }, .. } if args.len() == 2
        ));
    }

    #[test]
    fn test_lower_call_with_kwargs() {
        let hir = helper_lower_with_fns(
            vec![
                sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "h".to_string(),
                    type_params: vec![],
                    params: vec![make_param("x")],
                    return_ty: None,
                    body: vec![sp(Stmt::Return(None))],
                }),
                sp(Stmt::ExprStmt(sp(Expr::Call {
                    func: Box::new(sp(Expr::Ident("h".to_string()))),
                    args: vec![CallArg::Keyword {
                        name: "x".to_string(),
                        value: sp(Expr::IntLit(42)),
                    }],
                }))),
            ],
            &["h"],
        );
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::Call { args, .. }, .. } if args.len() == 1
        ));
    }

    #[test]
    fn test_lower_method_call() {
        // obj.method() — attr access call
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "obj".to_string(),
                ty: sp(TypeExpr::Named("Any".to_string())),
                value: sp(Expr::IntLit(0)),
            }),
            sp(Stmt::ExprStmt(sp(Expr::Call {
                func: Box::new(sp(Expr::Attr {
                    object: Box::new(sp(Expr::Ident("obj".to_string()))),
                    attr: "method".to_string(),
                })),
                args: vec![],
            }))),
        ]);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Expr {
                expr: HirExpr::Call { func, .. }, ..
            } if matches!(func.as_ref(), HirExpr::Attr { .. })
        ));
    }

    // -------------------------------------------------------------------------
    // 21. Lambda expressions
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_lambda_simple() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::Lambda {
            params: vec![make_param("x")],
            body: Box::new(sp(Expr::Ident("x".to_string()))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::Lambda { params, .. }, .. } if params.len() == 1
        ));
    }

    #[test]
    fn test_lower_lambda_multi_params() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::Lambda {
            params: vec![make_param("a"), make_param("b")],
            body: Box::new(sp(Expr::BinOp {
                op: BinOp::Add,
                lhs: Box::new(sp(Expr::Ident("a".to_string()))),
                rhs: Box::new(sp(Expr::Ident("b".to_string()))),
            })),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::Lambda { params, .. }, .. } if params.len() == 2
        ));
    }

    // -------------------------------------------------------------------------
    // 22. If expression (ternary)
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_if_expr_true_branch() {
        // `1 if True else 0` — then_val is the first branch (body in AST)
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::IfExpr {
            body: Box::new(sp(Expr::IntLit(1))),
            condition: Box::new(sp(Expr::BoolLit(true))),
            else_body: Box::new(sp(Expr::IntLit(0))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::IfExpr { then_val, else_val, .. }, ..
            } if matches!(then_val.as_ref(), HirExpr::IntLit(1, _))
              && matches!(else_val.as_ref(), HirExpr::IntLit(0, _))
        ));
    }

    #[test]
    fn test_lower_if_expr_false_branch() {
        // `"yes" if False else "no"`
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::IfExpr {
            body: Box::new(sp(Expr::StrLit("yes".to_string()))),
            condition: Box::new(sp(Expr::BoolLit(false))),
            else_body: Box::new(sp(Expr::StrLit("no".to_string()))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::IfExpr { then_val, else_val, .. }, ..
            } if matches!(then_val.as_ref(), HirExpr::StrLit(s, _) if s == "yes")
              && matches!(else_val.as_ref(), HirExpr::StrLit(s, _) if s == "no")
        ));
    }

    // -------------------------------------------------------------------------
    // Extra: return empty, multiple top-level, list literal
    // -------------------------------------------------------------------------

    #[test]
    fn test_lower_return_no_value() {
        let hir = helper_lower(vec![sp(Stmt::Return(None))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Return { value: None, .. }
        ));
    }

    #[test]
    fn test_lower_list_lit_empty() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::ListLit(vec![]))))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::List { elements, .. }, .. } if elements.is_empty()
        ));
    }

    #[test]
    fn test_lower_list_lit_with_elems() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::ListLit(vec![
            sp(Expr::IntLit(1)),
            sp(Expr::IntLit(2)),
            sp(Expr::IntLit(3)),
        ]))))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::List { elements, .. }, .. } if elements.len() == 3
        ));
    }

    #[test]
    fn test_lower_dict_lit_empty() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::DictLit(vec![]))))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::Dict { entries, .. }, .. } if entries.is_empty()
        ));
    }

    #[test]
    fn test_lower_dict_lit_with_entries() {
        let entries = vec![
            (Some(sp(Expr::StrLit("a".to_string()))), sp(Expr::IntLit(1))),
            (Some(sp(Expr::StrLit("b".to_string()))), sp(Expr::IntLit(2))),
        ];
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::DictLit(entries))))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr { expr: HirExpr::Dict { entries, .. }, .. } if entries.len() == 2
        ));
    }

    #[test]
    fn test_lower_multiple_top_level_stmts() {
        let hir = helper_lower(vec![
            sp(Stmt::ExprStmt(sp(Expr::IntLit(1)))),
            sp(Stmt::ExprStmt(sp(Expr::IntLit(2)))),
            sp(Stmt::ExprStmt(sp(Expr::IntLit(3)))),
        ]);
        assert_eq!(hir.top_level.len(), 3);
    }

    #[test]
    fn test_lower_bool_false_literal() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BoolLit(false))))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BoolLit(false, _),
                ..
            }
        ));
    }

    #[test]
    fn test_lower_binop_lshift() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
            op: BinOp::LShift,
            lhs: Box::new(sp(Expr::IntLit(1))),
            rhs: Box::new(sp(Expr::IntLit(3))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    op: HirBinOp::LShift,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_binop_rshift() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
            op: BinOp::RShift,
            lhs: Box::new(sp(Expr::IntLit(8))),
            rhs: Box::new(sp(Expr::IntLit(2))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    op: HirBinOp::RShift,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_unary_bitnot() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::UnaryOp {
            op: UnaryOp::BitNot,
            operand: Box::new(sp(Expr::IntLit(7))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::UnaryOp {
                    op: HirUnaryOp::BitNot,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_attr_access() {
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "obj".to_string(),
                ty: sp(TypeExpr::Named("Any".to_string())),
                value: sp(Expr::IntLit(0)),
            }),
            sp(Stmt::ExprStmt(sp(Expr::Attr {
                object: Box::new(sp(Expr::Ident("obj".to_string()))),
                attr: "field".to_string(),
            }))),
        ]);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Expr { expr: HirExpr::Attr { attr, .. }, .. } if attr == "field"
        ));
    }

    #[test]
    fn test_lower_index_access() {
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "lst".to_string(),
                ty: sp(TypeExpr::Named("Any".to_string())),
                value: sp(Expr::ListLit(vec![sp(Expr::IntLit(1))])),
            }),
            sp(Stmt::ExprStmt(sp(Expr::Index {
                object: Box::new(sp(Expr::Ident("lst".to_string()))),
                index: Box::new(sp(Expr::IntLit(0))),
            }))),
        ]);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Expr {
                expr: HirExpr::Index { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_yield_with_value() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::Yield(Some(Box::new(
            sp(Expr::IntLit(42)),
        ))))))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::Yield { value: Some(_), .. },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_yield_from_expr() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::YieldFrom(Box::new(sp(
            Expr::ListLit(vec![]),
        ))))))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::YieldFrom { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_await_expr() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::Await(Box::new(sp(
            Expr::IntLit(1),
        ))))))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::Await { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_fn_is_not_generator_by_default() {
        let hir = helper_lower_with_fns(
            vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "plain".to_string(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::Return(Some(sp(Expr::IntLit(0)))))],
            })],
            &["plain"],
        );
        assert!(!hir.functions[0].is_generator);
    }

    #[test]
    fn test_lower_augassign_mul() {
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "z".to_string(),
                ty: sp(TypeExpr::Named("int".to_string())),
                value: sp(Expr::IntLit(3)),
            }),
            sp(Stmt::AugAssign {
                target: sp(Expr::Ident("z".to_string())),
                op: AugOp::Mul,
                value: sp(Expr::IntLit(2)),
            }),
        ]);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Assign {
                value: HirExpr::BinOp {
                    op: HirBinOp::Mul,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_try_with_named_handler() {
        let hir = helper_lower(vec![sp(Stmt::Try {
            body: vec![sp(Stmt::Break)],
            handlers: vec![ExceptHandler {
                exc_type: None,
                name: None,
                body: vec![sp(Stmt::Break)],
                is_star: false,
                span: Span::dummy(),
            }],
            else_body: Some(vec![sp(Stmt::Pass)]),
            finally_body: None,
        })]);
        if let HirStmt::Try {
            handlers,
            else_body,
            ..
        } = &hir.top_level[0]
        {
            assert_eq!(handlers.len(), 1);
            // else body: Pass produces no HIR
            assert!(else_body.is_empty());
        } else {
            panic!("expected Try");
        }
    }

    #[test]
    fn test_lower_binop_in_operator() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
            op: BinOp::In,
            lhs: Box::new(sp(Expr::IntLit(1))),
            rhs: Box::new(sp(Expr::ListLit(vec![sp(Expr::IntLit(1))]))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    op: HirBinOp::In,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_binop_not_in() {
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
            op: BinOp::NotIn,
            lhs: Box::new(sp(Expr::IntLit(0))),
            rhs: Box::new(sp(Expr::ListLit(vec![]))),
        })))]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    op: HirBinOp::NotIn,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_fn_with_decorator_produces_placeholder() {
        // A function with a resolvable decorator emits a FuncDefPlaceholder in top_level.
        // "staticmethod" is a builtin and IS resolvable, so we expect a placeholder.
        let hir = helper_lower_with_fns(
            vec![sp(Stmt::FnDef {
                decorators: vec![sp(Expr::Ident("staticmethod".to_string()))],
                name: "decorated".to_string(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::Return(None))],
            })],
            &["decorated"],
        );
        assert_eq!(hir.functions.len(), 1);
        // Decorator is resolved → placeholder IS emitted in top_level
        assert_eq!(hir.top_level.len(), 1);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::FuncDefPlaceholder { .. }
        ));
    }

    #[test]
    fn test_lower_bare_annotation_produces_nothing() {
        let hir = helper_lower(vec![sp(Stmt::BareAnnotation {
            name: "x".to_string(),
            ty: sp(TypeExpr::Named("int".to_string())),
        })]);
        assert!(
            hir.top_level.is_empty(),
            "BareAnnotation should emit no HIR"
        );
    }

    #[test]
    fn test_lower_fstring_only_literal() {
        let parts = vec![FStringPart::Literal("hello".to_string())];
        let hir = helper_lower(vec![sp(Stmt::ExprStmt(sp(Expr::FString(parts))))]);
        if let HirStmt::Expr {
            expr: HirExpr::FString { parts, .. },
            ..
        } = &hir.top_level[0]
        {
            assert_eq!(parts.len(), 1);
            assert!(matches!(&parts[0], HirFStringPart::Literal(s) if s == "hello"));
        } else {
            panic!("expected FString");
        }
    }

    #[test]
    fn test_lower_while_with_else_body() {
        let hir = helper_lower(vec![sp(Stmt::While {
            condition: sp(Expr::BoolLit(false)),
            body: vec![sp(Stmt::Pass)],
            else_body: Some(vec![sp(Stmt::Break)]),
        })]);
        assert!(matches!(
            &hir.top_level[0],
            HirStmt::While { else_body, .. } if else_body.len() == 1
        ));
    }

    #[test]
    fn test_lower_assign_implicit_let() {
        // `x = 5` where x is not yet declared → emits Let (implicit declaration)
        let hir = helper_lower(vec![sp(Stmt::Assign {
            target: sp(Expr::Ident("new_var".to_string())),
            value: sp(Expr::IntLit(5)),
        })]);
        assert_eq!(hir.top_level.len(), 1);
        assert!(matches!(&hir.top_level[0], HirStmt::Let { .. }));
    }

    #[test]
    fn test_lower_assign_to_attr() {
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "o".to_string(),
                ty: sp(TypeExpr::Named("Any".to_string())),
                value: sp(Expr::IntLit(0)),
            }),
            sp(Stmt::Assign {
                target: sp(Expr::Attr {
                    object: Box::new(sp(Expr::Ident("o".to_string()))),
                    attr: "x".to_string(),
                }),
                value: sp(Expr::IntLit(1)),
            }),
        ]);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Assign {
                target: HirLValue::Attr { .. },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_augassign_floordiv() {
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "x".to_string(),
                ty: sp(TypeExpr::Named("int".to_string())),
                value: sp(Expr::IntLit(10)),
            }),
            sp(Stmt::AugAssign {
                target: sp(Expr::Ident("x".to_string())),
                op: AugOp::FloorDiv,
                value: sp(Expr::IntLit(3)),
            }),
        ]);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Assign {
                value: HirExpr::BinOp {
                    op: HirBinOp::FloorDiv,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_augassign_pow() {
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "x".to_string(),
                ty: sp(TypeExpr::Named("int".to_string())),
                value: sp(Expr::IntLit(2)),
            }),
            sp(Stmt::AugAssign {
                target: sp(Expr::Ident("x".to_string())),
                op: AugOp::Pow,
                value: sp(Expr::IntLit(4)),
            }),
        ]);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Assign {
                value: HirExpr::BinOp {
                    op: HirBinOp::Pow,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_augassign_bitxor() {
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "x".to_string(),
                ty: sp(TypeExpr::Named("int".to_string())),
                value: sp(Expr::IntLit(0b1010)),
            }),
            sp(Stmt::AugAssign {
                target: sp(Expr::Ident("x".to_string())),
                op: AugOp::BitXor,
                value: sp(Expr::IntLit(0b1100)),
            }),
        ]);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Assign {
                value: HirExpr::BinOp {
                    op: HirBinOp::BitXor,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_augassign_bitor() {
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "x".to_string(),
                ty: sp(TypeExpr::Named("int".to_string())),
                value: sp(Expr::IntLit(0b0101)),
            }),
            sp(Stmt::AugAssign {
                target: sp(Expr::Ident("x".to_string())),
                op: AugOp::BitOr,
                value: sp(Expr::IntLit(0b1010)),
            }),
        ]);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Assign {
                value: HirExpr::BinOp {
                    op: HirBinOp::BitOr,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn test_lower_augassign_bitand() {
        let hir = helper_lower(vec![
            sp(Stmt::VarDecl {
                name: "x".to_string(),
                ty: sp(TypeExpr::Named("int".to_string())),
                value: sp(Expr::IntLit(0b1111)),
            }),
            sp(Stmt::AugAssign {
                target: sp(Expr::Ident("x".to_string())),
                op: AugOp::BitAnd,
                value: sp(Expr::IntLit(0b1010)),
            }),
        ]);
        assert!(matches!(
            &hir.top_level[1],
            HirStmt::Assign {
                value: HirExpr::BinOp {
                    op: HirBinOp::BitAnd,
                    ..
                },
                ..
            }
        ));
    }
}
