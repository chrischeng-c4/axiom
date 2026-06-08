use super::check::TypeChecker;
use super::{Ty, TypeId};
use crate::parser::ast::*;
use crate::resolve::SymbolKind;
use crate::source::span::Spanned;

/// Statement type checking, function body checking, and helpers.
impl TypeChecker {
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
                }
                let sym = self.symbols.define(name.clone(), SymbolKind::Variable);
                self.set_sym_type(sym.0, declared_ty);
            }
            Stmt::Assign { target, value } => {
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
                ..
            }
            | Stmt::AsyncFnDef {
                name,
                type_params,
                params,
                return_ty,
                body,
                ..
            } => {
                // Re-register type params for body checking scope
                let _gp = self.register_type_params(type_params);
                self.check_fn_body(name, params, return_ty.as_ref(), body);
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
                self.symbols.push_scope();
                for s in body {
                    self.check_stmt(s);
                }
                self.symbols.pop_scope();
                if let Some(eb) = else_body {
                    self.symbols.push_scope();
                    for s in eb {
                        self.check_stmt(s);
                    }
                    self.symbols.pop_scope();
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
                self.symbols.push_scope();
                let ty = var_ty
                    .as_ref()
                    .map(|t| self.resolve_type_expr(t))
                    .unwrap_or_else(|| self.infer_iter_element(iter));
                for var in targets {
                    let sym = self.symbols.define(var.clone(), SymbolKind::Variable);
                    self.set_sym_type(sym.0, ty);
                }
                for s in body {
                    self.check_stmt(s);
                }
                if let Some(eb) = else_body {
                    for s in eb {
                        self.check_stmt(s);
                    }
                }
                self.symbols.pop_scope();
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
                for arm in arms {
                    self.symbols.push_scope();
                    // R4: flow-sensitive type narrowing (#827)
                    // When a case branch uses a class pattern, narrow the matched
                    // variable's type to the class type within the branch scope.
                    self.narrow_match_subject(expr, &arm.pattern);
                    // Propagate subject type into capture/star/AS bindings (#827).
                    let prev_subject_ty = self.current_match_subject_ty.replace(subject_ty);
                    self.check_pattern(&arm.pattern);
                    self.current_match_subject_ty = prev_subject_ty;
                    // Type-check guard expression within the arm's scope (#827)
                    // Guard must be boolean (same semantics as if/while conditions)
                    if let Some(guard) = &arm.guard {
                        let guard_ty = self.check_expr(guard);
                        if !self.types_compatible(self.tcx.bool(), guard_ty) {
                            self.error(guard.span, "match guard condition must be bool");
                        }
                    }
                    for s in &arm.body {
                        self.check_stmt(s);
                    }
                    self.symbols.pop_scope();
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
                for s in body {
                    self.check_stmt(s);
                }
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
    ) {
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
        for s in body {
            self.check_stmt(s);
        }
        self.current_return_ty = prev_ret;
        self.symbols.pop_scope();
        if self.symbols.lookup(name).is_none() {
            // Detect *args variadic and only include pre-star positional params in type.
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
            let fn_ty = self.tcx.intern(Ty::Fn {
                params: param_types,
                ret: ret_ty,
                variadic: is_variadic,
            });
            let sym = self.symbols.define(name.to_string(), SymbolKind::Function);
            self.set_sym_type(sym.0, fn_ty);
        }
    }

    /// Perform flow-sensitive type narrowing for a match arm (#827, R4).
    ///
    /// If `pattern` is a `ClassPattern` (or an `As` pattern wrapping one) and
    /// `subject` is a simple identifier, re-defines that identifier in the
    /// current (already-pushed) scope with the class's registered type.
    pub(crate) fn narrow_match_subject(
        &mut self,
        subject: &Spanned<Expr>,
        pattern: &Spanned<Pattern>,
    ) {
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
        self.narrow_match_subject_pat(subject, inner_pat, alias_name);
    }

    fn narrow_match_subject_pat(
        &mut self,
        subject: &Spanned<Expr>,
        pattern: &Spanned<Pattern>,
        alias_name: Option<String>,
    ) {
        let Expr::Ident(subject_name) = &subject.node else {
            return;
        };
        // Extract class name from either ClassPattern or Constructor (#827 R4)
        let class_name = match &pattern.node {
            Pattern::ClassPattern { cls, .. } => cls.last().map(|s| s.as_str()).unwrap_or(""),
            Pattern::Constructor { path, .. } => path.last().map(|s| s.as_str()).unwrap_or(""),
            _ => return,
        };
        if class_name.is_empty() {
            return;
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
            return;
        }

        let Some(class_sym) = self.symbols.lookup(class_name) else {
            return;
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
        }
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
