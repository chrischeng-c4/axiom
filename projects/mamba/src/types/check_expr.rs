use super::check::{
    expr_to_type_expr, ClassPatternTarget, FunctionParamSig, NumericRoot, TypeChecker,
};
use super::generic::{
    check_bounds, complete_type_args, infer_type_args, GenericParams, Substitution,
};
use super::ty::{
    ClassRole, ExternalCallable, ExternalCallableAccess, ExternalCallableRuntimeKind, ExternalClass,
    ExternalValue, LiteralValue, TypeParamDefault, TypeVarKind,
};
use super::{Ty, TypeId};
use crate::parser::ast::*;
use crate::resolve::{SymbolId, SymbolKind};
use crate::source::span::{Span, Spanned};

#[derive(Debug)]
enum StdlibSpecCandidate {
    Accepted(Option<TypeId>),
    Rejected(Span, String, u8),
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StrictRelation {
    Compatible,
    Incompatible,
    Indeterminate,
}

enum UserProtocolMethod {
    Found(
        super::protocol::MethodSig,
        Option<Vec<FunctionParamSig>>,
    ),
    Missing,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StdlibSpecAccess {
    ModuleFn,
    Constructor,
    ClassMember,
    BoundMember,
}

#[derive(Debug)]
struct ResolvedStdlibSpecCall {
    module: String,
    qualifier: String,
    name: String,
    access: StdlibSpecAccess,
    receiver: Option<ExternalClass>,
}

fn bind_explicit_inference_args(
    args: &[CallArg],
    checked_arg_types: &[Option<TypeId>],
    params: &[FunctionParamSig],
) -> (Vec<TypeId>, Vec<TypeId>) {
    let positional_params: Vec<_> = params
        .iter()
        .enumerate()
        .filter(|(_, param)| param.kind == ParamKind::Regular && !param.kw_only)
        .map(|(idx, _)| idx)
        .collect();
    let star_param = params
        .iter()
        .position(|param| param.kind == ParamKind::Star);
    let double_star_param = params
        .iter()
        .position(|param| param.kind == ParamKind::DoubleStar);
    let mut matched_params = Vec::new();
    let mut matched_args = Vec::new();
    let mut bound_regular_params = Vec::new();
    let mut positional_idx = 0;
    let mut dynamic_positionals = false;
    let mut dynamic_keywords = false;

    for (arg, actual) in args.iter().zip(checked_arg_types) {
        let param_idx = match arg {
            CallArg::Positional(_) => {
                if dynamic_positionals {
                    continue;
                }
                let param_idx = positional_params
                    .get(positional_idx)
                    .copied()
                    .or(star_param);
                positional_idx += 1;
                param_idx
            }
            CallArg::Keyword { name, .. } => {
                if dynamic_keywords {
                    continue;
                }
                params
                    .iter()
                    .position(|param| {
                        param.kind == ParamKind::Regular && !param.pos_only && param.name == *name
                    })
                    .or(double_star_param)
            }
            CallArg::StarArg(_) => {
                dynamic_positionals = true;
                continue;
            }
            CallArg::DoubleStarArg(_) => {
                dynamic_keywords = true;
                continue;
            }
        };
        let (Some(param_idx), Some(actual)) = (param_idx, actual) else {
            continue;
        };
        let param = &params[param_idx];
        if param.kind == ParamKind::Regular && bound_regular_params.contains(&param_idx) {
            continue;
        }
        if param.kind == ParamKind::Regular {
            bound_regular_params.push(param_idx);
        }
        matched_params.push(param.ty);
        matched_args.push(*actual);
    }

    (matched_params, matched_args)
}

fn bind_compact_positional_inference_args(
    args: &[CallArg],
    checked_arg_types: &[Option<TypeId>],
    params: &[TypeId],
) -> (Vec<TypeId>, Vec<TypeId>) {
    let mut matched_params = Vec::new();
    let mut matched_args = Vec::new();
    let mut positional_idx = 0;
    let mut dynamic_positionals = false;

    for (arg, actual) in args.iter().zip(checked_arg_types) {
        match arg {
            CallArg::StarArg(_) => {
                dynamic_positionals = true;
                continue;
            }
            CallArg::Positional(_) if !dynamic_positionals => {}
            _ => continue,
        }
        if let (Some(param), Some(actual)) = (params.get(positional_idx), actual) {
            matched_params.push(*param);
            matched_args.push(*actual);
        }
        positional_idx += 1;
    }

    (matched_params, matched_args)
}

fn is_pep695_lazy_thunk_arg(
    func_name: Option<&str>,
    positional_index: usize,
    arg: &Spanned<Expr>,
) -> bool {
    matches!(arg.node, Expr::Lambda { .. })
        && matches!(
            (func_name, positional_index),
            (Some("__mb_pep695_typevar__"), 2 | 3 | 4) | (Some("__mb_pep695_type_alias__"), 1)
        )
}

/// Expression, operator, and pattern type checking.
impl TypeChecker {
    pub(crate) fn check_expr(&mut self, expr: &Spanned<Expr>) -> TypeId {
        match &expr.node {
            Expr::IntLit(_) | Expr::BigIntLit(_) => self.tcx.int(),
            Expr::FloatLit(_) => self.tcx.float(),
            Expr::ComplexLit(_) => {
                self.external_class_instance("builtins", "complex", Vec::new())
            }
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
            Expr::BytesLit(_) => {
                self.external_class_instance("builtins", "bytes", Vec::new())
            }
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
                if matches!(*op, BinOp::And | BinOp::Or) {
                    let mut conditional = Vec::new();
                    crate::resolve::pass::collect_walrus_targets(&rhs.node, &mut conditional);
                    self.invalidate_conditional_binding_names(conditional);
                }
                self.check_binop(*op, lt, rt, expr.span)
            }
            Expr::UnaryOp { op, operand } => {
                let ot = self.check_expr(operand);
                match op {
                    UnaryOp::Pos => {
                        // Bool is a subtype of int in Python — `+True == 1`,
                        // `+False == 0`, `type(+True) is int`. #1031: a class
                        // deriving int/float (`class P(int): pass`) is
                        // numeric too — `is_numeric_like` covers both.
                        // #1041: a class that is NOT numeric-derived but
                        // defines `__pos__` itself (walking bases) is also
                        // accepted — the runtime dispatches to the override
                        // (#1030); its result type isn't inferable here, so
                        // it resolves to `Any`.
                        let numeric_like = self.is_numeric_like(ot);
                        let has_pos_dunder =
                            !numeric_like && self.class_defines_dunder(ot, "__pos__");
                        if !numeric_like && !has_pos_dunder {
                            self.error(operand.span, "unary `+` requires numeric type");
                        }
                        if has_pos_dunder {
                            self.tcx.any()
                        } else if matches!(self.tcx.get(ot), Ty::Bool) {
                            self.tcx.int()
                        } else {
                            self.numeric_derived_result_ty(ot).unwrap_or(ot)
                        }
                    }
                    UnaryOp::Neg => {
                        // Bool is a subtype of int — `-True == -1`, `-False == 0`,
                        // `type(-True) is int`. #1031: same numeric-derived-class
                        // acceptance as `+`. #1041: same non-numeric-but-
                        // dunder-carrying acceptance as `+`, via `__neg__`.
                        let numeric_like = self.is_numeric_like(ot);
                        let has_neg_dunder =
                            !numeric_like && self.class_defines_dunder(ot, "__neg__");
                        if !numeric_like && !has_neg_dunder {
                            self.error(operand.span, "unary `-` requires numeric type");
                        }
                        if has_neg_dunder {
                            self.tcx.any()
                        } else if matches!(self.tcx.get(ot), Ty::Bool) {
                            self.tcx.int()
                        } else {
                            self.numeric_derived_result_ty(ot).unwrap_or(ot)
                        }
                    }
                    UnaryOp::Not => {
                        // Python `not` works on any type via truthiness testing.
                        // Always returns bool.
                        self.tcx.bool()
                    }
                    UnaryOp::BitNot => {
                        // Bool is a subtype of int — ~True == -2, ~False == -1.
                        // #1031: `~` is int-only (unlike `+`/`-`) — a
                        // float-derived class must still be rejected, so use
                        // `is_int_like` rather than `is_numeric_like`.
                        // #1041: a class that is NOT int-derived but defines
                        // `__invert__` itself (walking bases) is also
                        // accepted; result type resolves to `Any` since the
                        // runtime dispatch (#1030) decides it dynamically.
                        let int_like = self.is_int_like(ot);
                        let has_invert_dunder =
                            !int_like && self.class_defines_dunder(ot, "__invert__");
                        if !int_like && !has_invert_dunder {
                            self.error(operand.span, "`~` requires int type");
                        }
                        if has_invert_dunder {
                            self.tcx.any()
                        } else {
                            self.tcx.int()
                        }
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
                let func_symbol = func_name
                    .as_deref()
                    .and_then(|name| self.symbols.lookup(name));
                let mut builtin_constructor_result = func_symbol
                    .and_then(|symbol| self.builtin_class_aliases.get(&symbol).copied())
                    .or_else(|| {
                        func_name.as_deref().and_then(|name| {
                            self.is_unshadowed_builtin(name)
                                .then(|| self.builtin_class_pattern_instance(name))
                                .flatten()
                        })
                    });
                if builtin_constructor_result.is_some_and(|ty| {
                    matches!(
                        self.tcx.get(ty),
                        Ty::Class {
                            external: Some(external),
                            ..
                        } if external.module == "builtins" && external.name == "type"
                    )
                }) {
                    builtin_constructor_result = None;
                }
                let method_key = self.user_method_key(func);
                let unbound_user_method = match &func.node {
                    Expr::Attr { object, .. } => self.user_class_object(object).is_some(),
                    _ => false,
                };
                // A resolved generated TypeSpec contract owns the call even
                // when its result is indeterminate. The compact scalar table
                // is only a fallback when no generated candidate exists.
                let structured_stdlib =
                    self.check_structured_stdlib_call(func, func_ty_id, args);
                let structured_stdlib_handled = structured_stdlib.is_some();
                let structured_stdlib_authoritative =
                    structured_stdlib_handled && func_name.is_some();
                let stdlib_ret = match structured_stdlib {
                    Some(ret) => ret,
                    None => self.check_stdlib_call(func, args),
                };
                match func_ty {
                    Ty::Fn {
                        params,
                        ret,
                        variadic,
                        ..
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
                        if !structured_stdlib_authoritative
                            && !has_star
                            && !has_kwargs
                            && !might_have_defaults
                        {
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
                        let mut checked_arg_types = Vec::with_capacity(args.len());
                        let mut user_param_sigs = method_key
                            .as_ref()
                            .and_then(|(class_symbol, method_name)| {
                                self.class_method_param_sigs
                                    .get(class_symbol)
                                    .and_then(|methods| methods.get(method_name))
                                    .cloned()
                            })
                            .or_else(|| {
                                func_symbol.and_then(|symbol| {
                                    self.function_param_sigs.get(&symbol).cloned()
                                })
                            });
                        if method_key.is_some() {
                            if let Some(sigs) = &mut user_param_sigs {
                                let mut specialized = params.iter();
                                if unbound_user_method {
                                    specialized.next();
                                }
                                for (sig, specialized) in sigs.iter_mut().zip(specialized) {
                                    sig.ty = *specialized;
                                }
                                if unbound_user_method {
                                    if let Some(receiver) = params.first() {
                                        sigs.insert(
                                            0,
                                            FunctionParamSig {
                                                name: "self".to_string(),
                                                ty: *receiver,
                                                kind: ParamKind::Regular,
                                                pos_only: true,
                                                kw_only: false,
                                            },
                                        );
                                    }
                                }
                            }
                        }
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
                                    checked_arg_types.push(Some(at));
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
                                        let at = self.refine_class_object_actual(expected, at, a);
                                        if let Some(last) = checked_arg_types.last_mut() {
                                            *last = Some(at);
                                        }
                                        let bytes_literal_str_mismatch = matches!(
                                            (self.tcx.get(expected), &a.node),
                                            (Ty::Str, Expr::BytesLit(_))
                                        );
                                        if !structured_stdlib_authoritative
                                            && (bytes_literal_str_mismatch
                                                || !self.types_compatible(expected, at))
                                        {
                                            let got = if bytes_literal_str_mismatch {
                                                "bytes".to_string()
                                            } else {
                                                self.ty_name(at)
                                            };
                                            self.error(
                                                a.span,
                                                format!(
                                                    "argument type mismatch: expected `{}`, got `{}`",
                                                    self.ty_name(expected),
                                                    got,
                                                ),
                                            );
                                        }
                                    } else if let Some(star_param) =
                                        user_param_sigs.as_ref().and_then(|sigs| {
                                            sigs.iter().find(|sig| sig.kind == ParamKind::Star)
                                        })
                                    {
                                        self.check_user_fn_argument_type(
                                            a.span,
                                            star_param.ty,
                                            at,
                                            &a.node,
                                        );
                                    }
                                    param_idx += 1;
                                }
                                CallArg::Keyword { name, value } => {
                                    let at = self.check_expr(value);
                                    checked_arg_types.push(Some(at));
                                    if let Some(param) = user_param_sigs.as_ref().and_then(|sigs| {
                                        sigs.iter()
                                            .find(|sig| {
                                                sig.name == *name
                                                    && sig.kind == ParamKind::Regular
                                                    && !sig.pos_only
                                                    && sig.kw_only
                                            })
                                            .or_else(|| {
                                                sigs.iter().find(|sig| {
                                                    sig.name == *name
                                                        && sig.kind == ParamKind::Regular
                                                        && !sig.pos_only
                                                        && !sig.kw_only
                                                })
                                            })
                                            .or_else(|| {
                                                sigs.iter()
                                                    .find(|sig| sig.kind == ParamKind::DoubleStar)
                                            })
                                    }) {
                                        let at = self.refine_class_object_actual(
                                            param.ty,
                                            at,
                                            value,
                                        );
                                        if let Some(last) = checked_arg_types.last_mut() {
                                            *last = Some(at);
                                        }
                                        self.check_user_fn_argument_type(
                                            value.span,
                                            param.ty,
                                            at,
                                            &value.node,
                                        );
                                    }
                                }
                                CallArg::StarArg(a) | CallArg::DoubleStarArg(a) => {
                                    self.check_expr(a);
                                    checked_arg_types.push(None);
                                }
                            }
                        }
                        let (inference_params, inference_args) = user_param_sigs
                            .as_deref()
                            .map(|sigs| {
                                bind_explicit_inference_args(args, &checked_arg_types, sigs)
                            })
                            .unwrap_or_else(|| {
                                bind_compact_positional_inference_args(
                                    args,
                                    &checked_arg_types,
                                    &params,
                                )
                            });
                        // If generic function, infer type args and check bounds
                        let generic_params = method_key
                            .as_ref()
                            .and_then(|key| self.class_method_generic_defs.get(key))
                            .cloned()
                            .or_else(|| {
                                func_symbol
                                    .and_then(|symbol| self.generic_defs.get(&symbol))
                                    .cloned()
                            });
                        if let Some(gp) = generic_params {
                            let (subst, conflicts) =
                                infer_type_args(&gp, &inference_params, &inference_args, &self.tcx);
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
                        builtin_constructor_result.unwrap_or(ret)
                    }
                    Ty::TypeObject(instance) => {
                        for arg in args {
                            self.check_call_arg(arg);
                        }
                        instance
                    }
                    // #246: calling a class constructor returns instance of that class
                    Ty::Class {
                        role,
                        user,
                        external,
                        ..
                    } => {
                        if role == ClassRole::Instance {
                            for arg in args {
                                self.check_call_arg(arg);
                            }
                            let callable = self.class_defines_dunder(func_ty_id, "__call__");
                            if callable {
                                return self.tcx.any();
                            }
                            self.error(expr.span, "called value is not a function");
                            return self.tcx.error();
                        }
                        if external.is_some() {
                            if !structured_stdlib_handled {
                                for arg in args {
                                    self.check_call_arg(arg);
                                }
                            }
                            return self.with_class_role(func_ty_id, ClassRole::Instance);
                        }
                        let class_symbol = user.as_ref().map(|user| user.symbol).or(func_symbol);
                        let init_params = class_symbol
                            .and_then(|symbol| self.class_method_param_sigs.get(&symbol))
                            .and_then(|methods| methods.get("__init__"))
                            .cloned()
                            .unwrap_or_default();
                        let mut checked_arg_types = Vec::with_capacity(args.len());
                        for arg in args {
                            match arg {
                                CallArg::Positional(value) => {
                                    let actual = self.check_expr(value);
                                    checked_arg_types.push(Some(actual));
                                }
                                CallArg::Keyword { value, .. } => {
                                    let actual = self.check_expr(value);
                                    checked_arg_types.push(Some(actual));
                                }
                                CallArg::StarArg(value) | CallArg::DoubleStarArg(value) => {
                                    self.check_expr(value);
                                    checked_arg_types.push(None);
                                }
                            }
                        }
                        let (inference_params, inference_args) =
                            bind_explicit_inference_args(args, &checked_arg_types, &init_params);
                        // If generic class, infer type params from constructor args
                        if let Some(gp) = class_symbol
                            .and_then(|symbol| self.generic_defs.get(&symbol))
                            .cloned()
                        {
                            let is_open = user.as_ref().is_some_and(|user| {
                                gp.params.len() == user.args.len()
                                    && gp.params.iter().zip(&user.args).all(|(param, arg)| {
                                        matches!(self.tcx.get(*arg), Ty::TypeVar(id) if *id == param.id)
                                    })
                            });
                            if !is_open {
                                if let Some(user) = &user {
                                    let mut explicit = Substitution::new();
                                    for (param, arg) in gp.params.iter().zip(&user.args) {
                                        explicit.insert(param.id, *arg);
                                    }
                                    self.check_substituted_constructor_args(
                                        &inference_params,
                                        &inference_args,
                                        &explicit,
                                        expr.span,
                                    );
                                }
                                return self.with_class_role(func_ty_id, ClassRole::Instance);
                            }
                            let (subst, conflicts) = if init_params.is_empty() {
                                (Substitution::new(), Vec::new())
                            } else {
                                infer_type_args(&gp, &inference_params, &inference_args, &self.tcx)
                            };
                            for err in conflicts {
                                self.error(expr.span, err);
                            }
                            if let Some((completed, resolved)) =
                                complete_type_args(&gp, subst.clone(), &mut self.tcx)
                            {
                                for err in check_bounds(&completed, &gp, &self.tcx) {
                                    self.error(expr.span, err);
                                }
                                self.check_substituted_constructor_args(
                                    &inference_params,
                                    &inference_args,
                                    &completed,
                                    expr.span,
                                );
                                if let Some(symbol) = class_symbol {
                                    return self.apply_user_class_specialization(
                                        symbol,
                                        func_ty_id,
                                        &completed,
                                        &resolved,
                                        ClassRole::Instance,
                                    );
                                }
                            } else {
                                for err in check_bounds(&subst, &gp, &self.tcx) {
                                    self.error(expr.span, err);
                                }
                            }
                        }
                        if user.is_some() {
                            self.check_substituted_constructor_args(
                                &inference_params,
                                &inference_args,
                                &Substitution::new(),
                                expr.span,
                            );
                        }
                        self.with_class_role(func_ty_id, ClassRole::Instance)
                    }
                    Ty::External(ExternalValue::Callable(_)) => {
                        if !structured_stdlib_handled {
                            for arg in args {
                                self.check_call_arg(arg);
                            }
                        }
                        stdlib_ret.unwrap_or_else(|| self.tcx.any())
                    }
                    Ty::External(ExternalValue::Module { .. }) => {
                        self.error(expr.span, "called value is not a function");
                        self.tcx.error()
                    }
                    Ty::Any => {
                        for arg in args {
                            self.check_call_arg(arg);
                        }
                        // #1021: a whole-module-import constructor call
                        // (`queue.Queue()`, `selectors.DefaultSelector()`)
                        // naming a known native class — give the call a
                        // concrete `Ty::Class` result (mirrors #982's
                        // dict-literal Any-hole fix) so the receiver has a
                        // type for `hir_to_mir.rs`'s `direct_method_fn` table
                        // to key off of instead of starving on a bare Any.
                        // Checked before the #887 stdlib-return fallback so
                        // it wins over a coarser scalar/Any StdlibSig guess.
                        if let Some(class_name) = self.native_ctor_class_call(func) {
                            return self.tcx.intern(Ty::Class {
                                name: class_name.to_string(),
                                role: ClassRole::Instance,
                                user: None,
                                external: None,
                                fields: Vec::new(),
                                match_args: None,
                            });
                        }
                        // #887: a from-imported / module-qualified stdlib
                        // callee resolves to `Ty::Any` at this layer (module
                        // bindings aren't typed as `Ty::Fn`) — feed the
                        // resolved `StdlibSig`'s concrete-scalar return type
                        // through when the ① hook found one, instead of
                        // always widening to `Any`.
                        stdlib_ret.unwrap_or_else(|| self.tcx.any())
                    }
                    Ty::Error => self.tcx.error(),
                    // #1586: heterogeneous-callable Union. `for C in set, list, ...:`
                    // binds C to a Union of Fn/Class types. If every member is
                    // callable, accept the call and return Any (join of return types).
                    Ty::Union(ref members)
                        if members.iter().all(|&member| match self.tcx.get(member) {
                            Ty::Fn { .. }
                            | Ty::External(ExternalValue::Callable(_))
                            | Ty::Any
                            | Ty::Error => true,
                            Ty::Class {
                                role: ClassRole::Object,
                                ..
                            } => true,
                            Ty::Class {
                                role: ClassRole::Instance,
                                ..
                            } => self.class_defines_dunder(member, "__call__"),
                            _ => false,
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
                if let Some(external) = self.resolve_external_value_attr(obj_ty_id, attr) {
                    return external;
                }
                if let Some(method_ty) =
                    self.resolve_unbound_class_method(object, obj_ty_id, attr)
                {
                    return method_ty;
                }
                if self.user_class_object(object).is_some() {
                    // Unknown dynamic class attributes remain callable through
                    // Any, but declared methods above retain their full
                    // receiver and parameter contract.
                    return self.tcx.any();
                }
                self.resolve_attr(obj_ty_id, attr, expr.span)
            }
            Expr::Index { object, index } => {
                if let Some(specialized) =
                    self.resolve_explicit_user_class_specialization(object, index, expr.span)
                {
                    return specialized;
                }
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
                    // Empty dict or unpack-only: Dict(Any, Any), mirroring the
                    // sibling []/set() empty-literal arms (List(Any)/Set(Any))
                    // so the dict fast path (#979) has a receiver type to key
                    // off of instead of starving on a bare Any.
                    let any = self.tcx.any();
                    self.tcx.intern(Ty::Dict(any, any))
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
                if elems.is_empty() {
                    let any = self.tcx.any();
                    self.tcx.intern(Ty::Set(any))
                } else {
                    let first = self.check_expr(&elems[0]);
                    let mut homogeneous = true;
                    for elem in &elems[1..] {
                        let et = self.check_expr(elem);
                        if !self.types_compatible(first, et) {
                            homogeneous = false;
                        }
                    }
                    let elem_ty = if homogeneous { first } else { self.tcx.any() };
                    self.tcx.intern(Ty::Set(elem_ty))
                }
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
                let et = self.check_expr(else_body);
                let joined = if bt == et {
                    bt
                } else if matches!(self.tcx.get(bt), Ty::External(_))
                    || matches!(self.tcx.get(et), Ty::External(_))
                {
                    self.tcx.any()
                } else if self.types_compatible(bt, et) {
                    bt
                } else if self.types_compatible(et, bt) {
                    et
                } else {
                    self.tcx.any()
                };
                let mut conditional = Vec::new();
                crate::resolve::pass::collect_walrus_targets(&body.node, &mut conditional);
                crate::resolve::pass::collect_walrus_targets(&else_body.node, &mut conditional);
                self.invalidate_conditional_binding_names(conditional);
                joined
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
                    param_spec: None,
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
                    let elem_ty = match self.semantic_ty(iter_ty) {
                        Ty::List(inner) => inner,
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
                        match self.semantic_ty(elem_ty) {
                            Ty::Tuple(ts) if ts.len() == gen.targets.len() => Some(ts),
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
                let mut conditional = Vec::new();
                crate::resolve::pass::collect_walrus_targets(&expr.node, &mut conditional);
                self.invalidate_conditional_binding_names(conditional);
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
                    let elem_ty = match self.semantic_ty(iter_ty) {
                        Ty::List(inner) => inner,
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
                        match self.semantic_ty(elem_ty) {
                            Ty::Tuple(ts) if ts.len() == gen.targets.len() => Some(ts),
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
                let key_ty = self.check_expr(key);
                let val_ty = self.check_expr(value);
                self.comprehension_depth -= 1;
                self.symbols.pop_scope();
                let mut conditional = Vec::new();
                crate::resolve::pass::collect_walrus_targets(&expr.node, &mut conditional);
                self.invalidate_conditional_binding_names(conditional);
                // Dict(key_ty, val_ty) from the comprehension's key/value
                // exprs, mirroring the dict-literal arm; fall back to
                // Dict(Any, Any) when a side is unresolvable (Ty::Error)
                // rather than widening the whole comprehension to error(),
                // which starved the #979 dict fast path of a receiver type.
                let any = self.tcx.any();
                let error = self.tcx.error();
                let final_kt = if key_ty == error { any } else { key_ty };
                let final_vt = if val_ty == error { any } else { val_ty };
                self.tcx.intern(Ty::Dict(final_kt, final_vt))
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

    fn check_user_fn_argument_type(
        &mut self,
        span: Span,
        expected: TypeId,
        actual: TypeId,
        actual_expr: &Expr,
    ) {
        let actual = self.refine_class_object_actual(
            expected,
            actual,
            &Spanned::new(actual_expr.clone(), span),
        );
        let bytes_literal_str_mismatch = matches!(
            (self.tcx.get(expected), actual_expr),
            (Ty::Str, Expr::BytesLit(_))
        );
        if bytes_literal_str_mismatch || !self.types_compatible(expected, actual) {
            let got = if bytes_literal_str_mismatch {
                "bytes".to_string()
            } else {
                self.ty_name(actual)
            };
            self.error(
                span,
                format!(
                    "argument type mismatch: expected `{}`, got `{}`",
                    self.ty_name(expected),
                    got,
                ),
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

    /// #1021: whole-module-import constructor calls (`queue.Queue()`,
    /// `selectors.DefaultSelector()`) that are known to construct a
    /// *specific* native stdlib class. Extensible allowlist — add `(module,
    /// class)` pairs here as more native classes gain receiver-type
    /// specialization in `hir_to_mir.rs`'s `direct_method_fn` table (#979).
    const NATIVE_CTOR_CLASSES: &[(&str, &str)] =
        &[("queue", "Queue"), ("selectors", "DefaultSelector")];

    /// #1021: resolve `func` to a concrete native-class name when it is
    /// unambiguously a whole-module-import attribute call (`queue.Queue(...)`)
    /// naming one of `NATIVE_CTOR_CLASSES`. Deliberately narrow
    /// (skip-when-unsure, matching this file's zero-false-positive goal):
    /// - Only fires for `Expr::Attr { object: Ident(base), attr }` — never a
    ///   bare `Ident` call, so a from-imported `Queue` (which could be
    ///   locally shadowed far more easily than a module name) never adopts
    ///   this path.
    /// - `base` must resolve through `import_origins` with an *empty*
    ///   qualifier — i.e. `base` is bound to the whole module (`import
    ///   queue` / `import queue as q`), not a from-imported class/value that
    ///   merely shares the module's name.
    /// - The `(module, attr)` pair must be an exact match in the allowlist;
    ///   any miss returns `None` and the caller falls back to `Any` as
    ///   before.
    ///
    /// Also recognizes a bare `Expr::Ident` callee that was previously bound
    /// to one of these classes via a *class-reference* alias assignment
    /// (`_Queue = queue.Queue`, then `_Queue()`) — the perf-tier "hoist
    /// convention" (#2097) that avoids a per-iteration module-attribute
    /// lookup. `class_ref_origins` is only ever populated by re-running this
    /// same Attr-shape check against the assignment's value (see
    /// `check_stmt.rs`), so this never widens what counts as a native
    /// constructor beyond the allowlist above.
    pub(crate) fn native_ctor_class_call(&self, func: &Spanned<Expr>) -> Option<&'static str> {
        match &func.node {
            Expr::Attr { object, attr } => {
                let Expr::Ident(base) = &object.node else {
                    return None;
                };
                let symbol = self.symbols.lookup(base)?;
                let (module, qual) = self.import_origins.get(&symbol)?;
                if !qual.is_empty() {
                    return None;
                }
                Self::NATIVE_CTOR_CLASSES
                    .iter()
                    .find(|&&(m, c)| m == module.as_str() && c == attr.as_str())
                    .map(|&(_, c)| c)
            }
            Expr::Ident(name) => self
                .symbols
                .lookup(name)
                .and_then(|symbol| self.class_ref_origins.get(&symbol).copied()),
            _ => None,
        }
    }

    /// #886: fall back to a receiver's already-*inferred* `Ty::Class` name to
    /// resolve a Method-row lookup when `instance_origins` misses. This covers
    /// receivers that `instance_origins` never sees because it is only
    /// populated by a direct `x = Cls(...)` / `object.__new__(Cls)` assignment
    /// through a *from-imported* qualifier (`check_stmt.rs`) — e.g. a
    /// receiver constructed from a builtin needing no import at all, such as
    /// `e = BaseException("boom")` (`Ty::Class{name:"BaseException"}` is
    /// already the inferred type of `e`, but `BaseException` never appears in
    /// `import_origins` so the direct instance-provenance path never fires).
    ///
    /// Deliberately conservative (skip-when-unsure, matching the file's
    /// zero-false-positive goal):
    /// - Never fires for a name that is a *user-defined* class (tracked in
    ///   `class_methods` for classes with methods, `user_bare_classes` for
    ///   bare ones) — a user class that happens to share a stdlib class's
    ///   name must not adopt the stdlib class's Method contract.
    /// - Only fires when the class name owns a `Method` row in *exactly one*
    ///   module across the sig tables. A name ambiguous across modules (no
    ///   real import to disambiguate) is left unresolved rather than guessed.
    fn stdlib_method_sig_by_class_name(
        &self,
        class_name: &str,
        attr: &str,
    ) -> Option<&'static super::stdlib_sigs::StdlibSig> {
        use super::stdlib_sigs::{SigKind, STDLIB_SIGS};
        use super::stdlib_sigs_generated::STDLIB_SIGS_GENERATED;
        if self.class_methods.contains_key(class_name)
            || self.user_bare_classes.contains(class_name)
        {
            return None;
        }
        let owns_class = |s: &&super::stdlib_sigs::StdlibSig| {
            matches!(s.kind, SigKind::Method) && s.qualifier == class_name
        };
        let mut modules: Vec<&'static str> = STDLIB_SIGS
            .iter()
            .chain(STDLIB_SIGS_GENERATED.iter())
            .filter(owns_class)
            .map(|s| s.module)
            .collect();
        modules.sort_unstable();
        modules.dedup();
        let [module] = modules.as_slice() else {
            return None;
        };
        super::stdlib_sigs::get(module, class_name, attr)
    }

    pub(crate) fn materialize_stdlib_type_param(
        &mut self,
        spec_id: super::stdlib_typespec::TypeParamSpecId,
    ) -> Option<TypeId> {
        use super::stdlib_typespec::{self as spec, TypeParamSpecKind};

        if self.stdlib_spec_type_param_failed.contains(&spec_id) {
            return None;
        }
        let var_id = if let Some(id) = self.stdlib_spec_type_params.get(&spec_id).copied() {
            id
        } else {
            let decl = spec::type_param(spec_id).clone();
            let kind = match decl.kind {
                TypeParamSpecKind::TypeVar => TypeVarKind::TypeVar,
                TypeParamSpecKind::TypeVarTuple => TypeVarKind::TypeVarTuple,
                TypeParamSpecKind::ParamSpec => TypeVarKind::ParamSpec,
            };
            let id = self.tcx.new_type_param(
                spec::string(decl.name).to_string(),
                kind,
                None,
                Vec::new(),
                TypeParamDefault::None,
            );
            self.stdlib_spec_type_params.insert(spec_id, id);
            id
        };
        let ty = self.tcx.intern(Ty::TypeVar(var_id));
        if self.stdlib_spec_type_param_initialized.contains(&spec_id)
            || self.stdlib_spec_type_param_initializing.contains(&spec_id)
        {
            return Some(ty);
        }

        self.stdlib_spec_type_param_initializing.insert(spec_id);
        let decl = spec::type_param(spec_id).clone();
        let metadata = (|| {
            let bound = match decl.bound {
                Some(bound) => Some(self.materialize_stdlib_type(bound)?),
                None => None,
            };
            let constraints = spec::edges(decl.constraints)
                .iter()
                .map(|constraint| self.materialize_stdlib_type(*constraint))
                .collect::<Option<Vec<_>>>()?;
            let default = match decl.default {
                Some(default) => {
                    TypeParamDefault::Resolved(self.materialize_stdlib_type(default)?)
                }
                None => TypeParamDefault::None,
            };
            Some((bound, constraints, default))
        })();
        self.stdlib_spec_type_param_initializing.remove(&spec_id);
        let Some((bound, constraints, default)) = metadata else {
            self.stdlib_spec_type_param_failed.insert(spec_id);
            return None;
        };
        self.tcx
            .set_type_var_metadata(var_id, bound, constraints, default);
        self.stdlib_spec_type_param_initialized.insert(spec_id);
        Some(ty)
    }

    fn stdlib_literal_values(
        &self,
        spec_id: super::stdlib_typespec::TypeSpecId,
    ) -> Option<Vec<LiteralValue>> {
        use super::stdlib_typespec::{self as spec, TypeSpecNode};

        match spec::node(spec_id) {
            TypeSpecNode::LiteralInt(value) => Some(vec![LiteralValue::Int(*value)]),
            TypeSpecNode::LiteralStr(value) => {
                Some(vec![LiteralValue::Str(spec::string(*value).to_string())])
            }
            TypeSpecNode::LiteralBool(value) => Some(vec![LiteralValue::Bool(*value)]),
            _ => None,
        }
    }

    fn materialize_stdlib_named_type(
        &mut self,
        module: &str,
        name: &str,
        kind: super::stdlib_typespec::TypeNameKind,
        args: Vec<TypeId>,
    ) -> Option<TypeId> {
        use super::stdlib_typespec::TypeNameKind;

        if kind == TypeNameKind::Alias && (module, name) == ("builtins", "_ClassInfo") {
            return Some(self.external_class_instance(module, name, args));
        }
        if kind == TypeNameKind::Alias {
            return self.materialize_stdlib_alias(module, name, args);
        }
        let ty = match (module, name) {
            ("builtins", "bool") if args.is_empty() => self.tcx.bool(),
            ("builtins", "int") if args.is_empty() => self.tcx.int(),
            ("builtins", "float") if args.is_empty() => self.tcx.float(),
            ("builtins", "str") if args.is_empty() => self.tcx.str(),
            ("builtins", "object") | ("typing", "Any") if args.is_empty() => {
                self.tcx.any()
            }
            ("typing" | "typing_extensions", "Never" | "NoReturn") if args.is_empty() => {
                self.tcx.never()
            }
            ("typing", "Self") | ("typing_extensions", "Self") if args.is_empty() => {
                self.tcx.intern(Ty::SelfType)
            }
            ("typing", "LiteralString") | ("typing_extensions", "LiteralString")
                if args.is_empty() =>
            {
                self.tcx.str()
            }
            ("builtins", "list") if args.is_empty() => {
                self.tcx.intern(Ty::List(self.tcx.any()))
            }
            ("builtins", "set") if args.is_empty() => {
                self.tcx.intern(Ty::Set(self.tcx.any()))
            }
            ("builtins", "dict") if args.is_empty() => {
                self.tcx.intern(Ty::Dict(self.tcx.any(), self.tcx.any()))
            }
            ("builtins", "tuple") if args.is_empty() => {
                self.tcx.intern(Ty::Tuple(Vec::new()))
            }
            ("builtins", "type") if args.is_empty() => {
                self.tcx.intern(Ty::TypeObject(self.tcx.any()))
            }
            _ if matches!(
                kind,
                TypeNameKind::Nominal
                    | TypeNameKind::Protocol
                    | TypeNameKind::Imported
                    | TypeNameKind::Builtin
            ) => {
                if let Some((_id, class)) = super::stdlib_typespec::class_spec(module, name) {
                    self.external_class_instance(
                        super::stdlib_typespec::string(class.module),
                        super::stdlib_typespec::string(class.qualifier),
                        args,
                    )
                } else if matches!(kind, TypeNameKind::Nominal | TypeNameKind::Builtin) {
                    self.external_class_instance(module, name, args)
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        Some(ty)
    }

    fn materialize_stdlib_alias(
        &mut self,
        module: &str,
        name: &str,
        args: Vec<TypeId>,
    ) -> Option<TypeId> {
        use super::stdlib_typespec as spec;

        let alias = spec::alias(module, name)?.clone();
        let key = (alias.module, alias.name);
        if !self.stdlib_spec_alias_initializing.insert(key) {
            return None;
        }
        let materialized = (|| {
            let target = self.materialize_stdlib_type(alias.target)?;
            let type_params = spec::type_param_edges(alias.type_params);
            if args.is_empty() {
                return Some(target);
            }
            if args.len() != type_params.len() {
                return None;
            }
            let mut substitution = Substitution::new();
            for (param, arg) in type_params.iter().zip(args) {
                let ty = self.materialize_stdlib_type_param(*param)?;
                let Ty::TypeVar(var) = self.tcx.get(ty) else {
                    return None;
                };
                substitution.insert(*var, arg);
            }
            Some(substitution.apply(target, &mut self.tcx))
        })();
        self.stdlib_spec_alias_initializing.remove(&key);
        materialized
    }

    fn materialize_stdlib_param_spec(
        &mut self,
        spec_id: super::stdlib_typespec::TypeSpecId,
    ) -> Option<super::ty::TypeVarId> {
        use super::stdlib_typespec::{self as spec, TypeParamSpecKind, TypeSpecNode};

        let TypeSpecNode::TypeParam(param_id) = spec::node(spec_id) else {
            return None;
        };
        if spec::type_param(*param_id).kind != TypeParamSpecKind::ParamSpec {
            return None;
        }
        let ty = self.materialize_stdlib_type_param(*param_id)?;
        let Ty::TypeVar(var) = self.tcx.get(ty) else {
            return None;
        };
        Some(*var)
    }

    fn materialize_stdlib_callable_params(
        &mut self,
        spec_id: super::stdlib_typespec::TypeSpecId,
    ) -> Option<(Vec<TypeId>, bool, Option<super::ty::TypeVarId>)> {
        use super::stdlib_typespec::{self as spec, TypeSpecNode};

        match spec::node(spec_id).clone() {
            TypeSpecNode::ParamList(range) => {
                let params = spec::edges(range)
                    .iter()
                    .map(|param| self.materialize_stdlib_type(*param))
                    .collect::<Option<Vec<_>>>()?;
                Some((params, false, None))
            }
            TypeSpecNode::Ellipsis => Some((Vec::new(), true, None)),
            TypeSpecNode::TypeParam(_) => {
                let param_spec = self.materialize_stdlib_param_spec(spec_id)?;
                Some((Vec::new(), false, Some(param_spec)))
            }
            TypeSpecNode::Apply { base, args } => {
                let TypeSpecNode::Name { module, name, .. } = spec::node(base) else {
                    return None;
                };
                if !matches!(
                    (spec::string(*module), spec::string(*name)),
                    ("typing", "Concatenate") | ("typing_extensions", "Concatenate")
                ) {
                    return None;
                }
                let args = spec::edges(args);
                let (tail, prefix) = args.split_last()?;
                let param_spec = self.materialize_stdlib_param_spec(*tail)?;
                let params = prefix
                    .iter()
                    .map(|param| self.materialize_stdlib_type(*param))
                    .collect::<Option<Vec<_>>>()?;
                Some((params, false, Some(param_spec)))
            }
            _ => None,
        }
    }

    pub(crate) fn materialize_stdlib_type(
        &mut self,
        spec_id: super::stdlib_typespec::TypeSpecId,
    ) -> Option<TypeId> {
        use super::stdlib_typespec::{self as spec, TypeSpecNode};

        if let Some(ty) = self.stdlib_spec_types.get(&spec_id).copied() {
            return Some(ty);
        }
        let node = spec::node(spec_id).clone();
        let ty = match node {
            TypeSpecNode::Missing | TypeSpecNode::Unsupported(_) => return None,
            TypeSpecNode::Any => self.tcx.any(),
            TypeSpecNode::Never => self.tcx.never(),
            TypeSpecNode::None | TypeSpecNode::LiteralNone => self.tcx.none(),
            TypeSpecNode::SelfType => self.tcx.intern(Ty::SelfType),
            TypeSpecNode::Ellipsis => self.tcx.any(),
            TypeSpecNode::TypeParam(id) => self.materialize_stdlib_type_param(id)?,
            TypeSpecNode::ParamSpecArgs(_) | TypeSpecNode::ParamSpecKwargs(_) => return None,
            TypeSpecNode::ForwardRef { target, .. } => {
                self.materialize_stdlib_type(target)?
            }
            TypeSpecNode::Name { module, name, kind } => {
                let module = spec::string(module);
                let name = spec::string(name);
                self.materialize_stdlib_named_type(module, name, kind, Vec::new())?
            }
            TypeSpecNode::Union(range) => {
                let members = spec::edges(range)
                    .iter()
                    .map(|member| self.materialize_stdlib_type(*member))
                    .collect::<Option<Vec<_>>>()?;
                self.tcx.intern(Ty::Union(members))
            }
            TypeSpecNode::Tuple(range) => {
                let members = spec::edges(range)
                    .iter()
                    .map(|member| self.materialize_stdlib_type(*member))
                    .collect::<Option<Vec<_>>>()?;
                self.tcx.intern(Ty::Tuple(members))
            }
            TypeSpecNode::ParamList(_) | TypeSpecNode::Unpack(_) => return None,
            TypeSpecNode::LiteralInt(_)
            | TypeSpecNode::LiteralStr(_)
            | TypeSpecNode::LiteralBool(_) => {
                self.tcx
                    .intern(Ty::Literal(self.stdlib_literal_values(spec_id)?))
            }
            TypeSpecNode::LiteralBytes(_) => return None,
            TypeSpecNode::Apply { base, args } => {
                let TypeSpecNode::Name {
                    module: base_module,
                    name: base_name,
                    kind: base_kind,
                } = spec::node(base)
                else {
                    return None;
                };
                let module = spec::string(*base_module);
                let name = spec::string(*base_name);
                let args = spec::edges(args);
                match (module, name) {
                    ("builtins", "list") if args.len() == 1 => {
                        let item = self.materialize_stdlib_type(args[0])?;
                        self.tcx.intern(Ty::List(item))
                    }
                    ("builtins", "set" | "frozenset") if args.len() == 1 => {
                        let item = self.materialize_stdlib_type(args[0])?;
                        if name == "set" {
                            self.tcx.intern(Ty::Set(item))
                        } else {
                            self.materialize_stdlib_named_type(
                                module,
                                name,
                                *base_kind,
                                vec![item],
                            )?
                        }
                    }
                    ("builtins", "dict") if args.len() == 2 => {
                        let key = self.materialize_stdlib_type(args[0])?;
                        let value = self.materialize_stdlib_type(args[1])?;
                        self.tcx.intern(Ty::Dict(key, value))
                    }
                    ("builtins", "tuple") => {
                        let items = args
                            .iter()
                            .map(|item| self.materialize_stdlib_type(*item))
                            .collect::<Option<Vec<_>>>()?;
                        self.tcx.intern(Ty::Tuple(items))
                    }
                    ("builtins", "type") if args.len() == 1 => {
                        let instance = self.materialize_stdlib_type(args[0])?;
                        self.tcx.intern(Ty::TypeObject(instance))
                    }
                    ("typing", "Optional") if args.len() == 1 => {
                        let item = self.materialize_stdlib_type(args[0])?;
                        self.tcx.intern(Ty::Union(vec![item, self.tcx.none()]))
                    }
                    ("typing", "Union") => {
                        let items = args
                            .iter()
                            .map(|item| self.materialize_stdlib_type(*item))
                            .collect::<Option<Vec<_>>>()?;
                        self.tcx.intern(Ty::Union(items))
                    }
                    ("typing", "Literal") | ("typing_extensions", "Literal") => {
                        let values = args
                            .iter()
                            .map(|item| self.stdlib_literal_values(*item))
                            .collect::<Option<Vec<_>>>()?
                            .into_iter()
                            .flatten()
                            .collect();
                        self.tcx.intern(Ty::Literal(values))
                    }
                    ("typing", "Callable") | ("collections.abc", "Callable")
                        if args.len() == 2 =>
                    {
                        let (params, variadic, param_spec) =
                            self.materialize_stdlib_callable_params(args[0])?;
                        let ret = self.materialize_stdlib_type(args[1])?;
                        self.tcx.intern(Ty::Fn {
                            params,
                            ret,
                            variadic,
                            param_spec,
                        })
                    }
                    ("typing", "TypeGuard")
                    | ("typing", "TypeIs")
                    | ("typing_extensions", "TypeGuard")
                    | ("typing_extensions", "TypeIs")
                        if args.len() == 1 =>
                    {
                        self.tcx.bool()
                    }
                    ("typing", "Annotated") | ("typing_extensions", "Annotated")
                        if !args.is_empty() =>
                    {
                        self.materialize_stdlib_type(args[0])?
                    }
                    ("typing", "ClassVar" | "Final" | "Required" | "NotRequired")
                    | (
                        "typing_extensions",
                        "ClassVar" | "Final" | "Required" | "NotRequired" | "ReadOnly",
                    ) if args.len() == 1 => self.materialize_stdlib_type(args[0])?,
                    _ => {
                        let materialized = args
                            .iter()
                            .map(|arg| self.materialize_stdlib_type(*arg))
                            .collect::<Option<Vec<_>>>()?;
                        self.materialize_stdlib_named_type(
                            module,
                            name,
                            *base_kind,
                            materialized,
                        )?
                    }
                }
            }
        };
        self.stdlib_spec_types.insert(spec_id, ty);
        Some(ty)
    }

    fn structured_stdlib_module_fn_exists(module: &str, name: &str) -> bool {
        super::stdlib_typespec::module_callable_exists(module, name)
    }

    fn structured_stdlib_member_owner(
        module: &str,
        qualifier: &str,
        name: &str,
    ) -> Option<(String, String)> {
        use super::stdlib_typespec::{self as spec, CallableSpecKind};
        spec::class_callable_owner(
            module,
            qualifier,
            name,
            &[
                CallableSpecKind::InstanceMethod,
                CallableSpecKind::ClassMethod,
                CallableSpecKind::StaticMethod,
            ],
        )
        .map(|(module, qualifier)| (module.to_string(), qualifier.to_string()))
    }

    fn structured_stdlib_direct_member_owner(
        module: &str,
        qualifier: &str,
        name: &str,
    ) -> Option<(String, String)> {
        use super::stdlib_typespec::{self as spec, CallableSpecKind};

        let resolution = spec::class_callable_resolution(
            module,
            qualifier,
            name,
            &[
                CallableSpecKind::InstanceMethod,
                CallableSpecKind::ClassMethod,
                CallableSpecKind::StaticMethod,
            ],
        )?;
        if !resolution.path.is_empty() {
            return None;
        }
        let owner = spec::class_by_id(resolution.owner);
        Some((
            spec::string(owner.module).to_string(),
            spec::string(owner.qualifier).to_string(),
        ))
    }

    fn structured_stdlib_constructor(
        module: &str,
        qualifier: &str,
    ) -> Option<(String, String, &'static str)> {
        for name in ["__init__", "__new__"] {
            if let Some((owner_module, owner_qualifier)) =
                Self::structured_stdlib_direct_member_owner(module, qualifier, name)
            {
                return Some((owner_module, owner_qualifier, name));
            }
        }
        for name in ["__init__", "__new__"] {
            if let Some((owner_module, owner_qualifier)) =
                Self::structured_stdlib_member_owner(module, qualifier, name)
            {
                return Some((owner_module, owner_qualifier, name));
            }
        }
        None
    }

    fn structured_stdlib_receiver(&self, ty: TypeId) -> Option<ExternalClass> {
        let receiver = match self.tcx.get(ty) {
            Ty::Bool => ExternalClass {
                module: "builtins".to_string(),
                name: "bool".to_string(),
                args: Vec::new(),
            },
            Ty::Int => ExternalClass {
                module: "builtins".to_string(),
                name: "int".to_string(),
                args: Vec::new(),
            },
            Ty::Float => ExternalClass {
                module: "builtins".to_string(),
                name: "float".to_string(),
                args: Vec::new(),
            },
            Ty::Str => ExternalClass {
                module: "builtins".to_string(),
                name: "str".to_string(),
                args: Vec::new(),
            },
            Ty::List(item) => ExternalClass {
                module: "builtins".to_string(),
                name: "list".to_string(),
                args: vec![*item],
            },
            Ty::Set(item) => ExternalClass {
                module: "builtins".to_string(),
                name: "set".to_string(),
                args: if self.tcx.get(*item).is_any() {
                    Vec::new()
                } else {
                    vec![*item]
                },
            },
            Ty::Dict(key, value) => ExternalClass {
                module: "builtins".to_string(),
                name: "dict".to_string(),
                args: vec![*key, *value],
            },
            Ty::Tuple(_) => ExternalClass {
                module: "builtins".to_string(),
                name: "tuple".to_string(),
                args: Vec::new(),
            },
            Ty::Class {
                external: Some(external),
                ..
            } => external.clone(),
            _ => return None,
        };
        Some(receiver)
    }

    fn external_callable_target(&self, ty: TypeId) -> Option<ResolvedStdlibSpecCall> {
        match self.tcx.get(ty) {
            Ty::External(ExternalValue::Callable(callable)) => Some(ResolvedStdlibSpecCall {
                module: callable.module.clone(),
                qualifier: callable.qualifier.clone(),
                name: callable.name.clone(),
                access: match callable.access {
                    ExternalCallableAccess::Module => StdlibSpecAccess::ModuleFn,
                    ExternalCallableAccess::ClassMember => StdlibSpecAccess::ClassMember,
                    ExternalCallableAccess::BoundMember => StdlibSpecAccess::BoundMember,
                },
                receiver: callable.receiver.clone(),
            }),
            Ty::Class {
                role: ClassRole::Object,
                external: Some(receiver),
                ..
            } => {
                let (module, qualifier, name) = Self::structured_stdlib_constructor(
                    &receiver.module,
                    &receiver.name,
                )?;
                Some(ResolvedStdlibSpecCall {
                    module,
                    qualifier,
                    name: name.to_string(),
                    access: StdlibSpecAccess::Constructor,
                    receiver: Some(receiver.clone()),
                })
            }
            _ => None,
        }
    }

    fn inferred_structured_stdlib_instance(
        &self,
        base: &str,
    ) -> Option<(String, String)> {
        let symbol = self.symbols.lookup(base)?;
        if let Some(origin) = self.instance_origins.get(&symbol) {
            return Some(origin.clone());
        }
        let ty = self.get_sym_type(symbol.0);
        if let Some(receiver) = self.structured_stdlib_receiver(ty) {
            return Some((receiver.module, receiver.name));
        }
        (self.symbols.get_symbol(symbol).kind == SymbolKind::Function)
            .then(|| ("builtins".to_string(), "function".to_string()))
    }

    fn resolve_structured_stdlib_call(
        &self,
        func: &Spanned<Expr>,
        func_ty: TypeId,
    ) -> Option<ResolvedStdlibSpecCall> {
        if let Some(target) = self.external_callable_target(func_ty) {
            return Some(target);
        }
        match &func.node {
            Expr::Ident(name) => self
                .symbols
                .lookup(name)
                .and_then(|symbol| self.import_origins.get(&symbol))
                .and_then(|(module, member)| {
                    let member = if member.is_empty() { name } else { member };
                    if Self::structured_stdlib_module_fn_exists(module, member) {
                        Some(ResolvedStdlibSpecCall {
                            module: module.clone(),
                            qualifier: String::new(),
                            name: member.to_string(),
                            access: StdlibSpecAccess::ModuleFn,
                            receiver: None,
                        })
                    } else {
                        Self::structured_stdlib_constructor(module, member).map(|(
                            owner_module,
                            owner_qualifier,
                            constructor,
                        )| {
                            ResolvedStdlibSpecCall {
                                module: owner_module,
                                qualifier: owner_qualifier,
                                name: constructor.to_string(),
                                access: StdlibSpecAccess::Constructor,
                                receiver: Some(ExternalClass {
                                    module: module.clone(),
                                    name: member.to_string(),
                                    args: Vec::new(),
                                }),
                            }
                        })
                    }
                })
                .or_else(|| {
                    if !self.is_unshadowed_builtin(name) {
                        return None;
                    }
                    if Self::structured_stdlib_module_fn_exists("builtins", name) {
                        Some(ResolvedStdlibSpecCall {
                            module: "builtins".to_string(),
                            qualifier: String::new(),
                            name: name.clone(),
                            access: StdlibSpecAccess::ModuleFn,
                            receiver: None,
                        })
                    } else {
                        Self::structured_stdlib_constructor("builtins", name).map(|(
                            owner_module,
                            owner_qualifier,
                            constructor,
                        )| {
                            ResolvedStdlibSpecCall {
                                module: owner_module,
                                qualifier: owner_qualifier,
                                name: constructor.to_string(),
                                access: StdlibSpecAccess::Constructor,
                                receiver: Some(ExternalClass {
                                    module: "builtins".to_string(),
                                    name: name.clone(),
                                    args: Vec::new(),
                                }),
                            }
                        })
                    }
                }),
            Expr::Attr { object, attr } => {
                if let Expr::Ident(base) = &object.node {
                    if let Some(symbol) = self.symbols.lookup(base) {
                        if let Some((module, qualifier)) = self.import_origins.get(&symbol) {
                            if qualifier.is_empty() {
                                if Self::structured_stdlib_module_fn_exists(module, attr) {
                                    return Some(ResolvedStdlibSpecCall {
                                        module: module.clone(),
                                        qualifier: String::new(),
                                        name: attr.clone(),
                                        access: StdlibSpecAccess::ModuleFn,
                                        receiver: None,
                                    });
                                }
                                if let Some((owner_module, owner_qualifier, constructor)) =
                                    Self::structured_stdlib_constructor(module, attr)
                                {
                                    return Some(ResolvedStdlibSpecCall {
                                        module: owner_module,
                                        qualifier: owner_qualifier,
                                        name: constructor.to_string(),
                                        access: StdlibSpecAccess::Constructor,
                                        receiver: Some(ExternalClass {
                                            module: module.clone(),
                                            name: attr.clone(),
                                            args: Vec::new(),
                                        }),
                                    });
                                }
                            } else if let Some((owner_module, owner_qualifier)) =
                                Self::structured_stdlib_member_owner(module, qualifier, attr)
                            {
                                return Some(ResolvedStdlibSpecCall {
                                    module: owner_module,
                                    qualifier: owner_qualifier,
                                    name: attr.clone(),
                                    access: StdlibSpecAccess::ClassMember,
                                    receiver: Some(ExternalClass {
                                        module: module.clone(),
                                        name: qualifier.clone(),
                                        args: Vec::new(),
                                    }),
                                });
                            }
                        }
                    }
                    if let Some((module, qualifier)) =
                        self.inferred_structured_stdlib_instance(base)
                    {
                        if let Some((owner_module, owner_qualifier)) =
                            Self::structured_stdlib_member_owner(&module, &qualifier, attr)
                        {
                            return Some(ResolvedStdlibSpecCall {
                                module: owner_module,
                                qualifier: owner_qualifier,
                                name: attr.clone(),
                                access: StdlibSpecAccess::BoundMember,
                                receiver: Some(ExternalClass {
                                    module,
                                    name: qualifier,
                                    args: Vec::new(),
                                }),
                            });
                        }
                    }
                }
                let Expr::Attr {
                    object: module_object,
                    attr: qualifier,
                } = &object.node
                else {
                    return None;
                };
                let Expr::Ident(module_alias) = &module_object.node else {
                    return None;
                };
                let symbol = self.symbols.lookup(module_alias)?;
                let (module, imported) = self.import_origins.get(&symbol)?;
                if !imported.is_empty() {
                    return None;
                }
                let (owner_module, owner_qualifier) =
                    Self::structured_stdlib_member_owner(module, qualifier, attr)?;
                Some(ResolvedStdlibSpecCall {
                    module: owner_module,
                    qualifier: owner_qualifier,
                    name: attr.clone(),
                    access: StdlibSpecAccess::ClassMember,
                    receiver: Some(ExternalClass {
                        module: module.clone(),
                        name: qualifier.clone(),
                        args: Vec::new(),
                    }),
                })
            }
            _ => None,
        }
    }

    fn stdlib_spec_generic_params(
        &mut self,
        sig: &super::stdlib_typespec::CallableSpec,
    ) -> Option<GenericParams> {
        use super::stdlib_typespec as spec;

        let mut params = GenericParams::new();
        for spec_id in spec::type_param_edges(sig.type_params) {
            let ty = self.materialize_stdlib_type_param(*spec_id)?;
            let Ty::TypeVar(var_id) = self.tcx.get(ty) else {
                return None;
            };
            let info = self.tcx.get_type_var(*var_id).clone();
            params.add_param(
                &info.name,
                *var_id,
                info.kind,
                info.bound,
                info.constraints,
                info.default,
            );
        }
        Some(params)
    }

    fn stdlib_receiver_substitution(
        &mut self,
        receiver: &ExternalClass,
        resolution: &super::stdlib_typespec::ClassCallableResolution,
    ) -> Option<Substitution> {
        use super::stdlib_typespec as spec;

        let root_id = resolution
            .path
            .first()
            .map(|step| step.child)
            .unwrap_or(resolution.owner);
        let root = spec::class_by_id(root_id);
        let root_params = spec::class_type_params(root);
        if receiver.args.len() > root_params.len() {
            return None;
        }
        let mut substitution = Substitution::new();
        for (param, arg) in root_params.iter().zip(&receiver.args) {
            let Some(param_ty) = self.materialize_stdlib_type_param(*param) else {
                continue;
            };
            let Ty::TypeVar(var) = self.tcx.get(param_ty) else {
                continue;
            };
            substitution.insert(*var, *arg);
        }
        let mut current = root_id;
        for step in &resolution.path {
            if step.child != current {
                return None;
            }
            let base = spec::class_by_id(step.base);
            let base_params = spec::class_type_params(base);
            let base_args = spec::edges(step.args);
            if base_params.len() != base_args.len() {
                return None;
            }
            let mut projected = Substitution::new();
            for (param, arg) in base_params.iter().zip(base_args) {
                let param_ty = self.materialize_stdlib_type_param(*param)?;
                let var = match self.tcx.get(param_ty) {
                    Ty::TypeVar(var) => *var,
                    _ => return None,
                };
                let arg = self.materialize_stdlib_type(*arg)?;
                let arg = substitution.apply(arg, &mut self.tcx);
                projected.insert(var, arg);
            }
            substitution = projected;
            current = step.base;
        }
        (current == resolution.owner).then_some(substitution)
    }

    fn known_stdlib_class_projection(
        &mut self,
        expected: &super::ty::ExternalClass,
        expected_ty: TypeId,
        actual: TypeId,
    ) -> Option<bool> {
        let shape = match (expected.module.as_str(), expected.name.as_str()) {
            ("builtins", "_ClassInfo") => match self.tcx.get(actual).clone() {
                Ty::TypeObject(_) => Some(true),
                Ty::Class {
                    role: ClassRole::Object,
                    ..
                } => Some(true),
                Ty::Tuple(items) => {
                    let mut unknown = false;
                    for item in items {
                        match self.known_stdlib_class_projection(expected, expected_ty, item) {
                            Some(true) => {}
                            Some(false) => return Some(false),
                            None => unknown = true,
                        }
                    }
                    if unknown { None } else { Some(true) }
                }
                Ty::Any | Ty::Error | Ty::TypeVar(_) | Ty::Infer(_) => None,
                _ => Some(false),
            },
            ("os", "PathLike") => match self.tcx.get(actual) {
                Ty::Any | Ty::Error | Ty::TypeVar(_) | Ty::Infer(_) => None,
                Ty::Class { .. } => None,
                _ => Some(false),
            },
            ("typing", "SupportsIndex") => match self.tcx.get(actual) {
                Ty::Int | Ty::Bool => Some(true),
                Ty::Float
                | Ty::Str
                | Ty::None
                | Ty::List(_)
                | Ty::Set(_)
                | Ty::Dict(_, _)
                | Ty::Tuple(_) => Some(false),
                _ => None,
            },
            ("typing", "Iterable" | "Collection") => match self.tcx.get(actual) {
                Ty::List(_)
                | Ty::Set(_)
                | Ty::Dict(_, _)
                | Ty::Tuple(_)
                | Ty::Str => Some(true),
                Ty::Int | Ty::Float | Ty::Bool | Ty::None => Some(false),
                Ty::Class {
                    external: Some(actual),
                    ..
                } if actual.module == "builtins"
                    && matches!(
                        actual.name.as_str(),
                        "bytes" | "bytearray" | "range" | "frozenset"
                    ) => Some(true),
                _ => None,
            },
            ("typing", "Sequence") => match self.tcx.get(actual) {
                Ty::List(_) | Ty::Tuple(_) | Ty::Str => Some(true),
                Ty::Int | Ty::Float | Ty::Bool | Ty::None | Ty::Set(_) | Ty::Dict(_, _) => {
                    Some(false)
                }
                Ty::Class {
                    external: Some(actual),
                    ..
                } if actual.module == "builtins"
                    && matches!(actual.name.as_str(), "bytes" | "bytearray" | "range") =>
                {
                    Some(true)
                }
                _ => None,
            },
            ("typing", "MutableSequence") => match self.tcx.get(actual) {
                Ty::List(_) => Some(true),
                Ty::Int
                | Ty::Float
                | Ty::Bool
                | Ty::Str
                | Ty::None
                | Ty::Set(_)
                | Ty::Dict(_, _)
                | Ty::Tuple(_) => Some(false),
                Ty::Class {
                    external: Some(actual),
                    ..
                } if actual.module == "builtins" && actual.name == "bytearray" => Some(true),
                Ty::Class {
                    external: Some(actual),
                    ..
                } if actual.module == "builtins"
                    && matches!(actual.name.as_str(), "bytes" | "range" | "frozenset") =>
                {
                    Some(false)
                }
                _ => None,
            },
            ("typing", "Mapping" | "MutableMapping") => match self.tcx.get(actual) {
                Ty::Dict(_, _) => Some(true),
                Ty::Int
                | Ty::Float
                | Ty::Bool
                | Ty::Str
                | Ty::None
                | Ty::List(_)
                | Ty::Set(_)
                | Ty::Tuple(_) => Some(false),
                _ => None,
            },
            _ => None,
        }?;
        if !shape || expected.args.is_empty() {
            return Some(shape);
        }
        Some(self.types_compatible(expected_ty, actual))
    }

    fn stdlib_protocol_marker(spec_id: super::stdlib_typespec::TypeSpecId) -> bool {
        use super::stdlib_typespec::{self as spec, TypeSpecNode};

        match spec::node(spec_id) {
            TypeSpecNode::Name { module, name, .. } => {
                spec::string(*module) == "typing"
                    && matches!(spec::string(*name), "Protocol" | "Generic")
            }
            TypeSpecNode::Apply { base, .. } => Self::stdlib_protocol_marker(*base),
            _ => false,
        }
    }

    fn user_protocol_method(
        &self,
        symbol: SymbolId,
        name: &str,
        visiting: &mut std::collections::HashSet<SymbolId>,
    ) -> UserProtocolMethod {
        if !visiting.insert(symbol) {
            return UserProtocolMethod::Indeterminate;
        }
        if let Some(method) = self
            .class_methods_by_symbol
            .get(&symbol)
            .and_then(|methods| methods.get(name))
            .cloned()
        {
            let params = self
                .class_method_param_sigs
                .get(&symbol)
                .and_then(|methods| methods.get(name))
                .cloned();
            visiting.remove(&symbol);
            return UserProtocolMethod::Found(method, params);
        }
        if self.class_inheritance_open.contains(&symbol) {
            visiting.remove(&symbol);
            return UserProtocolMethod::Indeterminate;
        }
        let mut found = None;
        let mut ambiguous = false;
        for base in self
            .class_base_symbols
            .get(&symbol)
            .into_iter()
            .flatten()
        {
            match self.user_protocol_method(*base, name, visiting) {
                UserProtocolMethod::Found(method, params) => {
                    if found.is_some() {
                        ambiguous = true;
                    } else {
                        found = Some((method, params));
                    }
                }
                UserProtocolMethod::Missing => {}
                UserProtocolMethod::Indeterminate => ambiguous = true,
            }
        }
        visiting.remove(&symbol);
        if ambiguous {
            UserProtocolMethod::Indeterminate
        } else if let Some((method, params)) = found {
            UserProtocolMethod::Found(method, params)
        } else {
            UserProtocolMethod::Missing
        }
    }

    fn stdlib_object_declares_method(name: &str) -> bool {
        use super::stdlib_typespec as spec;

        spec::class_spec("builtins", "object").is_some_and(|(_, class)| {
            spec::class_methods(class).any(|method| {
                method.py312
                    && method.kind == spec::CallableSpecKind::InstanceMethod
                    && spec::string(method.name) == name
            })
        })
    }

    fn stdlib_protocol_relation(
        &mut self,
        class: &super::stdlib_typespec::ClassSpec,
        expected_external: &super::ty::ExternalClass,
        actual: TypeId,
        visiting: &mut std::collections::HashSet<(TypeId, TypeId)>,
    ) -> StrictRelation {
        use super::stdlib_typespec::{self as spec, CallableSpecKind, ParamSpecKind};

        let Ty::Class {
            user: Some(actual_user),
            ..
        } = self.tcx.get(actual).clone()
        else {
            return StrictRelation::Indeterminate;
        };
        let class_params = spec::class_type_params(class);
        if !expected_external.args.is_empty() && expected_external.args.len() != class_params.len() {
            return StrictRelation::Indeterminate;
        }
        let mut substitution = Substitution::new();
        for (index, param) in class_params.iter().enumerate() {
            let Some(param_ty) = self.materialize_stdlib_type_param(*param) else {
                return StrictRelation::Indeterminate;
            };
            let Ty::TypeVar(var) = self.tcx.get(param_ty) else {
                return StrictRelation::Indeterminate;
            };
            let arg = expected_external
                .args
                .get(index)
                .copied()
                .unwrap_or_else(|| self.tcx.any());
            substitution.insert(*var, arg);
        }

        let mut method_names = Vec::new();
        for method in spec::class_methods(class).filter(|method| {
            method.py312 && method.kind == CallableSpecKind::InstanceMethod
        }) {
            let name = spec::string(method.name);
            if !method_names.contains(&name) {
                method_names.push(name);
            }
        }
        let mut saw_indeterminate = false;
        for name in method_names {
            let required: Vec<_> = spec::class_methods(class)
                .filter(|method| {
                    method.py312
                        && method.kind == CallableSpecKind::InstanceMethod
                        && spec::string(method.name) == name
                })
                .collect();
            let (actual_method, actual_params) = match self.user_protocol_method(
                actual_user.symbol,
                name,
                &mut std::collections::HashSet::new(),
            ) {
                UserProtocolMethod::Found(method, params) => (method, params),
                UserProtocolMethod::Indeterminate => {
                    saw_indeterminate = true;
                    continue;
                }
                UserProtocolMethod::Missing => {
                    return if Self::stdlib_object_declares_method(name) {
                        StrictRelation::Indeterminate
                    } else {
                        StrictRelation::Incompatible
                    };
                }
            };
            if required.len() != 1
                || spec::type_param_edges(required[0].type_params)
                    .iter()
                    .any(|param| !class_params.contains(param))
            {
                saw_indeterminate = true;
                continue;
            }
            let required = required[0];
            let required_params: Vec<_> = spec::params(required.params)
                .iter()
                .filter(|param| !param.implicit_receiver)
                .collect();
            if required_params.iter().any(|param| {
                matches!(param.kind, ParamSpecKind::VarPos | ParamSpecKind::VarKw)
            }) || required_params.len() != actual_method.params.len()
            {
                saw_indeterminate = true;
                continue;
            }
            let Some(actual_params) = actual_params else {
                saw_indeterminate = true;
                continue;
            };
            if actual_params.len() != required_params.len() {
                saw_indeterminate = true;
                continue;
            }
            for ((required_param, actual_param), actual_ty) in required_params
                .iter()
                .zip(&actual_params)
                .zip(&actual_method.params)
            {
                let kind_compatible = match required_param.kind {
                    ParamSpecKind::PosOnly => {
                        (actual_param.kind == ParamKind::Regular && !actual_param.kw_only)
                            || actual_param.kind == ParamKind::Star
                    }
                    ParamSpecKind::PosOrKw => {
                        actual_param.kind == ParamKind::Regular
                            && !actual_param.pos_only
                            && !actual_param.kw_only
                    }
                    ParamSpecKind::KwOnly => {
                        (actual_param.kind == ParamKind::Regular && !actual_param.pos_only)
                            || actual_param.kind == ParamKind::DoubleStar
                    }
                    ParamSpecKind::VarPos | ParamSpecKind::VarKw => false,
                };
                let required_name = spec::string(required_param.name);
                let name_compatible = match required_param.kind {
                    ParamSpecKind::PosOnly => true,
                    ParamSpecKind::PosOrKw => actual_param.name == required_name,
                    ParamSpecKind::KwOnly => {
                        actual_param.kind == ParamKind::DoubleStar
                            || actual_param.name == required_name
                    }
                    ParamSpecKind::VarPos | ParamSpecKind::VarKw => false,
                };
                if !kind_compatible || !name_compatible {
                    return StrictRelation::Incompatible;
                }
                let required_ty = spec::type_use(required_param.ty).0;
                let Some(required_ty) = self.materialize_stdlib_type(required_ty) else {
                    saw_indeterminate = true;
                    continue;
                };
                let required_ty = substitution.apply(required_ty, &mut self.tcx);
                match self.stdlib_type_relation_inner(*actual_ty, required_ty, visiting) {
                    StrictRelation::Compatible => {}
                    StrictRelation::Incompatible => return StrictRelation::Incompatible,
                    StrictRelation::Indeterminate => saw_indeterminate = true,
                }
                if required_param.has_default {
                    saw_indeterminate = true;
                }
            }
            let Some(required_ret) = self.materialize_stdlib_type(spec::type_use(required.ret).0)
            else {
                saw_indeterminate = true;
                continue;
            };
            let required_ret = substitution.apply(required_ret, &mut self.tcx);
            match self.stdlib_type_relation_inner(
                required_ret,
                actual_method.return_type,
                visiting,
            ) {
                StrictRelation::Compatible => {}
                StrictRelation::Incompatible => return StrictRelation::Incompatible,
                StrictRelation::Indeterminate => saw_indeterminate = true,
            }
        }

        for base in spec::class_bases(class) {
            if Self::stdlib_protocol_marker(*base) {
                continue;
            }
            let Some(base) = self.materialize_stdlib_type(*base) else {
                saw_indeterminate = true;
                continue;
            };
            let base = substitution.apply(base, &mut self.tcx);
            match self.stdlib_type_relation_inner(base, actual, visiting) {
                StrictRelation::Compatible => {}
                StrictRelation::Incompatible => return StrictRelation::Incompatible,
                StrictRelation::Indeterminate => saw_indeterminate = true,
            }
        }
        if !class.method_only_complete || saw_indeterminate {
            StrictRelation::Indeterminate
        } else {
            StrictRelation::Compatible
        }
    }

    fn stdlib_type_relation(&mut self, expected: TypeId, actual: TypeId) -> StrictRelation {
        self.stdlib_type_relation_inner(expected, actual, &mut std::collections::HashSet::new())
    }

    fn stdlib_callable_relation_inner(
        &mut self,
        expected: TypeId,
        actual: TypeId,
        visiting: &mut std::collections::HashSet<(TypeId, TypeId)>,
    ) -> StrictRelation {
        let Ty::Fn {
            params: expected_params,
            ret: expected_ret,
            variadic: expected_variadic,
            param_spec: expected_param_spec,
        } = self.tcx.get(expected).clone()
        else {
            return StrictRelation::Indeterminate;
        };
        let Ty::Fn {
            params: actual_params,
            ret: actual_ret,
            variadic: actual_variadic,
            param_spec: actual_param_spec,
        } = self.tcx.get(actual).clone()
        else {
            return match self.tcx.get(actual).clone() {
                Ty::External(ExternalValue::Callable(_)) | Ty::TypeObject(_) => {
                    StrictRelation::Indeterminate
                }
                Ty::Class {
                    role: ClassRole::Instance,
                    user: Some(user),
                    ..
                } => match self.user_protocol_method(
                    user.symbol,
                    "__call__",
                    &mut std::collections::HashSet::new(),
                ) {
                    UserProtocolMethod::Missing => StrictRelation::Incompatible,
                    UserProtocolMethod::Found(..) | UserProtocolMethod::Indeterminate => {
                        StrictRelation::Indeterminate
                    }
                },
                Ty::Class { .. } => StrictRelation::Indeterminate,
                _ => StrictRelation::Incompatible,
            };
        };

        let return_relation =
            self.stdlib_type_relation_inner(expected_ret, actual_ret, visiting);
        if return_relation == StrictRelation::Incompatible {
            return StrictRelation::Incompatible;
        }

        let mut compare_prefix = |limit: usize| {
            let mut unknown = false;
            for (&expected, &actual) in expected_params
                .iter()
                .zip(&actual_params)
                .take(limit)
            {
                match self.stdlib_type_relation_inner(actual, expected, visiting) {
                    StrictRelation::Compatible => {}
                    StrictRelation::Indeterminate => unknown = true,
                    StrictRelation::Incompatible => {
                        return StrictRelation::Incompatible;
                    }
                }
            }
            if unknown {
                StrictRelation::Indeterminate
            } else {
                StrictRelation::Compatible
            }
        };

        let parameter_relation = if expected_param_spec.is_some() {
            if actual_param_spec.is_none()
                && !actual_variadic
                && actual_params.len() < expected_params.len()
            {
                StrictRelation::Incompatible
            } else {
                match compare_prefix(expected_params.len().min(actual_params.len())) {
                    StrictRelation::Incompatible => StrictRelation::Incompatible,
                    StrictRelation::Compatible | StrictRelation::Indeterminate => {
                        StrictRelation::Indeterminate
                    }
                }
            }
        } else if actual_param_spec.is_some() {
            StrictRelation::Indeterminate
        } else if expected_variadic {
            if expected_params.is_empty() {
                StrictRelation::Indeterminate
            } else if !actual_variadic && actual_params.len() < expected_params.len() {
                StrictRelation::Incompatible
            } else {
                match compare_prefix(expected_params.len().min(actual_params.len())) {
                    StrictRelation::Incompatible => StrictRelation::Incompatible,
                    StrictRelation::Compatible | StrictRelation::Indeterminate => {
                        StrictRelation::Indeterminate
                    }
                }
            }
        } else if actual_variadic {
            if actual_params.len() > expected_params.len() {
                StrictRelation::Incompatible
            } else {
                compare_prefix(actual_params.len())
            }
        } else if expected_params.len() != actual_params.len() {
            StrictRelation::Incompatible
        } else {
            compare_prefix(expected_params.len())
        };

        if parameter_relation == StrictRelation::Incompatible {
            StrictRelation::Incompatible
        } else if parameter_relation == StrictRelation::Indeterminate
            || return_relation == StrictRelation::Indeterminate
        {
            StrictRelation::Indeterminate
        } else {
            StrictRelation::Compatible
        }
    }

    fn stdlib_type_relation_inner(
        &mut self,
        expected: TypeId,
        actual: TypeId,
        visiting: &mut std::collections::HashSet<(TypeId, TypeId)>,
    ) -> StrictRelation {
        use super::stdlib_typespec::{self as spec, ClassSpecKind};

        let expected_node = self.tcx.get(expected).clone();
        let actual_node = self.tcx.get(actual).clone();
        if matches!(actual_node, Ty::Any | Ty::Error | Ty::TypeVar(_) | Ty::Infer(_)) {
            return StrictRelation::Indeterminate;
        }
        if let Ty::TypeVar(var) = expected_node {
            let info = self.tcx.get_type_var(var).clone();
            if !info.constraints.is_empty() {
                let mut unknown = false;
                for constraint in info.constraints {
                    match self.stdlib_type_relation_inner(constraint, actual, visiting) {
                        StrictRelation::Compatible => return StrictRelation::Compatible,
                        StrictRelation::Indeterminate => unknown = true,
                        StrictRelation::Incompatible => {}
                    }
                }
                return if unknown {
                    StrictRelation::Indeterminate
                } else {
                    StrictRelation::Incompatible
                };
            }
            return match info.bound {
                Some(bound) => self.stdlib_type_relation_inner(bound, actual, visiting),
                None => StrictRelation::Indeterminate,
            };
        }
        if matches!(expected_node, Ty::Any | Ty::Error | Ty::Infer(_)) {
            return StrictRelation::Indeterminate;
        }
        if expected == actual {
            return StrictRelation::Compatible;
        }
        if !visiting.insert((expected, actual)) {
            return StrictRelation::Compatible;
        }
        let relation = match (expected_node.clone(), actual_node.clone()) {
            (Ty::Union(expected), Ty::Union(actual)) => {
                let mut unknown = false;
                let compatible = actual.into_iter().all(|actual| {
                    let mut branch_unknown = false;
                    let matched = expected.iter().any(|expected| {
                        match self.stdlib_type_relation_inner(*expected, actual, visiting) {
                            StrictRelation::Compatible => true,
                            StrictRelation::Indeterminate => {
                                branch_unknown = true;
                                false
                            }
                            StrictRelation::Incompatible => false,
                        }
                    });
                    unknown |= !matched && branch_unknown;
                    matched || branch_unknown
                });
                if !compatible {
                    StrictRelation::Incompatible
                } else if unknown {
                    StrictRelation::Indeterminate
                } else {
                    StrictRelation::Compatible
                }
            }
            (Ty::Union(expected), _) => {
                let mut unknown = false;
                let matched = expected.iter().any(|expected| {
                    match self.stdlib_type_relation_inner(*expected, actual, visiting) {
                        StrictRelation::Compatible => true,
                        StrictRelation::Indeterminate => {
                            unknown = true;
                            false
                        }
                        StrictRelation::Incompatible => false,
                    }
                });
                if matched {
                    StrictRelation::Compatible
                } else if unknown {
                    StrictRelation::Indeterminate
                } else {
                    StrictRelation::Incompatible
                }
            }
            (_, Ty::Union(actual)) => {
                let mut unknown = false;
                let mut incompatible = false;
                for actual in actual {
                    match self.stdlib_type_relation_inner(expected, actual, visiting) {
                        StrictRelation::Compatible => {}
                        StrictRelation::Indeterminate => unknown = true,
                        StrictRelation::Incompatible => incompatible = true,
                    }
                }
                if incompatible {
                    StrictRelation::Incompatible
                } else if unknown {
                    StrictRelation::Indeterminate
                } else {
                    StrictRelation::Compatible
                }
            }
            (Ty::Fn { .. }, _) => {
                self.stdlib_callable_relation_inner(expected, actual, visiting)
            }
            (
                Ty::Class {
                    external: Some(external),
                    role: ClassRole::Instance,
                    ..
                },
                _,
            ) => {
                if let Some(projected) =
                    self.known_stdlib_class_projection(&external, expected, actual)
                {
                    if projected {
                        StrictRelation::Compatible
                    } else {
                        StrictRelation::Incompatible
                    }
                } else if let Some((_class_id, class)) =
                    spec::class_spec(&external.module, &external.name)
                {
                    if class.kind == ClassSpecKind::Protocol {
                        self.stdlib_protocol_relation(class, &external, actual, visiting)
                    } else if self.types_compatible(expected, actual) {
                        StrictRelation::Compatible
                    } else if external.module == "builtins"
                        && matches!(
                            &actual_node,
                            Ty::Never
                                | Ty::None
                                | Ty::Bool
                                | Ty::Int
                                | Ty::Float
                                | Ty::Str
                                | Ty::List(_)
                                | Ty::Set(_)
                                | Ty::Dict(_, _)
                                | Ty::Tuple(_)
                                | Ty::Fn { .. }
                                | Ty::TypeObject(_)
                                | Ty::Enum { .. }
                                | Ty::Literal(_)
                        )
                    {
                        StrictRelation::Incompatible
                    } else if matches!(
                        &actual_node,
                        Ty::Class {
                            user: Some(user),
                            ..
                        } if !self.class_inheritance_open.contains(&user.symbol)
                    ) {
                        StrictRelation::Incompatible
                    } else {
                        StrictRelation::Indeterminate
                    }
                } else if self.types_compatible(expected, actual) {
                    StrictRelation::Compatible
                } else {
                    StrictRelation::Incompatible
                }
            }
            _ if self.types_compatible(expected, actual) => StrictRelation::Compatible,
            _ => StrictRelation::Incompatible,
        };
        visiting.remove(&(expected, actual));
        relation
    }

    fn stdlib_literal_argument_relation(
        &mut self,
        expected: TypeId,
        actual: TypeId,
        value: &Spanned<Expr>,
    ) -> StrictRelation {
        match self.tcx.get(expected).clone() {
            Ty::Literal(values) => match self.tcx.get(actual).clone() {
                Ty::Any | Ty::Error | Ty::TypeVar(_) | Ty::Infer(_) => {
                    StrictRelation::Indeterminate
                }
                Ty::Never => StrictRelation::Compatible,
                Ty::Literal(actual_values) => {
                    if actual_values.iter().all(|value| values.contains(value)) {
                        StrictRelation::Compatible
                    } else {
                        StrictRelation::Incompatible
                    }
                }
                Ty::Union(members) => {
                    let mut unknown = false;
                    for member in members {
                        match self.stdlib_literal_argument_relation(expected, member, value) {
                            StrictRelation::Compatible => {}
                            StrictRelation::Indeterminate => unknown = true,
                            StrictRelation::Incompatible => {
                                return StrictRelation::Incompatible;
                            }
                        }
                    }
                    if unknown {
                        StrictRelation::Indeterminate
                    } else {
                        StrictRelation::Compatible
                    }
                }
                _ => {
                    if values.iter().any(|literal| {
                        matches!(
                            (literal, &value.node),
                            (LiteralValue::Int(left), Expr::IntLit(right)) if left == right
                        ) || matches!(
                            (literal, &value.node),
                            (LiteralValue::Str(left), Expr::StrLit(right)) if left == right
                        ) || matches!(
                            (literal, &value.node),
                            (LiteralValue::Bool(left), Expr::BoolLit(right)) if left == right
                        )
                    }) {
                        StrictRelation::Compatible
                    } else {
                        StrictRelation::Incompatible
                    }
                }
            },
            Ty::Union(members) => {
                let mut unknown = false;
                for member in members {
                    match self.stdlib_literal_argument_relation(member, actual, value) {
                        StrictRelation::Compatible => return StrictRelation::Compatible,
                        StrictRelation::Indeterminate => unknown = true,
                        StrictRelation::Incompatible => {}
                    }
                }
                if unknown {
                    StrictRelation::Indeterminate
                } else {
                    StrictRelation::Incompatible
                }
            }
            _ => self.stdlib_type_relation(expected, actual),
        }
    }

    fn stdlib_unmaterialized_argument_relation(
        &self,
        expected: super::stdlib_typespec::TypeSpecId,
        actual: TypeId,
    ) -> StrictRelation {
        use super::stdlib_typespec::{self as spec, TypeSpecNode};

        let TypeSpecNode::Apply { base, .. } = spec::node(expected) else {
            return StrictRelation::Indeterminate;
        };
        let TypeSpecNode::Name { module, name, .. } = spec::node(*base) else {
            return StrictRelation::Indeterminate;
        };
        if !matches!(
            (spec::string(*module), spec::string(*name)),
            ("typing", "Callable") | ("collections.abc", "Callable")
        ) {
            return StrictRelation::Indeterminate;
        }
        match self.tcx.get(actual) {
            Ty::Any | Ty::Error | Ty::TypeVar(_) | Ty::Infer(_) | Ty::AliasRef(_) => {
                StrictRelation::Indeterminate
            }
            Ty::Fn { .. } | Ty::TypeObject(_) => StrictRelation::Indeterminate,
            Ty::Class {
                role: ClassRole::Object,
                ..
            } => StrictRelation::Indeterminate,
            Ty::Class {
                role: ClassRole::Instance,
                user: Some(user),
                ..
            } => {
                let callable = self.user_protocol_method(
                        user.symbol,
                        "__call__",
                        &mut std::collections::HashSet::new(),
                    );
                match callable {
                    UserProtocolMethod::Found(..) | UserProtocolMethod::Indeterminate => {
                        StrictRelation::Indeterminate
                    }
                    UserProtocolMethod::Missing => StrictRelation::Incompatible,
                }
            }
            Ty::Class { .. } | Ty::Union(_) | Ty::SelfType => StrictRelation::Indeterminate,
            _ => StrictRelation::Incompatible,
        }
    }

    fn stdlib_generic_bounds_relation(
        &mut self,
        substitution: &Substitution,
        params: &GenericParams,
    ) -> StrictRelation {
        let mut unknown = false;
        for param in &params.params {
            let Some(actual) = substitution.get(param.id) else {
                unknown = true;
                continue;
            };
            if !param.constraints.is_empty() {
                let mut matched = false;
                let mut constraint_unknown = false;
                for constraint in &param.constraints {
                    match self.stdlib_type_relation(*constraint, actual) {
                        StrictRelation::Compatible => matched = true,
                        StrictRelation::Indeterminate => constraint_unknown = true,
                        StrictRelation::Incompatible => {}
                    }
                }
                if !matched {
                    if constraint_unknown {
                        unknown = true;
                    } else {
                        return StrictRelation::Incompatible;
                    }
                }
            }
            if let Some(bound) = param.bound {
                match self.stdlib_type_relation(bound, actual) {
                    StrictRelation::Compatible => {}
                    StrictRelation::Indeterminate => unknown = true,
                    StrictRelation::Incompatible => return StrictRelation::Incompatible,
                }
            }
        }
        if unknown {
            StrictRelation::Indeterminate
        } else {
            StrictRelation::Compatible
        }
    }

    pub(crate) fn stdlib_generic_bounds_error(
        &mut self,
        substitution: &Substitution,
        params: &GenericParams,
    ) -> Option<String> {
        if self.stdlib_generic_bounds_relation(substitution, params)
            != StrictRelation::Incompatible
        {
            return None;
        }
        check_bounds(substitution, params, &self.tcx)
            .into_iter()
            .next()
            .or_else(|| Some("external generic type argument violates its bound".to_string()))
    }

    fn evaluate_stdlib_spec_candidate(
        &mut self,
        sig: &super::stdlib_typespec::CallableSpec,
        args: &[CallArg],
        checked: &[Option<TypeId>],
        hide_implicit_receiver: bool,
        preserve_return_type: bool,
        receiver: Option<&ExternalClass>,
    ) -> StdlibSpecCandidate {
        use super::stdlib_typespec::{self as spec, ParamSpecKind};

        if args
            .iter()
            .any(|arg| matches!(arg, CallArg::StarArg(_) | CallArg::DoubleStarArg(_)))
        {
            return StdlibSpecCandidate::Indeterminate;
        }
        let params = spec::params(sig.params);
        let visible: Vec<_> = params
            .iter()
            .filter(|param| !hide_implicit_receiver || !param.implicit_receiver)
            .collect();
        let positional: Vec<_> = visible
            .iter()
            .enumerate()
            .filter(|(_, param)| {
                matches!(param.kind, ParamSpecKind::PosOnly | ParamSpecKind::PosOrKw)
            })
            .map(|(index, _)| index)
            .collect();
        let var_pos = visible
            .iter()
            .position(|param| param.kind == ParamSpecKind::VarPos);
        let var_kw = visible
            .iter()
            .position(|param| param.kind == ParamSpecKind::VarKw);
        let mut bound = std::collections::HashSet::new();
        let mut positional_index = 0usize;
        let mut bound_args = Vec::new();
        for (arg_index, (arg, actual)) in args.iter().zip(checked).enumerate() {
            let Some(actual) = actual else {
                return StdlibSpecCandidate::Indeterminate;
            };
            let (param_index, span) = match arg {
                CallArg::Positional(value) => {
                    let index = positional.get(positional_index).copied().or(var_pos);
                    positional_index += 1;
                    (index, value.span)
                }
                CallArg::Keyword { name, value } => {
                    let index = visible
                        .iter()
                        .position(|param| {
                            matches!(param.kind, ParamSpecKind::PosOrKw | ParamSpecKind::KwOnly)
                                && spec::string(param.name) == name
                        })
                        .or(var_kw);
                    (index, value.span)
                }
                CallArg::StarArg(_) | CallArg::DoubleStarArg(_) => unreachable!(),
            };
            let Some(param_index) = param_index else {
                return StdlibSpecCandidate::Rejected(
                    span,
                    "call has an argument that no parameter accepts".to_string(),
                    0,
                );
            };
            let param = visible[param_index];
            if !matches!(param.kind, ParamSpecKind::VarPos | ParamSpecKind::VarKw)
                && !bound.insert(param_index)
            {
                return StdlibSpecCandidate::Rejected(
                    span,
                    format!(
                        "multiple values for parameter `{}`",
                        spec::string(param.name)
                    ),
                    0,
                );
            }
            bound_args.push((
                param_index,
                *actual,
                span,
                spec::string(param.name).to_string(),
                arg_index,
            ));
        }
        for (param_index, param) in visible.iter().enumerate() {
            if matches!(param.kind, ParamSpecKind::VarPos | ParamSpecKind::VarKw)
                || param.has_default
                || bound.contains(&param_index)
            {
                continue;
            }
            return StdlibSpecCandidate::Rejected(
                Span::default(),
                format!("missing required parameter `{}`", spec::string(param.name)),
                0,
            );
        }
        let mut matched = Vec::with_capacity(bound_args.len());
        let mut indeterminate = false;
        for (param_index, actual, span, name, arg_index) in bound_args {
            let expected_spec = spec::type_use(visible[param_index].ty).0;
            let expected = self.materialize_stdlib_type(expected_spec);
            indeterminate |= expected.is_none();
            if expected.is_none()
                && self.stdlib_unmaterialized_argument_relation(expected_spec, actual)
                    == StrictRelation::Incompatible
            {
                return StdlibSpecCandidate::Rejected(
                    span,
                    format!(
                        "argument type mismatch: expected a callable value, got `{}` for parameter `{name}`",
                        self.ty_name(actual),
                    ),
                    1,
                );
            }
            let actual = if let Some(expected) = expected {
                let value = match &args[arg_index] {
                    CallArg::Positional(value)
                    | CallArg::StarArg(value)
                    | CallArg::Keyword { value, .. }
                    | CallArg::DoubleStarArg(value) => value,
                };
                self.refine_class_object_actual(expected, actual, value)
            } else {
                actual
            };
            matched.push((expected, actual, span, name, arg_index));
        }

        let mut relation_substitution = None;
        let completed = if let Some(generic_params) = self.stdlib_spec_generic_params(sig) {
            let inference_pairs: Vec<_> = matched
                .iter()
                .filter_map(|item| item.0.map(|expected| (expected, item.1)))
                .collect();
            let inference_params: Vec<_> = inference_pairs.iter().map(|item| item.0).collect();
            let inference_args: Vec<_> = inference_pairs.iter().map(|item| item.1).collect();
            let (mut subst, conflicts) =
                infer_type_args(&generic_params, &inference_params, &inference_args, &self.tcx);
            if let Some(message) = conflicts.into_iter().next() {
                let span = matched.first().map(|item| item.2).unwrap_or_default();
                return StdlibSpecCandidate::Rejected(span, message, 1);
            }
            if let Some(receiver) = receiver {
                let resolution = spec::class_callable_resolution(
                    &receiver.module,
                    &receiver.name,
                    spec::string(sig.name),
                    &[sig.kind],
                );
                let receiver_substitution = resolution.as_ref().and_then(|resolution| {
                    self.stdlib_receiver_substitution(receiver, resolution)
                });
                if let Some(receiver_substitution) = receiver_substitution {
                    for param in &generic_params.params {
                        let Some(receiver_arg) = receiver_substitution.get(param.id) else {
                            continue;
                        };
                        if let Some(inferred_arg) = subst.get(param.id) {
                            match self.stdlib_type_relation(receiver_arg, inferred_arg) {
                                StrictRelation::Compatible => {}
                                StrictRelation::Indeterminate => indeterminate = true,
                                StrictRelation::Incompatible => {
                                    let span =
                                        matched.first().map(|item| item.2).unwrap_or_default();
                                    return StdlibSpecCandidate::Rejected(
                                        span,
                                        format!(
                                            "argument type mismatch: receiver expects `{}`, got `{}`",
                                            self.ty_name(receiver_arg),
                                            self.ty_name(inferred_arg),
                                        ),
                                        1,
                                    );
                                }
                            }
                        }
                        subst.insert(param.id, receiver_arg);
                    }
                } else {
                    indeterminate = true;
                }
            }
            relation_substitution = Some(subst.clone());
            match complete_type_args(&generic_params, subst, &mut self.tcx) {
                Some((completed, _)) => {
                    if let Some(message) = check_bounds(
                        &completed,
                        &generic_params,
                        &self.tcx,
                    )
                    .into_iter()
                    .next()
                    {
                        match self.stdlib_generic_bounds_relation(
                            &completed,
                            &generic_params,
                        ) {
                            StrictRelation::Compatible => {}
                            StrictRelation::Indeterminate => indeterminate = true,
                            StrictRelation::Incompatible => {
                                let span =
                                    matched.first().map(|item| item.2).unwrap_or_default();
                                return StdlibSpecCandidate::Rejected(span, message, 1);
                            }
                        }
                    }
                    relation_substitution = Some(completed.clone());
                    Some(completed)
                }
                None => {
                    indeterminate = true;
                    None
                }
            }
        } else {
            indeterminate = true;
            None
        };
        for (expected, actual, span, name, arg_index) in matched {
            let Some(mut expected) = expected else {
                continue;
            };
            if let Some(substitution) = &relation_substitution {
                expected = substitution.apply(expected, &mut self.tcx);
            } else if self.tcx.contains_type_var(expected) {
                indeterminate = true;
                continue;
            }
            let value = match &args[arg_index] {
                CallArg::Positional(value)
                | CallArg::StarArg(value)
                | CallArg::Keyword { value, .. }
                | CallArg::DoubleStarArg(value) => value,
            };
            match self.stdlib_literal_argument_relation(expected, actual, value) {
                StrictRelation::Compatible => {}
                StrictRelation::Indeterminate => {
                    indeterminate = true;
                }
                StrictRelation::Incompatible => {
                    return StdlibSpecCandidate::Rejected(
                        span,
                        format!(
                            "argument type mismatch: expected `{}`, got `{}` for parameter `{name}`",
                            self.ty_name(expected),
                            self.ty_name(actual),
                        ),
                        1,
                    );
                }
            }
        }
        if indeterminate {
            return StdlibSpecCandidate::Indeterminate;
        }
        let ret = if sig.is_async || !preserve_return_type {
            None
        } else {
            let ret = spec::type_use(sig.ret).0;
            self.materialize_stdlib_type(ret)
                .map(|ret| completed.expect("determinate candidate").apply(ret, &mut self.tcx))
        };
        StdlibSpecCandidate::Accepted(ret)
    }

    fn check_structured_stdlib_call(
        &mut self,
        func: &Spanned<Expr>,
        func_ty: TypeId,
        args: &[CallArg],
    ) -> Option<Option<TypeId>> {
        use super::stdlib_typespec as spec;

        use super::stdlib_typespec::CallableSpecKind;

        let target = self.resolve_structured_stdlib_call(func, func_ty)?;
        let constructor_result = if target.access == StdlibSpecAccess::Constructor {
            target.receiver.as_ref().map(|receiver| {
                self.external_class_instance(
                    &receiver.module,
                    &receiver.name,
                    receiver.args.clone(),
                )
            })
        } else {
            None
        };
        let accepts_kind = |kind: CallableSpecKind| match target.access {
            StdlibSpecAccess::ModuleFn => kind == CallableSpecKind::ModuleFn,
            StdlibSpecAccess::Constructor
            | StdlibSpecAccess::ClassMember
            | StdlibSpecAccess::BoundMember => matches!(
                kind,
                CallableSpecKind::InstanceMethod
                    | CallableSpecKind::ClassMethod
                    | CallableSpecKind::StaticMethod
            ),
        };
        let candidates: Vec<_> = spec::overloads(
            &target.module,
            &target.qualifier,
            &target.name,
        )
        .filter(|sig| accepts_kind(sig.kind))
        .cloned()
        .collect();
        if candidates.is_empty() {
            return None;
        }
        if matches!(
            target.access,
            StdlibSpecAccess::ClassMember | StdlibSpecAccess::BoundMember
        ) {
            let mut kinds = Vec::new();
            for candidate in &candidates {
                if !kinds.contains(&candidate.kind) {
                    kinds.push(candidate.kind);
                }
            }
            if kinds.len() > 1 {
                return Some(None);
            }
        }
        let checked: Vec<_> = args
            .iter()
            .map(|arg| match arg {
                CallArg::Positional(value) | CallArg::StarArg(value) => {
                    Some(self.check_expr(value))
                }
                CallArg::Keyword { value, .. } | CallArg::DoubleStarArg(value) => {
                    Some(self.check_expr(value))
                }
            })
            .collect();
        let mut accepted = Vec::new();
        let mut rejected = Vec::new();
        let mut indeterminate = false;
        for candidate in &candidates {
            let hide_implicit_receiver = match target.access {
                StdlibSpecAccess::ModuleFn | StdlibSpecAccess::ClassMember
                    if candidate.kind == CallableSpecKind::InstanceMethod =>
                {
                    false
                }
                StdlibSpecAccess::ModuleFn => false,
                StdlibSpecAccess::Constructor
                | StdlibSpecAccess::ClassMember
                | StdlibSpecAccess::BoundMember => true,
            };
            let candidate_receiver = if target.access == StdlibSpecAccess::ClassMember
                && candidate.kind == CallableSpecKind::InstanceMethod
            {
                checked
                    .first()
                    .and_then(|actual| *actual)
                    .and_then(|actual| self.structured_stdlib_receiver(actual))
            } else {
                target.receiver.clone()
            };
            let evaluated = self.evaluate_stdlib_spec_candidate(
                candidate,
                args,
                &checked,
                hide_implicit_receiver,
                target.access != StdlibSpecAccess::Constructor,
                candidate_receiver.as_ref(),
            );
            match evaluated {
                StdlibSpecCandidate::Accepted(ret) => accepted.push(ret.map(|ret| {
                    candidate_receiver
                        .as_ref()
                        .map(|receiver| self.contextualize_stdlib_return(ret, receiver))
                        .unwrap_or(ret)
                })),
                StdlibSpecCandidate::Rejected(span, message, priority) => {
                    rejected.push((span, message, priority))
                }
                StdlibSpecCandidate::Indeterminate => indeterminate = true,
            }
        }
        if accepted.is_empty() {
            if !indeterminate && rejected.len() == candidates.len() {
                if let Some((span, message, _)) = rejected.into_iter().max_by(|left, right| {
                    left.2
                        .cmp(&right.2)
                        .then_with(|| right.0.start.cmp(&left.0.start))
                })
                {
                    self.error(span, message);
                }
                return Some(constructor_result);
            }
            return Some(constructor_result);
        }
        if indeterminate {
            return Some(constructor_result);
        }
        if accepted.iter().any(Option::is_none) {
            return Some(constructor_result);
        }
        let mut returns: Vec<_> = accepted.into_iter().flatten().collect();
        returns.sort_unstable_by_key(|ty| ty.0);
        returns.dedup();
        Some(match returns.as_slice() {
            [] => None,
            [ret] => Some(*ret),
            _ => Some(self.tcx.intern(Ty::Union(returns))),
        })
    }

    /// Legacy compact-signature fallback for stdlib calls not handled by the
    /// generated TypeSpec contract. It rejects only disjoint concrete scalars
    /// and returns a modeled scalar result when one is available.
    fn check_stdlib_call(&mut self, func: &Spanned<Expr>, args: &[CallArg]) -> Option<TypeId> {
        // Resolve callee -> a concrete `StdlibSig`. We resolve to the signature
        // directly (rather than a `(module, qualifier, name)` triple) because a
        // bare stdlib name `Cls(...)` may be either a module function OR a class
        // constructor; we try the module-fn key first and fall back to the
        // class `__init__` key. Skip-when-unsure at every miss.
        let mut explicit_unbound_receiver = false;
        let sig: Option<&'static super::stdlib_sigs::StdlibSig> = match &func.node {
            // Bare name: a from-imported module function (`strerror(...)`) OR a
            // from-imported stdlib class called as a constructor (`Cls(...)`).
            //
            // Preserve the imported member independently of its local alias:
            // try the module-fn key `(module, "", member)` first, and on a miss
            // fall back to the constructor key `(module, member, "__init__")`.
            // The `self` receiver is already stripped from `__init__` param rows,
            // so positional alignment starts at the first real argument. Names not
            // in `import_origins` (user-defined classes, locals) resolve to None.
            Expr::Ident(name) => self
                .symbols
                .lookup(name)
                .and_then(|symbol| self.import_origins.get(&symbol))
                .and_then(|(module, member)| {
                    let member = if member.is_empty() { name } else { member };
                    super::stdlib_sigs::get(module, "", member)
                        .or_else(|| super::stdlib_sigs::get(module, member, "__init__"))
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
                    let base_symbol = self.symbols.lookup(base);
                    if let Some((module, qual)) =
                        base_symbol.and_then(|symbol| self.import_origins.get(&symbol))
                    {
                        // `base.attr(...)` is either a module function or a
                        // class/static method. Resolving `date.fromtimestamp(...)`
                        // needs us to try `base` itself as the class qualifier —
                        // `get(module, "date", "fromtimestamp")`.
                        // Try module-fn first, then class-method (base = class name),
                        // then any recorded qualifier. Still gated downstream by
                        // `sig.enforceable` + the concrete-scalar-disjoint check, so
                        // this only ADDS rejections of genuinely wrong-typed scalar
                        // args (it previously leaked `date.fromtimestamp("x")` etc.).
                        let module_sig = super::stdlib_sigs::get(module, "", attr);
                        if module_sig.is_some() {
                            module_sig
                        } else {
                            let class_sig =
                                super::stdlib_sigs::get(module, base, attr).or_else(|| {
                                    if qual.is_empty() {
                                        None
                                    } else {
                                        super::stdlib_sigs::get(module, qual, attr)
                                    }
                                });
                            if let Some(sig) = class_sig {
                                explicit_unbound_receiver =
                                    matches!(sig.kind, super::stdlib_sigs::SigKind::Method)
                                        && self
                                            .stdlib_call_has_explicit_unbound_receiver(base, args);
                            }
                            class_sig
                        }
                    } else if let Some((module, class_name)) =
                        base_symbol.and_then(|symbol| self.instance_origins.get(&symbol))
                    {
                        // `base` is a stdlib instance. Its immutable origin is
                        // independent of later rebinding of the constructor name.
                        super::stdlib_sigs::get(module, class_name, attr)
                    } else if let Some(sym) = self.symbols.lookup(base) {
                        if matches!(self.tcx.get(self.get_sym_type(sym.0)), Ty::List(_)) {
                            super::stdlib_sigs::get("builtins", "list", attr)
                        } else if matches!(self.tcx.get(self.get_sym_type(sym.0)), Ty::Tuple(_)) {
                            super::stdlib_sigs::get("builtins", "tuple", attr)
                        } else if self.symbols.get_symbol(sym).kind == SymbolKind::Function {
                            // User-defined Python functions are instances of
                            // builtins.function. This keeps descriptor walls such as
                            // `f.__get__(..., owner)` enforceable without pretending
                            // that `builtins.function` is importable in CPython.
                            super::stdlib_sigs::get("builtins", "function", attr)
                        } else if let Ty::Class {
                            name,
                            role: ClassRole::Instance,
                            user: None,
                            ..
                        } = self.tcx.get(self.get_sym_type(sym.0)).clone()
                        {
                            // #886: `instance_origins` missed (no direct
                            // `x = Cls(...)` provenance through an import) but
                            // inference already knows `base`'s class — fall
                            // back to it. See `stdlib_method_sig_by_class_name`.
                            self.stdlib_method_sig_by_class_name(&name, attr)
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

        let Some(sig) = sig else { return None };
        // #887: the callee's typeshed return type, independent of argument
        // enforceability below (a zero-arg call like `os.getcwd()` is never
        // `enforceable`, but its `str` return still must flow into inference).
        let ret_ty = self.core_ty_to_type_id(sig.ret);
        if !sig.enforceable {
            return ret_ty;
        }

        // Walk positional args against params. Stop at the first star/kwarg arg
        // and at the first star/unknown param. Only reject when BOTH the param
        // and the actual arg are concrete-and-disjoint scalars.
        let bytes_encoding_arg_is_positional = sig.module == "builtins"
            && sig.qualifier.is_empty()
            && matches!(sig.name, "bytes" | "bytearray")
            && matches!(args.get(1), Some(CallArg::Positional(_)));
        let mut param_idx = 0usize;
        let mut arg_idx = 0usize;
        for arg in args {
            let CallArg::Positional(a) = arg else {
                // Keyword / *args / **kwargs: stop *positional* enforcement
                // entirely. We do not know how positional alignment continues
                // past these. Keyword args are still checked below (#881) —
                // by name, independent of this position walk.
                break;
            };
            if explicit_unbound_receiver && arg_idx == 0 {
                self.check_expr(a);
                arg_idx += 1;
                continue;
            }
            let Some(param) = sig.params.get(param_idx) else {
                break;
            };
            if param.star {
                break; // never enforce past `*args`
            }
            let classinfo_param = sig.module == "builtins"
                && sig.qualifier.is_empty()
                && matches!(sig.name, "isinstance" | "issubclass")
                && param_idx == 1;
            if classinfo_param {
                let actual = self.check_expr(a);
                if let Some(name) = self
                    .classinfo_instance_name_from_type(actual)
                    .or_else(|| self.classinfo_bare_instance_name(a))
                {
                    self.error(
                        a.span,
                        format!(
                            "argument type mismatch: `{name}` does not satisfy parameter `{}`'s type",
                            param.name,
                        ),
                    );
                }
                param_idx += 1;
                arg_idx += 1;
                continue;
            }
            let bytes_encoding_source = bytes_encoding_arg_is_positional && param_idx == 0;
            self.check_stdlib_scalar_arg(param, a, bytes_encoding_source);
            param_idx += 1;
            arg_idx += 1;
        }

        // #881: keyword-arg alignment. The positional walk above breaks at the
        // first non-positional arg (position tracking becomes meaningless past
        // *args), but named keyword args don't need position — match each
        // `CallArg::Keyword{name, value}` to the like-named `ParamSig` (never a
        // `*args`/`**kwargs` boundary param) and run the same scalar/Typed
        // checks positional args get. Skip-when-unsure: an unknown keyword name
        // (typeshed-only kwarg not modeled, or a genuinely bad call CPython
        // itself will reject) is left unchecked here rather than guessed at.
        for arg in args {
            let CallArg::Keyword { name, value } = arg else {
                continue;
            };
            let Some(param) = sig
                .params
                .iter()
                .find(|p| !p.star && p.name == name.as_str())
            else {
                continue;
            };
            self.check_stdlib_scalar_arg(param, value, false);
        }
        ret_ty
    }

    /// Shared scalar/Typed check for a single (param, actual-arg) pair: the
    /// bare-user-class-instance rejection plus the CoreTy-specific and generic
    /// concrete-scalar-disjoint checks. Used by both the positional `param_idx`
    /// walk and (#881) keyword-name alignment in `check_stdlib_call` above.
    /// `bytes_encoding_source` only applies to the positional `bytes`/
    /// `bytearray` `encoding`-implies-`str`-source special case — always
    /// `false` for keyword calls (out of scope for #881).
    fn check_stdlib_scalar_arg(
        &mut self,
        param: &super::stdlib_sigs::ParamSig,
        a: &Spanned<Expr>,
        bytes_encoding_source: bool,
    ) {
        // Map the param's CoreTy to a concrete scalar; None => no positive
        // scalar mapping (Unknown / Typed / Bytes). Bytes is handled below as a
        // negative scalar wall because bytes literals currently infer to Any.
        let expected = self.core_ty_to_type_id(param.ty);
        let actual = self.check_expr(a);
        if matches!(param.ty, super::stdlib_sigs::CoreTy::Type) {
            if let Ty::Class {
                name,
                role: ClassRole::Instance,
                ..
            } = self.tcx.get(actual)
            {
                self.error(
                    a.span,
                    format!(
                        "argument type mismatch: `{name}` does not satisfy parameter `{}`'s type",
                        param.name,
                    ),
                );
                return;
            }
        }
        // A BARE user class instance (`class _W: pass` -> `_W()`) satisfies NO
        // concrete parameter contract: it is not a scalar (str/int/float/
        // bytes/bool; no relevant dunder), not a protocol (no dunders), and
        // not a nominal class (object is its only base). Reject it against any
        // param whose CoreTy names such a contract. `None`/`Unknown` params
        // are excluded because `None` is frequently an under-declared
        // Optional sentinel and Unknown remains skip-when-unsure.
        let concrete_param = matches!(
            param.ty,
            super::stdlib_sigs::CoreTy::Int
                | super::stdlib_sigs::CoreTy::Float
                | super::stdlib_sigs::CoreTy::Str
                | super::stdlib_sigs::CoreTy::Bytes
                | super::stdlib_sigs::CoreTy::MemoryView
                | super::stdlib_sigs::CoreTy::Complex
                | super::stdlib_sigs::CoreTy::IntOrStr
                | super::stdlib_sigs::CoreTy::PathOrFd
                | super::stdlib_sigs::CoreTy::List
                | super::stdlib_sigs::CoreTy::Tuple
                | super::stdlib_sigs::CoreTy::Dict
                | super::stdlib_sigs::CoreTy::Bool
                | super::stdlib_sigs::CoreTy::Typed
                | super::stdlib_sigs::CoreTy::TypedNamed(_)
                | super::stdlib_sigs::CoreTy::Type
        );
        // #885: a bare instance stashed in a variable (`w = _W(); f(w)`) has
        // no distinguishing expression shape at the call site, so the
        // syntactic `classinfo_bare_instance_name` helper alone misses it.
        // Fall back to the inferred type; class objects and instances are
        // distinguished by the universal ClassRole.
        let bare_arg = self
            .classinfo_bare_instance_name(a)
            .or_else(|| match param.ty {
                super::stdlib_sigs::CoreTy::Typed | super::stdlib_sigs::CoreTy::TypedNamed(_) => {
                    self.typed_literal_bare_instance_name(a)
                }
                _ => None,
            })
            .or_else(|| {
                if matches!(param.ty, super::stdlib_sigs::CoreTy::Type) {
                    return None;
                }
                match self.tcx.get(actual) {
                    Ty::Class {
                        name,
                        role: ClassRole::Instance,
                        user: Some(user),
                        ..
                    } if self.user_bare_class_symbols.contains(&user.symbol) => Some(name.clone()),
                    Ty::Class {
                        name,
                        role: ClassRole::Instance,
                        user: None,
                        ..
                    } if self.user_bare_classes.contains(name) => Some(name.clone()),
                    _ => None,
                }
            });
        if let (true, Some(name)) = (concrete_param, &bare_arg) {
            self.error(
                a.span,
                format!(
                    "argument type mismatch: `{name}` does not satisfy parameter `{}`'s type",
                    param.name,
                ),
            );
            return;
        }
        // A `None` actual argument is NEVER rejected: typeshed routinely
        // under-declares Optional (a `host: str` parameter is called with
        // `None` as a sentinel/clear, `set_proxy(host, None)` etc.), and
        // `None` is the single most common "looks wrong, is right" runtime
        // value. Skip-when-unsure — a missed enforcement is fine, a false
        // reject is not. (The ① type-wall fixtures probe with wrong
        // *scalars* — str-for-int, instance-for-bool — not bare `None`, so
        // this costs no type gain.)
        let actual_is_none = matches!(self.tcx.get(actual), Ty::None);
        if bytes_encoding_source
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
        } else if matches!(param.ty, super::stdlib_sigs::CoreTy::IntOrStr)
            && !actual_is_none
            && self.is_concrete_scalar(actual)
            && !matches!(self.tcx.get(actual), Ty::Int | Ty::Bool | Ty::Str)
        {
            self.error(
                a.span,
                format!(
                    "argument type mismatch: expected `int | str`, got `{}`",
                    self.ty_name(actual),
                ),
            );
        } else if matches!(param.ty, super::stdlib_sigs::CoreTy::PathOrFd)
            && !actual_is_none
            && self.is_concrete_scalar(actual)
            && !matches!(self.tcx.get(actual), Ty::Int | Ty::Bool | Ty::Str)
        {
            self.error(
                a.span,
                format!(
                    "argument type mismatch: expected `str | bytes | os.PathLike | int`, got `{}`",
                    self.ty_name(actual),
                ),
            );
        } else if matches!(
            param.ty,
            super::stdlib_sigs::CoreTy::Bytes
                | super::stdlib_sigs::CoreTy::MemoryView
                | super::stdlib_sigs::CoreTy::List
                | super::stdlib_sigs::CoreTy::Tuple
                | super::stdlib_sigs::CoreTy::Dict
                | super::stdlib_sigs::CoreTy::Type
        ) && !actual_is_none
            && self.is_concrete_scalar(actual)
        {
            let expected_name = match param.ty {
                super::stdlib_sigs::CoreTy::Bytes => "bytes",
                super::stdlib_sigs::CoreTy::MemoryView => "memoryview",
                super::stdlib_sigs::CoreTy::List => "list",
                super::stdlib_sigs::CoreTy::Tuple => "tuple",
                super::stdlib_sigs::CoreTy::Dict => "dict",
                super::stdlib_sigs::CoreTy::Type => "type",
                _ => unreachable!(),
            };
            self.error(
                a.span,
                format!(
                    "argument type mismatch: expected `{expected_name}`, got `{}`",
                    self.ty_name(actual),
                ),
            );
        } else if let super::stdlib_sigs::CoreTy::TypedNamed(contract) = param.ty {
            // #882: positive predicate for the seed named contracts. The
            // bare-class rejection above already fired for a bare instance;
            // this only rejects a *concrete scalar* actual that provably
            // cannot satisfy the named protocol. Any other name (reserved for
            // future seeds) is left skip-when-unsure, identical to `Typed`.
            let violates = match contract {
                // os.PathLike / StrPath / BytesPath / StrOrBytesPath /
                // GenericPath: `__fspath__`'s two valid concrete return
                // shapes are str and bytes, so only str is a concrete-scalar
                // match — bytes values currently infer to `Any` and stay
                // skip-safe there already. int/float/bool can never satisfy
                // `__fspath__`.
                "PathLike" => {
                    !actual_is_none
                        && self.is_concrete_scalar(actual)
                        && !matches!(self.tcx.get(actual), Ty::Str)
                }
                // SupportsIndex: `__index__` is satisfiable by int (and bool,
                // an int subtype at runtime); str/float never define it. A
                // user class defining `__index__` (the same protocol the
                // `chr`/`hex`/`oct`/`bin` check above recognizes) is never a
                // concrete scalar, so `is_concrete_scalar` already excludes
                // it here with no extra class-shape check needed.
                "SupportsIndex" => {
                    !actual_is_none
                        && self.is_concrete_scalar(actual)
                        && !matches!(self.tcx.get(actual), Ty::Int | Ty::Bool)
                }
                _ => false,
            };
            if violates {
                let expected_name = match contract {
                    "PathLike" => "str | bytes | os.PathLike",
                    other => other,
                };
                self.error(
                    a.span,
                    format!(
                        "argument type mismatch: expected `{expected_name}`, got `{}`",
                        self.ty_name(actual),
                    ),
                );
            }
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

    fn stdlib_call_has_explicit_unbound_receiver(
        &self,
        class_name: &str,
        args: &[CallArg],
    ) -> bool {
        let Some(CallArg::Positional(first)) = args.first() else {
            return false;
        };
        self.stdlib_unbound_receiver_placeholder(class_name, first)
    }

    fn stdlib_unbound_receiver_placeholder(&self, class_name: &str, expr: &Spanned<Expr>) -> bool {
        let Expr::Call { func, args } = &expr.node else {
            return false;
        };
        match &func.node {
            Expr::Ident(name) => name == class_name || (name == "object" && args.is_empty()),
            Expr::Attr { object, attr } if attr == "__new__" => {
                let Some(CallArg::Positional(arg0)) = args.first() else {
                    return false;
                };
                let Expr::Ident(cls) = &arg0.node else {
                    return false;
                };
                if cls != class_name {
                    return false;
                }
                matches!(&object.node, Expr::Ident(base) if base == "object" || base == class_name)
            }
            _ => false,
        }
    }

    /// Syntax fallback for bare-instance classinfo fixtures. This also covers
    /// shapes whose inferred type widened.
    fn classinfo_bare_instance_name(&self, expr: &Spanned<Expr>) -> Option<String> {
        match &expr.node {
            Expr::Call { func, .. } => match &func.node {
                Expr::Ident(name) => self
                    .symbols
                    .lookup(name)
                    .and_then(|symbol| match self.tcx.get(self.get_sym_type(symbol.0)) {
                        Ty::Class {
                            role: ClassRole::Object,
                            user: Some(user),
                            ..
                        } if self.user_bare_class_symbols.contains(&user.symbol) => {
                            Some(name.clone())
                        }
                        Ty::Class { user: None, .. }
                            if self.user_bare_classes.contains(name) =>
                        {
                            Some(name.clone())
                        }
                        _ => None,
                    })
                    .or_else(|| self.user_bare_classes.contains(name).then(|| name.clone())),
                _ => None,
            },
            Expr::TupleLit(elems) => elems
                .iter()
                .find_map(|elem| self.classinfo_bare_instance_name(elem)),
            _ => None,
        }
    }

    fn classinfo_instance_name_from_type(&self, ty: TypeId) -> Option<String> {
        match self.tcx.get(ty) {
            Ty::Class {
                name,
                role: ClassRole::Instance,
                ..
            } => Some(name.clone()),
            Ty::Tuple(elements) => elements
                .iter()
                .find_map(|element| self.classinfo_instance_name_from_type(*element)),
            _ => None,
        }
    }

    /// `Typed` walls keep skip-when-unsure for arbitrary iterables, but a
    /// literal list/tuple/set containing a syntactically bare user-class
    /// instance (`[_W()]`, `(_W(),)`, `{_W()}`) is concrete enough to reject.
    /// Dict key/value contracts stay out of scope.
    fn typed_literal_bare_instance_name(&self, expr: &Spanned<Expr>) -> Option<String> {
        match &expr.node {
            Expr::ListLit(elems) | Expr::SetLit(elems) | Expr::TupleLit(elems) => {
                elems.iter().find_map(|elem| {
                    self.classinfo_bare_instance_name(elem)
                        .or_else(|| self.typed_literal_bare_instance_name(elem))
                })
            }
            _ => None,
        }
    }

    /// Resolve a bound user-method call to its owner-specific metadata key.
    fn user_method_key(&self, func: &Spanned<Expr>) -> Option<(crate::resolve::SymbolId, String)> {
        let Expr::Attr { object, attr } = &func.node else {
            return None;
        };
        let object_ty = match &object.node {
            Expr::Ident(name) => {
                let symbol = self.symbols.lookup(name)?;
                self.get_sym_type(symbol.0)
            }
            Expr::Index { object, .. } => {
                let Expr::Ident(name) = &object.node else {
                    return None;
                };
                let symbol = self.symbols.lookup(name)?;
                self.get_sym_type(symbol.0)
            }
            Expr::Call { func, .. } => {
                let name = match &func.node {
                    Expr::Ident(name) => name,
                    Expr::Index { object, .. } => {
                        let Expr::Ident(name) = &object.node else {
                            return None;
                        };
                        name
                    }
                    _ => return None,
                };
                let symbol = self.symbols.lookup(name)?;
                self.get_sym_type(symbol.0)
            }
            _ => return None,
        };
        let Ty::Class {
            user: Some(user), ..
        } = self.tcx.get(object_ty)
        else {
            return None;
        };
        let class_symbol = user.symbol;
        Some((class_symbol, attr.clone()))
    }

    fn resolve_explicit_user_class_specialization(
        &mut self,
        object: &Spanned<Expr>,
        index: &Spanned<Expr>,
        span: Span,
    ) -> Option<TypeId> {
        let Expr::Ident(name) = &object.node else {
            return None;
        };
        let binding_symbol = self.symbols.lookup(name)?;
        let base_ty = self.get_sym_type(binding_symbol.0);
        match self.tcx.get(base_ty) {
            Ty::Class {
                role: ClassRole::Object,
                user: Some(_),
                ..
            } => {}
            _ => return None,
        }
        let expressions = match &index.node {
            Expr::TupleLit(items) => items.as_slice(),
            _ => std::slice::from_ref(index),
        };
        let mut supplied = Vec::with_capacity(expressions.len());
        for expression in expressions {
            let type_expr = expr_to_type_expr(expression)?;
            supplied.push(self.resolve_type_expr(&type_expr));
        }
        Some(self.specialize_user_class_as(
            name,
            binding_symbol,
            base_ty,
            Some(&supplied),
            span,
            ClassRole::Object,
        ))
    }

    fn check_substituted_constructor_args(
        &mut self,
        params: &[TypeId],
        args: &[TypeId],
        subst: &Substitution,
        span: Span,
    ) {
        for (param, actual) in params.iter().zip(args) {
            let expected = subst.apply(*param, &mut self.tcx);
            if !self.types_compatible(expected, *actual) {
                self.error(
                    span,
                    format!(
                        "argument type mismatch: expected `{}`, got `{}`",
                        self.ty_name(expected),
                        self.ty_name(*actual),
                    ),
                );
            }
        }
    }

    fn user_class_object<'a>(
        &self,
        object: &'a Spanned<Expr>,
    ) -> Option<(&'a str, crate::resolve::SymbolId)> {
        let (name, symbol) = match &object.node {
            Expr::Ident(name) => {
                let symbol = self.symbols.lookup(name)?;
                (name.as_str(), symbol)
            }
            Expr::Index { object, .. } => {
                let Expr::Ident(name) = &object.node else {
                    return None;
                };
                let symbol = self.symbols.lookup(name)?;
                (name.as_str(), symbol)
            }
            _ => return None,
        };
        let Ty::Class {
            role: ClassRole::Object,
            user: Some(user),
            ..
        } = self.tcx.get(self.get_sym_type(symbol.0))
        else {
            return None;
        };
        Some((name, user.symbol))
    }

    fn specialize_class_member_type(
        &mut self,
        class_symbol: SymbolId,
        class_args: &[TypeId],
        ty: TypeId,
    ) -> TypeId {
        let Some(params) = self.generic_defs.get(&class_symbol).cloned() else {
            return ty;
        };
        let mut subst = Substitution::new();
        for (param, arg) in params.params.iter().zip(class_args) {
            subst.insert(param.id, *arg);
        }
        subst.apply(ty, &mut self.tcx)
    }

    fn external_callable_type(
        &mut self,
        module: String,
        qualifier: String,
        name: &str,
        access: ExternalCallableAccess,
        receiver: Option<ExternalClass>,
    ) -> TypeId {
        self.tcx
            .intern(Ty::External(ExternalValue::Callable(ExternalCallable {
                module,
                qualifier,
                name: name.to_string(),
                access,
                runtime_kind: ExternalCallableRuntimeKind::Unknown,
                receiver,
            })))
    }

    fn contextualize_stdlib_return(
        &mut self,
        ty: TypeId,
        receiver: &ExternalClass,
    ) -> TypeId {
        match self.tcx.get(ty).clone() {
            Ty::SelfType => self.external_class_instance(
                &receiver.module,
                &receiver.name,
                receiver.args.clone(),
            ),
            Ty::List(item) => {
                let item = self.contextualize_stdlib_return(item, receiver);
                self.tcx.intern(Ty::List(item))
            }
            Ty::Set(item) => {
                let item = self.contextualize_stdlib_return(item, receiver);
                self.tcx.intern(Ty::Set(item))
            }
            Ty::Dict(key, value) => {
                let key = self.contextualize_stdlib_return(key, receiver);
                let value = self.contextualize_stdlib_return(value, receiver);
                self.tcx.intern(Ty::Dict(key, value))
            }
            Ty::Tuple(items) => {
                let items = items
                    .into_iter()
                    .map(|item| self.contextualize_stdlib_return(item, receiver))
                    .collect();
                self.tcx.intern(Ty::Tuple(items))
            }
            Ty::Union(items) => {
                let items = items
                    .into_iter()
                    .map(|item| self.contextualize_stdlib_return(item, receiver))
                    .collect();
                self.tcx.intern(Ty::Union(items))
            }
            Ty::TypeObject(instance) => {
                let instance = self.contextualize_stdlib_return(instance, receiver);
                self.tcx.intern(Ty::TypeObject(instance))
            }
            _ => ty,
        }
    }

    fn resolve_external_property_type(
        &mut self,
        receiver: &ExternalClass,
        attr: &str,
    ) -> Option<TypeId> {
        use super::stdlib_typespec::{self as spec, CallableSpecKind};

        let resolution = spec::class_callable_resolution(
            &receiver.module,
            &receiver.name,
            attr,
            &[CallableSpecKind::PropertyGet],
        )?;
        let owner = spec::class_by_id(resolution.owner);
        let module = spec::string(owner.module);
        let qualifier = spec::string(owner.qualifier);
        let receiver_substitution = self.stdlib_receiver_substitution(receiver, &resolution)?;
        let candidates: Vec<_> = spec::overloads(module, qualifier, attr)
            .filter(|sig| sig.kind == CallableSpecKind::PropertyGet)
            .cloned()
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let mut returns = Vec::new();
        for candidate in candidates {
            if candidate.is_async {
                return None;
            }
            let ret = self.materialize_stdlib_type(spec::type_use(candidate.ret).0)?;
            let ret = receiver_substitution.apply(ret, &mut self.tcx);
            let ret = self.contextualize_stdlib_return(ret, receiver);
            if self.tcx.contains_type_var(ret) {
                return None;
            }
            returns.push(ret);
        }
        returns.sort_unstable_by_key(|ty| ty.0);
        returns.dedup();
        match returns.as_slice() {
            [] => None,
            [ret] => Some(*ret),
            _ => Some(self.tcx.intern(Ty::Union(returns))),
        }
    }

    fn resolve_external_property_setter_type(
        &mut self,
        receiver: &ExternalClass,
        attr: &str,
    ) -> Option<TypeId> {
        use super::stdlib_typespec::{self as spec, CallableSpecKind};

        let resolution = spec::class_callable_resolution(
            &receiver.module,
            &receiver.name,
            attr,
            &[CallableSpecKind::PropertySet],
        )?;
        let owner = spec::class_by_id(resolution.owner);
        let module = spec::string(owner.module);
        let qualifier = spec::string(owner.qualifier);
        let receiver_substitution = self.stdlib_receiver_substitution(receiver, &resolution)?;
        let mut accepted = Vec::new();
        for candidate in spec::overloads(module, qualifier, attr)
            .filter(|sig| sig.kind == CallableSpecKind::PropertySet)
        {
            let visible: Vec<_> = spec::params(candidate.params)
                .iter()
                .filter(|param| !param.implicit_receiver)
                .collect();
            let [value] = visible.as_slice() else {
                return None;
            };
            let expected = self.materialize_stdlib_type(spec::type_use(value.ty).0)?;
            let expected = receiver_substitution.apply(expected, &mut self.tcx);
            let expected = self.contextualize_stdlib_return(expected, receiver);
            if self.tcx.contains_type_var(expected) {
                return None;
            }
            accepted.push(expected);
        }
        accepted.sort_unstable_by_key(|ty| ty.0);
        accepted.dedup();
        match accepted.as_slice() {
            [] => None,
            [expected] => Some(*expected),
            _ => Some(self.tcx.intern(Ty::Union(accepted))),
        }
    }

    fn resolve_external_value_attr(&mut self, object_ty: TypeId, attr: &str) -> Option<TypeId> {
        match self.tcx.get(object_ty).clone() {
            Ty::External(ExternalValue::Module { path, loaded }) => {
                let child = format!("{path}.{attr}");
                if loaded
                    .iter()
                    .any(|module| module == &child || module.starts_with(&format!("{child}.")))
                {
                    return Some(self.tcx.intern(Ty::External(ExternalValue::Module {
                        path: child,
                        loaded,
                    })));
                }
                if let Some((module, name)) =
                    super::stdlib_typespec::exported_class(&path, attr)
                {
                    return Some(self.external_class_object(module, name, Vec::new()));
                }
                if Self::structured_stdlib_module_fn_exists(&path, attr) {
                    return Some(self.external_callable_type(
                        path,
                        String::new(),
                        attr,
                        ExternalCallableAccess::Module,
                        None,
                    ));
                }
                None
            }
            Ty::Class {
                role,
                external: Some(receiver),
                ..
            } => {
                if role == ClassRole::Instance {
                    if let Some(property) = self.resolve_external_property_type(&receiver, attr) {
                        return Some(property);
                    }
                }
                let (module, qualifier) = Self::structured_stdlib_member_owner(
                    &receiver.module,
                    &receiver.name,
                    attr,
                )?;
                let access = if role == ClassRole::Object {
                    ExternalCallableAccess::ClassMember
                } else {
                    ExternalCallableAccess::BoundMember
                };
                Some(self.external_callable_type(
                    module,
                    qualifier,
                    attr,
                    access,
                    Some(receiver),
                ))
            }
            _ => None,
        }
    }

    fn resolve_generated_bound_member(&mut self, object_ty: TypeId, attr: &str) -> Option<TypeId> {
        let receiver = self.structured_stdlib_receiver(object_ty)?;
        let (module, qualifier) = Self::structured_stdlib_member_owner(
            &receiver.module,
            &receiver.name,
            attr,
        )?;
        Some(self.external_callable_type(
            module,
            qualifier,
            attr,
            ExternalCallableAccess::BoundMember,
            Some(receiver),
        ))
    }

    pub(crate) fn resolve_property_setter_type(
        &mut self,
        object_ty: TypeId,
        attr: &str,
    ) -> Option<TypeId> {
        match self.semantic_ty(object_ty) {
            Ty::Class {
                role: ClassRole::Instance,
                user: Some(user),
                ..
            } => {
                let setter_ty = self
                    .class_property_setters
                    .get(&user.symbol)?
                    .get(attr)
                    .copied()?;
                Some(self.specialize_class_member_type(user.symbol, &user.args, setter_ty))
            }
            Ty::Class {
                role: ClassRole::Instance,
                external: Some(receiver),
                ..
            } => self.resolve_external_property_setter_type(&receiver, attr),
            _ => None,
        }
    }

    /// Resolve attribute access (#246).
    fn resolve_unbound_class_method(
        &mut self,
        object: &Spanned<Expr>,
        object_ty: TypeId,
        attr: &str,
    ) -> Option<TypeId> {
        let (_, symbol) = self.user_class_object(object)?;
        let mut sig = self.class_unbound_methods.get(&symbol)?.get(attr)?.clone();
        let user = match self.tcx.get(object_ty) {
            Ty::Class {
                role: ClassRole::Object,
                user: Some(user),
                ..
            } if user.symbol == symbol => Some(user.clone()),
            _ => None,
        };
        if let Some(user) = user {
            if let Some(params) = self.generic_defs.get(&symbol).cloned() {
                let mut subst = Substitution::new();
                for (param, arg) in params.params.iter().zip(&user.args) {
                    subst.insert(param.id, *arg);
                }
                sig.params = sig
                    .params
                    .iter()
                    .map(|param| subst.apply(*param, &mut self.tcx))
                    .collect();
                sig.return_type = subst.apply(sig.return_type, &mut self.tcx);
            }
        }
        Some(self.tcx.intern(Ty::Fn {
            params: sig.params,
            ret: sig.return_type,
            variadic: false,
            param_spec: None,
        }))
    }

    /// Resolve attribute access (#246).
    fn resolve_attr(&mut self, obj_ty_id: TypeId, attr: &str, _span: Span) -> TypeId {
        match self.semantic_ty(obj_ty_id) {
            Ty::List(elem) => match attr {
                "append" | "remove" => self.tcx.intern(Ty::Fn {
                    params: vec![elem],
                    ret: self.tcx.none(),
                    variadic: false,
                    param_spec: None,
                }),
                "count" => self.tcx.intern(Ty::Fn {
                    params: vec![elem],
                    ret: self.tcx.int(),
                    variadic: false,
                    param_spec: None,
                }),
                "index" => self.tcx.intern(Ty::Fn {
                    params: vec![elem, self.tcx.int(), self.tcx.int()],
                    ret: self.tcx.int(),
                    variadic: false,
                    param_spec: None,
                }),
                _ => self
                    .resolve_generated_bound_member(obj_ty_id, attr)
                    .unwrap_or_else(|| self.tcx.any()),
            },
            Ty::Set(elem) => match attr {
                "add" | "remove" => self.tcx.intern(Ty::Fn {
                    params: vec![elem],
                    ret: self.tcx.none(),
                    variadic: false,
                    param_spec: None,
                }),
                _ => self
                    .resolve_generated_bound_member(obj_ty_id, attr)
                    .unwrap_or_else(|| self.tcx.any()),
            },
            Ty::Dict(key, value) => match attr {
                "__delitem__" => self.tcx.intern(Ty::Fn {
                    params: vec![key],
                    ret: self.tcx.none(),
                    variadic: false,
                    param_spec: None,
                }),
                "__getitem__" => self.tcx.intern(Ty::Fn {
                    params: vec![key],
                    ret: value,
                    variadic: false,
                    param_spec: None,
                }),
                "__setitem__" => self.tcx.intern(Ty::Fn {
                    params: vec![key, value],
                    ret: self.tcx.none(),
                    variadic: false,
                    param_spec: None,
                }),
                "get" | "pop" => {
                    let any = self.tcx.any();
                    self.tcx.intern(Ty::Fn {
                        params: vec![key, any],
                        ret: any,
                        variadic: false,
                        param_spec: None,
                    })
                }
                _ => self
                    .resolve_generated_bound_member(obj_ty_id, attr)
                    .unwrap_or_else(|| self.tcx.any()),
            },
            Ty::Class {
                role, user, fields, ..
            } => {
                if role == ClassRole::Object {
                    return self.tcx.any();
                }
                if let Some((symbol, args, property_ty)) = user.as_ref().and_then(|user| {
                    self.class_property_getters
                        .get(&user.symbol)
                        .and_then(|properties| properties.get(attr))
                        .copied()
                        .map(|ty| (user.symbol, user.args.clone(), ty))
                }) {
                    return self.specialize_class_member_type(symbol, &args, property_ty);
                }
                for (name, ty) in &fields {
                    if name == attr {
                        return *ty;
                    }
                }
                if let Some(method) = user
                    .as_ref()
                    .and_then(|user| self.class_methods_by_symbol.get(&user.symbol))
                    .and_then(|methods| methods.get(attr))
                    .cloned()
                {
                    let substitution = user.as_ref().and_then(|user| {
                        self.generic_defs.get(&user.symbol).map(|params| {
                            let mut subst = Substitution::new();
                            for (param, arg) in params.params.iter().zip(&user.args) {
                                subst.insert(param.id, *arg);
                            }
                            subst
                        })
                    });
                    let params = if let Some(subst) = &substitution {
                        method
                            .params
                            .iter()
                            .map(|param| subst.apply(*param, &mut self.tcx))
                            .collect()
                    } else {
                        method.params
                    };
                    let ret = substitution
                        .as_ref()
                        .map(|subst| subst.apply(method.return_type, &mut self.tcx))
                        .unwrap_or(method.return_type);
                    return self.tcx.intern(Ty::Fn {
                        params,
                        ret,
                        variadic: false,
                        param_spec: None,
                    });
                }
                self.tcx.any()
            }
            Ty::Any | Ty::Error => self.tcx.any(),
            _ => self
                .resolve_generated_bound_member(obj_ty_id, attr)
                .unwrap_or_else(|| self.tcx.any()),
        }
    }

    /// Resolve subscript / index access (#248).
    fn resolve_subscript(&mut self, obj_ty: TypeId, _span: Span) -> TypeId {
        match self.semantic_ty(obj_ty) {
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

    /// #1031: numeric builtin `ty` satisfies, if any — a plain numeric
    /// builtin satisfies its own root directly (`bool` collapses to `Int`,
    /// matching Python's `bool` being an `int` subtype), and `Ty::Class(name)`
    /// satisfies a root only when `name`'s base chain was recorded in
    /// `numeric_derived_classes` (i.e. it derives `int`/`float` — a bare
    /// `class Q: pass` does NOT reach this). Shared by every hard numeric-type
    /// wall below so an int-derived-class instance (`class P(int): pass`)
    /// compiles wherever a bare `int` would, without loosening rejection of
    /// genuinely non-numeric classes.
    fn numeric_root(&self, ty: &Ty) -> Option<NumericRoot> {
        match ty {
            Ty::Int | Ty::Bool => Some(NumericRoot::Int),
            Ty::Float => Some(NumericRoot::Float),
            Ty::Class {
                name,
                role: ClassRole::Instance,
                user,
                ..
            } => user
                .as_ref()
                .and_then(|user| {
                    self.numeric_derived_class_symbols
                        .get(&user.symbol)
                        .copied()
                })
                .or_else(|| {
                    user.is_none()
                        .then(|| self.numeric_derived_classes.get(name).copied())
                        .flatten()
                }),
            _ => None,
        }
    }

    /// #1031: true when `ty_id` may stand in for a numeric operand — a plain
    /// numeric builtin, `Any`/`Error` (already deferred/reported elsewhere),
    /// or a numeric-derived class instance ([`Self::numeric_root`]).
    fn is_numeric_like(&self, ty_id: TypeId) -> bool {
        let ty = self.tcx.get(ty_id);
        matches!(ty, Ty::Any | Ty::Error) || self.numeric_root(ty).is_some()
    }

    /// #1031: true when `ty_id` may stand in for an `int` operand
    /// specifically — plain `int`/`bool`, `Any`/`Error`, or a class whose
    /// base chain reaches `int` (NOT `float`: a `float`-derived class must
    /// still reject `~`/shifts exactly like a bare `float` does).
    fn is_int_like(&self, ty_id: TypeId) -> bool {
        let ty = self.tcx.get(ty_id);
        matches!(ty, Ty::Any | Ty::Error) || matches!(self.numeric_root(ty), Some(NumericRoot::Int))
    }

    /// #1041: true when `ty_id` is a `Ty::Class` whose method table — or any
    /// class in its base chain — defines `dunder`. Sibling of
    /// [`Self::numeric_root`]/[`Self::is_numeric_like`]/[`Self::is_int_like`]
    /// above (#1031), which only recognize classes deriving `int`/`float`;
    /// this lets a class with an explicit unary/shift dunder override
    /// (`__neg__`/`__pos__`/`__invert__`/`__lshift__`/`__rshift__`) pass the
    /// walls below even when it is NOT numeric-derived (`class V:
    /// def __neg__(self): ...`). Walks `class_bases` (#885/#886's
    /// `class_methods`, plus the base-name side table populated alongside it
    /// in `check.rs`) so an inherited dunder several levels up
    /// (`class W(V): pass`) is found too. A class without the dunder
    /// anywhere in its chain returns `false` — the walls' rejection of
    /// genuinely non-overriding classes is unchanged.
    fn class_defines_dunder(&self, ty_id: TypeId, dunder: &str) -> bool {
        let Ty::Class {
            name, role, user, ..
        } = self.tcx.get(ty_id)
        else {
            return false;
        };
        if *role == ClassRole::Object {
            return false;
        }
        if let Some(user) = user {
            let mut visited = std::collections::HashSet::new();
            let mut queue = vec![user.symbol];
            while let Some(symbol) = queue.pop() {
                if !visited.insert(symbol) {
                    continue;
                }
                if self
                    .class_methods_by_symbol
                    .get(&symbol)
                    .is_some_and(|methods| methods.contains_key(dunder))
                {
                    return true;
                }
                if let Some(bases) = self.class_base_symbols.get(&symbol) {
                    queue.extend(bases.iter().copied());
                }
            }
            return false;
        }
        let mut visited: std::collections::HashSet<&str> = std::collections::HashSet::new();
        let mut queue: Vec<&str> = vec![name.as_str()];
        while let Some(cur) = queue.pop() {
            if !visited.insert(cur) {
                continue;
            }
            if self
                .class_methods
                .get(cur)
                .is_some_and(|methods| methods.contains_key(dunder))
            {
                return true;
            }
            if let Some(bases) = self.class_bases.get(cur) {
                queue.extend(bases.iter().map(String::as_str));
            }
        }
        false
    }

    /// #1031: resolve the compile-time result type of a numeric-derived-class
    /// unary op. Builtin numeric operations on a subclass instance return a
    /// plain instance of the base builtin (never the subclass) when the
    /// subclass hasn't overridden the dunder — checker has no dunder-override
    /// visibility, so this always collapses to the root, matching the common
    /// (unoverridden) case. `None` when `ty_id` isn't a numeric-derived class
    /// (caller keeps its existing fallback).
    fn numeric_derived_result_ty(&mut self, ty_id: TypeId) -> Option<TypeId> {
        let Ty::Class {
            name, role, user, ..
        } = self.tcx.get(ty_id)
        else {
            return None;
        };
        if *role == ClassRole::Object {
            return None;
        }
        let root = user
            .as_ref()
            .and_then(|user| {
                self.numeric_derived_class_symbols
                    .get(&user.symbol)
                    .copied()
            })
            .or_else(|| {
                user.is_none()
                    .then(|| self.numeric_derived_classes.get(name).copied())
                    .flatten()
            });
        match root {
            Some(NumericRoot::Int) => Some(self.tcx.int()),
            Some(NumericRoot::Float) => Some(self.tcx.float()),
            None => None,
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
                // Set binops on statically-known sets route to the existing
                // runtime set operator support; keep the gate narrow to real
                // Set types so unrelated non-numeric `-` expressions still
                // hard-error.
                if matches!(op, BinOp::Sub)
                    && matches!(self.tcx.get(lt), Ty::Set(_))
                    && matches!(self.tcx.get(rt), Ty::Set(_))
                {
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
                if matches!(
                    self.tcx.get(lt),
                    Ty::Class {
                        role: ClassRole::Instance,
                        ..
                    }
                ) || matches!(
                    self.tcx.get(rt),
                    Ty::Class {
                        role: ClassRole::Instance,
                        ..
                    }
                ) {
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
                // Bool is a subtype of int — accept both. #1031: also accept
                // a class deriving int (`class P(int): pass; P(7) << 1`);
                // `is_int_like` rejects a float-derived class exactly like a
                // bare `float` operand. #1041: a `lt` class that is NOT
                // int-derived but defines the matching shift dunder itself
                // (walking bases) is also accepted for the left/receiver
                // operand — mirrors the unary wall's dunder acceptance.
                // `rt` (the shift amount) keeps the unchanged `is_int_like`
                // requirement; the receiver's own dunder decides dispatch at
                // runtime regardless of `rt`'s concrete type.
                let shift_dunder = if matches!(op, BinOp::LShift) {
                    "__lshift__"
                } else {
                    "__rshift__"
                };
                let lt_int_like = self.is_int_like(lt);
                let lt_has_dunder = !lt_int_like && self.class_defines_dunder(lt, shift_dunder);
                if (!lt_int_like && !lt_has_dunder) || !self.is_int_like(rt) {
                    self.error(span, "shift operators require int types");
                    return self.tcx.error();
                }
                if lt_has_dunder {
                    self.tcx.any()
                } else {
                    self.tcx.int()
                }
            }
            BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => {
                if matches!(self.tcx.get(lt), Ty::Set(_)) && matches!(self.tcx.get(rt), Ty::Set(_))
                {
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
                    return lt;
                }
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
                        | Ty::Set(_)
                        | Ty::Tuple(_)
                        | Ty::Any
                        | Ty::Class {
                            role: ClassRole::Instance,
                            ..
                        }
                );
                let rt_ok = matches!(
                    self.tcx.get(rt),
                    Ty::Int
                        | Ty::Float
                        | Ty::Bool
                        | Ty::Str
                        | Ty::List(_)
                        | Ty::Set(_)
                        | Ty::Tuple(_)
                        | Ty::Any
                        | Ty::Class {
                            role: ClassRole::Instance,
                            ..
                        }
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
                let class_target = self.class_pattern_target(cls);
                if class_target == ClassPatternTarget::Invalid {
                    self.error(pattern.span, "class pattern target must be a class");
                }

                // Built-in self-subject patterns: case int(x), case str(s), etc.
                // These have ONE positional arg that captures the subject itself.
                if let ClassPatternTarget::Instance(capture_ty) = class_target {
                    if matches!(self.tcx.get(capture_ty), Ty::Class { .. }) {
                        // User, builtin-exception, and FFI class metadata is handled below.
                    } else {
                        let prev = self.current_match_subject_ty.replace(capture_ty);
                        for (_, sub_pat) in patterns {
                            self.check_pattern(sub_pat);
                        }
                        self.current_match_subject_ty = prev;
                        return;
                    }
                }
                let (class_fields, explicit_match_args): (
                    Vec<(String, TypeId)>,
                    Option<Vec<String>>,
                ) = match class_target {
                    ClassPatternTarget::Instance(ty) => {
                        if let Ty::Class {
                            fields, match_args, ..
                        } = self.tcx.get(ty).clone()
                        {
                            (fields, match_args)
                        } else {
                            (Vec::new(), None)
                        }
                    }
                    ClassPatternTarget::Unknown | ClassPatternTarget::Invalid => (Vec::new(), None),
                };

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
                    Pattern::ClassPattern { cls, .. } => match self.class_pattern_target(cls) {
                        ClassPatternTarget::Instance(ty) => ty,
                        ClassPatternTarget::Unknown | ClassPatternTarget::Invalid => self.tcx.any(),
                    },
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
    use crate::types::ty::ClassRole;
    use crate::types::Ty;

    fn sp<T>(node: T) -> Spanned<T> {
        Spanned::new(node, Span::dummy())
    }

    fn bare_ctor_call(name: &str) -> Spanned<Expr> {
        sp(Expr::Call {
            func: Box::new(sp(Expr::Ident(name.to_string()))),
            args: Vec::new(),
        })
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

    #[test]
    fn test_typed_literal_bare_instance_name_detects_nested_container_element() {
        let mut checker = TypeChecker::new();
        checker.user_bare_classes.insert("_W".to_string());
        let expr = sp(Expr::ListLit(vec![
            sp(Expr::TupleLit(vec![bare_ctor_call("_W")])),
            sp(Expr::SetLit(vec![bare_ctor_call("_W")])),
        ]));
        assert_eq!(
            checker.typed_literal_bare_instance_name(&expr),
            Some("_W".to_string())
        );
    }

    #[test]
    fn test_typed_literal_bare_instance_name_skips_dict_literal() {
        let mut checker = TypeChecker::new();
        checker.user_bare_classes.insert("_W".to_string());
        let expr = sp(Expr::DictLit(vec![(
            Some(sp(Expr::StrLit("k".to_string()))),
            bare_ctor_call("_W"),
        )]));
        assert_eq!(checker.typed_literal_bare_instance_name(&expr), None);
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
        assert!(matches!(
            checker.tcx.get(ty),
            Ty::Class {
                role: ClassRole::Instance,
                external: Some(external),
                ..
            } if external.module == "builtins" && external.name == "complex"
        ));
    }

    #[test]
    fn test_check_expr_bytes_lit() {
        let mut checker = TypeChecker::new();
        let ty = checker.check_expr(&sp(Expr::BytesLit(vec![104, 105])));
        assert!(matches!(
            checker.tcx.get(ty),
            Ty::Class {
                role: ClassRole::Instance,
                external: Some(external),
                ..
            } if external.module == "builtins" && external.name == "bytes"
        ));
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
