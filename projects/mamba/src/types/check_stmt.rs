use super::check::TypeChecker;
use super::{Ty, TypeId};
use crate::parser::ast::*;
use crate::resolve::SymbolKind;
use crate::source::span::Spanned;

fn decorator_is_typing_overload(expr: &Expr) -> bool {
    match expr {
        Expr::Ident(n) => n == "overload",
        Expr::Attr { attr, .. } => attr == "overload",
        Expr::Call { func, .. } => decorator_is_typing_overload(&func.node),
        _ => false,
    }
}

/// Statement type checking, function body checking, and helpers.
impl TypeChecker {
    /// ① Type-wall PoC: is `cls` a from-imported name that names a stdlib *class*
    /// carrying at least one `Method` signature in the sig table (curated OR
    /// generated)? Used to bind instance provenance so that a later
    /// `obj.method(arg)` can resolve a `Method` signature and apply the same
    /// guarded scalar check.
    ///
    /// The import-time qualifier in `import_origins` only reflects the *curated*
    /// table (see `is_known_stdlib_class`), so generated stdlib classes are bound
    /// with an empty qualifier and the old `qual == cls` test misses them. We
    /// therefore consult the sig table directly: `cls` must be a from-imported
    /// name (present in `import_origins`) whose `(module, cls)` owns a `Method`
    /// row. A from-imported *module function* never owns a `Method` row keyed on
    /// its own name, so this never mis-binds a module fn as an instance class.
    fn stdlib_method_class(&self, cls: &str) -> Option<String> {
        use super::stdlib_sigs::{SigKind, STDLIB_SIGS};
        use super::stdlib_sigs_generated::STDLIB_SIGS_GENERATED;
        let (module, _qual) = self.import_origins.get(cls)?;
        let owns_method = |s: &super::stdlib_sigs::StdlibSig| {
            matches!(s.kind, SigKind::Method) && s.module == module && s.qualifier == cls
        };
        if STDLIB_SIGS.iter().any(owns_method) || STDLIB_SIGS_GENERATED.iter().any(owns_method) {
            Some(cls.to_string())
        } else {
            None
        }
    }

    /// ① Type-wall PoC: if `value` constructs an instance of a known imported
    /// stdlib class, return that class's qualifier. Recognizes
    /// `object.__new__(Cls)` and `Cls(...)` where `Cls` is a from-imported stdlib
    /// class with a `Method` signature in the sig table.
    fn stdlib_instance_class(&self, value: &Spanned<Expr>) -> Option<String> {
        let Expr::Call { func, args } = &value.node else {
            return None;
        };
        // `Cls(...)` — direct constructor call.
        if let Expr::Ident(cls) = &func.node {
            return self.stdlib_method_class(cls);
        }
        // `object.__new__(Cls)` — bare allocation. func is `object.__new__`.
        if let Expr::Attr { object, attr } = &func.node {
            if attr == "__new__" {
                if let Expr::Ident(base) = &object.node {
                    if base == "object" {
                        if let Some(CallArg::Positional(arg0)) = args.first() {
                            if let Expr::Ident(cls) = &arg0.node {
                                return self.stdlib_method_class(cls);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// ① Type-wall PoC: walk a statement sequence with one-statement lookahead so
    /// the "expected to raise at runtime" carve-out can fire. When an `ExprStmt`
    /// is immediately followed by a `raise`, the probe call's value-vs-annotation
    /// enforcement (constructor/method/module-fn arg check) is SUPPRESSED for that
    /// statement only — the program's correct behavior depends on the call raising
    /// at runtime, so a compile-time rejection would abort it (see
    /// `SUPPRESS_STDLIB_ARG_CHECK` in check_expr.rs). Every other statement is
    /// checked exactly as before. Used for block bodies that can host the
    /// auto-ported manual-`assertRaises` idiom (`try`/function/module bodies).
    pub(crate) fn check_stmt_seq(&mut self, body: &[Spanned<Stmt>]) {
        for (i, s) in body.iter().enumerate() {
            let next_is_raise = body
                .get(i + 1)
                .map(|n| matches!(n.node, Stmt::Raise { .. }))
                .unwrap_or(false);
            if next_is_raise && matches!(s.node, Stmt::ExprStmt(_)) {
                // Suppress value-vs-annotation arg enforcement for THIS probe
                // statement only, then restore.
                let prev = super::check_expr::set_stdlib_arg_check_suppressed(true);
                self.check_stmt(s);
                super::check_expr::restore_stdlib_arg_check(prev);
            } else {
                self.check_stmt(s);
            }
        }
    }

    pub(crate) fn check_stmt(&mut self, stmt: &Spanned<Stmt>) {
        match &stmt.node {
            Stmt::VarDecl { name, ty, value } => {
                let declared_ty = self.resolve_type_expr(ty);
                let value_ty = self.check_expr(value);
                if !self.types_compatible(declared_ty, value_ty) {
                    self.error(
                        value.span,
                        format!(
                            "type mismatch: expected `{}`, got `{}`",
                            self.ty_name(declared_ty),
                            self.ty_name(value_ty),
                        ),
                    );
                } else {
                    // Element-level check for container literals: a heterogeneous
                    // literal like `[1, "two"]` widens to `list[Any]`, which
                    // passes the whole-value compatibility check above. Verify
                    // each literal element against the declared element type so
                    // `xs: list[int] = [1, "two"]` is rejected.
                    self.check_container_literal_elements(declared_ty, value);
                }
                let sym = self.symbols.define(name.clone(), SymbolKind::Variable);
                self.set_sym_type(sym.0, declared_ty);
            }
            Stmt::Assign { target, value } => {
                // ① Type-wall PoC: record stdlib-instance provenance so that a
                // later `obj.method(arg)` can resolve a `Method` signature.
                // `obj = object.__new__(Cls)` or `obj = Cls(...)` where `Cls` is
                // a known imported stdlib class binds `obj` -> class qualifier.
                if let Expr::Ident(var) = &target.node {
                    if let Some(cls) = self.stdlib_instance_class(value) {
                        self.instance_origins.insert(var.clone(), cls);
                    } else {
                        // Reassignment to something else clears the origin.
                        self.instance_origins.remove(var);
                    }
                }
                // Implicit variable declaration: bare `x = val` where x is not
                // yet in scope behaves like Python — declares x with the
                // inferred type of val.
                if let Expr::Ident(name) = &target.node {
                    if self.symbols.lookup(name).is_none() {
                        let value_ty = self.check_expr(value);
                        let sym = self.symbols.define(name.clone(), SymbolKind::Variable);
                        self.set_sym_type(sym.0, value_ty);
                        return;
                    }
                }
                // Tuple/star unpack target: `a, *b, c = rhs` or `(a, b) = rhs`.
                // Define each variable in the target before type-checking.
                if matches!(&target.node, Expr::TupleLit(_) | Expr::UnpackTarget(_)) {
                    self.check_expr(value);
                    fn define_unpack_targets(checker: &mut TypeChecker, expr: &Expr) {
                        match expr {
                            Expr::Ident(name) => {
                                if checker.symbols.lookup(name).is_none() {
                                    let sym =
                                        checker.symbols.define(name.clone(), SymbolKind::Variable);
                                    let any_ty = checker.tcx.any();
                                    checker.set_sym_type(sym.0, any_ty);
                                }
                            }
                            Expr::Starred(inner) => define_unpack_targets(checker, &inner.node),
                            Expr::TupleLit(elems) | Expr::UnpackTarget(elems) => {
                                for elem in elems {
                                    define_unpack_targets(checker, &elem.node);
                                }
                            }
                            _ => {}
                        }
                    }
                    define_unpack_targets(self, &target.node);
                    return;
                }
                if let Expr::Index { object, index } = &target.node {
                    self.check_subscript_assignment(object, index, value);
                    return;
                }

                let target_ty = self.check_expr(target);
                let value_ty = self.check_expr(value);
                if !self.types_compatible(target_ty, value_ty) {
                    // Python allows rebinding a variable to a different type
                    // (`a = 5; a = "str"`). If the target is a bare identifier,
                    // widen its symbol type to Any rather than rejecting the
                    // assignment. Structural targets (attr / index / unpack)
                    // still error — those respect the object's type contract.
                    if let Expr::Ident(name) = &target.node {
                        if let Some(sym) = self.symbols.lookup(name) {
                            let any_ty = self.tcx.any();
                            self.set_sym_type(sym.0, any_ty);
                            return;
                        }
                    }
                    self.error(
                        value.span,
                        format!(
                            "type mismatch in assignment: expected `{}`, got `{}`",
                            self.ty_name(target_ty),
                            self.ty_name(value_ty),
                        ),
                    );
                }
            }
            Stmt::FnDef {
                name,
                type_params,
                params,
                return_ty,
                body,
                decorators,
                ..
            }
            | Stmt::AsyncFnDef {
                name,
                type_params,
                params,
                return_ty,
                body,
                decorators,
                ..
            } => {
                // Re-register type params for body checking scope
                let _gp = self.register_type_params(type_params);
                let overload_decorated = decorators
                    .iter()
                    .any(|d| decorator_is_typing_overload(&d.node));
                self.check_fn_body(name, params, return_ty.as_ref(), body, overload_decorated);
                self.unregister_type_params(type_params);
            }
            Stmt::If {
                condition,
                body,
                elif_clauses,
                else_body,
            } => {
                let _cond_ty = self.check_expr(condition);
                // Python: any type can be used in a condition (truthiness via __bool__/__len__)
                for s in body {
                    self.check_stmt(s);
                }

                for (cond, elif_body) in elif_clauses {
                    let _ct = self.check_expr(cond);
                    for s in elif_body {
                        self.check_stmt(s);
                    }
                }

                if let Some(eb) = else_body {
                    for s in eb {
                        self.check_stmt(s);
                    }
                }
            }
            Stmt::While {
                condition,
                body,
                else_body,
            } => {
                let _cond_ty = self.check_expr(condition);
                // Python: any type can be used in a while condition
                //
                // Python has no block scope: assignments inside a `while` body
                // (or its `else`) bind in the enclosing scope and remain
                // visible after the loop. Mirror the `Stmt::Try` handling —
                // do NOT push a scope here, or post-loop reads of body-assigned
                // names raise bogus "undefined name" type errors.
                for s in body {
                    self.check_stmt(s);
                }
                if let Some(eb) = else_body {
                    for s in eb {
                        self.check_stmt(s);
                    }
                }
            }
            Stmt::For {
                targets,
                var_ty,
                iter,
                body,
                else_body,
            }
            | Stmt::AsyncFor {
                targets,
                var_ty,
                iter,
                body,
                else_body,
            } => {
                // Python has no block scope: loop targets and body assignments
                // bind in the enclosing scope and persist after the loop (the
                // very common read-loop-var-after-loop idiom). Mirror the
                // `Stmt::Try` handling and the resolver pass — do NOT push a
                // scope here. Comprehensions DO scope their variables; that is
                // handled separately in `check_expr` and stays scoped.
                let ty = var_ty
                    .as_ref()
                    .map(|t| self.resolve_type_expr(t))
                    .unwrap_or_else(|| self.infer_iter_element(iter));
                if targets.len() > 1 {
                    // Tuple-destructuring targets: `for a, b in [(17, 5)]`
                    // binds each target to the corresponding tuple ELEMENT
                    // type, not the whole element type (which made `a // b`
                    // a bogus tuple//tuple hard error). On shape mismatch or
                    // a non-tuple element type, fall back to Any and defer
                    // to runtime unpacking.
                    let elem_tys: Option<Vec<TypeId>> = match self.tcx.get(ty) {
                        Ty::Tuple(ts) if ts.len() == targets.len() => Some(ts.clone()),
                        _ => None,
                    };
                    for (i, var) in targets.iter().enumerate() {
                        let t = elem_tys
                            .as_ref()
                            .map(|ts| ts[i])
                            .unwrap_or_else(|| self.tcx.any());
                        let sym = self.symbols.define(var.clone(), SymbolKind::Variable);
                        self.set_sym_type(sym.0, t);
                    }
                } else {
                    for var in targets {
                        let sym = self.symbols.define(var.clone(), SymbolKind::Variable);
                        self.set_sym_type(sym.0, ty);
                    }
                }
                for s in body {
                    self.check_stmt(s);
                }
                if let Some(eb) = else_body {
                    for s in eb {
                        self.check_stmt(s);
                    }
                }
            }
            Stmt::Return(value) => {
                let val_ty = value
                    .as_ref()
                    .map(|v| self.check_expr(v))
                    .unwrap_or(self.tcx.none());
                if let Some(expected) = self.current_return_ty {
                    if !self.types_compatible(expected, val_ty) {
                        let span = value.as_ref().map(|v| v.span).unwrap_or(stmt.span);
                        self.error(
                            span,
                            format!(
                                "return type mismatch: expected `{}`, got `{}`",
                                self.ty_name(expected),
                                self.ty_name(val_ty),
                            ),
                        );
                    }
                }
            }
            Stmt::ExprStmt(expr) => {
                self.check_expr(expr);
            }
            Stmt::Match { expr, arms } => {
                let subject_ty = self.check_expr(expr);
                // Python has no block scope: pattern captures (incl. AS
                // aliases) and case-body assignments bind in the enclosing
                // scope and remain visible after the match. Mirror the
                // `Stmt::Try` handling — do NOT push a per-arm scope.
                //
                // Per-arm class-pattern narrowing of the subject (#827, R4)
                // must stay arm-local, so snapshot the subject's binding type
                // before narrowing and restore it after the arm body when (and
                // only when) narrowing actually re-bound it.
                let subject_name = match &expr.node {
                    Expr::Ident(n) => Some(n.clone()),
                    _ => None,
                };
                for arm in arms {
                    let saved_subject_ty = subject_name
                        .as_ref()
                        .and_then(|n| self.symbols.lookup(n))
                        .map(|s| self.get_sym_type(s.0));
                    // R4: flow-sensitive type narrowing (#827)
                    // When a case branch uses a class pattern, narrow the matched
                    // variable's type to the class type within the branch body.
                    let narrowed = self.narrow_match_subject(expr, &arm.pattern);
                    // Propagate subject type into capture/star/AS bindings (#827).
                    let prev_subject_ty = self.current_match_subject_ty.replace(subject_ty);
                    self.check_pattern(&arm.pattern);
                    self.current_match_subject_ty = prev_subject_ty;
                    // Type-check guard expression (#827) for its side effects
                    // (walrus bindings, name resolution, sub-expression
                    // inference). A guard may be ANY expression — it is
                    // truthy-tested at runtime, exactly like the `if`/`while`
                    // conditions above, neither of which enforces `bool`.
                    // Enforcing bool here spuriously rejected valid guards such
                    // as `case n if (doubled := n * 2):` (the walrus value is
                    // `int`, not `bool`).
                    if let Some(guard) = &arm.guard {
                        let _guard_ty = self.check_expr(guard);
                    }
                    for s in &arm.body {
                        self.check_stmt(s);
                    }
                    // Un-narrow: restore the subject's pre-arm type so the
                    // narrowing does not leak into later arms or past the match.
                    if narrowed {
                        if let (Some(name), Some(orig_ty)) =
                            (subject_name.as_ref(), saved_subject_ty)
                        {
                            let sym = self.symbols.define(name.clone(), SymbolKind::Variable);
                            self.set_sym_type(sym.0, orig_ty);
                        }
                    }
                }
            }
            Stmt::ClassDef {
                name,
                type_params,
                body,
                ..
            } => {
                let _gp = self.register_type_params(type_params);
                let prev_class = self.current_class.replace(name.clone());
                self.symbols.push_scope();
                for s in body {
                    self.check_stmt(s);
                }
                self.symbols.pop_scope();
                self.current_class = prev_class;
                self.unregister_type_params(type_params);
            }
            Stmt::EnumDef { .. } => { /* handled in first pass */ }
            Stmt::AugAssign { target, value, .. } => {
                self.check_expr(target);
                self.check_expr(value);
            }
            Stmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
            } => {
                // Python semantics: variables assigned inside try/except/else/finally
                // are visible in the enclosing scope after the block — no new scope
                // is pushed for the block bodies themselves.
                //
                // Use the sibling-aware walker: a `try` body is where the
                // auto-ported manual-`assertRaises` idiom (`probe(); raise ...`)
                // lives, so a probe immediately followed by a `raise` has its
                // value-vs-annotation arg enforcement suppressed (it must raise at
                // runtime, not be rejected at compile time).
                self.check_stmt_seq(body);
                for handler in handlers {
                    // The exception alias (`as exc`) is defined in the outer scope.
                    // In CPython it's deleted at the end of the except clause, but
                    // we leave it as Any to avoid false "undefined name" errors.
                    if let Some(name) = &handler.name {
                        let any_ty = self.tcx.any();
                        let sym = self.symbols.define(name.clone(), SymbolKind::Variable);
                        self.set_sym_type(sym.0, any_ty);
                    }
                    for s in &handler.body {
                        self.check_stmt(s);
                    }
                }
                if let Some(eb) = else_body {
                    for s in eb {
                        self.check_stmt(s);
                    }
                }
                if let Some(fb) = finally_body {
                    for s in fb {
                        self.check_stmt(s);
                    }
                }
            }
            Stmt::Raise { value, from } => {
                if let Some(v) = value {
                    self.check_expr(v);
                }
                if let Some(f) = from {
                    self.check_expr(f);
                }
            }
            Stmt::With { items, body } | Stmt::AsyncWith { items, body } => {
                // Python's `with` is NOT a scope — `as v` aliases bind in the
                // enclosing scope and remain visible after the with-block exits.
                // (CPython language ref §8.5.) Defining the alias in a fresh
                // scope and popping it dropped the binding, so post-with reads
                // of `v` raised an undefined-name type error.
                for item in items {
                    self.check_expr(&item.context);
                    if let Some(alias) = &item.alias {
                        let sym = self.symbols.define(alias.clone(), SymbolKind::Variable);
                        self.set_sym_type(sym.0, self.tcx.any());
                    }
                }
                for s in body {
                    self.check_stmt(s);
                }
            }
            Stmt::Assert { test, msg } => {
                self.check_expr(test);
                if let Some(m) = msg {
                    self.check_expr(m);
                }
            }
            Stmt::Del(expr) => {
                self.check_expr(expr);
            }
            Stmt::Global(names) => {
                use crate::resolve::VariableClass;
                for name in names {
                    let id = if let Some(existing) = self.symbols.lookup(name) {
                        existing
                    } else {
                        self.symbols.define(name.clone(), SymbolKind::Variable)
                    };
                    self.symbols.set_var_class(id, VariableClass::Global);
                }
            }
            Stmt::Nonlocal(names) => {
                use crate::resolve::VariableClass;
                let current = self.symbols.current_scope_idx();
                for name in names {
                    let mut scope_idx = self.symbols.parent_scope(current);
                    let mut found = false;
                    while let Some(si) = scope_idx {
                        if let Some(outer_id) = self.symbols.lookup_in_scope(si, name) {
                            self.symbols.set_var_class(outer_id, VariableClass::Cell);
                            let inner_id = if let Some(existing) =
                                self.symbols.lookup_in_scope(current, name)
                            {
                                existing
                            } else {
                                self.symbols.define(name.clone(), SymbolKind::Variable)
                            };
                            self.symbols.set_var_class(inner_id, VariableClass::Free);
                            found = true;
                            break;
                        }
                        scope_idx = self.symbols.parent_scope(si);
                    }
                    if !found {
                        let id = self.symbols.define(name.clone(), SymbolKind::Variable);
                        self.symbols.set_var_class(id, VariableClass::Free);
                    }
                }
            }
            Stmt::TypeAlias { .. } => { /* handled in first pass */ }
            Stmt::Pass | Stmt::Break | Stmt::Continue => {}
            Stmt::Import {
                names,
                module_alias,
                module,
            } => {
                // Imported names get type Any — external modules are not resolved.
                // This prevents false "undefined name" errors for third-party imports.
                let any_ty = self.tcx.any();
                if let Some(import_names) = names {
                    // `from module import X` / `from module import X as Y`
                    for (name, alias) in import_names {
                        let effective = alias.as_ref().unwrap_or(name);
                        let sym = self.symbols.define(effective.clone(), SymbolKind::Variable);
                        self.set_sym_type(sym.0, any_ty);
                    }
                } else if let Some(alias) = module_alias {
                    // `import module as alias`
                    let sym = self.symbols.define(alias.clone(), SymbolKind::Variable);
                    self.set_sym_type(sym.0, any_ty);
                } else {
                    // `import module` — register the root module name
                    if let Some(root) = module.first() {
                        let sym = self.symbols.define(root.clone(), SymbolKind::Variable);
                        self.set_sym_type(sym.0, any_ty);
                    }
                }
            }
            Stmt::BareAnnotation { name, ty } => {
                let declared_ty = self.resolve_type_expr(ty);
                let sym = self.symbols.define(name.clone(), SymbolKind::Variable);
                self.set_sym_type(sym.0, declared_ty);
            }
        }
    }

    pub(crate) fn check_fn_body(
        &mut self,
        name: &str,
        params: &[Param],
        return_ty: Option<&Spanned<TypeExpr>>,
        body: &[Spanned<Stmt>],
        overload_decorated: bool,
    ) {
        // A parameter default value must satisfy the parameter's annotation
        // (`def f(c: int = "3")` is a type error). Mirrors the var-decl
        // `x: int = "3"` check. Defaults are evaluated in the *enclosing*
        // scope (Python semantics), so check them before pushing the function
        // scope and before any params are defined. Only `Regular` params can
        // carry defaults; `*args`/`**kwargs` annotate the element/value type,
        // not the collection, so they are skipped.
        for param in params {
            if param.kind != crate::parser::ast::ParamKind::Regular {
                continue;
            }
            if let Some(default) = &param.default {
                let declared_ty = self.resolve_type_expr(&param.ty);
                let default_ty = self.check_expr(default);
                if !self.types_compatible(declared_ty, default_ty) {
                    self.error(
                        default.span,
                        format!(
                            "type mismatch: expected `{}`, got `{}`",
                            self.ty_name(declared_ty),
                            self.ty_name(default_ty),
                        ),
                    );
                }
            }
        }
        self.symbols.push_scope();
        for param in params {
            let ty = self.resolve_type_expr(&param.ty);
            let sym = self
                .symbols
                .define(param.name.clone(), SymbolKind::Parameter);
            self.set_sym_type(sym.0, ty);
        }
        // Python scoping rule: any name assigned anywhere in the body is
        // local. Pre-define each such name as Any before walking the body
        // so identifier lookups find the local binding rather than walking
        // up to an outer-scope symbol (which would type-commit the outer
        // and reject rebinds).
        let mut assigned: Vec<String> = Vec::new();
        let mut declared: Vec<String> = Vec::new();
        crate::resolve::pass::collect_assignment_targets(body, &mut assigned, &mut declared);
        crate::resolve::pass::collect_walrus_targets_in_stmts(body, &mut assigned);
        let any_ty = self.tcx.any();
        for n in &assigned {
            if declared.iter().any(|d| d == n) {
                continue;
            }
            if params.iter().any(|p| &p.name == n) {
                continue;
            }
            let sym = self.symbols.define(n.clone(), SymbolKind::Variable);
            self.set_sym_type(sym.0, any_ty);
        }
        let ret_ty = match return_ty {
            Some(t) => self.resolve_type_expr(t),
            None => {
                // #240: missing return annotation defaults to Any
                self.tcx.any()
            }
        };
        let prev_ret = self.current_return_ty.replace(ret_ty);
        self.check_stmt_seq(body);
        self.current_return_ty = prev_ret;
        self.symbols.pop_scope();
        if self.symbols.lookup(name).is_none() {
            // Detect *args variadic and only include pre-star positional params in type.
            let (param_types, ret_ty, is_variadic) = if overload_decorated {
                (Vec::new(), self.tcx.any(), true)
            } else {
                let star_pos = params
                    .iter()
                    .position(|p| p.kind == crate::parser::ast::ParamKind::Star);
                let is_variadic = star_pos.is_some()
                    || params
                        .iter()
                        .any(|p| p.kind == crate::parser::ast::ParamKind::DoubleStar);
                let effective_params = star_pos.map_or(params, |pos| &params[..pos]);
                let param_types: Vec<TypeId> = effective_params
                    .iter()
                    .filter(|p| p.kind == crate::parser::ast::ParamKind::Regular)
                    .map(|p| self.resolve_type_expr(&p.ty))
                    .collect();
                (param_types, ret_ty, is_variadic)
            };
            let fn_ty = self.tcx.intern(Ty::Fn {
                params: param_types,
                ret: ret_ty,
                variadic: is_variadic,
            });
            let sym = self.symbols.define(name.to_string(), SymbolKind::Function);
            self.set_sym_type(sym.0, fn_ty);
        }
    }

    /// Verify each literal element of a container against its declared element
    /// type (`xs: list[int] = [1, "two"]` must be rejected).
    ///
    /// A heterogeneous literal widens to `list[Any]`/`dict[Any, Any]`, which
    /// passes the whole-value `types_compatible` check, so the element mismatch
    /// is invisible there. This walks the matching literal shape and reuses the
    /// SAME `types_compatible` relation per element — so int->float promotion,
    /// bool->int, and `Any` element annotations remain accepted (zero false
    /// positives). The check only fires when the literal shape matches the
    /// declared container shape and the declared element type is concrete; any
    /// other case (non-literal value, `Any` element, shape mismatch) is left to
    /// the whole-value check.
    pub(crate) fn check_container_literal_elements(
        &mut self,
        declared_ty: TypeId,
        value: &Spanned<Expr>,
    ) {
        match (self.tcx.get(declared_ty).clone(), &value.node) {
            (Ty::List(elem_ty), Expr::ListLit(elems)) => {
                if self.tcx.get(elem_ty).is_any() {
                    return;
                }
                for elem in elems {
                    // Skip starred unpacks (`*rest`) — their element type is not
                    // a single value we can directly compare.
                    if matches!(elem.node, Expr::Starred(_)) {
                        continue;
                    }
                    let et = self.check_expr(elem);
                    if !self.types_compatible(elem_ty, et) {
                        self.error(
                            elem.span,
                            format!(
                                "type mismatch: expected `{}`, got `{}`",
                                self.ty_name(elem_ty),
                                self.ty_name(et),
                            ),
                        );
                    } else {
                        // Recurse for nested containers (`list[list[int]]`).
                        self.check_container_literal_elements(elem_ty, elem);
                    }
                }
            }
            (Ty::Dict(key_ty, val_ty), Expr::DictLit(pairs)) => {
                let key_any = self.tcx.get(key_ty).is_any();
                let val_any = self.tcx.get(val_ty).is_any();
                if key_any && val_any {
                    return;
                }
                for (k, v) in pairs {
                    // Skip `**other` dict unpacks (key is None).
                    let Some(k) = k else { continue };
                    if !key_any {
                        let kt = self.check_expr(k);
                        if !self.types_compatible(key_ty, kt) {
                            self.error(
                                k.span,
                                format!(
                                    "type mismatch: expected `{}`, got `{}`",
                                    self.ty_name(key_ty),
                                    self.ty_name(kt),
                                ),
                            );
                        }
                    }
                    if !val_any {
                        let vt = self.check_expr(v);
                        if !self.types_compatible(val_ty, vt) {
                            self.error(
                                v.span,
                                format!(
                                    "type mismatch: expected `{}`, got `{}`",
                                    self.ty_name(val_ty),
                                    self.ty_name(vt),
                                ),
                            );
                        } else {
                            self.check_container_literal_elements(val_ty, v);
                        }
                    }
                }
            }
            (Ty::Tuple(elem_tys), Expr::TupleLit(elems)) => {
                // Only check fixed-arity tuple annotations whose element count
                // matches the literal (`tuple[int, str]`). A bare `tuple` (empty
                // element list) or any arity mismatch is left to the whole-value
                // check to avoid false positives on variadic forms.
                if elem_tys.is_empty() || elem_tys.len() != elems.len() {
                    return;
                }
                // Skip if any element is starred (variadic unpack in the literal).
                if elems.iter().any(|e| matches!(e.node, Expr::Starred(_))) {
                    return;
                }
                for (decl_elem, elem) in elem_tys.iter().zip(elems.iter()) {
                    let decl_elem = *decl_elem;
                    if self.tcx.get(decl_elem).is_any() {
                        continue;
                    }
                    let et = self.check_expr(elem);
                    if !self.types_compatible(decl_elem, et) {
                        self.error(
                            elem.span,
                            format!(
                                "type mismatch: expected `{}`, got `{}`",
                                self.ty_name(decl_elem),
                                self.ty_name(et),
                            ),
                        );
                    } else {
                        self.check_container_literal_elements(decl_elem, elem);
                    }
                }
            }
            _ => {}
        }
    }

    /// Perform flow-sensitive type narrowing for a match arm (#827, R4).
    ///
    /// If `pattern` is a `ClassPattern` (or an `As` pattern wrapping one) and
    /// `subject` is a simple identifier, re-defines that identifier in the
    /// current scope with the class's registered type.
    ///
    /// Returns `true` when the subject binding was actually re-bound, so the
    /// caller can restore the original type after the arm body (match arms are
    /// not a scope, but narrowing must stay arm-local).
    pub(crate) fn narrow_match_subject(
        &mut self,
        subject: &Spanned<Expr>,
        pattern: &Spanned<Pattern>,
    ) -> bool {
        // Unwrap outer AS layer if present
        let (inner_pat, alias_name) = match &pattern.node {
            Pattern::As {
                pattern: inner,
                name: alias,
            } => (inner.as_ref(), Some(alias.clone())),
            other_pat => {
                // Wrap in a temporary spanned shell so we can recurse uniformly
                let tmp = Spanned {
                    node: other_pat.clone(),
                    span: pattern.span,
                };
                return self.narrow_match_subject_pat(subject, &tmp, None);
            }
        };
        self.narrow_match_subject_pat(subject, inner_pat, alias_name)
    }

    fn narrow_match_subject_pat(
        &mut self,
        subject: &Spanned<Expr>,
        pattern: &Spanned<Pattern>,
        alias_name: Option<String>,
    ) -> bool {
        let Expr::Ident(subject_name) = &subject.node else {
            return false;
        };
        // Extract class name from either ClassPattern or Constructor (#827 R4)
        let class_name = match &pattern.node {
            Pattern::ClassPattern { cls, .. } => cls.last().map(|s| s.as_str()).unwrap_or(""),
            Pattern::Constructor { path, .. } => path.last().map(|s| s.as_str()).unwrap_or(""),
            _ => return false,
        };
        if class_name.is_empty() {
            return false;
        }

        // Built-in self-subject patterns: narrow to the built-in type directly.
        let builtin_narrow_ty = match class_name {
            "int" => Some(self.tcx.int()),
            "bool" => Some(self.tcx.bool()),
            "str" => Some(self.tcx.str()),
            "float" => Some(self.tcx.float()),
            "list" => Some(self.tcx.any()),
            "tuple" => Some(self.tcx.any()),
            "dict" => Some(self.tcx.any()),
            _ => None,
        };
        if let Some(narrow_ty) = builtin_narrow_ty {
            let sym = self
                .symbols
                .define(subject_name.clone(), crate::resolve::SymbolKind::Variable);
            self.set_sym_type(sym.0, narrow_ty);
            if let Some(alias) = alias_name {
                let alias_sym = self
                    .symbols
                    .define(alias, crate::resolve::SymbolKind::Variable);
                self.set_sym_type(alias_sym.0, narrow_ty);
            }
            return true;
        }

        let Some(class_sym) = self.symbols.lookup(class_name) else {
            return false;
        };
        let class_ty = self.get_sym_type(class_sym.0);
        // Only narrow if the looked-up type is actually a class (not error/any)
        if matches!(self.tcx.get(class_ty), super::Ty::Class { .. }) {
            // Re-define the subject variable in the current scope with the narrowed type
            let sym = self
                .symbols
                .define(subject_name.clone(), crate::resolve::SymbolKind::Variable);
            self.set_sym_type(sym.0, class_ty);
            // Also narrow the AS-alias to the same class type (#827)
            if let Some(alias) = alias_name {
                let alias_sym = self
                    .symbols
                    .define(alias, crate::resolve::SymbolKind::Variable);
                self.set_sym_type(alias_sym.0, class_ty);
            }
            return true;
        }
        false
    }

    /// Collect class fields from a class body for type resolution (#246).
    pub(crate) fn collect_class_fields(&mut self, body: &[Spanned<Stmt>]) -> Vec<(String, TypeId)> {
        let mut fields = Vec::new();
        for stmt in body {
            if let Stmt::VarDecl { name, ty, .. } = &stmt.node {
                let ty_id = self.resolve_type_expr(ty);
                fields.push((name.clone(), ty_id));
            }
        }
        fields
    }

    /// Determine `__match_args__` order for a class (#827).
    ///
    /// Scans the class body for an explicit `__match_args__ = ("x", "y")` or
    /// `__match_args__: tuple = ("x", "y")` assignment.
    ///
    /// Returns:
    /// - `Some(names)` when explicit `__match_args__` found (even `Some(vec![])` for `= ()`).
    /// - `None` when not found; callers fall back to `__init__` param order or field order.
    pub(crate) fn collect_match_args(&self, body: &[Spanned<Stmt>]) -> Option<Vec<String>> {
        for stmt in body {
            match &stmt.node {
                // `__match_args__ = ("x", "y")` — bare assignment
                Stmt::Assign { target, value } => {
                    if let (Expr::Ident(name), Expr::TupleLit(elems)) = (&target.node, &value.node)
                    {
                        if name == "__match_args__" {
                            let names: Vec<String> = elems
                                .iter()
                                .filter_map(|e| {
                                    if let Expr::StrLit(s) = &e.node {
                                        Some(s.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            return Some(names); // authoritative even if empty
                        }
                    }
                }
                // `__match_args__: tuple = ("x", "y")` — typed var declaration
                Stmt::VarDecl { name, value, .. } => {
                    if name == "__match_args__" {
                        if let Expr::TupleLit(elems) = &value.node {
                            let names: Vec<String> = elems
                                .iter()
                                .filter_map(|e| {
                                    if let Expr::StrLit(s) = &e.node {
                                        Some(s.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            return Some(names); // authoritative even if empty
                        }
                    }
                }
                _ => {}
            }
        }
        None // no explicit __match_args__
    }

    fn check_subscript_assignment(
        &mut self,
        object: &Spanned<Expr>,
        index: &Spanned<Expr>,
        value: &Spanned<Expr>,
    ) {
        let obj_ty = self.check_expr(object);
        self.check_expr(index);
        let value_ty = self.check_expr(value);
        let is_slice = matches!(index.node, Expr::Slice { .. });
        let expected = match self.tcx.get(obj_ty).clone() {
            Ty::List(elem) if is_slice => Some(self.tcx.intern(Ty::List(elem))),
            Ty::List(elem) => Some(elem),
            Ty::Dict(_, value) => Some(value),
            Ty::Any | Ty::Error => None,
            _ => None,
        };

        let Some(expected_ty) = expected else {
            return;
        };
        if !self.types_compatible(expected_ty, value_ty) {
            self.error(
                value.span,
                format!(
                    "type mismatch in assignment: expected `{}`, got `{}`",
                    self.ty_name(expected_ty),
                    self.ty_name(value_ty),
                ),
            );
        }
    }

    /// Infer element type from an iterable expression (#248).
    pub(crate) fn infer_iter_element(&mut self, iter: &Spanned<Expr>) -> TypeId {
        // Special case: range() always yields Int elements.
        if let Expr::Call { func, .. } = &iter.node {
            if let Expr::Ident(name) = &func.node {
                if name == "range" {
                    return self.tcx.int();
                }
            }
        }
        let iter_ty = self.check_expr(iter);
        match self.tcx.get(iter_ty).clone() {
            Ty::List(elem) => elem,
            Ty::Dict(k, _) => k,
            Ty::Str => self.tcx.str(),
            Ty::Tuple(ts) if !ts.is_empty() => {
                // Dedupe so a tuple of homogeneous strings/ints/etc. yields the
                // bare element type rather than a Union[Str,Str,...] that fails
                // the Str+Str fast-path in BinOp::Add (#1562).
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
            _ => self.tcx.any(),
        }
    }
}
