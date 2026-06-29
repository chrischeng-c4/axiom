use super::check::TypeChecker;
use super::generic::{check_bounds, infer_type_args};
use super::{Ty, TypeId};
use crate::parser::ast::*;
use crate::resolve::SymbolKind;
use crate::source::span::{Span, Spanned};

thread_local! {
    /// ① Type-wall PoC: when set, `check_stdlib_call` suppresses CONSTRUCTOR /
    /// METHOD argument enforcement for the call it is currently checking.
    ///
    /// This is the "expected to raise at runtime" carve-out. The auto-ported
    /// CPython idiom
    ///
    /// ```python
    /// try:
    ///     Cls(wrong_arg)            # probe that MUST raise at runtime
    ///     raise AssertionError(...) # never reached when the probe raises
    /// except TypeError:
    ///     pass
    /// ```
    ///
    /// makes the program's CORRECT output depend on `Cls(wrong_arg)` raising at
    /// RUNTIME (so the trailing `raise` is skipped and the `except` swallows it).
    /// A compile-time `argument type mismatch` would abort the whole module
    /// before it runs, turning a behavior PASS into a RED. The ① type-wall
    /// fixtures never use this idiom — their probe is followed by a `print`, not
    /// a `raise` — so suppressing on a trailing `raise` keeps every type gain
    /// while eliminating the behavior false positive. `check_stmt` sets this flag
    /// (via [`set_stdlib_arg_check_suppressed`] / [`restore_stdlib_arg_check`])
    /// only while checking an `ExprStmt` whose immediate sibling is a `raise`.
    static SUPPRESS_STDLIB_ARG_CHECK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// ① Type-wall PoC: is constructor/method/module-fn arg enforcement currently
/// suppressed for the call being checked? (see [`SUPPRESS_STDLIB_ARG_CHECK`]).
pub(crate) fn stdlib_arg_check_suppressed() -> bool {
    SUPPRESS_STDLIB_ARG_CHECK.with(|c| c.get())
}

/// ① Type-wall PoC: set the suppress flag, returning its previous value so the
/// caller can restore it. Paired with [`restore_stdlib_arg_check`]. Kept as bare
/// set/restore (not an RAII closure) so the caller can run `&mut self` methods
/// inside the suppressed window without a borrow conflict.
pub(crate) fn set_stdlib_arg_check_suppressed(v: bool) -> bool {
    SUPPRESS_STDLIB_ARG_CHECK.with(|c| c.replace(v))
}

/// ① Type-wall PoC: restore the suppress flag to a value previously returned by
/// [`set_stdlib_arg_check_suppressed`].
pub(crate) fn restore_stdlib_arg_check(prev: bool) {
    SUPPRESS_STDLIB_ARG_CHECK.with(|c| c.set(prev));
}

fn is_pep695_lazy_thunk_arg(
    func_name: Option<&str>,
    positional_index: usize,
    arg: &Spanned<Expr>,
) -> bool {
    matches!(arg.node, Expr::Lambda { .. })
        && matches!(
            (func_name, positional_index),
            (Some("__mb_pep695_typevar__"), 2 | 3) | (Some("__mb_pep695_type_alias__"), 1)
        )
}

/// Expression, operator, and pattern type checking.
impl TypeChecker {
    pub(crate) fn check_expr(&mut self, expr: &Spanned<Expr>) -> TypeId {
        match &expr.node {
            Expr::IntLit(_) | Expr::BigIntLit(_) => self.tcx.int(),
            Expr::FloatLit(_) => self.tcx.float(),
            Expr::ComplexLit(_) => self.tcx.any(), // heap ObjData::Complex (ast_to_hir lowers to `complex(0, N)`)
            Expr::StrLit(_) => self.tcx.str(),
            Expr::FString(parts) => {
                // Walk replacement fields for their binding side effects
                // (walrus targets must be declared in the enclosing scope:
                // `f"{(z := 10)}"` leaks z), but suppress any new type
                // errors — field expressions are formatted dynamically and
                // were historically unchecked.
                fn walk(checker: &mut TypeChecker, parts: &[crate::parser::ast::FStringPart]) {
                    for p in parts {
                        if let crate::parser::ast::FStringPart::Expr(e, spec) = p {
                            let mark = checker.errors_mark();
                            let _ = checker.check_expr(e);
                            checker.truncate_errors(mark);
                            if let Some(sp) = spec {
                                walk(checker, sp);
                            }
                        }
                    }
                }
                walk(self, parts);
                self.tcx.str()
            }
            Expr::BytesLit(_) => self.tcx.any(),
            Expr::BoolLit(_) => self.tcx.bool(),
            Expr::NoneLit => self.tcx.none(),
            // `...` is a real runtime singleton (the `ellipsis` type) — type
            // it as Any so stub bodies and Ellipsis-valued expressions
            // compile and lower to the interned Ellipsis value.
            Expr::Ellipsis => self.tcx.any(),
            Expr::Ident(name) => {
                match self.symbols.lookup(name) {
                    Some(sym) => self.get_sym_type(sym.0),
                    None => {
                        // #1588: Python defers free-name lookup inside fn bodies to
                        // call time. If we're inside a function (current_return_ty is
                        // set), treat the undefined name as Any rather than erroring.
                        // Module-level free names stay hard errors.
                        if self.current_return_ty.is_some() || self.allow_runtime_unresolved_names {
                            self.tcx.any()
                        } else {
                            self.error(expr.span, format!("undefined name: `{name}`"));
                            self.tcx.error()
                        }
                    }
                }
            }
            Expr::BinOp { op, lhs, rhs } => {
                let lt = self.check_expr(lhs);
                let rt = self.check_expr(rhs);
                self.check_binop(*op, lt, rt, expr.span)
            }
            Expr::UnaryOp { op, operand } => {
                let ot = self.check_expr(operand);
                match op {
                    UnaryOp::Pos => {
                        // Bool is a subtype of int in Python — `+True == 1`,
                        // `+False == 0`, `type(+True) is int`.
                        if !matches!(
                            self.tcx.get(ot),
                            Ty::Int | Ty::Float | Ty::Bool | Ty::Error | Ty::Any
                        ) {
                            self.error(operand.span, "unary `+` requires numeric type");
                        }
                        if matches!(self.tcx.get(ot), Ty::Bool) {
                            self.tcx.int()
                        } else {
                            ot
                        }
                    }
                    UnaryOp::Neg => {
                        // Bool is a subtype of int — `-True == -1`, `-False == 0`,
                        // `type(-True) is int`.
                        if !matches!(
                            self.tcx.get(ot),
                            Ty::Int | Ty::Float | Ty::Bool | Ty::Error | Ty::Any
                        ) {
                            self.error(operand.span, "unary `-` requires numeric type");
                        }
                        if matches!(self.tcx.get(ot), Ty::Bool) {
                            self.tcx.int()
                        } else {
                            ot
                        }
                    }
                    UnaryOp::Not => {
                        // Python `not` works on any type via truthiness testing.
                        // Always returns bool.
                        self.tcx.bool()
                    }
                    UnaryOp::BitNot => {
                        // Bool is a subtype of int — ~True == -2, ~False == -1
                        if !matches!(self.tcx.get(ot), Ty::Int | Ty::Bool | Ty::Error | Ty::Any) {
                            self.error(operand.span, "`~` requires int type");
                        }
                        self.tcx.int()
                    }
                }
            }
            Expr::Call { func, args } => {
                let func_ty_id = self.check_expr(func);
                let func_ty = self.tcx.get(func_ty_id).clone();
                let func_name = if let Expr::Ident(n) = &func.node {
                    Some(n.clone())
                } else {
                    None
                };
                // ① Type-wall PoC HOOK: stdlib argument enforcement. ADDITIVE —
                // runs before the existing `match func_ty` and never changes the
                // Any-path return. It only *emits* the existing arg-mismatch
                // error when a known stdlib signature is genuinely violated by a
                // concrete-scalar argument. Skip-when-unsure at every branch.
                self.check_stdlib_call(func, args);
                self.check_dict_operator_call(func, args);
                match func_ty {
                    Ty::Fn {
                        params,
                        ret,
                        variadic,
                    } => {
                        let has_star = args
                            .iter()
                            .any(|a| matches!(a, CallArg::StarArg(_) | CallArg::DoubleStarArg(_)));
                        let has_kwargs = args.iter().any(|a| matches!(a, CallArg::Keyword { .. }));
                        let positional_count = args
                            .iter()
                            .filter(|a| matches!(a, CallArg::Positional(_)))
                            .count();
                        // Skip arity check when spread args, kwargs, or fewer-than-max
                        // positional args are present (defaults fill the gap at lowering).
                        // Includes the zero-arg case (#1600): defaults aren't surfaced
                        // through `Ty::Fn`, so an all-default fn looks identical to a
                        // required-arg fn at this layer; lowering / runtime catches
                        // genuinely-missing required args.
                        let might_have_defaults = positional_count < params.len();
                        if !has_star && !has_kwargs && !might_have_defaults {
                            if variadic {
                                // Variadic: only check minimum args
                                if positional_count < params.len() {
                                    self.error(
                                        expr.span,
                                        format!(
                                            "expected at least {} arguments, got {}",
                                            params.len(),
                                            positional_count,
                                        ),
                                    );
                                }
                            } else if positional_count != params.len() {
                                self.error(
                                    expr.span,
                                    format!(
                                        "expected {} arguments, got {}",
                                        params.len(),
                                        positional_count,
                                    ),
                                );
                            }
                        }
                        let mut arg_types = Vec::new();
                        let mut param_idx = 0;
                        for arg in args {
                            match arg {
                                CallArg::Positional(a) => {
                                    let at = if is_pep695_lazy_thunk_arg(
                                        func_name.as_deref(),
                                        param_idx,
                                        a,
                                    ) {
                                        self.tcx.any()
                                    } else {
                                        self.check_expr(a)
                                    };
                                    arg_types.push(at);
                                    if matches!(
                                        func_name.as_deref(),
                                        Some("isinstance" | "issubclass")
                                    ) && param_idx == 1
                                        && matches!(a.node, Expr::StrLit(_))
                                    {
                                        self.error(
                                            a.span,
                                            format!(
                                                "{}() arg 2 must be a type or tuple of types",
                                                func_name.as_deref().unwrap_or("isinstance"),
                                            ),
                                        );
                                    }
                                    if let Some(&expected) = params.get(param_idx) {
                                        // chr/hex/oct/bin accept any class
                                        // defining __index__ (SupportsIndex
                                        // protocol — CPython calls the dunder
                                        // at runtime). The wall stays up for
                                        // scalars without the protocol
                                        // (chr(1.5) is still rejected here).
                                        let index_protocol_ok = matches!(
                                            func_name.as_deref(),
                                            Some("chr" | "hex" | "oct" | "bin")
                                        ) && matches!(
                                            self.tcx.get(expected),
                                            Ty::Int
                                        ) && match self.tcx.get(at) {
                                            Ty::Class { name, .. } => self
                                                .class_methods
                                                .get(name)
                                                .is_some_and(|m| m.contains_key("__index__")),
                                            _ => false,
                                        };
                                        if !index_protocol_ok
                                            && !self.types_compatible(expected, at)
                                        {
                                            self.error(
                                                a.span,
                                                format!(
                                                    "argument type mismatch: expected `{}`, got `{}`",
                                                    self.ty_name(expected),
                                                    self.ty_name(at),
                                                ),
                                            );
                                        }
                                    }
                                    param_idx += 1;
                                }
                                CallArg::Keyword { value, .. } => {
                                    self.check_expr(value);
                                }
                                CallArg::StarArg(a) | CallArg::DoubleStarArg(a) => {
                                    self.check_expr(a);
                                }
                            }
                        }
                        // If generic function, infer type args and check bounds
                        if let Some(ref fname) = func_name {
                            if let Some(gp) = self.generic_defs.get(fname).cloned() {
                                let (subst, conflicts) =
                                    infer_type_args(&gp, &params, &arg_types, &self.tcx);
                                for err in conflicts {
                                    self.error(expr.span, err);
                                }
                                let bound_errors = check_bounds(&subst, &gp, &self.tcx);
                                for err in bound_errors {
                                    self.error(expr.span, err);
                                }
                                let applied = subst.apply(ret, &mut self.tcx);
                                // ABI honesty: a bare-TypeVar return crosses
                                // the call boundary as a boxed MbValue in the
                                // integer register (the generic callee
                                // compiles to the boxed I64 ABI). Substituting
                                // `float` would make codegen read an F64
                                // register that was never written — degrade to
                                // Any so the boxed value is handled
                                // dynamically. Int/Bool share the I64 register
                                // file and round-trip unchanged.
                                if matches!(self.tcx.get(ret), Ty::TypeVar(_))
                                    && matches!(self.tcx.get(applied), Ty::Float)
                                {
                                    return self.tcx.any();
                                }
                                return applied;
                            }
                        }
                        ret
                    }
                    // #246: calling a class constructor returns instance of that class
                    Ty::Class {
                        name: ref class_name,
                        ..
                    } => {
                        for arg in args {
                            self.check_call_arg(arg);
                        }
                        // If generic class, infer type params from constructor args
                        if let Some(gp) = self.generic_defs.get(class_name).cloned() {
                            let init_methods = self
                                .class_methods
                                .get(class_name)
                                .cloned()
                                .unwrap_or_default();
                            if let Some(init_sig) = init_methods.get("__init__") {
                                let arg_types: Vec<TypeId> = args
                                    .iter()
                                    .filter_map(|a| match a {
                                        CallArg::Positional(e) => Some(self.check_expr(e)),
                                        _ => None,
                                    })
                                    .collect();
                                let (subst, conflicts) =
                                    infer_type_args(&gp, &init_sig.params, &arg_types, &self.tcx);
                                for err in conflicts {
                                    self.error(expr.span, err);
                                }
                                let bound_errors = check_bounds(&subst, &gp, &self.tcx);
                                for err in bound_errors {
                                    self.error(expr.span, err);
                                }
                            }
                        }
                        func_ty_id
                    }
                    Ty::Any => {
                        for arg in args {
                            self.check_call_arg(arg);
                        }
                        self.tcx.any()
                    }
                    Ty::Error => self.tcx.error(),
                    // #1586: heterogeneous-callable Union. `for C in set, list, ...:`
                    // binds C to a Union of Fn/Class types. If every member is
                    // callable, accept the call and return Any (join of return types).
                    Ty::Union(ref members)
                        if members.iter().all(|&m| {
                            matches!(
                                self.tcx.get(m),
                                Ty::Fn { .. } | Ty::Class { .. } | Ty::Any | Ty::Error
                            )
                        }) =>
                    {
                        for arg in args {
                            self.check_call_arg(arg);
                        }
                        self.tcx.any()
                    }
                    _ => {
                        self.error(expr.span, "called value is not a function");
                        self.tcx.error()
                    }
                }
            }
            Expr::Attr { object, attr } => {
                let obj_ty_id = self.check_expr(object);
                self.resolve_attr(obj_ty_id, attr, expr.span)
            }
            Expr::Index { object, index } => {
                let obj_ty = self.check_expr(object);
                self.check_expr(index);
                // Slice index returns the container type itself, not the
                // element type. `lst[1:3]` is a list; `lst[1:3] = [...]`
                // needs to type-check as list-to-list, not element-to-list.
                if matches!(index.node, Expr::Slice { .. }) {
                    return obj_ty;
                }
                self.resolve_subscript(obj_ty, expr.span)
            }
            Expr::Slice { start, stop, step } => {
                if let Some(s) = start {
                    self.check_expr(s);
                }
                if let Some(s) = stop {
                    self.check_expr(s);
                }
                if let Some(s) = step {
                    self.check_expr(s);
                }
                self.tcx.any()
            }
            Expr::ListLit(elems) => {
                if elems.is_empty() {
                    let any = self.tcx.any();
                    self.tcx.intern(Ty::List(any))
                } else {
                    let first = self.check_expr(&elems[0]);
                    let mut homogeneous = true;
                    for elem in &elems[1..] {
                        let et = self.check_expr(elem);
                        if !self.types_compatible(first, et) {
                            homogeneous = false;
                        }
                    }
                    // Heterogeneous list literals infer List[Any] (CPython-compatible).
                    let elem_ty = if homogeneous { first } else { self.tcx.any() };
                    self.tcx.intern(Ty::List(elem_ty))
                }
            }
            Expr::DictLit(pairs) => {
                // Collect only explicit key-value pairs (skip unpack entries where key=None).
                let kv_pairs: Vec<_> = pairs
                    .iter()
                    .filter_map(|(k, v)| k.as_ref().map(|key| (key, v)))
                    .collect();
                // Also type-check unpack expressions (values where key is None).
                for (k, v) in pairs {
                    if k.is_none() {
                        self.check_expr(v);
                    }
                }
                if kv_pairs.is_empty() {
                    // Empty dict or unpack-only: return any type
                    self.tcx.any()
                } else {
                    let kt = self.check_expr(kv_pairs[0].0);
                    let vt = self.check_expr(kv_pairs[0].1);
                    let mut key_uniform = true;
                    let mut val_uniform = true;
                    for (k, v) in &kv_pairs[1..] {
                        let kk = self.check_expr(k);
                        let vv = self.check_expr(v);
                        if !self.types_compatible(kt, kk) {
                            key_uniform = false;
                        }
                        if !self.types_compatible(vt, vv) {
                            val_uniform = false;
                        }
                    }
                    // Python dicts are heterogeneous — widen to Any when types differ
                    let final_kt = if key_uniform { kt } else { self.tcx.any() };
                    let final_vt = if val_uniform { vt } else { self.tcx.any() };
                    self.tcx.intern(Ty::Dict(final_kt, final_vt))
                }
            }
            Expr::SetLit(elems) => {
                for elem in elems {
                    self.check_expr(elem);
                }
                self.tcx.error()
            }
            Expr::TupleLit(elems) => {
                let types: Vec<TypeId> = elems.iter().map(|e| self.check_expr(e)).collect();
                self.tcx.intern(Ty::Tuple(types))
            }
            Expr::IfExpr {
                body,
                condition,
                else_body,
            } => {
                self.check_expr(condition);
                let bt = self.check_expr(body);
                self.check_expr(else_body);
                bt
            }
            Expr::Lambda { params, body } => {
                self.symbols.push_scope();
                let param_types: Vec<TypeId> = params
                    .iter()
                    .map(|p| {
                        let ty = self.resolve_type_expr(&p.ty);
                        let sym = self.symbols.define(p.name.clone(), SymbolKind::Parameter);
                        self.set_sym_type(sym.0, ty);
                        ty
                    })
                    .collect();
                let ret = self.check_expr(body);
                self.symbols.pop_scope();
                self.tcx.intern(Ty::Fn {
                    params: param_types,
                    ret,
                    variadic: false,
                })
            }
            Expr::ListComp {
                element,
                generators,
            }
            | Expr::SetComp {
                element,
                generators,
            }
            | Expr::GeneratorExpr {
                element,
                generators,
            } => {
                self.symbols.push_scope();
                self.comprehension_depth += 1;
                for gen in generators {
                    let iter_ty = self.check_expr(&gen.iter);
                    // Infer element type from iterable (List[T] → T, else Any)
                    let elem_ty = match self.tcx.get(iter_ty) {
                        Ty::List(inner) => *inner,
                        _ => self.tcx.any(),
                    };
                    // Tuple-destructuring targets (`for a, b in pairs`) bind
                    // each target to the corresponding tuple ELEMENT type, not
                    // the whole element type — otherwise `a * b` became a bogus
                    // tuple*tuple "arithmetic requires numeric types" hard error.
                    // Mirrors the statement-`for` handling in check_stmt; shape
                    // mismatch / non-tuple element → Any, deferring to runtime
                    // unpacking.
                    let target_elem_tys: Option<Vec<TypeId>> = if gen.unpack_target {
                        match self.tcx.get(elem_ty) {
                            Ty::Tuple(ts) if ts.len() == gen.targets.len() => Some(ts.clone()),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    for (i, name) in gen.targets.iter().enumerate() {
                        let t = if gen.unpack_target {
                            target_elem_tys
                                .as_ref()
                                .map(|ts| ts[i])
                                .unwrap_or_else(|| self.tcx.any())
                        } else {
                            elem_ty
                        };
                        let sym = self.symbols.define(name.clone(), SymbolKind::Variable);
                        self.set_sym_type(sym.0, t);
                    }
                    for cond in &gen.conditions {
                        self.check_expr(cond);
                    }
                }
                self.check_expr(element);
                self.comprehension_depth -= 1;
                self.symbols.pop_scope();
                self.tcx.any()
            }
            Expr::DictComp {
                key,
                value,
                generators,
            } => {
                self.symbols.push_scope();
                self.comprehension_depth += 1;
                for gen in generators {
                    let iter_ty = self.check_expr(&gen.iter);
                    let elem_ty = match self.tcx.get(iter_ty) {
                        Ty::List(inner) => *inner,
                        _ => self.tcx.any(),
                    };
                    // Tuple-destructuring targets (`for a, b in pairs`) bind
                    // each target to the corresponding tuple ELEMENT type, not
                    // the whole element type — otherwise `a * b` became a bogus
                    // tuple*tuple "arithmetic requires numeric types" hard error.
                    // Mirrors the statement-`for` handling in check_stmt; shape
                    // mismatch / non-tuple element → Any, deferring to runtime
                    // unpacking.
                    let target_elem_tys: Option<Vec<TypeId>> = if gen.unpack_target {
                        match self.tcx.get(elem_ty) {
                            Ty::Tuple(ts) if ts.len() == gen.targets.len() => Some(ts.clone()),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    for (i, name) in gen.targets.iter().enumerate() {
                        let t = if gen.unpack_target {
                            target_elem_tys
                                .as_ref()
                                .map(|ts| ts[i])
                                .unwrap_or_else(|| self.tcx.any())
                        } else {
                            elem_ty
                        };
                        let sym = self.symbols.define(name.clone(), SymbolKind::Variable);
                        self.set_sym_type(sym.0, t);
                    }
                    for cond in &gen.conditions {
                        self.check_expr(cond);
                    }
                }
                self.check_expr(key);
                self.check_expr(value);
                self.comprehension_depth -= 1;
                self.symbols.pop_scope();
                self.tcx.error()
            }
            Expr::Yield(val) => {
                if let Some(v) = val {
                    self.check_expr(v);
                }
                self.tcx.error()
            }
            Expr::YieldFrom(expr) | Expr::Await(expr) | Expr::Starred(expr) => {
                self.check_expr(expr);
                self.tcx.error()
            }
            Expr::Walrus { target, value } => {
                let vt = self.check_expr(value);
                // PEP 572: walrus := only "leaks out of" comprehension scope.
                // In any other context (statement, while/if condition,
                // function body) it binds in the current scope like a normal
                // assignment. Always-enclosing was wrong: an inner function
                // walrus on the same name as an outer module variable
                // re-defined the symbol at module scope and corrupted the
                // outer variable's type (e.g. outer `i = 0` flipped from int
                // to float when an inner `(i := i + 1)` walrus was lowered).
                let sym = if self.comprehension_depth > 0 {
                    // Escape ALL enclosing comprehension scopes (one pushed per
                    // nesting level) to the nearest non-comprehension scope, so a
                    // walrus in a nested comprehension still binds in the real
                    // enclosing scope (nested_comp_walrus_leaks_enclosing).
                    self.symbols.define_levels_up(
                        self.comprehension_depth as usize,
                        target.clone(),
                        SymbolKind::Variable,
                    )
                } else {
                    self.symbols.define(target.clone(), SymbolKind::Variable)
                };
                self.set_sym_type(sym.0, vt);
                vt
            }
            Expr::ChainedCompare { operands, ops } => {
                // Type-check each adjacent pair of operands with its comparison op.
                for i in 0..ops.len() {
                    let lt = self.check_expr(&operands[i]);
                    let rt = self.check_expr(&operands[i + 1]);
                    self.check_binop(ops[i], lt, rt, expr.span);
                }
                self.tcx.bool()
            }
            Expr::UnpackTarget(elems) => {
                for elem in elems {
                    self.check_expr(elem);
                }
                self.tcx.error()
            }
        }
    }

    /// Strict dict operator wall for receiver-known `dict[...]` values.
    ///
    /// `dict | other` accepts mapping-like values such as `UserDict`, while
    /// `dict |= other` also accepts iterable key/value pairs. Until protocols are
    /// modeled here, only reject values that are provably neither: concrete
    /// scalars and bare user-class instances.
    fn check_dict_operator_call(&mut self, func: &Spanned<Expr>, args: &[CallArg]) {
        if stdlib_arg_check_suppressed() {
            return;
        }
        let Expr::Attr { object, attr } = &func.node else {
            return;
        };
        if !matches!(attr.as_str(), "__ior__" | "__or__" | "__ror__") {
            return;
        }
        let receiver_ty = self.check_expr(object);
        if !matches!(self.tcx.get(receiver_ty), Ty::Dict(_, _)) {
            return;
        }
        let Some(CallArg::Positional(arg)) = args.first() else {
            return;
        };
        let actual = self.check_expr(arg);
        let bare_arg = match self.tcx.get(actual) {
            Ty::Class { name, .. } if self.user_bare_classes.contains(name) => Some(name.clone()),
            _ => None,
        };
        if self.is_concrete_scalar(actual) || bare_arg.is_some() {
            let got = bare_arg.unwrap_or_else(|| self.ty_name(actual));
            self.error(
                arg.span,
                format!("argument type mismatch: expected `mapping`, got `{got}`"),
            );
        }
    }

    /// Check a single call argument (helper for non-Fn call sites).
    fn check_call_arg(&mut self, arg: &CallArg) {
        match arg {
            CallArg::Positional(a) => {
                self.check_expr(a);
            }
            CallArg::Keyword { value, .. } => {
                self.check_expr(value);
            }
            CallArg::StarArg(a) | CallArg::DoubleStarArg(a) => {
                self.check_expr(a);
            }
        }
    }

    /// ① Type-wall PoC HOOK. Resolve a call's callee to `(module, qualifier,
    /// name)` via import/instance provenance, look it up in the hardcoded
    /// `stdlib_sigs` table, and — only when the signature is enforceable —
    /// reject a concrete-scalar positional argument that is genuinely disjoint
    /// from a concrete-scalar param. ADDITIVE: only ever *emits* an error; never
    /// changes any return type or inference. Skip-when-unsure at every step so
    /// correct calls (the ② behavior oracle) are never newly rejected.
    fn check_stdlib_call(&mut self, func: &Spanned<Expr>, args: &[CallArg]) {
        // "Expected to raise at runtime" carve-out: a probe statement whose
        // immediate sibling is a `raise` (the auto-ported manual-assertRaises
        // idiom) needs the call to raise at RUNTIME, not be rejected at compile
        // time — otherwise the whole module aborts and a behavior PASS turns RED.
        // The ① type-wall fixtures never use this idiom (their probe is followed
        // by a `print`), so this never costs a type gain. See
        // `SUPPRESS_STDLIB_ARG_CHECK`.
        if stdlib_arg_check_suppressed() {
            return;
        }
        // Resolve callee -> a concrete `StdlibSig`. We resolve to the signature
        // directly (rather than a `(module, qualifier, name)` triple) because a
        // bare stdlib name `Cls(...)` may be either a module function OR a class
        // constructor; we try the module-fn key first and fall back to the
        // class `__init__` key. Skip-when-unsure at every miss.
        let sig: Option<&'static super::stdlib_sigs::StdlibSig> = match &func.node {
            // Bare name: a from-imported module function (`strerror(...)`) OR a
            // from-imported stdlib class called as a constructor (`Cls(...)`).
            //
            // The import-time qualifier (recorded in `import_origins`) is only a
            // hint — `is_known_stdlib_class` consults the curated table, so most
            // generated stdlib classes are bound with an empty qualifier and
            // look like module functions here. So we DO NOT trust the qualifier:
            // we try the module-fn key `(module, "", name)` first, and on a miss
            // fall back to the constructor key `(module, name, "__init__")`.
            // The `self` receiver is already stripped from `__init__` param rows,
            // so positional alignment starts at the first real argument. Names not
            // in `import_origins` (user-defined classes, locals) resolve to None.
            Expr::Ident(name) => self
                .import_origins
                .get(name)
                .and_then(|(module, _qual)| {
                    super::stdlib_sigs::get(module, "", name)
                        .or_else(|| super::stdlib_sigs::get(module, name, "__init__"))
                })
                .or_else(|| {
                    if self.is_unshadowed_builtin(name) {
                        super::stdlib_sigs::get("builtins", "", name)
                            .or_else(|| super::stdlib_sigs::get("builtins", name, "__init__"))
                    } else {
                        None
                    }
                }),
            // Attribute access: `os.strerror(...)` (module fn) or
            // `obj.handle_entityref(...)` (instance method).
            Expr::Attr { object, attr } => {
                if let Expr::Ident(base) = &object.node {
                    if let Some((module, qual)) = self.import_origins.get(base) {
                        // `base.attr(...)` is either a module function or a
                        // class/static method. Most from-imported stdlib classes are
                        // bound with an EMPTY qualifier (they "look like module
                        // functions" here, see the Ident branch above), so resolving
                        // `date.fromtimestamp(...)` needs us to also try `base` itself
                        // as the class qualifier — `get(module, "date", "fromtimestamp")`.
                        // Try module-fn first, then class-method (base = class name),
                        // then any recorded qualifier. Still gated downstream by
                        // `sig.enforceable` + the concrete-scalar-disjoint check, so
                        // this only ADDS rejections of genuinely wrong-typed scalar
                        // args (it previously leaked `date.fromtimestamp("x")` etc.).
                        super::stdlib_sigs::get(module, "", attr)
                            .or_else(|| super::stdlib_sigs::get(module, base, attr))
                            .or_else(|| {
                                if qual.is_empty() {
                                    None
                                } else {
                                    super::stdlib_sigs::get(module, qual, attr)
                                }
                            })
                    } else if let Some(cls) = self.instance_origins.get(base).cloned() {
                        // `base` is a stdlib instance — recover its module from
                        // the class's import origin, then resolve a method sig.
                        self.import_origins
                            .get(&cls)
                            .and_then(|(module, _q)| super::stdlib_sigs::get(module, &cls, attr))
                    } else if let Some(sym) = self.symbols.lookup(base) {
                        if matches!(self.tcx.get(self.get_sym_type(sym.0)), Ty::List(_)) {
                            super::stdlib_sigs::get("builtins", "list", attr)
                        } else if self.symbols.get_symbol(sym).kind == SymbolKind::Function {
                            // User-defined Python functions are instances of
                            // builtins.function. This keeps descriptor walls such as
                            // `f.__get__(..., owner)` enforceable without pretending
                            // that `builtins.function` is importable in CPython.
                            super::stdlib_sigs::get("builtins", "function", attr)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        };

        let Some(sig) = sig else { return };
        let strict_keyword_wall = self.strict_type_fixture
            && sig.module == "keyword"
            && sig.qualifier.is_empty()
            && matches!(sig.name, "iskeyword" | "issoftkeyword");
        if !sig.enforceable && !strict_keyword_wall {
            return;
        }

        // Walk positional args against params. Stop at the first star/kwarg arg
        // and at the first star/unknown param. Only reject when BOTH the param
        // and the actual arg are concrete-and-disjoint scalars.
        let bytes_encoding_arg_is_positional = sig.module == "builtins"
            && sig.qualifier.is_empty()
            && matches!(sig.name, "bytes" | "bytearray")
            && matches!(args.get(1), Some(CallArg::Positional(_)));
        let mut param_idx = 0usize;
        for arg in args {
            let CallArg::Positional(a) = arg else {
                // Keyword / *args / **kwargs: stop enforcement entirely. We do
                // not know how positional alignment continues past these.
                break;
            };
            let Some(param) = sig.params.get(param_idx) else {
                break;
            };
            if param.star {
                break; // never enforce past `*args`
            }
            // Map the param's CoreTy to a concrete scalar; None => no positive
            // scalar mapping (Unknown / Typed / Bytes). Still advance param_idx.
            // Bytes is handled below as a negative scalar wall because bytes
            // literals currently infer to Any.
            let expected = self.core_ty_to_type_id(param.ty);
            let actual = self.check_expr(a);
            let classinfo_param = sig.module == "builtins"
                && sig.qualifier.is_empty()
                && matches!(sig.name, "isinstance" | "issubclass")
                && param_idx == 1;
            if classinfo_param {
                if let Some(name) = self.classinfo_bare_instance_name(a) {
                    self.error(
                        a.span,
                        format!(
                            "argument type mismatch: `{name}` does not satisfy parameter `{}`'s type",
                            param.name,
                        ),
                    );
                }
                param_idx += 1;
                continue;
            }
            // A BARE user class instance (`class _W: pass` -> `_W()`) satisfies NO
            // concrete parameter contract: it is not a scalar (str/int/float/
            // bytes/bool; no relevant dunder), not a protocol (no dunders), and
            // not a nominal class (object is its only base). Reject it against any
            // param whose CoreTy names such a contract. Use expression shape,
            // not just `Ty::Class`: the current type model represents both `C`
            // and `C()` as `Ty::Class`, and descriptor/type params must accept
            // class objects such as `f.__get__(None, C)` and
            // `object.__subclasshook__(C)`. `None`/`Unknown` params are excluded
            // because `None` is frequently an under-declared Optional sentinel
            // and Unknown remains skip-when-unsure.
            let concrete_param = matches!(
                param.ty,
                super::stdlib_sigs::CoreTy::Int
                    | super::stdlib_sigs::CoreTy::Float
                    | super::stdlib_sigs::CoreTy::Str
                    | super::stdlib_sigs::CoreTy::Bytes
                    | super::stdlib_sigs::CoreTy::MemoryView
                    | super::stdlib_sigs::CoreTy::Complex
                    | super::stdlib_sigs::CoreTy::List
                    | super::stdlib_sigs::CoreTy::Bool
                    | super::stdlib_sigs::CoreTy::Typed
            );
            let bare_arg = self.classinfo_bare_instance_name(a);
            if let (true, Some(name)) = (concrete_param, &bare_arg) {
                self.error(
                    a.span,
                    format!(
                        "argument type mismatch: `{name}` does not satisfy parameter `{}`'s type",
                        param.name,
                    ),
                );
            } else {
                // A `None` actual argument is NEVER rejected: typeshed routinely
                // under-declares Optional (a `host: str` parameter is called with
                // `None` as a sentinel/clear, `set_proxy(host, None)` etc.), and
                // `None` is the single most common "looks wrong, is right" runtime
                // value. Skip-when-unsure — a missed enforcement is fine, a false
                // reject is not. (The ① type-wall fixtures probe with wrong
                // *scalars* — str-for-int, instance-for-bool — not bare `None`, so
                // this costs no type gain.)
                let actual_is_none = matches!(self.tcx.get(actual), Ty::None);
                if bytes_encoding_arg_is_positional
                    && param_idx == 0
                    && !actual_is_none
                    && self.is_concrete_scalar(actual)
                    && !matches!(self.tcx.get(actual), Ty::Str)
                {
                    self.error(
                        a.span,
                        format!(
                            "argument type mismatch: expected `str` source when `encoding` is provided, got `{}`",
                            self.ty_name(actual),
                        ),
                    );
                } else if matches!(param.ty, super::stdlib_sigs::CoreTy::Complex)
                    && !actual_is_none
                    && self.is_concrete_scalar(actual)
                    && !matches!(self.tcx.get(actual), Ty::Int | Ty::Float | Ty::Bool)
                {
                    self.error(
                        a.span,
                        format!(
                            "argument type mismatch: expected `complex`, got `{}`",
                            self.ty_name(actual),
                        ),
                    );
                } else if matches!(
                    param.ty,
                    super::stdlib_sigs::CoreTy::Bytes
                        | super::stdlib_sigs::CoreTy::MemoryView
                        | super::stdlib_sigs::CoreTy::List
                ) && !actual_is_none
                    && self.is_concrete_scalar(actual)
                {
                    let expected_name = match param.ty {
                        super::stdlib_sigs::CoreTy::Bytes => "bytes",
                        super::stdlib_sigs::CoreTy::MemoryView => "memoryview",
                        super::stdlib_sigs::CoreTy::List => "list",
                        _ => unreachable!(),
                    };
                    self.error(
                        a.span,
                        format!(
                            "argument type mismatch: expected `{expected_name}`, got `{}`",
                            self.ty_name(actual),
                        ),
                    );
                } else if let Some(expected) = expected {
                    // Both must be concrete scalars, and genuinely incompatible
                    // (types_compatible already allows Bool->Int and Int->Float).
                    if !actual_is_none
                        && self.is_concrete_scalar(actual)
                        && !self.types_compatible(expected, actual)
                    {
                        self.error(
                            a.span,
                            format!(
                                "argument type mismatch: expected `{}`, got `{}`",
                                self.ty_name(expected),
                                self.ty_name(actual),
                            ),
                        );
                    }
                }
            }
            param_idx += 1;
        }
    }

    /// `isinstance`/`issubclass` classinfo accepts type objects and tuples of
    /// type objects. The current type model represents both `C` and `C()` as
    /// `Ty::Class`, so this narrow hook uses expression shape to reject only
    /// the provably-wrong bare-instance fixtures.
    fn classinfo_bare_instance_name(&self, expr: &Spanned<Expr>) -> Option<String> {
        match &expr.node {
            Expr::Call { func, .. } => match &func.node {
                Expr::Ident(name) if self.user_bare_classes.contains(name) => Some(name.clone()),
                _ => None,
            },
            Expr::TupleLit(elems) => elems
                .iter()
                .find_map(|elem| self.classinfo_bare_instance_name(elem)),
            _ => None,
        }
    }

    /// Resolve attribute access (#246).
    fn resolve_attr(&mut self, obj_ty_id: TypeId, attr: &str, _span: Span) -> TypeId {
        match self.tcx.get(obj_ty_id).clone() {
            Ty::List(elem) => match attr {
                "append" | "remove" => self.tcx.intern(Ty::Fn {
                    params: vec![elem],
                    ret: self.tcx.none(),
                    variadic: false,
                }),
                "count" => self.tcx.intern(Ty::Fn {
                    params: vec![elem],
                    ret: self.tcx.int(),
                    variadic: false,
                }),
                "index" => self.tcx.intern(Ty::Fn {
                    params: vec![elem, self.tcx.int(), self.tcx.int()],
                    ret: self.tcx.int(),
                    variadic: false,
                }),
                _ => self.tcx.any(),
            },
            Ty::Dict(key, value) => match attr {
                "__delitem__" => self.tcx.intern(Ty::Fn {
                    params: vec![key],
                    ret: self.tcx.none(),
                    variadic: false,
                }),
                "__getitem__" => self.tcx.intern(Ty::Fn {
                    params: vec![key],
                    ret: value,
                    variadic: false,
                }),
                "__setitem__" => self.tcx.intern(Ty::Fn {
                    params: vec![key, value],
                    ret: self.tcx.none(),
                    variadic: false,
                }),
                "get" | "pop" => {
                    let any = self.tcx.any();
                    self.tcx.intern(Ty::Fn {
                        params: vec![key, any],
                        ret: any,
                        variadic: false,
                    })
                }
                _ => self.tcx.any(),
            },
            Ty::Class { fields, .. } => {
                for (name, ty) in &fields {
                    if name == attr {
                        return *ty;
                    }
                }
                // Method lookup would go here; for now return Any
                self.tcx.any()
            }
            Ty::Any | Ty::Error => self.tcx.any(),
            _ => self.tcx.any(),
        }
    }

    /// Resolve subscript / index access (#248).
    fn resolve_subscript(&mut self, obj_ty: TypeId, _span: Span) -> TypeId {
        match self.tcx.get(obj_ty).clone() {
            Ty::List(elem) => elem,
            Ty::Dict(_, v) => v,
            Ty::Tuple(ts) if !ts.is_empty() => {
                // Static tuple index: return the union of all element types,
                // deduped like `infer_iter_element` (#1562) so a homogeneous
                // tuple subscripts to the bare element type rather than a
                // degenerate Union[Int, Int, ...].
                let mut uniq: Vec<TypeId> = Vec::with_capacity(ts.len());
                for t in ts {
                    if !uniq.contains(&t) {
                        uniq.push(t);
                    }
                }
                if uniq.len() == 1 {
                    uniq.into_iter().next().unwrap()
                } else {
                    self.tcx.intern(Ty::Union(uniq))
                }
            }
            Ty::Str => self.tcx.str(),
            Ty::Any | Ty::Error => self.tcx.any(),
            _ => self.tcx.any(),
        }
    }

    /// Numeric tower promotion: int+float → Any (routes through runtime dispatch).
    /// Returns Any so the codegen uses `mb_dispatch_binop` which handles coercion.
    fn numeric_promotion(&self, lt: TypeId, rt: TypeId) -> Option<TypeId> {
        let l = self.tcx.get(lt);
        let r = self.tcx.get(rt);
        match (l, r) {
            // int + float → float (via Any for now)
            (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Some(self.tcx.any()),
            // bool + float → float
            (Ty::Bool, Ty::Float) | (Ty::Float, Ty::Bool) => Some(self.tcx.any()),
            // bool + int or bool + bool → int
            (Ty::Bool, Ty::Int) | (Ty::Int, Ty::Bool) => Some(self.tcx.int()),
            (Ty::Bool, Ty::Bool) => Some(self.tcx.int()),
            _ => None,
        }
    }

    /// True when `t` is a `Union` whose members are ALL numeric
    /// (Int/Float/Bool). Such unions arise from subscripting heterogeneous
    /// numeric tuples; arithmetic on them is safe to defer to runtime
    /// dispatch. Unions with any non-numeric member return false.
    fn is_all_numeric_union(&self, t: TypeId) -> bool {
        match self.tcx.get(t) {
            Ty::Union(ts) => ts.iter().all(|m| self.tcx.get(*m).is_numeric()),
            _ => false,
        }
    }

    pub(crate) fn check_binop(&mut self, op: BinOp, lt: TypeId, rt: TypeId, span: Span) -> TypeId {
        if self.tcx.get(lt).is_error() || self.tcx.get(rt).is_error() {
            return self.tcx.error();
        }
        // Any propagates through operations (#240)
        if self.tcx.get(lt).is_any() || self.tcx.get(rt).is_any() {
            return self.tcx.any();
        }
        match op {
            BinOp::Add
            | BinOp::Sub
            | BinOp::Mul
            | BinOp::Div
            | BinOp::FloorDiv
            | BinOp::Mod
            | BinOp::Pow
            | BinOp::MatMul => {
                // Str + Str → Str (string concatenation): early branch before numeric guards
                if matches!(op, BinOp::Add)
                    && matches!(self.tcx.get(lt), Ty::Str)
                    && matches!(self.tcx.get(rt), Ty::Str)
                {
                    return self.tcx.str();
                }
                // List + List → List (concatenation)
                if matches!(op, BinOp::Add)
                    && matches!(self.tcx.get(lt), Ty::List(_))
                    && matches!(self.tcx.get(rt), Ty::List(_))
                {
                    return lt;
                }
                // Tuple + Tuple → Tuple (concatenation)
                if matches!(op, BinOp::Add)
                    && matches!(self.tcx.get(lt), Ty::Tuple(_))
                    && matches!(self.tcx.get(rt), Ty::Tuple(_))
                {
                    return self.tcx.any();
                }
                // List * Int or Int * List → List (repetition)
                if matches!(op, BinOp::Mul) {
                    if (matches!(self.tcx.get(lt), Ty::List(_))
                        && matches!(self.tcx.get(rt), Ty::Int))
                        || (matches!(self.tcx.get(lt), Ty::Int)
                            && matches!(self.tcx.get(rt), Ty::List(_)))
                    {
                        return if matches!(self.tcx.get(lt), Ty::List(_)) {
                            lt
                        } else {
                            rt
                        };
                    }
                    // Tuple * Int or Int * Tuple → Tuple (repetition)
                    if (matches!(self.tcx.get(lt), Ty::Tuple(_))
                        && matches!(self.tcx.get(rt), Ty::Int))
                        || (matches!(self.tcx.get(lt), Ty::Int)
                            && matches!(self.tcx.get(rt), Ty::Tuple(_)))
                    {
                        return self.tcx.any();
                    }
                    // Str * Int or Int * Str → Str (repetition)
                    if (matches!(self.tcx.get(lt), Ty::Str) && matches!(self.tcx.get(rt), Ty::Int))
                        || (matches!(self.tcx.get(lt), Ty::Int)
                            && matches!(self.tcx.get(rt), Ty::Str))
                    {
                        return self.tcx.str();
                    }
                }
                // Str % X → Str (printf-style formatting). X can be any
                // single value or a tuple of values — the runtime parses
                // the template at format time.
                if matches!(op, BinOp::Mod) && matches!(self.tcx.get(lt), Ty::Str) {
                    return self.tcx.str();
                }
                // Numeric tower promotion: int+float → float
                if let Some(promoted) = self.numeric_promotion(lt, rt) {
                    return promoted;
                }
                // Class instances may define __add__/__sub__/__mul__/... —
                // defer to runtime dunder dispatch via Any.
                if matches!(self.tcx.get(lt), Ty::Class { .. })
                    || matches!(self.tcx.get(rt), Ty::Class { .. })
                {
                    return self.tcx.any();
                }
                // Union-of-numerics (e.g. a subscript on a heterogeneous
                // numeric tuple yields Union[Int, Float]): every member
                // supports arithmetic, so defer to runtime dispatch via Any.
                // Unions containing ANY non-numeric member do NOT qualify —
                // those still hard-error below (force-typed policy, Option A).
                let l_num_union = self.is_all_numeric_union(lt);
                let r_num_union = self.is_all_numeric_union(rt);
                if (l_num_union || r_num_union)
                    && (l_num_union || self.tcx.get(lt).is_numeric())
                    && (r_num_union || self.tcx.get(rt).is_numeric())
                {
                    return self.tcx.any();
                }
                if !self.types_compatible(lt, rt) {
                    self.error(
                        span,
                        format!(
                            "operand type mismatch: `{}` vs `{}`",
                            self.ty_name(lt),
                            self.ty_name(rt),
                        ),
                    );
                    return self.tcx.error();
                }
                if !self.tcx.get(lt).is_numeric() {
                    self.error(span, "arithmetic requires numeric types");
                    return self.tcx.error();
                }
                // Python true division always returns float, even for int/int (#2104).
                // HIR/MIR lowering already routes Int/Int through `mb_div` (which boxes
                // the float result), so the static type must reflect that — otherwise
                // downstream consumers (print formatting, int() coercion) treat the
                // raw f64 bits as an i64 and emit garbage.
                if matches!(op, BinOp::Div)
                    && matches!(self.tcx.get(lt), Ty::Int | Ty::Bool)
                    && matches!(self.tcx.get(rt), Ty::Int | Ty::Bool)
                {
                    return self.tcx.float();
                }
                lt
            }
            BinOp::LShift | BinOp::RShift => {
                // Bool is a subtype of int — accept both
                if !matches!(self.tcx.get(lt), Ty::Int | Ty::Bool)
                    || !matches!(self.tcx.get(rt), Ty::Int | Ty::Bool)
                {
                    self.error(span, "shift operators require int types");
                    return self.tcx.error();
                }
                self.tcx.int()
            }
            BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
                // Bool is a subtype of int — accept both for bitwise ops.
                // Python: bool & bool → bool, bool & int → int, int & int → int
                if matches!(self.tcx.get(lt), Ty::Bool) && matches!(self.tcx.get(rt), Ty::Bool) {
                    self.tcx.bool()
                } else if matches!(self.tcx.get(lt), Ty::Int | Ty::Bool)
                    && matches!(self.tcx.get(rt), Ty::Int | Ty::Bool)
                {
                    self.tcx.int()
                } else {
                    self.tcx.any()
                }
            }
            BinOp::Eq | BinOp::NotEq | BinOp::Is | BinOp::IsNot => self.tcx.bool(),
            BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
                // Bool is a subtype of int — accept for ordered comparisons
                // Class instances accepted — may define __lt__/__le__/__gt__/__ge__
                let lt_ok = matches!(
                    self.tcx.get(lt),
                    Ty::Int
                        | Ty::Float
                        | Ty::Bool
                        | Ty::Str
                        | Ty::List(_)
                        | Ty::Tuple(_)
                        | Ty::Any
                        | Ty::Class { .. }
                );
                let rt_ok = matches!(
                    self.tcx.get(rt),
                    Ty::Int
                        | Ty::Float
                        | Ty::Bool
                        | Ty::Str
                        | Ty::List(_)
                        | Ty::Tuple(_)
                        | Ty::Any
                        | Ty::Class { .. }
                );
                if !lt_ok || !rt_ok {
                    self.error(span, "comparison requires numeric types");
                }
                self.tcx.bool()
            }
            BinOp::And | BinOp::Or => {
                // Python and/or accept any type with short-circuit semantics
                self.tcx.any()
            }
            BinOp::In | BinOp::NotIn => self.tcx.bool(),
        }
    }

    pub(crate) fn check_pattern(&mut self, pattern: &Spanned<Pattern>) {
        match &pattern.node {
            Pattern::Wildcard => {}
            Pattern::Binding(name) => {
                let sym = self.symbols.define(name.clone(), SymbolKind::Variable);
                // Propagate the match subject type into the capture binding (#827).
                let ty = self
                    .current_match_subject_ty
                    .unwrap_or_else(|| self.tcx.error());
                self.set_sym_type(sym.0, ty);
            }
            Pattern::Constructor { fields, .. } => {
                for field in fields {
                    let sym = self.symbols.define(field.clone(), SymbolKind::Variable);
                    self.set_sym_type(sym.0, self.tcx.error());
                }
            }
            Pattern::Literal(_) => {}
            Pattern::Or(patterns) => {
                // PEP 634: all OR alternatives must bind the same names (#827).
                // Check each alternative and track per-binding types so we can merge
                // soundly. The arm body must not see a single-alternative type when
                // alternatives bind with different types (e.g. int(v) | str(v)).
                if patterns.len() > 1 {
                    let first_bindings = collect_pattern_bindings(&patterns[0]);
                    for alt in &patterns[1..] {
                        let alt_bindings = collect_pattern_bindings(alt);
                        if first_bindings != alt_bindings {
                            self.error(
                                alt.span,
                                format!(
                                    "PEP 634: OR pattern alternatives must bind the same names; \
                                     expected bindings {:?} but got {:?}",
                                    first_bindings.iter().collect::<Vec<_>>(),
                                    alt_bindings.iter().collect::<Vec<_>>(),
                                ),
                            );
                        }
                    }
                }
                // Collect the binding names from the first alternative.
                let binding_names = collect_pattern_bindings(&patterns[0]);
                // Check each alternative and accumulate the per-name types.
                let mut per_name_types: std::collections::HashMap<String, Vec<TypeId>> =
                    binding_names
                        .iter()
                        .map(|n| (n.clone(), Vec::new()))
                        .collect();
                for p in patterns {
                    self.check_pattern(p);
                    for name in &binding_names {
                        if let Some(sym) = self.symbols.lookup(name) {
                            let ty = self.get_sym_type(sym.0);
                            if let Some(v) = per_name_types.get_mut(name) {
                                v.push(ty);
                            }
                        }
                    }
                }
                // Re-define each binding with the merged type. If all alternatives agree
                // on the same type, keep that type; otherwise fall back to Any (#827).
                for name in &binding_names {
                    let types = per_name_types
                        .get(name)
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);
                    let merged = if types.is_empty() {
                        self.tcx.any()
                    } else if types.iter().all(|&t| t == types[0]) {
                        types[0]
                    } else {
                        self.tcx.any() // conservative: heterogeneous types collapse to Any
                    };
                    let sym = self.symbols.define(name.clone(), SymbolKind::Variable);
                    self.set_sym_type(sym.0, merged);
                }
            }
            Pattern::Sequence(patterns) => {
                // Derive per-position element type from the current match subject (#827).
                // For tuples we use the indexed slot type; for lists all positions share
                // the uniform element type. This prevents Union-collapse that loses
                // per-slot precision (e.g. `match (1, 2): case (n, _): return n + 1`).
                let subj = self.current_match_subject_ty;
                let subj_ty_clone = subj.map(|id| self.tcx.get(id).clone());
                for (i, p) in patterns.iter().enumerate() {
                    let elem_ty = match &subj_ty_clone {
                        Some(Ty::List(inner)) => *inner,
                        Some(Ty::Tuple(ts)) => {
                            if i < ts.len() {
                                ts[i]
                            } else {
                                self.tcx.any()
                            }
                        }
                        _ => self.tcx.any(),
                    };
                    let saved = self.current_match_subject_ty;
                    // Star sub-pattern captures a list of the element type.
                    self.current_match_subject_ty = Some(match &p.node {
                        Pattern::Star(_) => self.tcx.intern(Ty::List(elem_ty)),
                        _ => elem_ty,
                    });
                    self.check_pattern(p);
                    self.current_match_subject_ty = saved;
                }
            }
            Pattern::Mapping { pairs, rest } => {
                // Derive the value type from the current match subject (#827).
                let (full_dict_ty, val_ty) = if let Some(subj) = self.current_match_subject_ty {
                    match self.tcx.get(subj).clone() {
                        Ty::Dict(_, v) => (subj, v),
                        _ => (subj, self.tcx.any()),
                    }
                } else {
                    let any = self.tcx.any();
                    (any, any)
                };
                for (_key, pat) in pairs {
                    let saved = self.current_match_subject_ty;
                    self.current_match_subject_ty = Some(val_ty);
                    self.check_pattern(pat);
                    self.current_match_subject_ty = saved;
                }
                // Register rest-capture variable as the full dict type (#827)
                if let Some(r) = rest {
                    let sym = self.symbols.define(r.clone(), SymbolKind::Variable);
                    self.set_sym_type(sym.0, full_dict_ty);
                }
            }
            Pattern::ClassPattern { cls, patterns } => {
                // Look up the class type so we can propagate field types into
                // keyword sub-patterns (#827).  E.g. `case Point(x=a):` should
                // type `a` as `int` (the type of `Point.x`), not as `Point`.
                let class_name = cls.last().map(|s| s.as_str()).unwrap_or("");

                // Built-in self-subject patterns: case int(x), case str(s), etc.
                // These have ONE positional arg that captures the subject itself.
                let builtin_capture_ty = match class_name {
                    "int" => Some(self.tcx.int()),
                    "bool" => Some(self.tcx.bool()),
                    "str" => Some(self.tcx.str()),
                    "float" => Some(self.tcx.float()),
                    "list" => Some(self.tcx.any()),
                    "tuple" => Some(self.tcx.any()),
                    "dict" => Some(self.tcx.any()),
                    _ => None,
                };
                if let Some(capture_ty) = builtin_capture_ty {
                    let prev = self.current_match_subject_ty.replace(capture_ty);
                    for (_, sub_pat) in patterns {
                        self.check_pattern(sub_pat);
                    }
                    self.current_match_subject_ty = prev;
                    return;
                }
                let (class_fields, explicit_match_args): (
                    Vec<(String, TypeId)>,
                    Option<Vec<String>>,
                ) = self
                    .symbols
                    .lookup(class_name)
                    .map(|sym| {
                        let ty = self.get_sym_type(sym.0);
                        if let Ty::Class {
                            fields, match_args, ..
                        } = self.tcx.get(ty).clone()
                        {
                            (fields, match_args)
                        } else {
                            (Vec::new(), None)
                        }
                    })
                    .unwrap_or_default();

                // Build positional field types (#827):
                // - explicit `__match_args__` present (even empty): use it (empty → no positional slots).
                // - absent (None): fall back to class field declaration order.
                let positional_names: Vec<String> = match explicit_match_args {
                    Some(names) => names,
                    None => class_fields.iter().map(|(n, _)| n.clone()).collect(),
                };
                let positional_field_types: Vec<TypeId> = positional_names
                    .iter()
                    .map(|arg_name| {
                        class_fields
                            .iter()
                            .find(|(n, _)| n == arg_name)
                            .map(|(_, t)| *t)
                            .unwrap_or_else(|| self.tcx.any())
                    })
                    .collect();

                let mut positional_idx = 0usize;
                for (name, pat) in patterns {
                    let field_ty = match name {
                        Some(attr_name) => {
                            // Keyword: look up the field by name
                            class_fields
                                .iter()
                                .find(|(n, _)| n == attr_name)
                                .map(|(_, t)| *t)
                                .unwrap_or_else(|| self.tcx.any())
                        }
                        None => {
                            // Positional: use __match_args__ order
                            let ty = positional_field_types
                                .get(positional_idx)
                                .copied()
                                .unwrap_or_else(|| self.tcx.any());
                            positional_idx += 1;
                            ty
                        }
                    };
                    // Temporarily set the match subject type to the field type
                    // so that nested `Pattern::Binding` picks up the right type.
                    let saved = self.current_match_subject_ty;
                    self.current_match_subject_ty = Some(field_ty);
                    self.check_pattern(pat);
                    self.current_match_subject_ty = saved;
                }
            }
            Pattern::Star(name) => {
                if let Some(n) = name {
                    let sym = self.symbols.define(n.clone(), SymbolKind::Variable);
                    // Star capture gets list[subject_ty] when subject type is known (#827).
                    let ty = if let Some(subj) = self.current_match_subject_ty {
                        self.tcx.intern(crate::types::Ty::List(subj))
                    } else {
                        self.tcx.error()
                    };
                    self.set_sym_type(sym.0, ty);
                }
            }
            Pattern::As { pattern, name } => {
                // Check inner pattern, then register the AS binding (#827).
                // Propagate the narrowed class type to the alias if the inner pattern
                // is a ClassPattern or Constructor — otherwise use the match subject type.
                self.check_pattern(pattern);
                let sym = self.symbols.define(name.clone(), SymbolKind::Variable);
                let alias_ty = match &pattern.node {
                    Pattern::ClassPattern { cls, .. } => {
                        let class_name = cls.last().map(|s| s.as_str()).unwrap_or("");
                        self.symbols
                            .lookup(class_name)
                            .map(|s| self.get_sym_type(s.0))
                            .filter(|&ty| {
                                matches!(self.tcx.get(ty), crate::types::Ty::Class { .. })
                            })
                            .unwrap_or_else(|| self.tcx.any())
                    }
                    Pattern::Constructor { path, .. } => {
                        let class_name = path.last().map(|s| s.as_str()).unwrap_or("");
                        self.symbols
                            .lookup(class_name)
                            .map(|s| self.get_sym_type(s.0))
                            .filter(|&ty| {
                                matches!(self.tcx.get(ty), crate::types::Ty::Class { .. })
                            })
                            .unwrap_or_else(|| self.tcx.any())
                    }
                    // For non-class patterns, propagate the match subject type (#827).
                    _ => self
                        .current_match_subject_ty
                        .unwrap_or_else(|| self.tcx.any()),
                };
                self.set_sym_type(sym.0, alias_ty);
            }
        }
    }
}

/// Collect all binding names introduced by a pattern (PEP 634 validation helper).
fn collect_pattern_bindings(pat: &Spanned<Pattern>) -> std::collections::BTreeSet<String> {
    let mut names = std::collections::BTreeSet::new();
    collect_bindings_inner(&pat.node, &mut names);
    names
}

fn collect_bindings_inner(pat: &Pattern, names: &mut std::collections::BTreeSet<String>) {
    match pat {
        Pattern::Binding(name) => {
            names.insert(name.clone());
        }
        Pattern::Or(alts) => {
            // Don't recurse into nested OR — validate at each level
            for alt in alts {
                collect_bindings_inner(&alt.node, names);
            }
        }
        Pattern::Sequence(pats) => {
            for p in pats {
                collect_bindings_inner(&p.node, names);
            }
        }
        Pattern::As { pattern, name } => {
            collect_bindings_inner(&pattern.node, names);
            names.insert(name.clone());
        }
        Pattern::ClassPattern { patterns, .. } => {
            for (_, p) in patterns {
                collect_bindings_inner(&p.node, names);
            }
        }
        Pattern::Mapping { pairs, rest } => {
            for (_, p) in pairs {
                collect_bindings_inner(&p.node, names);
            }
            if let Some(r) = rest {
                names.insert(r.clone());
            }
        }
        Pattern::Star(Some(name)) => {
            names.insert(name.clone());
        }
        Pattern::Constructor { fields, .. } => {
            for f in fields {
                names.insert(f.clone());
            }
        }
        Pattern::Wildcard | Pattern::Literal(_) | Pattern::Star(None) => {}
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::*;
    use crate::source::span::{Span, Spanned};
    use crate::types::check::TypeChecker;
    use crate::types::Ty;

    fn sp<T>(node: T) -> Spanned<T> {
        Spanned::new(node, Span::dummy())
    }

    // --- Literal types (via check_expr, which is pub(crate)) ---

    #[test]
    fn test_check_expr_int_lit() {
        let mut checker = TypeChecker::new();
        let ty = checker.check_expr(&sp(Expr::IntLit(42)));
        assert_eq!(checker.tcx.get(ty), &Ty::Int);
    }

    #[test]
    fn test_check_expr_float_lit() {
        let mut checker = TypeChecker::new();
        let ty = checker.check_expr(&sp(Expr::FloatLit(3.14)));
        assert_eq!(checker.tcx.get(ty), &Ty::Float);
    }

    #[test]
    fn test_check_expr_bool_lit() {
        let mut checker = TypeChecker::new();
        let ty = checker.check_expr(&sp(Expr::BoolLit(true)));
        assert_eq!(checker.tcx.get(ty), &Ty::Bool);
    }

    #[test]
    fn test_check_expr_str_lit() {
        let mut checker = TypeChecker::new();
        let ty = checker.check_expr(&sp(Expr::StrLit("hello".to_string())));
        assert_eq!(checker.tcx.get(ty), &Ty::Str);
    }

    #[test]
    fn test_check_expr_none_lit() {
        let mut checker = TypeChecker::new();
        let ty = checker.check_expr(&sp(Expr::NoneLit));
        assert_eq!(checker.tcx.get(ty), &Ty::None);
    }

    // --- Undefined ident → check_module returns errors ---

    #[test]
    fn test_check_expr_undefined_ident_emits_error() {
        let mut checker = TypeChecker::new();
        let module = Module {
            stmts: vec![sp(Stmt::ExprStmt(sp(Expr::Ident(
                "undefined_xyz_999".to_string(),
            ))))],
        };
        let errors = checker.check_module(&module);
        assert!(!errors.is_empty());
    }

    // --- UnaryOp error branches (via check_module) ---

    #[test]
    fn test_check_expr_unary_neg_on_string_emits_error() {
        let mut checker = TypeChecker::new();
        let module = Module {
            stmts: vec![
                sp(Stmt::VarDecl {
                    name: "s".to_string(),
                    ty: sp(TypeExpr::Named("str".to_string())),
                    value: sp(Expr::StrLit("hello".to_string())),
                }),
                sp(Stmt::ExprStmt(sp(Expr::UnaryOp {
                    op: UnaryOp::Neg,
                    operand: Box::new(sp(Expr::Ident("s".to_string()))),
                }))),
            ],
        };
        let errors = checker.check_module(&module);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_check_expr_unary_not_on_int_no_error() {
        // Python: `not 5` is valid (truthiness test), returns False
        let mut checker = TypeChecker::new();
        let module = Module {
            stmts: vec![
                sp(Stmt::VarDecl {
                    name: "n".to_string(),
                    ty: sp(TypeExpr::Named("int".to_string())),
                    value: sp(Expr::IntLit(5)),
                }),
                sp(Stmt::ExprStmt(sp(Expr::UnaryOp {
                    op: UnaryOp::Not,
                    operand: Box::new(sp(Expr::Ident("n".to_string()))),
                }))),
            ],
        };
        let errors = checker.check_module(&module);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_expr_unary_bitnot_on_float_emits_error() {
        let mut checker = TypeChecker::new();
        let module = Module {
            stmts: vec![
                sp(Stmt::VarDecl {
                    name: "f".to_string(),
                    ty: sp(TypeExpr::Named("float".to_string())),
                    value: sp(Expr::FloatLit(3.14)),
                }),
                sp(Stmt::ExprStmt(sp(Expr::UnaryOp {
                    op: UnaryOp::BitNot,
                    operand: Box::new(sp(Expr::Ident("f".to_string()))),
                }))),
            ],
        };
        let errors = checker.check_module(&module);
        assert!(!errors.is_empty());
    }

    // --- Special literal types ---

    #[test]
    fn test_check_expr_complex_lit() {
        let mut checker = TypeChecker::new();
        let ty = checker.check_expr(&sp(Expr::ComplexLit(2.0)));
        assert_eq!(checker.tcx.get(ty), &Ty::Any);
    }

    #[test]
    fn test_check_expr_bytes_lit() {
        let mut checker = TypeChecker::new();
        let ty = checker.check_expr(&sp(Expr::BytesLit(vec![104, 105])));
        assert_eq!(checker.tcx.get(ty), &Ty::Any);
    }

    #[test]
    fn test_check_expr_ellipsis() {
        let mut checker = TypeChecker::new();
        let ty = checker.check_expr(&sp(Expr::Ellipsis));
        assert_eq!(checker.tcx.get(ty), &Ty::Any);
    }

    // --- BinOp type mismatch ---

    #[test]
    fn test_check_expr_binop_int_add_str_emits_error() {
        let mut checker = TypeChecker::new();
        let module = Module {
            stmts: vec![sp(Stmt::ExprStmt(sp(Expr::BinOp {
                op: BinOp::Add,
                lhs: Box::new(sp(Expr::IntLit(1))),
                rhs: Box::new(sp(Expr::StrLit("a".to_string()))),
            })))],
        };
        let errors = checker.check_module(&module);
        assert!(!errors.is_empty());
    }
}
