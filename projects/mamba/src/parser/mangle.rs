//! Private name mangling (CPython `_Py_Mangle`, PEP-classic).
//!
//! Inside a class body, an identifier of the form `__spam` (at least two
//! leading underscores, not ending in two underscores) is textually rewritten
//! to `_<classname>__spam`, where the class name has its own leading
//! underscores stripped. Mangling applies to every such identifier within the
//! *textually enclosing* class — including nested functions/lambdas — but a
//! nested class re-roots mangling under its own name.
//!
//! Run as a post-parse AST pass so the type checker and the HIR lowering both
//! observe the mangled names (attributes, locals, parameters, captures).

use super::ast::*;
use crate::source::span::Spanned;

/// Rewrite every mangled private name in the module in place.
pub fn mangle_module(module: &mut Module) {
    for stmt in &mut module.stmts {
        mangle_stmt(stmt, None);
    }
}

/// The mangled form of `name` under class context `class`, or `None` when the
/// name is not eligible (no class context, missing leading `__`, trailing
/// `__`, or an all-underscore class name).
fn mangled(name: &str, class: Option<&str>) -> Option<String> {
    let class = class?;
    if !name.starts_with("__") || name.ends_with("__") {
        return None;
    }
    let stripped = class.trim_start_matches('_');
    if stripped.is_empty() {
        return None;
    }
    Some(format!("_{stripped}{name}"))
}

fn mangle_in_place(name: &mut String, class: Option<&str>) {
    if let Some(m) = mangled(name, class) {
        *name = m;
    }
}

fn mangle_stmt(stmt: &mut Spanned<Stmt>, class: Option<&str>) {
    match &mut stmt.node {
        Stmt::VarDecl { name, value, .. } => {
            mangle_in_place(name, class);
            mangle_expr(value, class);
        }
        Stmt::Assign { target, value } => {
            mangle_expr(target, class);
            mangle_expr(value, class);
        }
        Stmt::AugAssign { target, value, .. } => {
            mangle_expr(target, class);
            mangle_expr(value, class);
        }
        Stmt::FnDef { decorators, name, params, body, .. }
        | Stmt::AsyncFnDef { decorators, name, params, body, .. } => {
            for d in decorators.iter_mut() {
                mangle_expr(d, class);
            }
            mangle_in_place(name, class);
            for p in params.iter_mut() {
                mangle_param(p, class);
            }
            // Nested functions inherit the enclosing class's mangling context.
            for s in body.iter_mut() {
                mangle_stmt(s, class);
            }
        }
        Stmt::ClassDef { decorators, name, bases, keyword_args, body, .. } => {
            for d in decorators.iter_mut() {
                mangle_expr(d, class);
            }
            for b in bases.iter_mut() {
                mangle_expr(b, class);
            }
            for (_, e) in keyword_args.iter_mut() {
                mangle_expr(e, class);
            }
            // The body mangles under THIS class's (original) name; the class's
            // own name is an identifier in the enclosing context.
            let inner = name.clone();
            mangle_in_place(name, class);
            for s in body.iter_mut() {
                mangle_stmt(s, Some(&inner));
            }
        }
        Stmt::EnumDef { variants, .. } => {
            for v in variants.iter_mut() {
                mangle_in_place(&mut v.name, class);
            }
        }
        Stmt::If { condition, body, elif_clauses, else_body, .. } => {
            mangle_expr(condition, class);
            mangle_stmts(body, class);
            for (cond, eb) in elif_clauses.iter_mut() {
                mangle_expr(cond, class);
                mangle_stmts(eb, class);
            }
            if let Some(eb) = else_body {
                mangle_stmts(eb, class);
            }
        }
        Stmt::While { condition, body, else_body, .. } => {
            mangle_expr(condition, class);
            mangle_stmts(body, class);
            if let Some(eb) = else_body {
                mangle_stmts(eb, class);
            }
        }
        Stmt::For { targets, iter, body, else_body, .. }
        | Stmt::AsyncFor { targets, iter, body, else_body, .. } => {
            for t in targets.iter_mut() {
                mangle_in_place(t, class);
            }
            mangle_expr(iter, class);
            mangle_stmts(body, class);
            if let Some(eb) = else_body {
                mangle_stmts(eb, class);
            }
        }
        Stmt::Match { expr, arms } => {
            mangle_expr(expr, class);
            for arm in arms.iter_mut() {
                mangle_pattern(&mut arm.pattern, class);
                if let Some(g) = &mut arm.guard {
                    mangle_expr(g, class);
                }
                mangle_stmts(&mut arm.body, class);
            }
        }
        Stmt::Return(Some(e)) => mangle_expr(e, class),
        Stmt::ExprStmt(e) => mangle_expr(e, class),
        Stmt::Del(e) => mangle_expr(e, class),
        Stmt::Assert { test, msg } => {
            mangle_expr(test, class);
            if let Some(m) = msg {
                mangle_expr(m, class);
            }
        }
        Stmt::Raise { value, from } => {
            if let Some(v) = value {
                mangle_expr(v, class);
            }
            if let Some(f) = from {
                mangle_expr(f, class);
            }
        }
        Stmt::Try { body, handlers, else_body, finally_body } => {
            mangle_stmts(body, class);
            for h in handlers.iter_mut() {
                if let Some(t) = &mut h.exc_type {
                    mangle_expr(t, class);
                }
                if let Some(n) = &mut h.name {
                    mangle_in_place(n, class);
                }
                mangle_stmts(&mut h.body, class);
            }
            if let Some(eb) = else_body {
                mangle_stmts(eb, class);
            }
            if let Some(fb) = finally_body {
                mangle_stmts(fb, class);
            }
        }
        Stmt::With { items, body, .. } | Stmt::AsyncWith { items, body, .. } => {
            for item in items.iter_mut() {
                mangle_expr(&mut item.context, class);
                if let Some(a) = &mut item.alias {
                    mangle_in_place(a, class);
                }
            }
            mangle_stmts(body, class);
        }
        Stmt::BareAnnotation { name, .. } => mangle_in_place(name, class),
        Stmt::Global(names) | Stmt::Nonlocal(names) => {
            for n in names.iter_mut() {
                mangle_in_place(n, class);
            }
        }
        // No mangleable identifiers / sub-expressions.
        Stmt::Return(None)
        | Stmt::Pass
        | Stmt::Break
        | Stmt::Continue
        | Stmt::Import { .. }
        | Stmt::TypeAlias { .. } => {}
    }
}

fn mangle_stmts(stmts: &mut [Spanned<Stmt>], class: Option<&str>) {
    for s in stmts.iter_mut() {
        mangle_stmt(s, class);
    }
}

fn mangle_param(p: &mut Param, class: Option<&str>) {
    mangle_in_place(&mut p.name, class);
    if let Some(d) = &mut p.default {
        mangle_expr(d, class);
    }
}

fn mangle_expr(expr: &mut Spanned<Expr>, class: Option<&str>) {
    match &mut expr.node {
        Expr::Ident(name) => mangle_in_place(name, class),
        Expr::Attr { object, attr } => {
            mangle_expr(object, class);
            mangle_in_place(attr, class);
        }
        Expr::BinOp { lhs, rhs, .. } => {
            mangle_expr(lhs, class);
            mangle_expr(rhs, class);
        }
        Expr::UnaryOp { operand, .. } => mangle_expr(operand, class),
        Expr::Call { func, args } => {
            mangle_expr(func, class);
            for arg in args.iter_mut() {
                match arg {
                    // Keyword argument *names* are not mangled (CPython binds
                    // them by literal name); only the value expression is.
                    CallArg::Positional(e)
                    | CallArg::StarArg(e)
                    | CallArg::DoubleStarArg(e)
                    | CallArg::Keyword { value: e, .. } => mangle_expr(e, class),
                }
            }
        }
        Expr::Index { object, index } => {
            mangle_expr(object, class);
            mangle_expr(index, class);
        }
        Expr::Slice { start, stop, step } => {
            if let Some(s) = start {
                mangle_expr(s, class);
            }
            if let Some(s) = stop {
                mangle_expr(s, class);
            }
            if let Some(s) = step {
                mangle_expr(s, class);
            }
        }
        Expr::ListLit(elems) | Expr::SetLit(elems) | Expr::TupleLit(elems) => {
            for e in elems.iter_mut() {
                mangle_expr(e, class);
            }
        }
        Expr::DictLit(entries) => {
            for (k, v) in entries.iter_mut() {
                if let Some(k) = k {
                    mangle_expr(k, class);
                }
                mangle_expr(v, class);
            }
        }
        Expr::IfExpr { body, condition, else_body } => {
            mangle_expr(body, class);
            mangle_expr(condition, class);
            mangle_expr(else_body, class);
        }
        Expr::Lambda { params, body } => {
            for p in params.iter_mut() {
                mangle_param(p, class);
            }
            mangle_expr(body, class);
        }
        Expr::ListComp { element, generators }
        | Expr::SetComp { element, generators }
        | Expr::GeneratorExpr { element, generators } => {
            mangle_comprehensions(generators, class);
            mangle_expr(element, class);
        }
        Expr::DictComp { key, value, generators } => {
            mangle_comprehensions(generators, class);
            mangle_expr(key, class);
            mangle_expr(value, class);
        }
        Expr::Walrus { target, value } => {
            mangle_in_place(target, class);
            mangle_expr(value, class);
        }
        Expr::FString(parts) => mangle_fstring(parts, class),
        Expr::Yield(Some(e)) | Expr::YieldFrom(e) | Expr::Await(e) | Expr::Starred(e) => {
            mangle_expr(e, class)
        }
        Expr::UnpackTarget(elems) => {
            for e in elems.iter_mut() {
                mangle_expr(e, class);
            }
        }
        Expr::ChainedCompare { operands, .. } => {
            for e in operands.iter_mut() {
                mangle_expr(e, class);
            }
        }
        Expr::Yield(None)
        | Expr::IntLit(_)
        | Expr::FloatLit(_)
        | Expr::ComplexLit(_)
        | Expr::StrLit(_)
        | Expr::BytesLit(_)
        | Expr::BoolLit(_)
        | Expr::NoneLit
        | Expr::Ellipsis => {}
    }
}

fn mangle_comprehensions(generators: &mut [Comprehension], class: Option<&str>) {
    for gen in generators.iter_mut() {
        for t in gen.targets.iter_mut() {
            mangle_in_place(t, class);
        }
        mangle_expr(&mut gen.iter, class);
        for c in gen.conditions.iter_mut() {
            mangle_expr(c, class);
        }
    }
}

fn mangle_fstring(parts: &mut [FStringPart], class: Option<&str>) {
    for part in parts.iter_mut() {
        if let FStringPart::Expr(e, spec) = part {
            mangle_expr(e, class);
            if let Some(sp) = spec {
                mangle_fstring(sp, class);
            }
        }
    }
}

fn mangle_pattern(pat: &mut Spanned<Pattern>, class: Option<&str>) {
    match &mut pat.node {
        Pattern::Binding(name) => mangle_in_place(name, class),
        Pattern::Literal(e) => {
            let mut tmp = Spanned { node: std::mem::replace(e, Expr::NoneLit), span: pat.span };
            mangle_expr(&mut tmp, class);
            *e = tmp.node;
        }
        Pattern::Or(pats) | Pattern::Sequence(pats) => {
            for p in pats.iter_mut() {
                mangle_pattern(p, class);
            }
        }
        Pattern::Mapping { pairs, rest } => {
            for (k, p) in pairs.iter_mut() {
                mangle_expr(k, class);
                mangle_pattern(p, class);
            }
            if let Some(r) = rest {
                mangle_in_place(r, class);
            }
        }
        Pattern::ClassPattern { patterns, .. } => {
            for (_, p) in patterns.iter_mut() {
                mangle_pattern(p, class);
            }
        }
        Pattern::Star(Some(name)) => mangle_in_place(name, class),
        Pattern::As { pattern, name } => {
            mangle_pattern(pattern, class);
            mangle_in_place(name, class);
        }
        Pattern::Constructor { fields, .. } => {
            for f in fields.iter_mut() {
                mangle_in_place(f, class);
            }
        }
        Pattern::Wildcard | Pattern::Star(None) => {}
    }
}
