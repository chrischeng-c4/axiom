use super::scope::{SymbolId, SymbolKind, SymbolTable, VariableClass};
/// Name resolution pass (#276): walks the AST and resolves names to SymbolIds.
///
/// Populates a SymbolTable with all variable, function, class, and parameter bindings.
/// Reports unresolved names as errors.
use crate::error::MambaError;
use crate::parser::ast::*;
use crate::source::span::{Span, Spanned};

/// Result of name resolution: the populated symbol table.
pub struct ResolveResult {
    pub symbols: SymbolTable,
    pub errors: Vec<MambaError>,
    /// Map from AST name occurrences to resolved SymbolIds.
    pub name_map: Vec<(Span, SymbolId)>,
}

/// Resolve all names in a module.
pub fn resolve_module(module: &Module) -> ResolveResult {
    let mut resolver = Resolver::new();
    // First pass: register top-level names
    resolver.register_top_level(module);
    // Second pass: resolve bodies
    for stmt in &module.stmts {
        resolver.resolve_stmt(stmt);
    }
    ResolveResult {
        symbols: resolver.symbols,
        errors: resolver.errors,
        name_map: resolver.name_map,
    }
}

struct Resolver {
    symbols: SymbolTable,
    errors: Vec<MambaError>,
    name_map: Vec<(Span, SymbolId)>,
    /// Depth of comprehension nesting (for walrus scope fix, PEP 572).
    comprehension_depth: usize,
    /// Scope indices representing function scope boundaries (for walrus target placement).
    function_scope_stack: Vec<usize>,
}

impl Resolver {
    fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            errors: Vec::new(),
            name_map: Vec::new(),
            comprehension_depth: 0,
            function_scope_stack: vec![0], // global scope as default
        }
    }

    fn register_top_level(&mut self, module: &Module) {
        self.register_defs_in(&module.stmts);
    }

    /// Pre-register def/class/enum names, descending into compound-statement
    /// bodies (try/if/while/for/with) — a class defined inside a module-level
    /// `try:` is still a module-scope binding. Function bodies are NOT
    /// descended into (their defs are their own scope).
    fn register_defs_in(&mut self, stmts: &[Spanned<Stmt>]) {
        for stmt in stmts {
            match &stmt.node {
                Stmt::FnDef { name, .. } | Stmt::AsyncFnDef { name, .. } => {
                    let id = self.symbols.define(name.clone(), SymbolKind::Function);
                    self.name_map.push((stmt.span, id));
                }
                Stmt::ClassDef { name, .. } => {
                    let id = self.symbols.define(name.clone(), SymbolKind::Class);
                    self.name_map.push((stmt.span, id));
                }
                Stmt::EnumDef { name, .. } => {
                    let id = self.symbols.define(name.clone(), SymbolKind::Enum);
                    self.name_map.push((stmt.span, id));
                }
                Stmt::ExprStmt(_) => {
                    if let Some(fn_def) =
                        crate::exec_literal::global_literal_exec_fn_def(&stmt.node)
                    {
                        let id = self.symbols.define(fn_def.name, SymbolKind::Function);
                        self.name_map.push((stmt.span, id));
                    }
                }
                Stmt::Try {
                    body,
                    handlers,
                    else_body,
                    finally_body,
                } => {
                    self.register_defs_in(body);
                    for h in handlers {
                        self.register_defs_in(&h.body);
                    }
                    if let Some(eb) = else_body {
                        self.register_defs_in(eb);
                    }
                    if let Some(fb) = finally_body {
                        self.register_defs_in(fb);
                    }
                }
                Stmt::If {
                    body,
                    elif_clauses,
                    else_body,
                    ..
                } => {
                    self.register_defs_in(body);
                    for (_, eb) in elif_clauses {
                        self.register_defs_in(eb);
                    }
                    if let Some(eb) = else_body {
                        self.register_defs_in(eb);
                    }
                }
                Stmt::While {
                    body, else_body, ..
                }
                | Stmt::For {
                    body, else_body, ..
                } => {
                    self.register_defs_in(body);
                    if let Some(eb) = else_body {
                        self.register_defs_in(eb);
                    }
                }
                Stmt::With { body, .. } => {
                    self.register_defs_in(body);
                }
                _ => {}
            }
        }
    }

    fn resolve_stmt(&mut self, stmt: &Spanned<Stmt>) {
        match &stmt.node {
            Stmt::VarDecl { name, value, .. } => {
                self.resolve_expr(value);
                let id = self.symbols.define(name.clone(), SymbolKind::Variable);
                self.name_map.push((stmt.span, id));
            }
            Stmt::Assign { target, value, .. } => {
                // Implicit variable declaration: bare `x = val` where x is
                // not yet in scope defines x as a new variable.
                if let Expr::Ident(name) = &target.node {
                    if self.symbols.lookup(name).is_none() {
                        self.resolve_expr(value);
                        let id = self.symbols.define(name.clone(), SymbolKind::Variable);
                        self.name_map.push((stmt.span, id));
                        return;
                    }
                }
                self.resolve_expr(target);
                self.resolve_expr(value);
            }
            Stmt::AugAssign { target, value, .. } => {
                self.resolve_expr(target);
                self.resolve_expr(value);
            }
            Stmt::FnDef { params, body, .. } | Stmt::AsyncFnDef { params, body, .. } => {
                self.symbols.push_scope();
                let func_scope = self.symbols.current_scope_idx();
                self.function_scope_stack.push(func_scope);
                for param in params {
                    let id = self
                        .symbols
                        .define(param.name.clone(), SymbolKind::Parameter);
                    self.name_map.push((param.span, id));
                }
                // Python: any name assigned anywhere in a function body is
                // local (unless declared `global` / `nonlocal`). Pre-scan the
                // body so subsequent `lookup()` calls find the local binding
                // rather than walking up to an outer-scope symbol.
                let mut assigned: Vec<String> = Vec::new();
                let mut declared: Vec<String> = Vec::new();
                collect_assignment_targets(body, &mut assigned, &mut declared);
                collect_walrus_targets_in_stmts(body, &mut assigned);
                for name in assigned {
                    if declared.iter().any(|n: &String| n == &name) {
                        continue;
                    }
                    if self.symbols.lookup_in_scope(func_scope, &name).is_some() {
                        continue;
                    }
                    let id = self.symbols.define(name.clone(), SymbolKind::Variable);
                    self.name_map.push((stmt.span, id));
                }
                for s in body {
                    self.resolve_stmt(s);
                }
                self.function_scope_stack.pop();
                self.symbols.pop_scope();
            }
            Stmt::ClassDef { body, .. } => {
                self.symbols.push_scope();
                for s in body {
                    self.resolve_stmt(s);
                }
                self.symbols.pop_scope();
            }
            Stmt::If {
                condition,
                body,
                elif_clauses,
                else_body,
                ..
            } => {
                self.resolve_expr(condition);
                for s in body {
                    self.resolve_stmt(s);
                }
                for (cond, elif_body) in elif_clauses {
                    self.resolve_expr(cond);
                    for s in elif_body {
                        self.resolve_stmt(s);
                    }
                }
                if let Some(eb) = else_body {
                    for s in eb {
                        self.resolve_stmt(s);
                    }
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.resolve_expr(condition);
                for s in body {
                    self.resolve_stmt(s);
                }
            }
            Stmt::For {
                targets,
                iter,
                body,
                ..
            }
            | Stmt::AsyncFor {
                targets,
                iter,
                body,
                ..
            } => {
                self.resolve_expr(iter);
                for var in targets {
                    let id = self.symbols.define(var.clone(), SymbolKind::Variable);
                    self.name_map.push((stmt.span, id));
                }
                for s in body {
                    self.resolve_stmt(s);
                }
            }
            Stmt::Return(Some(expr)) => {
                self.resolve_expr(expr);
            }
            Stmt::Return(None) | Stmt::Pass | Stmt::Break | Stmt::Continue => {}
            Stmt::ExprStmt(expr) => {
                self.resolve_expr(expr);
            }
            Stmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
            } => {
                for s in body {
                    self.resolve_stmt(s);
                }
                for handler in handlers {
                    if let Some(name) = &handler.name {
                        let id = self.symbols.define(name.clone(), SymbolKind::Variable);
                        self.name_map.push((handler.span, id));
                    }
                    for s in &handler.body {
                        self.resolve_stmt(s);
                    }
                }
                if let Some(eb) = else_body {
                    for s in eb {
                        self.resolve_stmt(s);
                    }
                }
                if let Some(fb) = finally_body {
                    for s in fb {
                        self.resolve_stmt(s);
                    }
                }
            }
            Stmt::Raise { value, from } => {
                if let Some(v) = value {
                    self.resolve_expr(v);
                }
                if let Some(f) = from {
                    self.resolve_expr(f);
                }
            }
            Stmt::With { items, body, .. } | Stmt::AsyncWith { items, body, .. } => {
                for item in items {
                    self.resolve_expr(&item.context);
                    if let Some(alias) = &item.alias {
                        let id = self.symbols.define(alias.clone(), SymbolKind::Variable);
                        self.name_map.push((stmt.span, id));
                    }
                }
                for s in body {
                    self.resolve_stmt(s);
                }
            }
            Stmt::Assert { test, msg } => {
                self.resolve_expr(test);
                if let Some(m) = msg {
                    self.resolve_expr(m);
                }
            }
            Stmt::Del(expr) => {
                self.resolve_expr(expr);
            }
            Stmt::Match { expr, arms } => {
                self.resolve_expr(expr);
                for arm in arms {
                    self.resolve_pattern(&arm.pattern);
                    if let Some(guard) = &arm.guard {
                        self.resolve_expr(guard);
                    }
                    for s in &arm.body {
                        self.resolve_stmt(s);
                    }
                }
            }
            Stmt::EnumDef { variants, .. } => {
                for variant in variants {
                    let id = self
                        .symbols
                        .define(variant.name.clone(), SymbolKind::EnumVariant);
                    self.name_map.push((variant.span, id));
                }
            }
            Stmt::Global(names) => {
                for name in names {
                    // Define or look up the symbol, then classify as Global
                    let id = if let Some(existing) = self.symbols.lookup(name) {
                        existing
                    } else {
                        self.symbols.define(name.clone(), SymbolKind::Variable)
                    };
                    self.symbols.set_var_class(id, VariableClass::Global);
                    self.name_map.push((stmt.span, id));
                }
            }
            Stmt::Nonlocal(names) => {
                // Walk enclosing function scopes to find the binding. Python
                // `nonlocal` cannot bind module globals, and class scopes do
                // not count as enclosing function scopes.
                for name in names {
                    let current = self.symbols.current_scope_idx();
                    let mut found = false;
                    let mut scope_idx = self.symbols.parent_scope(current);
                    while let Some(si) = scope_idx {
                        if si == 0 {
                            break;
                        }
                        if !self.function_scope_stack.contains(&si) {
                            scope_idx = self.symbols.parent_scope(si);
                            continue;
                        }
                        if let Some(outer_id) = self.symbols.lookup_in_scope(si, name) {
                            // Mark the outer variable as Cell (captured by inner)
                            self.symbols.set_var_class(outer_id, VariableClass::Cell);
                            // Define or re-use in current scope as Free
                            let inner_id = if let Some(existing) =
                                self.symbols.lookup_in_scope(current, name)
                            {
                                existing
                            } else {
                                self.symbols.define(name.clone(), SymbolKind::Variable)
                            };
                            self.symbols.set_var_class(inner_id, VariableClass::Free);
                            self.symbols.set_nonlocal_mapping(inner_id, outer_id);
                            self.name_map.push((stmt.span, inner_id));
                            found = true;
                            break;
                        }
                        scope_idx = self.symbols.parent_scope(si);
                    }
                    if !found {
                        self.errors.push(MambaError::name(
                            stmt.span,
                            format!("no binding for nonlocal `{name}` found"),
                        ));
                    }
                }
            }
            // Bare annotation `x: int` — record the name in scope (no value to resolve).
            Stmt::BareAnnotation { name, .. } => {
                let id = self.symbols.define(name.clone(), SymbolKind::Variable);
                self.name_map.push((stmt.span, id));
            }
            Stmt::Import {
                module,
                names,
                module_alias,
            } => {
                // R2 (#1132): Define imported names in the symbol table so that
                // subsequent references resolve to valid SymbolIds.
                if let Some(names) = names {
                    // `from X import Y, Z as W` → define Y, W as Variables
                    // @spec .aw/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#R5
                    // Skip `from X import *` — star imports bind names dynamically
                    // at runtime, not statically at resolve time.
                    for (name, alias) in names {
                        if name == "*" {
                            continue;
                        }
                        let bound = alias.as_deref().unwrap_or(name.as_str());
                        let id = self.symbols.define(bound.to_string(), SymbolKind::Variable);
                        self.name_map.push((stmt.span, id));
                    }
                } else {
                    // `import X` or `import X as alias`
                    let bound = if let Some(alias) = module_alias {
                        alias.clone()
                    } else {
                        // For `import os.path`, Python binds `os` (first component)
                        module.first().cloned().unwrap_or_default()
                    };
                    if !bound.is_empty() {
                        let id = self.symbols.define(bound, SymbolKind::Variable);
                        self.name_map.push((stmt.span, id));
                    }
                }
            }
            Stmt::TypeAlias { .. } => {}
        }
    }

    fn resolve_expr(&mut self, expr: &Spanned<Expr>) {
        match &expr.node {
            Expr::Ident(name) => {
                if let Some(id) = self.symbols.lookup(name) {
                    self.name_map.push((expr.span, id));
                } else {
                    self.errors.push(MambaError::name(
                        expr.span,
                        format!("undefined name `{name}`"),
                    ));
                }
            }
            Expr::BinOp { lhs, rhs, .. } => {
                self.resolve_expr(lhs);
                self.resolve_expr(rhs);
            }
            Expr::UnaryOp { operand, .. } => {
                self.resolve_expr(operand);
            }
            Expr::Call { func, args } => {
                self.resolve_expr(func);
                for arg in args {
                    match arg {
                        CallArg::Positional(e)
                        | CallArg::StarArg(e)
                        | CallArg::DoubleStarArg(e) => self.resolve_expr(e),
                        CallArg::Keyword { value, .. } => self.resolve_expr(value),
                    }
                }
            }
            Expr::Attr { object, .. } => {
                self.resolve_expr(object);
            }
            Expr::Index { object, index } => {
                self.resolve_expr(object);
                self.resolve_expr(index);
            }
            Expr::ListLit(elems) | Expr::SetLit(elems) | Expr::TupleLit(elems) => {
                for e in elems {
                    self.resolve_expr(e);
                }
            }
            Expr::DictLit(entries) => {
                for (k, v) in entries {
                    if let Some(key) = k {
                        self.resolve_expr(key);
                    }
                    self.resolve_expr(v);
                }
            }
            Expr::IfExpr {
                body,
                condition,
                else_body,
            } => {
                self.resolve_expr(condition);
                self.resolve_expr(body);
                self.resolve_expr(else_body);
            }
            Expr::Lambda { params, body } => {
                self.symbols.push_scope();
                let lambda_scope = self.symbols.current_scope_idx();
                self.function_scope_stack.push(lambda_scope);
                for p in params {
                    let id = self.symbols.define(p.name.clone(), SymbolKind::Parameter);
                    self.name_map.push((p.span, id));
                }
                self.resolve_expr(body);
                self.function_scope_stack.pop();
                self.symbols.pop_scope();
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
                self.comprehension_depth += 1;
                self.symbols.push_scope();
                for gen in generators {
                    self.resolve_expr(&gen.iter);
                    for name in &gen.targets {
                        let id = self.symbols.define(name.clone(), SymbolKind::Variable);
                        self.name_map.push((gen.iter.span, id));
                    }
                    for cond in &gen.conditions {
                        self.resolve_expr(cond);
                    }
                }
                self.resolve_expr(element);
                self.symbols.pop_scope();
                self.comprehension_depth -= 1;
            }
            Expr::DictComp {
                key,
                value,
                generators,
            } => {
                self.comprehension_depth += 1;
                self.symbols.push_scope();
                for gen in generators {
                    self.resolve_expr(&gen.iter);
                    for name in &gen.targets {
                        let id = self.symbols.define(name.clone(), SymbolKind::Variable);
                        self.name_map.push((gen.iter.span, id));
                    }
                    for cond in &gen.conditions {
                        self.resolve_expr(cond);
                    }
                }
                self.resolve_expr(key);
                self.resolve_expr(value);
                self.symbols.pop_scope();
                self.comprehension_depth -= 1;
            }
            Expr::Walrus { target, value } => {
                self.resolve_expr(value);
                let id = if self.comprehension_depth > 0 {
                    // PEP 572: bind walrus target in enclosing function scope,
                    // not the comprehension's inner scope.
                    let func_scope = *self.function_scope_stack.last().unwrap_or(&0);
                    self.symbols
                        .define_in_scope(func_scope, target.clone(), SymbolKind::Variable)
                } else {
                    self.symbols.define(target.clone(), SymbolKind::Variable)
                };
                self.name_map.push((expr.span, id));
            }
            Expr::Slice { start, stop, step } => {
                if let Some(s) = start {
                    self.resolve_expr(s);
                }
                if let Some(s) = stop {
                    self.resolve_expr(s);
                }
                if let Some(s) = step {
                    self.resolve_expr(s);
                }
            }
            Expr::FString(parts) => {
                for part in parts {
                    if let FStringPart::Expr(e, spec) = part {
                        self.resolve_expr(e);
                        if let Some(sp) = spec {
                            fn walk_spec(this: &mut Resolver, parts: &[FStringPart]) {
                                for p in parts {
                                    if let FStringPart::Expr(e, spec) = p {
                                        this.resolve_expr(e);
                                        if let Some(sp) = spec {
                                            walk_spec(this, sp);
                                        }
                                    }
                                }
                            }
                            walk_spec(self, sp);
                        }
                    }
                }
            }
            Expr::Yield(Some(e)) | Expr::YieldFrom(e) | Expr::Await(e) | Expr::Starred(e) => {
                self.resolve_expr(e);
            }
            Expr::Yield(None) => {}
            Expr::UnpackTarget(elems) => {
                for e in elems {
                    self.resolve_expr(e);
                }
            }
            Expr::ChainedCompare { operands, .. } => {
                for e in operands {
                    self.resolve_expr(e);
                }
            }
            Expr::IntLit(_)
            | Expr::BigIntLit(_)
            | Expr::FloatLit(_)
            | Expr::ComplexLit(_)
            | Expr::StrLit(_)
            | Expr::BytesLit(_)
            | Expr::BoolLit(_)
            | Expr::NoneLit
            | Expr::Ellipsis => {}
        }
    }

    fn resolve_pattern(&mut self, pattern: &Spanned<Pattern>) {
        match &pattern.node {
            Pattern::Binding(name) => {
                let id = self.symbols.define(name.clone(), SymbolKind::Variable);
                self.name_map.push((pattern.span, id));
            }
            Pattern::Or(pats) | Pattern::Sequence(pats) => {
                for p in pats {
                    self.resolve_pattern(p);
                }
            }
            Pattern::Constructor { fields, .. } => {
                for name in fields {
                    let id = self.symbols.define(name.clone(), SymbolKind::Variable);
                    self.name_map.push((pattern.span, id));
                }
            }
            Pattern::Mapping { pairs, rest } => {
                for (k, v) in pairs {
                    self.resolve_expr(k);
                    self.resolve_pattern(v);
                }
                // Register rest-capture variable if present (#827)
                if let Some(r) = rest {
                    let id = self.symbols.define(r.clone(), SymbolKind::Variable);
                    self.name_map.push((pattern.span, id));
                }
            }
            Pattern::ClassPattern { patterns, .. } => {
                for (_, p) in patterns {
                    self.resolve_pattern(p);
                }
            }
            Pattern::Star(Some(name)) => {
                let id = self.symbols.define(name.clone(), SymbolKind::Variable);
                self.name_map.push((pattern.span, id));
            }
            Pattern::As {
                pattern: inner,
                name,
            } => {
                // Resolve the inner pattern, then register the AS binding (#827)
                self.resolve_pattern(inner);
                let id = self.symbols.define(name.clone(), SymbolKind::Variable);
                self.name_map.push((pattern.span, id));
            }
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::Star(None) => {}
        }
    }
}

/// Pre-scan a function body for names that appear as assignment targets, so
/// the resolver can define them as local BEFORE walking the statements. Needed
/// because Python scopes on assignment (any name assigned in a function body
/// is local unless declared `global` / `nonlocal`), while a naive lookup()
/// would find an outer-scope symbol and rebind that instead.
///
/// Does NOT descend into nested FnDef / AsyncFnDef / ClassDef bodies — those
/// carry their own scope. `declared` collects names shadowed by `global` /
/// `nonlocal` so the caller can skip pre-defining them.
pub fn collect_assignment_targets(
    stmts: &[Spanned<Stmt>],
    assigned: &mut Vec<String>,
    declared: &mut Vec<String>,
) {
    for stmt in stmts {
        match &stmt.node {
            Stmt::VarDecl { name, .. } => {
                if !assigned.iter().any(|n| n == name) {
                    assigned.push(name.clone());
                }
            }
            Stmt::Assign { target, .. } | Stmt::AugAssign { target, .. } => {
                collect_expr_targets(&target.node, assigned);
            }
            Stmt::For {
                targets,
                body,
                else_body,
                ..
            }
            | Stmt::AsyncFor {
                targets,
                body,
                else_body,
                ..
            } => {
                for t in targets {
                    if !assigned.iter().any(|n| n == t) {
                        assigned.push(t.clone());
                    }
                }
                collect_assignment_targets(body, assigned, declared);
                if let Some(eb) = else_body {
                    collect_assignment_targets(eb, assigned, declared);
                }
            }
            Stmt::With { items, body } | Stmt::AsyncWith { items, body } => {
                for item in items {
                    if let Some(name) = &item.alias {
                        if !assigned.iter().any(|n| n == name) {
                            assigned.push(name.clone());
                        }
                    }
                }
                collect_assignment_targets(body, assigned, declared);
            }
            Stmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
            } => {
                collect_assignment_targets(body, assigned, declared);
                for h in handlers {
                    if let Some(name) = &h.name {
                        if !assigned.iter().any(|n| n == name) {
                            assigned.push(name.clone());
                        }
                    }
                    collect_assignment_targets(&h.body, assigned, declared);
                }
                if let Some(eb) = else_body {
                    collect_assignment_targets(eb, assigned, declared);
                }
                if let Some(fb) = finally_body {
                    collect_assignment_targets(fb, assigned, declared);
                }
            }
            Stmt::If {
                body,
                elif_clauses,
                else_body,
                ..
            } => {
                collect_assignment_targets(body, assigned, declared);
                for (_, b) in elif_clauses {
                    collect_assignment_targets(b, assigned, declared);
                }
                if let Some(eb) = else_body {
                    collect_assignment_targets(eb, assigned, declared);
                }
            }
            Stmt::While {
                body, else_body, ..
            } => {
                collect_assignment_targets(body, assigned, declared);
                if let Some(eb) = else_body {
                    collect_assignment_targets(eb, assigned, declared);
                }
            }
            Stmt::Match { arms, .. } => {
                for arm in arms {
                    collect_assignment_targets(&arm.body, assigned, declared);
                }
            }
            Stmt::Global(names) | Stmt::Nonlocal(names) => {
                for n in names {
                    declared.push(n.clone());
                }
            }
            Stmt::Import {
                names: Some(names),
                module,
                module_alias,
            } => {
                for (n, alias) in names {
                    let binding = alias.clone().unwrap_or_else(|| n.clone());
                    if !assigned.iter().any(|x| x == &binding) {
                        assigned.push(binding);
                    }
                }
                // `import a.b as c` binds `c`; `import a` binds `a`.
                if names.is_empty() {
                    if let Some(alias) = module_alias {
                        if !assigned.iter().any(|x| x == alias) {
                            assigned.push(alias.clone());
                        }
                    } else if let Some(first) = module.first() {
                        if !assigned.iter().any(|x| x == first) {
                            assigned.push(first.clone());
                        }
                    }
                }
            }
            Stmt::Import {
                module,
                module_alias,
                names: None,
            } => {
                let binding = module_alias.clone().or_else(|| module.first().cloned());
                if let Some(b) = binding {
                    if !assigned.iter().any(|x| x == &b) {
                        assigned.push(b);
                    }
                }
            }
            // Nested function / class definitions bind the name but do NOT
            // contribute their body's assignments to the outer scope.
            Stmt::FnDef { name, .. }
            | Stmt::AsyncFnDef { name, .. }
            | Stmt::ClassDef { name, .. }
            | Stmt::EnumDef { name, .. } => {
                if !assigned.iter().any(|n| n == name) {
                    assigned.push(name.clone());
                }
            }
            _ => {}
        }
    }
}

/// Recursive expression walk for identifier-targets inside tuple / unpack
/// assignments.
fn collect_expr_targets(expr: &Expr, assigned: &mut Vec<String>) {
    match expr {
        Expr::Ident(name) => {
            if !assigned.iter().any(|n| n == name) {
                assigned.push(name.clone());
            }
        }
        Expr::TupleLit(elems) | Expr::UnpackTarget(elems) => {
            for e in elems {
                collect_expr_targets(&e.node, assigned);
            }
        }
        Expr::Starred(inner) => {
            collect_expr_targets(&inner.node, assigned);
        }
        _ => {}
    }
}

/// Scan an expression for walrus targets. Walrus `x := expr` names are
/// locals of the enclosing function (PEP 572), so the assignment-target
/// pre-scan must visit every expression position in the body — including
/// conditions, return values, comprehensions — not just Assign statement
/// targets.
fn collect_walrus_targets(expr: &Expr, assigned: &mut Vec<String>) {
    match expr {
        Expr::Walrus { target, value } => {
            if !assigned.iter().any(|n| n == target) {
                assigned.push(target.clone());
            }
            collect_walrus_targets(&value.node, assigned);
        }
        Expr::BinOp { lhs, rhs, .. } => {
            collect_walrus_targets(&lhs.node, assigned);
            collect_walrus_targets(&rhs.node, assigned);
        }
        Expr::UnaryOp { operand, .. } => collect_walrus_targets(&operand.node, assigned),
        Expr::Call { func, args } => {
            collect_walrus_targets(&func.node, assigned);
            for arg in args {
                match arg {
                    CallArg::Positional(e) | CallArg::StarArg(e) | CallArg::DoubleStarArg(e) => {
                        collect_walrus_targets(&e.node, assigned)
                    }
                    CallArg::Keyword { value, .. } => collect_walrus_targets(&value.node, assigned),
                }
            }
        }
        Expr::Attr { object, .. } => collect_walrus_targets(&object.node, assigned),
        Expr::Index { object, index } => {
            collect_walrus_targets(&object.node, assigned);
            collect_walrus_targets(&index.node, assigned);
        }
        Expr::Slice { start, stop, step } => {
            if let Some(s) = start {
                collect_walrus_targets(&s.node, assigned);
            }
            if let Some(s) = stop {
                collect_walrus_targets(&s.node, assigned);
            }
            if let Some(s) = step {
                collect_walrus_targets(&s.node, assigned);
            }
        }
        Expr::TupleLit(elems) | Expr::ListLit(elems) | Expr::SetLit(elems) => {
            for e in elems {
                collect_walrus_targets(&e.node, assigned);
            }
        }
        Expr::IfExpr {
            body,
            condition,
            else_body,
        } => {
            collect_walrus_targets(&condition.node, assigned);
            collect_walrus_targets(&body.node, assigned);
            collect_walrus_targets(&else_body.node, assigned);
        }
        Expr::ChainedCompare { operands, .. } => {
            for o in operands {
                collect_walrus_targets(&o.node, assigned);
            }
        }
        Expr::Starred(inner) => collect_walrus_targets(&inner.node, assigned),
        _ => {}
    }
}

/// Walk every statement body and descend into expression positions to
/// collect walrus targets. Mirrors collect_assignment_targets but only
/// chases walrus, since walrus targets need a *separate* local-define
/// pass (they can appear inside conditions that aren't assignment stmts).
pub fn collect_walrus_targets_in_stmts(stmts: &[Spanned<Stmt>], assigned: &mut Vec<String>) {
    for stmt in stmts {
        match &stmt.node {
            Stmt::Assign { target, value } => {
                collect_walrus_targets(&target.node, assigned);
                collect_walrus_targets(&value.node, assigned);
            }
            Stmt::AugAssign { target, value, .. } => {
                collect_walrus_targets(&target.node, assigned);
                collect_walrus_targets(&value.node, assigned);
            }
            Stmt::VarDecl { value, .. } => collect_walrus_targets(&value.node, assigned),
            Stmt::ExprStmt(e) => collect_walrus_targets(&e.node, assigned),
            Stmt::Return(Some(e)) => collect_walrus_targets(&e.node, assigned),
            Stmt::If {
                condition,
                body,
                elif_clauses,
                else_body,
            } => {
                collect_walrus_targets(&condition.node, assigned);
                collect_walrus_targets_in_stmts(body, assigned);
                for (c, b) in elif_clauses {
                    collect_walrus_targets(&c.node, assigned);
                    collect_walrus_targets_in_stmts(b, assigned);
                }
                if let Some(eb) = else_body {
                    collect_walrus_targets_in_stmts(eb, assigned);
                }
            }
            Stmt::While {
                condition,
                body,
                else_body,
            } => {
                collect_walrus_targets(&condition.node, assigned);
                collect_walrus_targets_in_stmts(body, assigned);
                if let Some(eb) = else_body {
                    collect_walrus_targets_in_stmts(eb, assigned);
                }
            }
            Stmt::For {
                iter,
                body,
                else_body,
                ..
            }
            | Stmt::AsyncFor {
                iter,
                body,
                else_body,
                ..
            } => {
                collect_walrus_targets(&iter.node, assigned);
                collect_walrus_targets_in_stmts(body, assigned);
                if let Some(eb) = else_body {
                    collect_walrus_targets_in_stmts(eb, assigned);
                }
            }
            Stmt::With { items, body } | Stmt::AsyncWith { items, body } => {
                for item in items {
                    collect_walrus_targets(&item.context.node, assigned);
                }
                collect_walrus_targets_in_stmts(body, assigned);
            }
            Stmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
            } => {
                collect_walrus_targets_in_stmts(body, assigned);
                for h in handlers {
                    if let Some(e) = &h.exc_type {
                        collect_walrus_targets(&e.node, assigned);
                    }
                    collect_walrus_targets_in_stmts(&h.body, assigned);
                }
                if let Some(eb) = else_body {
                    collect_walrus_targets_in_stmts(eb, assigned);
                }
                if let Some(fb) = finally_body {
                    collect_walrus_targets_in_stmts(fb, assigned);
                }
            }
            Stmt::Match { expr, arms } => {
                collect_walrus_targets(&expr.node, assigned);
                for arm in arms {
                    collect_walrus_targets_in_stmts(&arm.body, assigned);
                }
            }
            Stmt::Assert { test, msg } => {
                collect_walrus_targets(&test.node, assigned);
                if let Some(m) = msg {
                    collect_walrus_targets(&m.node, assigned);
                }
            }
            // Nested function/class definitions carry their own scope; their
            // walrus targets belong to them, not to us.
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::span::Span;

    fn sp<T>(node: T) -> Spanned<T> {
        Spanned::new(node, Span::dummy())
    }

    #[test]
    fn test_resolve_simple_function() {
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "add".into(),
                type_params: vec![],
                params: vec![
                    Param {
                        name: "a".into(),
                        ty: sp(TypeExpr::Named("int".into())),
                        default: None,
                        kind: ParamKind::Regular,
                        pos_only: false,
                        kw_only: false,
                        span: Span::dummy(),
                    },
                    Param {
                        name: "b".into(),
                        ty: sp(TypeExpr::Named("int".into())),
                        default: None,
                        kind: ParamKind::Regular,
                        pos_only: false,
                        kw_only: false,
                        span: Span::dummy(),
                    },
                ],
                return_ty: Some(sp(TypeExpr::Named("int".into()))),
                body: vec![sp(Stmt::Return(Some(sp(Expr::BinOp {
                    op: BinOp::Add,
                    lhs: Box::new(sp(Expr::Ident("a".into()))),
                    rhs: Box::new(sp(Expr::Ident("b".into()))),
                }))))],
            })],
        };

        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        // Should have: "add" (function), "a" (param), "b" (param), plus name refs
        assert!(result.name_map.len() >= 3);
    }

    #[test]
    fn test_resolve_undefined_name() {
        let module = Module {
            stmts: vec![sp(Stmt::ExprStmt(sp(Expr::Ident("undefined_var".into()))))],
        };

        let result = resolve_module(&module);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_resolve_variable_declaration() {
        let module = Module {
            stmts: vec![
                sp(Stmt::VarDecl {
                    name: "x".into(),
                    ty: sp(TypeExpr::Named("int".into())),
                    value: sp(Expr::IntLit(42)),
                }),
                sp(Stmt::ExprStmt(sp(Expr::Ident("x".into())))),
            ],
        };

        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_resolve_global_declaration() {
        // def f(): global x; x = 42
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "f".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![
                    sp(Stmt::Global(vec!["x".into()])),
                    sp(Stmt::Assign {
                        target: sp(Expr::Ident("x".into())),
                        value: sp(Expr::IntLit(42)),
                    }),
                ],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        // Find the SymbolId for x in name_map (global decl produces a mapping)
        let x_id = result
            .name_map
            .iter()
            .map(|(_, id)| *id)
            .find(|id| result.symbols.get_symbol(*id).name == "x")
            .expect("x should have a name_map entry");
        assert_eq!(result.symbols.get_var_class(x_id), VariableClass::Global,);
    }

    #[test]
    fn test_resolve_nonlocal_declaration() {
        // def outer(): x = 1; def inner(): nonlocal x; x = 2
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "outer".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![
                    sp(Stmt::Assign {
                        target: sp(Expr::Ident("x".into())),
                        value: sp(Expr::IntLit(1)),
                    }),
                    sp(Stmt::FnDef {
                        decorators: vec![],
                        name: "inner".into(),
                        type_params: vec![],
                        params: vec![],
                        return_ty: None,
                        body: vec![
                            sp(Stmt::Nonlocal(vec!["x".into()])),
                            sp(Stmt::Assign {
                                target: sp(Expr::Ident("x".into())),
                                value: sp(Expr::IntLit(2)),
                            }),
                        ],
                    }),
                ],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_resolve_nonlocal_not_found_error() {
        // def f(): nonlocal z  (z not in any enclosing scope)
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "f".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::Nonlocal(vec!["z".into()]))],
            })],
        };
        let result = resolve_module(&module);
        assert_eq!(result.errors.len(), 1);
        assert!(
            result.errors[0].to_string().contains("nonlocal"),
            "error should mention nonlocal: {:?}",
            result.errors[0]
        );
    }

    // ── Group 1: Module-level name registration ────────────────────────────

    #[test]
    fn test_top_level_variable_registered() {
        // x = 42  →  lookup("x") is Some
        let module = Module {
            stmts: vec![sp(Stmt::Assign {
                target: sp(Expr::Ident("x".into())),
                value: sp(Expr::IntLit(42)),
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert!(
            result.symbols.lookup("x").is_some(),
            "x should be registered"
        );
    }

    #[test]
    fn test_top_level_function_registered() {
        // def f(): pass  →  lookup("f") has kind=Function
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "f".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::Pass)],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let id = result.symbols.lookup("f").expect("f should be registered");
        assert_eq!(result.symbols.get_symbol(id).kind, SymbolKind::Function);
    }

    #[test]
    fn test_top_level_class_registered() {
        // class C: pass  →  lookup("C") has kind=Class
        let module = Module {
            stmts: vec![sp(Stmt::ClassDef {
                decorators: vec![],
                name: "C".into(),
                type_params: vec![],
                bases: vec![],
                keyword_args: vec![],
                body: vec![sp(Stmt::Pass)],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let id = result.symbols.lookup("C").expect("C should be registered");
        assert_eq!(result.symbols.get_symbol(id).kind, SymbolKind::Class);
    }

    #[test]
    fn test_multiple_top_level_defs() {
        // def f(): pass; def g(): pass  →  both registered
        let module = Module {
            stmts: vec![
                sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "f".into(),
                    type_params: vec![],
                    params: vec![],
                    return_ty: None,
                    body: vec![sp(Stmt::Pass)],
                }),
                sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "g".into(),
                    type_params: vec![],
                    params: vec![],
                    return_ty: None,
                    body: vec![sp(Stmt::Pass)],
                }),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert!(
            result.symbols.lookup("f").is_some(),
            "f should be registered"
        );
        assert!(
            result.symbols.lookup("g").is_some(),
            "g should be registered"
        );
    }

    #[test]
    fn test_top_level_import_registers_name() {
        // import os  →  "os" resolvable
        let module = Module {
            stmts: vec![sp(Stmt::Import {
                module: vec!["os".into()],
                names: None,
                module_alias: None,
            })],
        };
        // Per R2 (#1132), `import os` defines `os` as a Variable in the symbol table
        // so subsequent references (e.g. `os.path`) resolve to a valid SymbolId.
        let result = resolve_module(&module);
        assert!(
            result.errors.is_empty(),
            "import should not produce errors: {:?}",
            result.errors
        );
        assert!(
            result.symbols.lookup("os").is_some(),
            "import should define `os` in symbol table"
        );
    }

    #[test]
    fn test_from_import_with_alias_registers_bound_name() {
        // from os import path as p  →  "p" resolvable, "path" NOT resolvable (unbound alias source)
        let module = Module {
            stmts: vec![sp(Stmt::Import {
                module: vec!["os".into()],
                names: Some(vec![("path".into(), Some("p".into()))]),
                module_alias: None,
            })],
        };
        let result = resolve_module(&module);
        assert!(
            result.errors.is_empty(),
            "from-import-as should not error: {:?}",
            result.errors
        );
        assert!(
            result.symbols.lookup("p").is_some(),
            "alias `p` should be defined"
        );
        assert!(
            result.symbols.lookup("path").is_none(),
            "source name `path` should NOT be defined (only alias binds)"
        );
    }

    #[test]
    fn test_from_import_star_skips_symbol_definition() {
        // from os import *  →  no symbol defined (star imports are dynamic)
        let module = Module {
            stmts: vec![sp(Stmt::Import {
                module: vec!["os".into()],
                names: Some(vec![("*".into(), None)]),
                module_alias: None,
            })],
        };
        let result = resolve_module(&module);
        assert!(
            result.errors.is_empty(),
            "star-import should not error: {:?}",
            result.errors
        );
        assert!(
            result.symbols.lookup("*").is_none(),
            "star-import should not define `*`"
        );
    }

    // ── Group 2: LEGB resolution order ────────────────────────────────────

    #[test]
    fn test_local_shadows_outer() {
        // x = 1; def f(): x = 2; use x  →  inner x resolves locally, no error
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("x".into())),
                    value: sp(Expr::IntLit(1)),
                }),
                sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "f".into(),
                    type_params: vec![],
                    params: vec![],
                    return_ty: None,
                    body: vec![
                        sp(Stmt::Assign {
                            target: sp(Expr::Ident("x".into())),
                            value: sp(Expr::IntLit(2)),
                        }),
                        sp(Stmt::ExprStmt(sp(Expr::Ident("x".into())))),
                    ],
                }),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_use_before_assign_is_error() {
        // use y before assigning it at module scope → error
        let module = Module {
            stmts: vec![
                sp(Stmt::ExprStmt(sp(Expr::Ident("y".into())))),
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("y".into())),
                    value: sp(Expr::IntLit(0)),
                }),
            ],
        };
        let result = resolve_module(&module);
        // y is not yet defined when first used (top-level registration only hoists
        // functions/classes, not bare assignments).
        assert!(
            !result.errors.is_empty(),
            "should get an undefined-name error for y"
        );
    }

    #[test]
    fn test_builtin_name_resolvable() {
        // len, print, range are builtins — no errors when used without prior definition
        // The resolver currently only errors on names that are absent from the symbol table.
        // Builtins are not pre-populated, so using them produces errors; this test documents
        // that the resolver has a known limitation (no builtin pre-population).
        // We simply verify no panic occurs.
        let module = Module {
            stmts: vec![sp(Stmt::ExprStmt(sp(Expr::Call {
                func: Box::new(sp(Expr::Ident("len".into()))),
                args: vec![CallArg::Positional(sp(Expr::ListLit(vec![])))],
            })))],
        };
        let result = resolve_module(&module);
        // Either 0 errors (if builtins pre-populated) or 1 error (len unknown) — no panic.
        let _ = result.errors.len();
    }

    #[test]
    fn test_nested_function_sees_outer_var() {
        // outer x=1; def inner(): use x  →  x found in enclosing scope, no error
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "outer".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![
                    sp(Stmt::Assign {
                        target: sp(Expr::Ident("x".into())),
                        value: sp(Expr::IntLit(1)),
                    }),
                    sp(Stmt::FnDef {
                        decorators: vec![],
                        name: "inner".into(),
                        type_params: vec![],
                        params: vec![],
                        return_ty: None,
                        body: vec![sp(Stmt::ExprStmt(sp(Expr::Ident("x".into()))))],
                    }),
                ],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_nested_function_local_doesnt_leak() {
        // def outer(): def inner(): y = 1; use y  →  y is inside inner only
        // outer body uses y → error
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "outer".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![
                    sp(Stmt::FnDef {
                        decorators: vec![],
                        name: "inner".into(),
                        type_params: vec![],
                        params: vec![],
                        return_ty: None,
                        body: vec![sp(Stmt::Assign {
                            target: sp(Expr::Ident("y".into())),
                            value: sp(Expr::IntLit(1)),
                        })],
                    }),
                    sp(Stmt::ExprStmt(sp(Expr::Ident("y".into())))),
                ],
            })],
        };
        let result = resolve_module(&module);
        assert!(
            !result.errors.is_empty(),
            "y defined only inside inner should not be visible in outer"
        );
    }

    #[test]
    fn test_function_param_visible_in_body() {
        // def f(n: int): return n  →  n visible in body, no error
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "f".into(),
                type_params: vec![],
                params: vec![Param {
                    name: "n".into(),
                    ty: sp(TypeExpr::Named("int".into())),
                    default: None,
                    kind: ParamKind::Regular,
                    pos_only: false,
                    kw_only: false,
                    span: Span::dummy(),
                }],
                return_ty: None,
                body: vec![sp(Stmt::Return(Some(sp(Expr::Ident("n".into())))))],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    // ── Group 3: Global declarations ──────────────────────────────────────

    #[test]
    fn test_global_marks_variable() {
        // def f(): global x  →  x has var_class == Global
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "f".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::Global(vec!["x".into()]))],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let x_id = result
            .name_map
            .iter()
            .map(|(_, id)| *id)
            .find(|id| result.symbols.get_symbol(*id).name == "x")
            .expect("x should have a name_map entry");
        assert_eq!(result.symbols.get_var_class(x_id), VariableClass::Global);
    }

    #[test]
    fn test_global_allows_use_without_local_define() {
        // def f(): global x; x = 10  →  no error (x treated as module-level)
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "f".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![
                    sp(Stmt::Global(vec!["x".into()])),
                    sp(Stmt::Assign {
                        target: sp(Expr::Ident("x".into())),
                        value: sp(Expr::IntLit(10)),
                    }),
                ],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_multiple_globals() {
        // def f(): global x, y  →  both x and y marked Global
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "f".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::Global(vec!["x".into(), "y".into()]))],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let x_id = result
            .name_map
            .iter()
            .map(|(_, id)| *id)
            .find(|id| result.symbols.get_symbol(*id).name == "x")
            .expect("x should be in name_map");
        let y_id = result
            .name_map
            .iter()
            .map(|(_, id)| *id)
            .find(|id| result.symbols.get_symbol(*id).name == "y")
            .expect("y should be in name_map");
        assert_eq!(result.symbols.get_var_class(x_id), VariableClass::Global);
        assert_eq!(result.symbols.get_var_class(y_id), VariableClass::Global);
    }

    #[test]
    fn test_global_in_nested_function() {
        // def outer(): def inner(): global g  →  g is Global
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "outer".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "inner".into(),
                    type_params: vec![],
                    params: vec![],
                    return_ty: None,
                    body: vec![sp(Stmt::Global(vec!["g".into()]))],
                })],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let g_id = result
            .name_map
            .iter()
            .map(|(_, id)| *id)
            .find(|id| result.symbols.get_symbol(*id).name == "g")
            .expect("g should be in name_map");
        assert_eq!(result.symbols.get_var_class(g_id), VariableClass::Global);
    }

    // ── Group 4: Nonlocal declarations ────────────────────────────────────

    #[test]
    fn test_nonlocal_marks_variable() {
        // outer x=1; inner nonlocal x  →  inner x has var_class == Free
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "outer".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![
                    sp(Stmt::Assign {
                        target: sp(Expr::Ident("x".into())),
                        value: sp(Expr::IntLit(1)),
                    }),
                    sp(Stmt::FnDef {
                        decorators: vec![],
                        name: "inner".into(),
                        type_params: vec![],
                        params: vec![],
                        return_ty: None,
                        body: vec![sp(Stmt::Nonlocal(vec!["x".into()]))],
                    }),
                ],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        // The inner x should be classified as Free
        let free_x = result.name_map.iter().map(|(_, id)| *id).find(|id| {
            result.symbols.get_symbol(*id).name == "x"
                && result.symbols.get_var_class(*id) == VariableClass::Free
        });
        assert!(free_x.is_some(), "inner x should be classified as Free");
    }

    #[test]
    fn test_nonlocal_not_found_in_direct_outer() {
        // def outer(): def inner(): nonlocal w  (w not in outer either)
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "outer".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "inner".into(),
                    type_params: vec![],
                    params: vec![],
                    return_ty: None,
                    body: vec![sp(Stmt::Nonlocal(vec!["w".into()]))],
                })],
            })],
        };
        let result = resolve_module(&module);
        assert_eq!(
            result.errors.len(),
            1,
            "should have exactly one nonlocal-not-found error"
        );
    }

    #[test]
    fn test_nonlocal_in_deeply_nested() {
        // 3-level nesting: outermost defines x, middle does nothing, innermost nonlocal x
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "level1".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![
                    sp(Stmt::Assign {
                        target: sp(Expr::Ident("x".into())),
                        value: sp(Expr::IntLit(1)),
                    }),
                    sp(Stmt::FnDef {
                        decorators: vec![],
                        name: "level2".into(),
                        type_params: vec![],
                        params: vec![],
                        return_ty: None,
                        body: vec![sp(Stmt::FnDef {
                            decorators: vec![],
                            name: "level3".into(),
                            type_params: vec![],
                            params: vec![],
                            return_ty: None,
                            body: vec![sp(Stmt::Nonlocal(vec!["x".into()]))],
                        })],
                    }),
                ],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_nonlocal_does_not_resolve_to_global() {
        // module-level x = 1; def f(): nonlocal x
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("x".into())),
                    value: sp(Expr::IntLit(1)),
                }),
                sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "f".into(),
                    type_params: vec![],
                    params: vec![],
                    return_ty: None,
                    body: vec![sp(Stmt::Nonlocal(vec!["x".into()]))],
                }),
            ],
        };
        let result = resolve_module(&module);
        assert_eq!(
            result.errors.len(),
            1,
            "module globals must not satisfy nonlocal: {:?}",
            result.errors
        );
        assert!(
            result.errors[0].to_string().contains("nonlocal"),
            "error should mention nonlocal: {:?}",
            result.errors[0]
        );
    }

    // ── Group 5: Class scope ───────────────────────────────────────────────

    #[test]
    fn test_class_body_defines_names() {
        // class C: x = 1  →  x registered in class scope (no error)
        let module = Module {
            stmts: vec![sp(Stmt::ClassDef {
                decorators: vec![],
                name: "C".into(),
                type_params: vec![],
                bases: vec![],
                keyword_args: vec![],
                body: vec![sp(Stmt::Assign {
                    target: sp(Expr::Ident("x".into())),
                    value: sp(Expr::IntLit(1)),
                })],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_class_method_registered() {
        // class C: def method(self): pass  →  no errors
        let module = Module {
            stmts: vec![sp(Stmt::ClassDef {
                decorators: vec![],
                name: "C".into(),
                type_params: vec![],
                bases: vec![],
                keyword_args: vec![],
                body: vec![sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "method".into(),
                    type_params: vec![],
                    params: vec![Param {
                        name: "self".into(),
                        ty: sp(TypeExpr::Named("C".into())),
                        default: None,
                        kind: ParamKind::Regular,
                        pos_only: false,
                        kw_only: false,
                        span: Span::dummy(),
                    }],
                    return_ty: None,
                    body: vec![sp(Stmt::Pass)],
                })],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_class_name_registered_at_module() {
        // class MyClass: pass  →  lookup("MyClass") returns Class symbol
        let module = Module {
            stmts: vec![sp(Stmt::ClassDef {
                decorators: vec![],
                name: "MyClass".into(),
                type_params: vec![],
                bases: vec![],
                keyword_args: vec![],
                body: vec![sp(Stmt::Pass)],
            })],
        };
        let result = resolve_module(&module);
        let id = result
            .symbols
            .lookup("MyClass")
            .expect("MyClass should be registered");
        assert_eq!(result.symbols.get_symbol(id).kind, SymbolKind::Class);
    }

    #[test]
    fn test_class_with_base_resolves_base() {
        // class A: pass; class B(A): pass  →  no error resolving A as base
        let module = Module {
            stmts: vec![
                sp(Stmt::ClassDef {
                    decorators: vec![],
                    name: "A".into(),
                    type_params: vec![],
                    bases: vec![],
                    keyword_args: vec![],
                    body: vec![sp(Stmt::Pass)],
                }),
                sp(Stmt::ClassDef {
                    decorators: vec![],
                    name: "B".into(),
                    type_params: vec![],
                    bases: vec![sp(Expr::Ident("A".into()))],
                    keyword_args: vec![],
                    body: vec![sp(Stmt::Pass)],
                }),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    // ── Group 6: Import forms ──────────────────────────────────────────────

    #[test]
    fn test_import_registers_module_name() {
        // import os  →  no parse errors; Import is currently a no-op in resolver
        let module = Module {
            stmts: vec![sp(Stmt::Import {
                module: vec!["os".into()],
                names: None,
                module_alias: None,
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_import_as_registers_alias() {
        // import os as operating_system  →  no errors
        let module = Module {
            stmts: vec![sp(Stmt::Import {
                module: vec!["os".into()],
                names: None,
                module_alias: Some("operating_system".into()),
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_from_import_registers_name() {
        // from sys import argv  →  no errors
        let module = Module {
            stmts: vec![sp(Stmt::Import {
                module: vec!["sys".into()],
                names: Some(vec![("argv".into(), None)]),
                module_alias: None,
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_from_import_as_registers_alias() {
        // from os import path as p  →  no errors
        let module = Module {
            stmts: vec![sp(Stmt::Import {
                module: vec!["os".into()],
                names: Some(vec![("path".into(), Some("p".into()))]),
                module_alias: None,
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_import_used_in_expression() {
        // x: int = 1 then use x in an expression — verifies import+use pattern doesn't panic
        let module = Module {
            stmts: vec![
                sp(Stmt::Import {
                    module: vec!["os".into()],
                    names: None,
                    module_alias: None,
                }),
                sp(Stmt::VarDecl {
                    name: "result".into(),
                    ty: sp(TypeExpr::Named("int".into())),
                    value: sp(Expr::IntLit(0)),
                }),
                sp(Stmt::ExprStmt(sp(Expr::Ident("result".into())))),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    // ── Group 7: Assignment forms ──────────────────────────────────────────

    #[test]
    fn test_augassign_requires_prior_definition() {
        // x += 1 where x was not previously defined → resolver tries to resolve x → error
        let module = Module {
            stmts: vec![sp(Stmt::AugAssign {
                target: sp(Expr::Ident("x".into())),
                op: AugOp::Add,
                value: sp(Expr::IntLit(1)),
            })],
        };
        let result = resolve_module(&module);
        // AugAssign resolves target as an expression, so undefined x → error
        assert!(
            !result.errors.is_empty(),
            "augmented assign to undefined variable should error"
        );
    }

    #[test]
    fn test_assign_defines_new_variable() {
        // x = 1; y = x  →  both x and y defined, no errors
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("x".into())),
                    value: sp(Expr::IntLit(1)),
                }),
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("y".into())),
                    value: sp(Expr::Ident("x".into())),
                }),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert!(result.symbols.lookup("x").is_some());
        assert!(result.symbols.lookup("y").is_some());
    }

    #[test]
    fn test_vardecl_visible_in_rest_of_scope() {
        // x: int = 5; then use x  →  no error
        let module = Module {
            stmts: vec![
                sp(Stmt::VarDecl {
                    name: "x".into(),
                    ty: sp(TypeExpr::Named("int".into())),
                    value: sp(Expr::IntLit(5)),
                }),
                sp(Stmt::ExprStmt(sp(Expr::Ident("x".into())))),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_assign_to_attribute_no_new_symbol() {
        // obj = SomeObj(); obj.x = 1  →  no new top-level symbol for "x"
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("obj".into())),
                    value: sp(Expr::IntLit(0)),
                }),
                sp(Stmt::Assign {
                    target: sp(Expr::Attr {
                        object: Box::new(sp(Expr::Ident("obj".into()))),
                        attr: "x".into(),
                    }),
                    value: sp(Expr::IntLit(1)),
                }),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        // "x" should not be a standalone symbol
        assert!(
            result.symbols.lookup("x").is_none(),
            "attribute assignment should not register x as a symbol"
        );
    }

    // ── Group 8: Comprehension / For scope ────────────────────────────────

    #[test]
    fn test_for_loop_target_defines_variable() {
        // for i in []: pass  →  "i" defined after loop, no error
        let module = Module {
            stmts: vec![sp(Stmt::For {
                targets: vec!["i".into()],
                var_ty: None,
                iter: sp(Expr::ListLit(vec![])),
                body: vec![sp(Stmt::Pass)],
                else_body: None,
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert!(
            result.symbols.lookup("i").is_some(),
            "loop variable i should be defined"
        );
    }

    #[test]
    fn test_list_comp_registers_iter_var() {
        // [x for x in items] where items defined  →  no error
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("items".into())),
                    value: sp(Expr::ListLit(vec![])),
                }),
                sp(Stmt::ExprStmt(sp(Expr::ListComp {
                    element: Box::new(sp(Expr::Ident("x".into()))),
                    generators: vec![Comprehension {
                        targets: vec!["x".into()],
                        unpack_target: false,
                        target_reads_before_bind: Vec::new(),
                        iter: sp(Expr::Ident("items".into())),
                        conditions: vec![],
                        is_async: false,
                    }],
                }))),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_for_target_reusable_after_loop() {
        // for i in []: pass; then use i  →  i still in scope, no error
        let module = Module {
            stmts: vec![
                sp(Stmt::For {
                    targets: vec!["i".into()],
                    var_ty: None,
                    iter: sp(Expr::ListLit(vec![])),
                    body: vec![sp(Stmt::Pass)],
                    else_body: None,
                }),
                sp(Stmt::ExprStmt(sp(Expr::Ident("i".into())))),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    // ── Group 9: Error cases ───────────────────────────────────────────────

    #[test]
    fn test_undefined_name_in_function() {
        // def f(): return z  (z never defined)
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "f".into(),
                type_params: vec![],
                params: vec![],
                return_ty: None,
                body: vec![sp(Stmt::Return(Some(sp(Expr::Ident("z".into())))))],
            })],
        };
        let result = resolve_module(&module);
        assert_eq!(
            result.errors.len(),
            1,
            "should have exactly one undefined-name error"
        );
    }

    #[test]
    fn test_undefined_in_class_body() {
        // class C: x = undefined_thing
        let module = Module {
            stmts: vec![sp(Stmt::ClassDef {
                decorators: vec![],
                name: "C".into(),
                type_params: vec![],
                bases: vec![],
                keyword_args: vec![],
                body: vec![sp(Stmt::Assign {
                    target: sp(Expr::Ident("x".into())),
                    value: sp(Expr::Ident("undefined_thing".into())),
                })],
            })],
        };
        let result = resolve_module(&module);
        assert_eq!(
            result.errors.len(),
            1,
            "class body should report undefined names"
        );
    }

    #[test]
    fn test_multiple_undefined_names() {
        // use x, y, z all undefined at module level
        let module = Module {
            stmts: vec![
                sp(Stmt::ExprStmt(sp(Expr::Ident("x".into())))),
                sp(Stmt::ExprStmt(sp(Expr::Ident("y".into())))),
                sp(Stmt::ExprStmt(sp(Expr::Ident("z".into())))),
            ],
        };
        let result = resolve_module(&module);
        assert_eq!(
            result.errors.len(),
            3,
            "should report 3 undefined-name errors"
        );
    }

    #[test]
    fn test_self_referential_assignment() {
        // x = x  where x not yet defined → error resolving rhs
        let module = Module {
            stmts: vec![sp(Stmt::Assign {
                target: sp(Expr::Ident("x".into())),
                value: sp(Expr::Ident("x".into())),
            })],
        };
        let result = resolve_module(&module);
        // The resolver defines x after resolving rhs, so rhs x is undefined → error
        assert!(
            !result.errors.is_empty(),
            "self-referential assignment should error"
        );
    }

    #[test]
    fn test_empty_module_has_no_errors() {
        // Empty module: no stmts → 0 errors, empty name_map
        let module = Module { stmts: vec![] };
        let result = resolve_module(&module);
        assert!(
            result.errors.is_empty(),
            "empty module should have no errors"
        );
        assert!(
            result.name_map.is_empty(),
            "empty module should have empty name_map"
        );
    }

    // ── Group 10: Name map / symbol counts ────────────────────────────────

    #[test]
    fn test_name_map_populated() {
        // VarDecl creates an entry in name_map
        let module = Module {
            stmts: vec![sp(Stmt::VarDecl {
                name: "v".into(),
                ty: sp(TypeExpr::Named("int".into())),
                value: sp(Expr::IntLit(0)),
            })],
        };
        let result = resolve_module(&module);
        assert!(
            !result.name_map.is_empty(),
            "VarDecl should create a name_map entry"
        );
        let has_v = result
            .name_map
            .iter()
            .any(|(_, id)| result.symbols.get_symbol(*id).name == "v");
        assert!(has_v, "name_map should contain entry for v");
    }

    #[test]
    fn test_symbol_count_multiple_functions() {
        // 3 top-level functions → at least 3 symbols
        let module = Module {
            stmts: vec![
                sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "f1".into(),
                    type_params: vec![],
                    params: vec![],
                    return_ty: None,
                    body: vec![sp(Stmt::Pass)],
                }),
                sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "f2".into(),
                    type_params: vec![],
                    params: vec![],
                    return_ty: None,
                    body: vec![sp(Stmt::Pass)],
                }),
                sp(Stmt::FnDef {
                    decorators: vec![],
                    name: "f3".into(),
                    type_params: vec![],
                    params: vec![],
                    return_ty: None,
                    body: vec![sp(Stmt::Pass)],
                }),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert!(
            result.name_map.len() >= 3,
            "should have at least 3 name_map entries for 3 functions"
        );
    }

    #[test]
    fn test_function_params_in_name_map() {
        // def f(a: int, b: int): pass  →  2 param entries in name_map
        let module = Module {
            stmts: vec![sp(Stmt::FnDef {
                decorators: vec![],
                name: "f".into(),
                type_params: vec![],
                params: vec![
                    Param {
                        name: "a".into(),
                        ty: sp(TypeExpr::Named("int".into())),
                        default: None,
                        kind: ParamKind::Regular,
                        pos_only: false,
                        kw_only: false,
                        span: Span::dummy(),
                    },
                    Param {
                        name: "b".into(),
                        ty: sp(TypeExpr::Named("int".into())),
                        default: None,
                        kind: ParamKind::Regular,
                        pos_only: false,
                        kw_only: false,
                        span: Span::dummy(),
                    },
                ],
                return_ty: None,
                body: vec![sp(Stmt::Pass)],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let param_entries = result
            .name_map
            .iter()
            .filter(|(_, id)| result.symbols.get_symbol(*id).kind == SymbolKind::Parameter)
            .count();
        assert_eq!(
            param_entries, 2,
            "should have 2 parameter entries in name_map"
        );
    }

    #[test]
    fn test_class_def_in_name_map() {
        // class Foo: pass  →  name_map has entry for Foo
        let module = Module {
            stmts: vec![sp(Stmt::ClassDef {
                decorators: vec![],
                name: "Foo".into(),
                type_params: vec![],
                bases: vec![],
                keyword_args: vec![],
                body: vec![sp(Stmt::Pass)],
            })],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let has_foo = result
            .name_map
            .iter()
            .any(|(_, id)| result.symbols.get_symbol(*id).name == "Foo");
        assert!(has_foo, "name_map should contain entry for Foo");
    }

    // ── Group 11: Comprehension scope isolation (P0-R5) ─────────────────

    #[test]
    fn test_list_comp_scope_isolation() {
        // x = 1; [x for x in items]; use x → x still resolves to outer
        // The comprehension's x should NOT leak into the enclosing scope.
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("items".into())),
                    value: sp(Expr::ListLit(vec![])),
                }),
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("x".into())),
                    value: sp(Expr::IntLit(99)),
                }),
                sp(Stmt::ExprStmt(sp(Expr::ListComp {
                    element: Box::new(sp(Expr::Ident("x".into()))),
                    generators: vec![Comprehension {
                        targets: vec!["x".into()],
                        unpack_target: false,
                        target_reads_before_bind: Vec::new(),
                        iter: sp(Expr::Ident("items".into())),
                        conditions: vec![],
                        is_async: false,
                    }],
                }))),
                // After comprehension, x should still resolve to the outer scope's x
                sp(Stmt::ExprStmt(sp(Expr::Ident("x".into())))),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        // The outer x should still be accessible (no error for last use of x)
    }

    #[test]
    fn test_dict_comp_scope_isolation() {
        // k = "outer"; {k: k for k in items}; use k → k resolves to outer
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("items".into())),
                    value: sp(Expr::ListLit(vec![])),
                }),
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("k".into())),
                    value: sp(Expr::StrLit("outer".into())),
                }),
                sp(Stmt::ExprStmt(sp(Expr::DictComp {
                    key: Box::new(sp(Expr::Ident("k".into()))),
                    value: Box::new(sp(Expr::Ident("k".into()))),
                    generators: vec![Comprehension {
                        targets: vec!["k".into()],
                        unpack_target: false,
                        target_reads_before_bind: Vec::new(),
                        iter: sp(Expr::Ident("items".into())),
                        conditions: vec![],
                        is_async: false,
                    }],
                }))),
                // After dict comprehension, k should still resolve to outer scope's k
                sp(Stmt::ExprStmt(sp(Expr::Ident("k".into())))),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_set_comp_scope_isolation() {
        // v = "keep"; {v for v in items}; use v → v resolves to outer
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("items".into())),
                    value: sp(Expr::ListLit(vec![])),
                }),
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("v".into())),
                    value: sp(Expr::StrLit("keep".into())),
                }),
                sp(Stmt::ExprStmt(sp(Expr::SetComp {
                    element: Box::new(sp(Expr::Ident("v".into()))),
                    generators: vec![Comprehension {
                        targets: vec!["v".into()],
                        unpack_target: false,
                        target_reads_before_bind: Vec::new(),
                        iter: sp(Expr::Ident("items".into())),
                        conditions: vec![],
                        is_async: false,
                    }],
                }))),
                // After set comprehension, v should still resolve to outer scope's v
                sp(Stmt::ExprStmt(sp(Expr::Ident("v".into())))),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_generator_expr_scope_isolation() {
        // n = "outer"; sum(n for n in items); use n → n resolves to outer
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("items".into())),
                    value: sp(Expr::ListLit(vec![])),
                }),
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("n".into())),
                    value: sp(Expr::StrLit("outer".into())),
                }),
                sp(Stmt::ExprStmt(sp(Expr::GeneratorExpr {
                    element: Box::new(sp(Expr::Ident("n".into()))),
                    generators: vec![Comprehension {
                        targets: vec!["n".into()],
                        unpack_target: false,
                        target_reads_before_bind: Vec::new(),
                        iter: sp(Expr::Ident("items".into())),
                        conditions: vec![],
                        is_async: false,
                    }],
                }))),
                // After generator expression, n should still resolve to outer scope's n
                sp(Stmt::ExprStmt(sp(Expr::Ident("n".into())))),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn test_comprehension_iter_var_not_in_outer_scope() {
        // No prior definition of fresh_var; [fresh_var for fresh_var in items]; use fresh_var → error
        // The comprehension should define fresh_var only in its inner scope.
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("items".into())),
                    value: sp(Expr::ListLit(vec![])),
                }),
                sp(Stmt::ExprStmt(sp(Expr::ListComp {
                    element: Box::new(sp(Expr::Ident("fresh_var".into()))),
                    generators: vec![Comprehension {
                        targets: vec!["fresh_var".into()],
                        unpack_target: false,
                        target_reads_before_bind: Vec::new(),
                        iter: sp(Expr::Ident("items".into())),
                        conditions: vec![],
                        is_async: false,
                    }],
                }))),
                // fresh_var was only defined inside comprehension scope — should not be visible here
                sp(Stmt::ExprStmt(sp(Expr::Ident("fresh_var".into())))),
            ],
        };
        let result = resolve_module(&module);
        // fresh_var should be undefined in the outer scope
        assert!(
            !result.errors.is_empty(),
            "comprehension iter variable should not leak to outer scope"
        );
    }

    // ── Group 12: Walrus operator := scope (P0-R6, PEP 572) ────────────

    #[test]
    fn test_walrus_outside_comprehension() {
        // y := 42 at module level → y defined in current scope
        let module = Module {
            stmts: vec![
                sp(Stmt::ExprStmt(sp(Expr::Walrus {
                    target: "y".into(),
                    value: Box::new(sp(Expr::IntLit(42))),
                }))),
                sp(Stmt::ExprStmt(sp(Expr::Ident("y".into())))),
            ],
        };
        let result = resolve_module(&module);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert!(
            result.symbols.lookup("y").is_some(),
            "y should be defined by walrus operator"
        );
    }

    #[test]
    fn test_walrus_in_list_comp_binds_enclosing() {
        // items = []; [y := x for x in items]; use y → y defined in enclosing scope
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("items".into())),
                    value: sp(Expr::ListLit(vec![])),
                }),
                sp(Stmt::ExprStmt(sp(Expr::ListComp {
                    element: Box::new(sp(Expr::Walrus {
                        target: "y".into(),
                        value: Box::new(sp(Expr::Ident("x".into()))),
                    })),
                    generators: vec![Comprehension {
                        targets: vec!["x".into()],
                        unpack_target: false,
                        target_reads_before_bind: Vec::new(),
                        iter: sp(Expr::Ident("items".into())),
                        conditions: vec![],
                        is_async: false,
                    }],
                }))),
                // y should be accessible after the comprehension (PEP 572)
                sp(Stmt::ExprStmt(sp(Expr::Ident("y".into())))),
            ],
        };
        let result = resolve_module(&module);
        assert!(
            result.errors.is_empty(),
            "walrus target should be visible after comprehension: {:?}",
            result.errors
        );
        assert!(
            result.symbols.lookup("y").is_some(),
            "walrus target y should be defined in enclosing scope"
        );
    }

    #[test]
    fn test_walrus_in_comp_filter_binds_enclosing() {
        // items = []; [x for x in items if (z := x) > 2]; use z → z defined in enclosing scope
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("items".into())),
                    value: sp(Expr::ListLit(vec![])),
                }),
                sp(Stmt::ExprStmt(sp(Expr::ListComp {
                    element: Box::new(sp(Expr::Ident("x".into()))),
                    generators: vec![Comprehension {
                        targets: vec!["x".into()],
                        unpack_target: false,
                        target_reads_before_bind: Vec::new(),
                        iter: sp(Expr::Ident("items".into())),
                        conditions: vec![sp(Expr::BinOp {
                            op: BinOp::Gt,
                            lhs: Box::new(sp(Expr::Walrus {
                                target: "z".into(),
                                value: Box::new(sp(Expr::Ident("x".into()))),
                            })),
                            rhs: Box::new(sp(Expr::IntLit(2))),
                        })],
                        is_async: false,
                    }],
                }))),
                // z should be accessible after the comprehension (PEP 572)
                sp(Stmt::ExprStmt(sp(Expr::Ident("z".into())))),
            ],
        };
        let result = resolve_module(&module);
        assert!(
            result.errors.is_empty(),
            "walrus target in filter should be visible after comprehension: {:?}",
            result.errors
        );
        assert!(
            result.symbols.lookup("z").is_some(),
            "walrus target z should be defined in enclosing scope"
        );
    }

    #[test]
    fn test_walrus_comprehension_iter_var_still_isolated() {
        // [y := x for x in items]; use x → error (x is comp iter var, isolated)
        // but y is walrus target → should be visible
        let module = Module {
            stmts: vec![
                sp(Stmt::Assign {
                    target: sp(Expr::Ident("items".into())),
                    value: sp(Expr::ListLit(vec![])),
                }),
                sp(Stmt::ExprStmt(sp(Expr::ListComp {
                    element: Box::new(sp(Expr::Walrus {
                        target: "y".into(),
                        value: Box::new(sp(Expr::Ident("x".into()))),
                    })),
                    generators: vec![Comprehension {
                        targets: vec!["x".into()],
                        unpack_target: false,
                        target_reads_before_bind: Vec::new(),
                        iter: sp(Expr::Ident("items".into())),
                        conditions: vec![],
                        is_async: false,
                    }],
                }))),
                // x should NOT be accessible (iter var is isolated)
                sp(Stmt::ExprStmt(sp(Expr::Ident("x".into())))),
                // y SHOULD be accessible (walrus target in enclosing scope)
                sp(Stmt::ExprStmt(sp(Expr::Ident("y".into())))),
            ],
        };
        let result = resolve_module(&module);
        // x is undefined (iter var leaked), y is defined (walrus target)
        assert_eq!(
            result.errors.len(),
            1,
            "should have exactly 1 error (x undefined), got: {:?}",
            result.errors
        );
        assert!(
            result.symbols.lookup("y").is_some(),
            "walrus target y should be defined even though x is not"
        );
    }

    // @spec .aw/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#R5
    #[test]
    fn test_resolve_star_import_no_star_symbol() {
        // `from somemod import *` should NOT define `*` as a symbol.
        // Star imports bind names dynamically at runtime, not statically.
        let module = Module {
            stmts: vec![sp(Stmt::Import {
                module: vec!["somemod".to_string()],
                names: Some(vec![("*".to_string(), None)]),
                module_alias: None,
            })],
        };
        let result = resolve_module(&module);
        assert!(
            result.errors.is_empty(),
            "from X import * should not produce resolve errors, got: {:?}",
            result.errors
        );
        assert!(
            result.symbols.lookup("*").is_none(),
            "* should NOT be defined as a symbol in the resolve pass"
        );
    }
}
