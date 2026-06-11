//! PEP 695 runtime desugaring (#233).
//!
//! Rewrites `def f[T](...)`, `class C[T]: ...`, and `type X = V` so the type
//! parameters and aliases exist as *runtime* objects with CPython semantics:
//!
//! * every `[T]`-syntax parameter becomes a runtime TypeVar instance bound to
//!   a same-named variable in the enclosing (non-class) scope, created by the
//!   `__mb_pep695_typevar__` intrinsic just before the def/class statement;
//! * the function/class gains a `__type_params__` tuple (classes also gain a
//!   matching `__parameters__`) assigned right after the definition;
//! * `type X = V` additionally binds `X` to a TypeAliasType instance built by
//!   the `__mb_pep695_type_alias__` intrinsic whose `__value__` is evaluated
//!   lazily via a zero-arg lambda (enabling recursive aliases). The original
//!   `TypeAlias` statement is kept for compile-time annotation resolution
//!   (it lowers to nothing).
//!
//! Bounds and constraints are wrapped in zero-arg lambdas so they are
//! evaluated lazily on first `__bound__` / `__constraints__` access, matching
//! CPython's deferred-evaluation semantics.
//!
//! Scope honesty: TypeVar bindings for parameters declared on methods or on
//! class-body aliases are hoisted to the nearest enclosing non-class scope
//! (class-body bindings are invisible to method bodies). This makes the
//! params *visible* in a wider scope than CPython's dedicated type-param
//! scope; PEP 695's scope-isolation corner cases (out-of-scope NameError) are
//! out of scope here.

use crate::parser::ast::{
    CallArg, Expr, Module, Stmt, TypeParam, TypeParamKind,
};
use crate::source::span::{Span, Spanned};

/// Name of the TypeVar-construction intrinsic (see `runtime::pep695`).
pub const TYPEVAR_INTRINSIC: &str = "__mb_pep695_typevar__";
/// Name of the TypeAliasType-construction intrinsic (see `runtime::pep695`).
pub const TYPE_ALIAS_INTRINSIC: &str = "__mb_pep695_type_alias__";

/// Desugar all PEP 695 constructs in a module (in place).
pub fn desugar_module(module: &mut Module) {
    let hoist = desugar_block(&mut module.stmts, false);
    // Module scope absorbs everything; nothing can hoist past it.
    debug_assert!(hoist.before.is_empty() && hoist.after.is_empty());
}

fn sp(node: Expr, span: Span) -> Spanned<Expr> {
    Spanned::new(node, span)
}

/// Build `name = __mb_pep695_typevar__("name", kind, bound, constraints)`.
fn typevar_assign(param: &TypeParam, span: Span) -> Spanned<Stmt> {
    let kind = match param.kind {
        TypeParamKind::TypeVar => 0,
        TypeParamKind::TypeVarTuple => 1,
        TypeParamKind::ParamSpec => 2,
    };
    let bound_arg = match &param.bound {
        Some(b) => sp(
            Expr::Lambda { params: vec![], body: Box::new(b.clone()) },
            b.span,
        ),
        None => sp(Expr::NoneLit, span),
    };
    let constraints_arg = match &param.constraints {
        Some(items) => sp(
            Expr::Lambda {
                params: vec![],
                body: Box::new(sp(Expr::TupleLit(items.clone()), span)),
            },
            span,
        ),
        None => sp(Expr::NoneLit, span),
    };
    let call = Expr::Call {
        func: Box::new(sp(Expr::Ident(TYPEVAR_INTRINSIC.to_string()), span)),
        args: vec![
            CallArg::Positional(sp(Expr::StrLit(param.name.clone()), span)),
            CallArg::Positional(sp(Expr::IntLit(kind), span)),
            CallArg::Positional(bound_arg),
            CallArg::Positional(constraints_arg),
        ],
    };
    Spanned::new(
        Stmt::Assign {
            target: sp(Expr::Ident(param.name.clone()), span),
            value: sp(call, span),
        },
        span,
    )
}

/// Build a `(P1, P2, ...)` tuple expression referencing the typevar bindings.
fn params_tuple(type_params: &[TypeParam], span: Span) -> Spanned<Expr> {
    sp(
        Expr::TupleLit(
            type_params
                .iter()
                .map(|p| sp(Expr::Ident(p.name.clone()), span))
                .collect(),
        ),
        span,
    )
}

/// Build `path[0].path[1]....attr = (P1, P2, ...)`.
fn attr_tuple_assign(
    path: &[Name],
    attr: &str,
    type_params: &[TypeParam],
    span: Span,
) -> Spanned<Stmt> {
    let mut object = sp(Expr::Ident(path[0].clone()), span);
    for seg in &path[1..] {
        object = sp(
            Expr::Attr { object: Box::new(object), attr: seg.clone() },
            span,
        );
    }
    Spanned::new(
        Stmt::Assign {
            target: sp(
                Expr::Attr { object: Box::new(object), attr: attr.to_string() },
                span,
            ),
            value: params_tuple(type_params, span),
        },
        span,
    )
}

use crate::parser::ast::Name;

/// Statements hoisted out of a class body.
///
/// * `before` — TypeVar bindings that must exist before the class statement
///   executes (class-level params, class-body alias params).
/// * `after` — per-definition TypeVar bindings plus the dotted
///   `Cls.meth.__type_params__ = (...)` assignments that must run after the
///   class exists. Method param bindings are deliberately placed *after* the
///   enclosing class's own `__type_params__` capture so a same-named method
///   param (`class Outer[T]: def meth[T]`) rebinds the shared name without
///   contaminating the class's already-captured tuple (method bodies resolve
///   the name dynamically at call time, so the later binding is the one they
///   see — matching the inner scope shadowing CPython gives them).
struct ClassHoists {
    before: Vec<Spanned<Stmt>>,
    after: Vec<AfterItem>,
}

/// One definition's deferred type-param wiring (see [`ClassHoists::after`]).
struct AfterItem {
    /// `T = __mb_pep695_typevar__(...)` bindings for this definition.
    tv_assigns: Vec<Spanned<Stmt>>,
    /// Attribute path *within* the enclosing class (e.g. `["meth"]`);
    /// each enclosing class prepends its own name while bubbling up.
    path: Vec<Name>,
    /// `(attr, params)` assignments, e.g. `("__type_params__", [T])`.
    attrs: Vec<(String, Vec<TypeParam>)>,
    span: Span,
}

/// Desugar one statement block. `in_class` marks class bodies, whose typevar
/// bindings must hoist out to the nearest non-class scope. Returns the
/// hoists destined for the enclosing block (non-empty only for class bodies).
fn desugar_block(stmts: &mut Vec<Spanned<Stmt>>, in_class: bool) -> ClassHoists {
    let old = std::mem::take(stmts);
    let mut out: Vec<Spanned<Stmt>> = Vec::new();
    let mut hoist_up = ClassHoists { before: Vec::new(), after: Vec::new() };

    // Emit one deferred wiring item into a non-class block.
    fn emit_after(out: &mut Vec<Spanned<Stmt>>, item: AfterItem) {
        out.extend(item.tv_assigns);
        for (attr, tps) in &item.attrs {
            out.push(attr_tuple_assign(&item.path, attr, tps, item.span));
        }
    }

    for mut st in old {
        let span = st.span;
        match &mut st.node {
            // ── class definitions (recursion + emission together) ──
            Stmt::ClassDef { name, type_params, body, .. } => {
                let name = name.clone();
                let tps = type_params.clone();
                let mut h = desugar_block(body, true);
                // Items leaving this class body gain its name as path prefix.
                for item in &mut h.after {
                    item.path.insert(0, name.clone());
                }
                let tv_assigns: Vec<Spanned<Stmt>> =
                    tps.iter().map(|p| typevar_assign(p, span)).collect();
                let attrs = if tps.is_empty() {
                    Vec::new()
                } else {
                    vec![
                        ("__type_params__".to_string(), tps.clone()),
                        // Generic classes also expose a matching __parameters__.
                        ("__parameters__".to_string(), tps),
                    ]
                };
                if in_class {
                    // Nested class: its body executes while the outer class
                    // body runs, so everything needed *before* keeps
                    // bubbling; the attribute wiring keeps deferring.
                    hoist_up.before.extend(h.before);
                    hoist_up.before.extend(tv_assigns);
                    out.push(st);
                    if !attrs.is_empty() {
                        hoist_up.after.push(AfterItem {
                            tv_assigns: Vec::new(),
                            path: vec![name],
                            attrs,
                            span,
                        });
                    }
                    hoist_up.after.extend(h.after);
                } else {
                    out.extend(h.before);
                    out.extend(tv_assigns);
                    out.push(st);
                    for (attr, ps) in &attrs {
                        out.push(attr_tuple_assign(&[name.clone()], attr, ps, span));
                    }
                    for item in h.after {
                        emit_after(&mut out, item);
                    }
                }
            }
            // ── function definitions ──
            Stmt::FnDef { name, type_params, body, .. }
            | Stmt::AsyncFnDef { name, type_params, body, .. } => {
                let name = name.clone();
                let tps = type_params.clone();
                let _ = desugar_block(body, false);
                if tps.is_empty() {
                    out.push(st);
                } else {
                    let tv_assigns: Vec<Spanned<Stmt>> =
                        tps.iter().map(|p| typevar_assign(p, span)).collect();
                    if in_class {
                        // Defer the whole wiring to after the enclosing class
                        // so a same-named class param's already-captured tuple
                        // stays intact (see ClassHoists docs).
                        out.push(st);
                        hoist_up.after.push(AfterItem {
                            tv_assigns,
                            path: vec![name],
                            attrs: vec![("__type_params__".to_string(), tps)],
                            span,
                        });
                    } else {
                        out.extend(tv_assigns);
                        out.push(st);
                        out.push(attr_tuple_assign(&[name], "__type_params__", &tps, span));
                    }
                }
            }
            // ── type aliases ──
            Stmt::TypeAlias { name, type_params, value } => {
                let name = name.clone();
                let tps = type_params.clone();
                let value = value.clone();
                // Param typevars must exist when the alias's constructor call
                // runs (it captures the params tuple eagerly), so they go
                // before the (outermost) class statement when in a class body.
                let tv_assigns: Vec<Spanned<Stmt>> =
                    tps.iter().map(|p| typevar_assign(p, span)).collect();
                if in_class {
                    hoist_up.before.extend(tv_assigns);
                } else {
                    out.extend(tv_assigns);
                }
                // Keep the original statement: the type checker registers the
                // compile-time alias from it; HIR lowering skips it.
                out.push(st);
                // Placeholder binding (an empty TypeAliasType, type `any`) so
                // the lazy value thunk below may reference the alias's own
                // name — recursive aliases (`type R = R | None`) resolve.
                let placeholder = Expr::Call {
                    func: Box::new(sp(
                        Expr::Ident(TYPE_ALIAS_INTRINSIC.to_string()),
                        span,
                    )),
                    args: vec![
                        CallArg::Positional(sp(Expr::StrLit(name.clone()), span)),
                        CallArg::Positional(sp(Expr::NoneLit, span)),
                        CallArg::Positional(sp(Expr::TupleLit(vec![]), span)),
                    ],
                };
                out.push(Spanned::new(
                    Stmt::Assign {
                        target: sp(Expr::Ident(name.clone()), span),
                        value: sp(placeholder, span),
                    },
                    span,
                ));
                // X = __mb_pep695_type_alias__("X", lambda: V, (params...))
                let thunk = sp(
                    Expr::Lambda { params: vec![], body: Box::new(value) },
                    span,
                );
                let call = Expr::Call {
                    func: Box::new(sp(
                        Expr::Ident(TYPE_ALIAS_INTRINSIC.to_string()),
                        span,
                    )),
                    args: vec![
                        CallArg::Positional(sp(Expr::StrLit(name.clone()), span)),
                        CallArg::Positional(thunk),
                        CallArg::Positional(params_tuple(&tps, span)),
                    ],
                };
                out.push(Spanned::new(
                    Stmt::Assign {
                        target: sp(Expr::Ident(name), span),
                        value: sp(call, span),
                    },
                    span,
                ));
            }
            // ── same-scope compound statements ──
            _ => {
                // Route hoists from a same-scope sub-block (if/try/...):
                // for non-class blocks emit immediately (paths complete);
                // for class blocks keep bubbling.
                let mut route = |h: ClassHoists,
                                 out: &mut Vec<Spanned<Stmt>>,
                                 hoist_up: &mut ClassHoists| {
                    if in_class {
                        hoist_up.before.extend(h.before);
                        hoist_up.after.extend(h.after);
                    } else {
                        out.extend(h.before);
                        for item in h.after {
                            emit_after(out, item);
                        }
                    }
                };
                match &mut st.node {
                    Stmt::If { body, elif_clauses, else_body, .. } => {
                        let h = desugar_block(body, in_class);
                        route(h, &mut out, &mut hoist_up);
                        for (_, b) in elif_clauses {
                            let h = desugar_block(b, in_class);
                            route(h, &mut out, &mut hoist_up);
                        }
                        if let Some(b) = else_body {
                            let h = desugar_block(b, in_class);
                            route(h, &mut out, &mut hoist_up);
                        }
                    }
                    Stmt::While { body, else_body, .. }
                    | Stmt::For { body, else_body, .. }
                    | Stmt::AsyncFor { body, else_body, .. } => {
                        let h = desugar_block(body, in_class);
                        route(h, &mut out, &mut hoist_up);
                        if let Some(b) = else_body {
                            let h = desugar_block(b, in_class);
                            route(h, &mut out, &mut hoist_up);
                        }
                    }
                    Stmt::With { body, .. } | Stmt::AsyncWith { body, .. } => {
                        let h = desugar_block(body, in_class);
                        route(h, &mut out, &mut hoist_up);
                    }
                    Stmt::Try { body, handlers, else_body, finally_body } => {
                        let h = desugar_block(body, in_class);
                        route(h, &mut out, &mut hoist_up);
                        for handler in handlers {
                            let h = desugar_block(&mut handler.body, in_class);
                            route(h, &mut out, &mut hoist_up);
                        }
                        if let Some(b) = else_body {
                            let h = desugar_block(b, in_class);
                            route(h, &mut out, &mut hoist_up);
                        }
                        if let Some(b) = finally_body {
                            let h = desugar_block(b, in_class);
                            route(h, &mut out, &mut hoist_up);
                        }
                    }
                    Stmt::Match { arms, .. } => {
                        for arm in arms {
                            let h = desugar_block(&mut arm.body, in_class);
                            route(h, &mut out, &mut hoist_up);
                        }
                    }
                    _ => {}
                }
                out.push(st);
            }
        }
    }

    *stmts = out;
    hoist_up
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::source::span::FileId;

    fn desugared(src: &str) -> Module {
        let mut module = parser::parse(src, FileId(0)).expect("parse failed");
        desugar_module(&mut module);
        module
    }

    #[test]
    fn generic_fn_injects_typevar_and_type_params() {
        let m = desugared("def f[T]():\n    return T\n");
        // T = __mb_pep695_typevar__(...), def f, f.__type_params__ = (T,)
        assert_eq!(m.stmts.len(), 3);
        assert!(matches!(
            &m.stmts[0].node,
            Stmt::Assign { target, .. }
                if matches!(&target.node, Expr::Ident(n) if n == "T")
        ));
        assert!(matches!(&m.stmts[1].node, Stmt::FnDef { .. }));
        assert!(matches!(
            &m.stmts[2].node,
            Stmt::Assign { target, .. }
                if matches!(&target.node, Expr::Attr { attr, .. } if attr == "__type_params__")
        ));
    }

    #[test]
    fn generic_class_injects_parameters_too() {
        let m = desugared("class C[T]:\n    pass\n");
        assert_eq!(m.stmts.len(), 4);
        assert!(matches!(&m.stmts[1].node, Stmt::ClassDef { .. }));
        let attrs: Vec<&str> = m.stmts[2..]
            .iter()
            .filter_map(|s| match &s.node {
                Stmt::Assign { target, .. } => match &target.node {
                    Expr::Attr { attr, .. } => Some(attr.as_str()),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        assert_eq!(attrs, vec!["__type_params__", "__parameters__"]);
    }

    #[test]
    fn method_typevars_hoist_out_of_class_body() {
        let m = desugared(
            "class C:\n    def meth[U](self):\n        return U\n",
        );
        // Deferred wiring: class first, then U = typevar(...), then
        // `C.meth.__type_params__ = (U,)` (dotted path).
        assert_eq!(m.stmts.len(), 3);
        match &m.stmts[0].node {
            Stmt::ClassDef { body, .. } => {
                assert_eq!(body.len(), 1);
                assert!(matches!(&body[0].node, Stmt::FnDef { .. }));
            }
            other => panic!("expected ClassDef, got {other:?}"),
        }
        assert!(matches!(
            &m.stmts[1].node,
            Stmt::Assign { target, .. }
                if matches!(&target.node, Expr::Ident(n) if n == "U")
        ));
        match &m.stmts[2].node {
            Stmt::Assign { target, .. } => match &target.node {
                Expr::Attr { object, attr } => {
                    assert_eq!(attr, "__type_params__");
                    assert!(matches!(
                        &object.node,
                        Expr::Attr { object: cls, attr: meth }
                            if meth == "meth"
                                && matches!(&cls.node, Expr::Ident(n) if n == "C")
                    ));
                }
                other => panic!("expected dotted Attr target, got {other:?}"),
            },
            other => panic!("expected Assign, got {other:?}"),
        }
    }

    #[test]
    fn type_alias_keeps_stmt_and_adds_binding() {
        let m = desugared("type X = int\n");
        // TypeAlias + placeholder binding + real binding.
        assert_eq!(m.stmts.len(), 3);
        assert!(matches!(&m.stmts[0].node, Stmt::TypeAlias { .. }));
        for s in &m.stmts[1..] {
            assert!(matches!(
                &s.node,
                Stmt::Assign { target, .. }
                    if matches!(&target.node, Expr::Ident(n) if n == "X")
            ));
        }
    }
}
