use super::generic::{GenericParams, Substitution};
use super::protocol::ProtocolRegistry;
use super::ty::TypeVarId;
use super::{Ty, TypeContext, TypeId};
use crate::error::MambaError;
use crate::parser::ast::*;
use crate::resolve::{SymbolId, SymbolKind, SymbolTable};
use crate::source::span::{Span, Spanned};
use std::collections::{HashMap, HashSet};

/// Diagnostic severity for warnings vs errors (#244).
#[derive(Debug, Clone, PartialEq)]
pub enum DiagLevel {
    Warning,
    Error,
}

/// A diagnostic produced during type checking.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagLevel,
    pub span: Span,
    pub message: String,
}

/// Check if a class name belongs to the built-in exception hierarchy.
/// True if `e` is a call to a PEP 484 type-variable factory — `TypeVar`,
/// `ParamSpec`, or `TypeVarTuple` — whether referenced bare (`TypeVar("T")`)
/// or dotted (`typing.TypeVar("T")`). Used to recognise classic
/// `T = TypeVar("T")` assignments so the bound name resolves as a TypeVar in
/// later annotations.
fn is_type_var_factory_call(e: &Expr) -> bool {
    let Expr::Call { func, .. } = e else {
        return false;
    };
    let fname = match &func.node {
        Expr::Ident(n) => n.as_str(),
        Expr::Attr { attr, .. } => attr.as_str(),
        _ => return false,
    };
    matches!(fname, "TypeVar" | "ParamSpec" | "TypeVarTuple")
}

fn is_typing_overload_decorator(expr: &Expr) -> bool {
    match expr {
        Expr::Ident(n) => n == "overload",
        Expr::Attr { attr, .. } => attr == "overload",
        Expr::Call { func, .. } => is_typing_overload_decorator(&func.node),
        _ => false,
    }
}

fn is_exception_class_name(name: &str) -> bool {
    matches!(
        name,
        "BaseException"
            | "SystemExit"
            | "KeyboardInterrupt"
            | "GeneratorExit"
            | "Exception"
            | "StopIteration"
            | "StopAsyncIteration"
            | "ArithmeticError"
            | "ZeroDivisionError"
            | "OverflowError"
            | "FloatingPointError"
            | "LookupError"
            | "IndexError"
            | "KeyError"
            | "OSError"
            | "IOError"
            | "FileNotFoundError"
            | "PermissionError"
            | "FileExistsError"
            | "TypeError"
            | "ValueError"
            | "AttributeError"
            | "NameError"
            | "RuntimeError"
            | "RecursionError"
            | "NotImplementedError"
            | "ImportError"
            | "ModuleNotFoundError"
            | "SyntaxError"
            | "IndentationError"
            | "UnicodeError"
            | "UnicodeDecodeError"
            | "UnicodeEncodeError"
            | "UnicodeTranslateError"
            | "AssertionError"
            | "BufferError"
            | "EOFError"
            | "MemoryError"
            | "ConnectionError"
            | "TimeoutError"
            | "ExceptionGroup"
            | "BaseExceptionGroup"
            | "Warning"
            | "UserWarning"
            | "DeprecationWarning"
            | "PendingDeprecationWarning"
            | "SyntaxWarning"
            | "RuntimeWarning"
            | "FutureWarning"
            | "ImportWarning"
            | "UnicodeWarning"
            | "BytesWarning"
            | "ResourceWarning"
            | "EncodingWarning"
    )
}

/// Type checker: walks the AST, resolves names, and checks types.
#[derive(Debug, Clone)]
pub(crate) struct FunctionParamSig {
    pub(crate) name: String,
    pub(crate) ty: TypeId,
    pub(crate) kind: ParamKind,
    pub(crate) kw_only: bool,
}

pub struct TypeChecker {
    pub tcx: TypeContext,
    pub symbols: SymbolTable,
    /// Map from SymbolId to TypeId
    sym_types: Vec<Option<TypeId>>,
    /// Current function return type (for checking return statements)
    pub(crate) current_return_ty: Option<TypeId>,
    /// Current class name for `Self` type resolution (#243).
    pub(crate) current_class: Option<String>,
    /// Strict mode: treat Any-inference warnings as errors (#244).
    pub strict: bool,
    /// Source carries `# mamba-strict-type:`. A few stdlib APIs are runtime
    /// permissive but still have explicit type-wall fixtures; keep those
    /// compile-time checks scoped to strict fixtures.
    pub strict_type_fixture: bool,
    /// Suppress Any-inference warnings (#244).
    pub no_warn_any: bool,
    /// Runtime/JIT execution mode: unresolved names should become runtime
    /// NameError instead of compile-time undefined-name diagnostics.
    pub allow_runtime_unresolved_names: bool,
    errors: Vec<MambaError>,
    pub diagnostics: Vec<Diagnostic>,
    /// Generic parameter lists for functions/classes (#314).
    pub(crate) generic_defs: HashMap<String, GenericParams>,
    /// Full user-function parameter metadata. `Ty::Fn` deliberately keeps a
    /// compact ABI-facing shape, so keyword-only and variadic annotation checks
    /// live in this checker-only side channel.
    pub(crate) function_param_sigs: HashMap<String, Vec<FunctionParamSig>>,
    /// Protocol registry for structural subtyping (#314).
    pub(crate) protocol_registry: ProtocolRegistry,
    /// Counter for TypeVarId allocation.
    pub(crate) next_type_var_id: u32,
    /// Class method signatures for protocol conformance checking (#314).
    pub(crate) class_methods: HashMap<String, HashMap<String, super::protocol::MethodSig>>,
    /// Class method signatures for bare-class unbound calls such as
    /// `Box.get(obj, arg)`. These include the explicit receiver parameter.
    pub(crate) class_unbound_methods: HashMap<String, HashMap<String, super::protocol::MethodSig>>,
    /// User classes declared with `TypedDict` in their base chain. Runtime
    /// instances of these classes are plain dict values, so a variable annotated
    /// as the TypedDict class accepts dict literals/values.
    pub(crate) typed_dict_classes: HashSet<String>,
    /// User classes that are BARE: no base class (other than `object`) and no
    /// methods. A bare class instance (`class _W: pass` → `_W()`) can satisfy
    /// neither a protocol (it has no dunders) nor a nominal type (it has no
    /// superclass), so the ① hook rejects it against a `CoreTy::Typed` param.
    /// Classes with any base or any method are NOT recorded here, so they are
    /// always skipped — keeping the bare-class rejection false-positive-clean.
    pub(crate) user_bare_classes: std::collections::HashSet<String>,
    /// Subject type of the enclosing `match` statement, used to propagate type
    /// into capture / star / AS bindings in `check_pattern` (#827).
    pub(crate) current_match_subject_ty: Option<TypeId>,
    /// Depth counter for comprehension scopes. Walrus (PEP 572) targets in a
    /// comprehension must bind in the enclosing function scope, not the
    /// comp's own scope; outside any comprehension the target binds in the
    /// current scope as for any other assignment. Without this distinction,
    /// `(i := i + 1)` inside a function body re-defined `i` at module scope,
    /// poisoning a same-named outer variable's type.
    pub(crate) comprehension_depth: u32,
    /// ① Type-wall PoC: import provenance for stdlib call resolution. Maps a
    /// bound *name* to its `(dotted-module, qualifier)` origin so a later call
    /// site can recover `(module, qualifier, name)` and consult the hardcoded
    /// `stdlib_sigs` table. The bound *symbol type* stays `Any` — this map is a
    /// purely additive side channel that never changes inference.
    ///
    /// Conventions:
    /// - `import os` / `import os as o`     -> name -> ("os", "")
    /// - `from os import strerror`          -> "strerror" -> ("os", "")
    /// - `from html.parser import HTMLParser` (a class) ->
    ///       "HTMLParser" -> ("html.parser", "HTMLParser")
    ///   (qualifier == name marks a class binding so instance vars can adopt it)
    pub(crate) import_origins: HashMap<String, (String, String)>,
    /// ① Type-wall PoC: maps a local variable name to the class (qualifier) of
    /// the stdlib instance it holds, populated when a var is assigned
    /// `object.__new__(Cls)` or `Cls(...)` where `Cls` is a known imported
    /// stdlib class. Lets `obj.method(arg)` resolve to a `Method` signature.
    pub(crate) instance_origins: HashMap<String, String>,
    /// Original symbol ids for builtin function names registered during
    /// TypeChecker construction. If the current lookup no longer matches this
    /// id, user code has shadowed the builtin and stdlib signature enforcement
    /// must not apply to the bare name.
    pub(crate) builtin_symbols: HashMap<String, SymbolId>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut tc = Self {
            tcx: TypeContext::new(),
            symbols: SymbolTable::new(),
            sym_types: Vec::new(),
            current_return_ty: None,
            current_class: None,
            strict: false,
            strict_type_fixture: false,
            no_warn_any: false,
            allow_runtime_unresolved_names: false,
            errors: Vec::new(),
            diagnostics: Vec::new(),
            generic_defs: HashMap::new(),
            function_param_sigs: HashMap::new(),
            protocol_registry: ProtocolRegistry::new(),
            next_type_var_id: 0,
            class_methods: HashMap::new(),
            class_unbound_methods: HashMap::new(),
            typed_dict_classes: HashSet::new(),
            user_bare_classes: std::collections::HashSet::new(),
            current_match_subject_ty: None,
            comprehension_depth: 0,
            import_origins: HashMap::new(),
            instance_origins: HashMap::new(),
            builtin_symbols: HashMap::new(),
        };
        tc.register_builtins();
        tc
    }

    pub(crate) fn set_sym_type(&mut self, sym_idx: u32, ty: TypeId) {
        let idx = sym_idx as usize;
        if idx >= self.sym_types.len() {
            self.sym_types.resize(idx + 1, None);
        }
        self.sym_types[idx] = Some(ty);
    }

    pub(crate) fn is_unshadowed_builtin(&self, name: &str) -> bool {
        let Some(builtin_id) = self.builtin_symbols.get(name).copied() else {
            return false;
        };
        self.symbols.lookup(name) == Some(builtin_id)
    }

    pub(crate) fn get_sym_type(&self, sym_idx: u32) -> TypeId {
        self.sym_types
            .get(sym_idx as usize)
            .and_then(|t| *t)
            .unwrap_or(self.tcx.error())
    }

    pub(crate) fn record_function_param_sigs(
        &mut self,
        name: &str,
        params: &[Param],
        overload_decorated: bool,
    ) {
        if overload_decorated {
            self.function_param_sigs.remove(name);
            return;
        }

        let sigs = params
            .iter()
            .map(|param| FunctionParamSig {
                name: param.name.clone(),
                ty: self.resolve_type_expr(&param.ty),
                kind: param.kind,
                kw_only: param.kw_only,
            })
            .collect();
        self.function_param_sigs.insert(name.to_string(), sigs);
    }

    pub(crate) fn error(&mut self, span: Span, msg: impl Into<String>) {
        self.errors.push(MambaError::type_err(span, msg));
    }

    /// Current error count — pair with `truncate_errors` to speculatively
    /// check an expression for its binding side effects (walrus targets in
    /// f-string fields) without surfacing new type errors.
    pub(crate) fn errors_mark(&self) -> usize {
        self.errors.len()
    }

    /// Drop errors recorded after `mark` (see `errors_mark`).
    pub(crate) fn truncate_errors(&mut self, mark: usize) {
        self.errors.truncate(mark);
    }

    /// Emit an Any-inference warning (#244). If strict mode, emits error instead.
    #[allow(dead_code)]
    pub(crate) fn warn_any(&mut self, span: Span, msg: impl Into<String>) {
        if self.no_warn_any {
            return;
        }
        let message = msg.into();
        if self.strict {
            self.errors.push(MambaError::type_err(span, message));
        } else {
            self.diagnostics.push(Diagnostic {
                level: DiagLevel::Warning,
                span,
                message,
            });
        }
    }

    /// Register type parameters as TypeVars and return GenericParams.
    pub(crate) fn register_type_params(
        &mut self,
        type_params: &[crate::parser::ast::TypeParam],
    ) -> GenericParams {
        let mut gp = GenericParams::new();
        for param in type_params {
            let name = &param.name;
            let var_id = TypeVarId(self.next_type_var_id);
            self.next_type_var_id += 1;
            self.tcx.new_type_var(name.clone(), None, Vec::new());
            gp.add(name, var_id, None);

            // Register in type alias scope so `T` resolves as a TypeVar
            let tv_ty = self.tcx.intern(Ty::TypeVar(var_id));
            self.tcx.register_alias(name.clone(), tv_ty);
        }
        gp
    }

    /// Remove type parameter aliases to prevent leaking outside scope.
    pub(crate) fn unregister_type_params(&mut self, type_params: &[crate::parser::ast::TypeParam]) {
        for param in type_params {
            self.tcx.unregister_alias(&param.name);
        }
    }

    /// Get the TypeId for a SymbolId, if known (#1190).
    pub fn get_symbol_type(&self, sym: crate::resolve::SymbolId) -> Option<crate::types::TypeId> {
        self.sym_types.get(sym.0 as usize).and_then(|t| *t)
    }

    /// Check a module. Returns accumulated errors.
    /// First-pass def/class/enum/alias pre-registration, descending into
    /// compound-statement bodies (try/if/while/for/with): a class defined in
    /// a module-level `try:` is still a module-scope binding.
    fn preregister_defs(&mut self, stmts: &[Spanned<Stmt>]) {
        for stmt in stmts {
            match &stmt.node {
                Stmt::FnDef {
                    name,
                    type_params,
                    params,
                    return_ty,
                    decorators,
                    ..
                }
                | Stmt::AsyncFnDef {
                    name,
                    type_params,
                    params,
                    return_ty,
                    decorators,
                    ..
                } => {
                    // Register generic type params before resolving param/ret types
                    let gp = self.register_type_params(type_params);

                    let sym = self.symbols.define(name.clone(), SymbolKind::Function);
                    let overload_decorated = decorators
                        .iter()
                        .any(|d| is_typing_overload_decorator(&d.node));
                    self.record_function_param_sigs(name, params, overload_decorated);
                    let (param_types, ret, is_variadic) = if overload_decorated {
                        (Vec::new(), self.tcx.any(), true)
                    } else {
                        // Detect *args variadic parameter and exclude it from param_types.
                        // Only positional params before the *args are required at call sites.
                        let star_pos = params
                            .iter()
                            .position(|p| p.kind == crate::parser::ast::ParamKind::Star);
                        let is_variadic = star_pos.is_some()
                            || params
                                .iter()
                                .any(|p| p.kind == crate::parser::ast::ParamKind::DoubleStar);
                        let effective_params =
                            star_pos.map_or(params.as_slice(), |pos| &params[..pos]);
                        let param_types: Vec<TypeId> = effective_params
                            .iter()
                            .filter(|p| p.kind == crate::parser::ast::ParamKind::Regular)
                            .map(|p| self.resolve_type_expr(&p.ty))
                            .collect();
                        let ret = return_ty
                            .as_ref()
                            .map(|t| self.resolve_type_expr(t))
                            .unwrap_or(self.tcx.any());
                        (param_types, ret, is_variadic)
                    };
                    let fn_ty = self.tcx.intern(Ty::Fn {
                        params: param_types,
                        ret,
                        variadic: is_variadic,
                    });
                    self.set_sym_type(sym.0, fn_ty);

                    if !gp.is_empty() {
                        self.generic_defs.insert(name.clone(), gp);
                    }
                    // Clean up type parameter aliases to prevent leaking
                    self.unregister_type_params(type_params);
                }
                Stmt::ClassDef {
                    name,
                    type_params,
                    bases,
                    body,
                    ..
                } => {
                    // Register generic type params for the class
                    let gp = self.register_type_params(type_params);

                    let fields = self.collect_class_fields(body);
                    let match_args = self.collect_match_args(body);
                    let is_typed_dict = bases.iter().any(|b| self.base_is_typed_dict(&b.node));
                    let class_ty = self.tcx.intern(Ty::Class {
                        name: name.clone(),
                        fields,
                        match_args,
                    });
                    let sym = self.symbols.define(name.clone(), SymbolKind::Class);
                    self.set_sym_type(sym.0, class_ty);
                    if is_typed_dict {
                        self.typed_dict_classes.insert(name.clone());
                    }

                    if !gp.is_empty() {
                        self.generic_defs.insert(name.clone(), gp);
                    }

                    // Collect class methods for protocol conformance
                    self.collect_class_methods(name, body);

                    // Record a BARE class (no base other than `object`, no
                    // methods): such an instance can satisfy neither a protocol
                    // nor a nominal type, so the ① hook may reject it against a
                    // `CoreTy::Typed` param. Any base or any method disqualifies.
                    let only_object_base = bases
                        .iter()
                        .all(|b| matches!(&b.node, Expr::Ident(n) if n == "object"));
                    let has_method = body
                        .iter()
                        .any(|s| matches!(&s.node, Stmt::FnDef { .. } | Stmt::AsyncFnDef { .. }));
                    if only_object_base && !has_method {
                        self.user_bare_classes.insert(name.clone());
                    }

                    // Detect Protocol base class and register
                    let is_protocol = bases
                        .iter()
                        .any(|b| matches!(&b.node, Expr::Ident(n) if n == "Protocol"));
                    if is_protocol {
                        self.register_protocol(name, body);
                    }

                    // Clean up type parameter aliases to prevent leaking
                    self.unregister_type_params(type_params);
                }
                Stmt::EnumDef { name, variants, .. } => {
                    let v: Vec<(String, Vec<TypeId>)> = variants
                        .iter()
                        .map(|v| {
                            let ftypes = v
                                .fields
                                .iter()
                                .map(|f| self.resolve_type_expr(&f.ty))
                                .collect();
                            (v.name.clone(), ftypes)
                        })
                        .collect();
                    let enum_ty = self.tcx.intern(Ty::Enum {
                        name: name.clone(),
                        variants: v,
                    });
                    let sym = self.symbols.define(name.clone(), SymbolKind::Enum);
                    self.set_sym_type(sym.0, enum_ty);
                }
                Stmt::ExprStmt(_) => {
                    if let Some(fn_def) =
                        crate::exec_literal::global_literal_exec_fn_def(&stmt.node)
                    {
                        let sym = self.symbols.lookup(&fn_def.name).unwrap_or_else(|| {
                            self.symbols
                                .define(fn_def.name.clone(), SymbolKind::Function)
                        });
                        self.record_function_param_sigs(&fn_def.name, &fn_def.params, false);
                        let star_pos = fn_def
                            .params
                            .iter()
                            .position(|p| p.kind == crate::parser::ast::ParamKind::Star);
                        let is_variadic = star_pos.is_some()
                            || fn_def
                                .params
                                .iter()
                                .any(|p| p.kind == crate::parser::ast::ParamKind::DoubleStar);
                        let effective_params =
                            star_pos.map_or(fn_def.params.as_slice(), |pos| &fn_def.params[..pos]);
                        let param_types: Vec<TypeId> = effective_params
                            .iter()
                            .filter(|p| p.kind == crate::parser::ast::ParamKind::Regular)
                            .map(|p| self.resolve_type_expr(&p.ty))
                            .collect();
                        let fn_ty = self.tcx.intern(Ty::Fn {
                            params: param_types,
                            ret: self.tcx.any(),
                            variadic: is_variadic,
                        });
                        self.set_sym_type(sym.0, fn_ty);
                    }
                }
                Stmt::TypeAlias {
                    name,
                    type_params,
                    value,
                } => {
                    // The alias value is a general expression (PEP 695). For
                    // compile-time annotation use (`x: Alias`), convert the
                    // type-shaped subset back into a TypeExpr; non-type-shaped
                    // values (e.g. lambdas) only exist as runtime
                    // TypeAliasType objects and are skipped here.
                    if let Some(te) = expr_to_type_expr(value) {
                        // Pre-register the alias as Any so a recursive alias
                        // (`type R = R | None`) resolves instead of erroring.
                        let any = self.tcx.any();
                        self.tcx.register_alias(name.clone(), any);
                        // The alias's own params (`type Pair[T] = list[T]`)
                        // resolve as TypeVars within the value.
                        let _gp = self.register_type_params(type_params);
                        let resolved = self.resolve_type_expr(&te);
                        self.unregister_type_params(type_params);
                        self.tcx.register_alias(name.clone(), resolved);
                    }
                }
                // Register imported names as Any in the first pass so that
                // collect_class_methods / collect_class_fields can resolve
                // types from third-party or relative imports in class signatures.
                Stmt::Import {
                    names,
                    module_alias,
                    module,
                } => {
                    let any_ty = self.tcx.any();
                    // ① Type-wall PoC: record import provenance. The dotted
                    // module path is the source-of-truth key into stdlib_sigs.
                    let dotted = module.join(".");
                    if let Some(import_names) = names {
                        for (name, alias) in import_names {
                            let effective = alias.as_ref().unwrap_or(name);
                            if self.symbols.lookup(effective).is_none() {
                                let sym =
                                    self.symbols.define(effective.clone(), SymbolKind::Variable);
                                self.set_sym_type(sym.0, any_ty);
                            }
                            // Provenance: `from MOD import N [as B]`. If `N`
                            // names a stdlib class we know, record it as a class
                            // binding (qualifier == N) so instances can adopt
                            // the qualifier; otherwise it's a module fn/value
                            // bound directly (qualifier == "").
                            let qualifier = if Self::is_known_stdlib_class(&dotted, name) {
                                name.clone()
                            } else {
                                String::new()
                            };
                            self.import_origins
                                .insert(effective.clone(), (dotted.clone(), qualifier));
                        }
                    } else if let Some(alias) = module_alias {
                        if self.symbols.lookup(alias).is_none() {
                            let sym = self.symbols.define(alias.clone(), SymbolKind::Variable);
                            self.set_sym_type(sym.0, any_ty);
                        }
                        // `import MOD as A` -> A is the module.
                        self.import_origins
                            .insert(alias.clone(), (dotted.clone(), String::new()));
                    } else if let Some(root) = module.first() {
                        if self.symbols.lookup(root).is_none() {
                            let sym = self.symbols.define(root.clone(), SymbolKind::Variable);
                            self.set_sym_type(sym.0, any_ty);
                        }
                        // `import MOD` (possibly dotted) -> the *root* name binds
                        // to the *root* module. Dotted submodule access through
                        // the root is a separate runtime quirk; for the PoC we
                        // only key the directly-bound root module.
                        self.import_origins
                            .insert(root.clone(), (root.clone(), String::new()));
                    }
                }
                // Classic PEP 484 type-variable definitions:
                // `T = TypeVar("T")`, `P = ParamSpec("P")`,
                // `Ts = TypeVarTuple("Ts")`. The PEP 695 `[T]` syntax is
                // handled by register_type_params, but the assignment form is
                // not — so a later annotation `-> T` would fall through to the
                // `unknown type: T` error. Register the bound name as a TypeVar
                // alias (compatible with any type, see is_assignable) so such
                // annotations type-check the way they do under CPython.
                Stmt::Assign { target, value } => {
                    if let Expr::Ident(name) = &target.node {
                        if is_type_var_factory_call(&value.node) {
                            let var_id = TypeVarId(self.next_type_var_id);
                            self.next_type_var_id += 1;
                            self.tcx.new_type_var(name.clone(), None, Vec::new());
                            let tv_ty = self.tcx.intern(Ty::TypeVar(var_id));
                            self.tcx.register_alias(name.clone(), tv_ty);
                        }
                    }
                }
                _ => {}
            }
            match &stmt.node {
                Stmt::Try {
                    body,
                    handlers,
                    else_body,
                    finally_body,
                } => {
                    self.preregister_defs(body);
                    for h in handlers {
                        self.preregister_defs(&h.body);
                    }
                    if let Some(eb) = else_body {
                        self.preregister_defs(eb);
                    }
                    if let Some(fb) = finally_body {
                        self.preregister_defs(fb);
                    }
                }
                Stmt::If {
                    body,
                    elif_clauses,
                    else_body,
                    ..
                } => {
                    self.preregister_defs(body);
                    for (_, eb) in elif_clauses {
                        self.preregister_defs(eb);
                    }
                    if let Some(eb) = else_body {
                        self.preregister_defs(eb);
                    }
                }
                Stmt::While {
                    body, else_body, ..
                }
                | Stmt::For {
                    body, else_body, ..
                } => {
                    self.preregister_defs(body);
                    if let Some(eb) = else_body {
                        self.preregister_defs(eb);
                    }
                }
                Stmt::With { body, .. } => {
                    self.preregister_defs(body);
                }
                _ => {}
            }
        }
    }

    pub fn check_module(&mut self, module: &Module) -> Vec<MambaError> {
        // First pass: register all top-level function/class/enum/alias names
        self.preregister_defs(&module.stmts);

        // Second pass: check bodies
        for stmt in &module.stmts {
            self.check_stmt(stmt);
        }

        std::mem::take(&mut self.errors)
    }

    pub(crate) fn resolve_type_expr(&mut self, ty: &Spanned<TypeExpr>) -> TypeId {
        match &ty.node {
            TypeExpr::Named(name) => match name.as_str() {
                "int" => self.tcx.int(),
                "float" => self.tcx.float(),
                "bool" => self.tcx.bool(),
                "str" => self.tcx.str(),
                "None" => self.tcx.none(),
                "Any" => self.tcx.any(),
                // Bare collection type names: resolve to concrete Ty so that
                // annotations like `-> dict` or `list[dict]` don't accidentally
                // pick up the symbol-table entry for the builtin callable.
                "dict" => {
                    let a = self.tcx.any();
                    self.tcx.intern(Ty::Dict(a, a))
                }
                "list" => {
                    let a = self.tcx.any();
                    self.tcx.intern(Ty::List(a))
                }
                "tuple" => self.tcx.intern(Ty::Tuple(vec![])),
                "set" | "frozenset" => {
                    let a = self.tcx.any();
                    self.tcx.intern(Ty::Set(a))
                }
                // Other builtin types with no dedicated Ty variant. Without
                // these arms the call falls through to the symbol-table lookup
                // of the builtin callable, mistyping `b: bytes = ...` as
                // `() -> Any` so that `len(b)` / `b[0]` / iteration are
                // rejected at compile time. Resolve to Any (like set/frozenset)
                // so the annotated variable supports the full dynamic surface.
                "bytes" | "bytearray" | "memoryview" | "complex" | "range" | "slice" => {
                    self.tcx.any()
                }
                // `type` as a type expression (e.g. `type[BaseModel]` bare name):
                // the class-object type is represented as Any for now.
                "type" | "object" => self.tcx.any(),
                n if crate::parser::ast::strip_forward_ref_name(n).is_some() => self.tcx.any(),
                "Self" => {
                    // #243: resolve Self to current class type
                    if self.current_class.is_some() {
                        self.tcx.intern(Ty::SelfType)
                    } else {
                        // Outside a class, `self` is just a regular parameter name.
                        // Resolve to Any so standalone functions can be used as methods
                        // via type() 3-arg dynamic class creation.
                        self.tcx.any()
                    }
                }
                _ => {
                    // Check type aliases first (#241)
                    if let Some(alias_ty) = self.tcx.resolve_alias(name) {
                        return alias_ty;
                    }
                    // User-defined type — look up in symbols
                    if let Some(sym) = self.symbols.lookup(name) {
                        self.get_sym_type(sym.0)
                    } else if name.contains('.') {
                        // Dotted reference like `collections.abc.Mapping`
                        // (#1576): external/forward type — treat as Any so
                        // CPython-style annotations type-check.
                        self.tcx.any()
                    } else if name.contains(' ') {
                        // PEP 484 / PEP 563 string-literal forward references
                        // with freeform content (e.g. `'This is a new
                        // annotation'`, #1578): not a valid Python identifier —
                        // CPython does not evaluate these strings, so treat as
                        // Any rather than emitting an error.
                        self.tcx.any()
                    } else if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                        // Numeric-literal annotation (`-> 42`, PEP 3107
                        // arbitrary expression preserved textually by the
                        // parser for introspection): annotations are never
                        // validated as types in CPython — treat as Any.
                        self.tcx.any()
                    } else {
                        self.error(ty.span, format!("unknown type: `{name}`"));
                        self.tcx.error()
                    }
                }
            },
            TypeExpr::Generic { name, args } => {
                let inner: Vec<TypeId> = args.iter().map(|a| self.resolve_type_expr(a)).collect();
                match name.as_str() {
                    "list" if inner.len() == 1 => self.tcx.intern(Ty::List(inner[0])),
                    "dict" if inner.len() == 2 => self.tcx.intern(Ty::Dict(inner[0], inner[1])),
                    "tuple" => self.tcx.intern(Ty::Tuple(inner)),
                    "set" | "frozenset" if inner.len() == 1 => self.tcx.intern(Ty::Set(inner[0])),
                    "Callable" if inner.len() >= 2 => {
                        // #243: Callable[[params...], ret] → Fn type
                        let ret = inner[inner.len() - 1];
                        let params = inner[..inner.len() - 1].to_vec();
                        self.tcx.intern(Ty::Fn {
                            params,
                            ret,
                            variadic: false,
                        })
                    }
                    _ => {
                        // Support user-defined generic types like Box[int]
                        if let Some(sym) = self.symbols.lookup(name) {
                            let base_ty = self.get_sym_type(sym.0);
                            // Create a parameterized version with args
                            if let Ty::Class { fields, .. } = self.tcx.get(base_ty).clone() {
                                let arg_names: Vec<String> =
                                    inner.iter().map(|a| self.ty_name(*a)).collect();
                                let param_name = format!("{}[{}]", name, arg_names.join(", "));
                                // Substitute type params in fields so Box[int].value = int
                                let new_fields = if let Some(gp) =
                                    self.generic_defs.get(name).cloned()
                                {
                                    let mut subst = Substitution::new();
                                    for (tv, concrete) in gp.params.iter().zip(inner.iter()) {
                                        subst.insert(tv.id, *concrete);
                                    }
                                    fields
                                        .iter()
                                        .map(|(n, t)| (n.clone(), subst.apply(*t, &mut self.tcx)))
                                        .collect()
                                } else {
                                    fields
                                };
                                self.tcx.intern(Ty::Class {
                                    name: param_name,
                                    fields: new_fields,
                                    match_args: None,
                                })
                            } else {
                                base_ty
                            }
                        } else if let Some(alias_ty) = self.tcx.resolve_alias(name) {
                            alias_ty
                        } else if name.contains('.') {
                            // Dotted generic reference like `typing.Iterable`
                            // or `collections.abc.Mapping[K, V]` (#1578):
                            // external/forward generic — treat as Any so
                            // CPython-style annotations type-check.
                            self.tcx.any()
                        } else {
                            self.error(ty.span, format!("unknown generic type: `{name}`"));
                            self.tcx.error()
                        }
                    }
                }
            }
            TypeExpr::Optional(inner) => {
                let inner_ty = self.resolve_type_expr(inner);
                let none_ty = self.tcx.none();
                self.tcx.intern(Ty::Union(vec![inner_ty, none_ty]))
            }
            TypeExpr::Union(types) => {
                let inner: Vec<TypeId> = types.iter().map(|t| self.resolve_type_expr(t)).collect();
                self.tcx.intern(Ty::Union(inner))
            }
            TypeExpr::Fn { params, ret } => {
                let param_types: Vec<TypeId> =
                    params.iter().map(|p| self.resolve_type_expr(p)).collect();
                let ret_ty = self.resolve_type_expr(ret);
                self.tcx.intern(Ty::Fn {
                    params: param_types,
                    ret: ret_ty,
                    variadic: false,
                })
            }
            TypeExpr::Tuple(types) => {
                let inner: Vec<TypeId> = types.iter().map(|t| self.resolve_type_expr(t)).collect();
                self.tcx.intern(Ty::Tuple(inner))
            }
        }
    }

    /// ① Type-wall PoC: does the hardcoded sig table contain a `Method` whose
    /// owning class is `class_name` in `module`? Used at import time to mark a
    /// from-imported class binding so its instances can resolve method sigs.
    pub(crate) fn is_known_stdlib_class(module: &str, class_name: &str) -> bool {
        super::stdlib_sigs::STDLIB_SIGS.iter().any(|s| {
            matches!(s.kind, super::stdlib_sigs::SigKind::Method)
                && s.module == module
                && s.qualifier == class_name
        })
    }

    /// ① Type-wall PoC: map a [`CoreTy`] to a concrete scalar [`TypeId`], or
    /// `None` when the param is non-scalar / unenforceable. `Bytes` and
    /// `MemoryView`, and `Complex` have no dedicated scalar `Ty` (buffer/complex
    /// expressions infer to `Any`), so scalar rejection for them lives in the
    /// stdlib call hook rather than this positive mapper.
    pub(crate) fn core_ty_to_type_id(&self, ct: super::stdlib_sigs::CoreTy) -> Option<TypeId> {
        use super::stdlib_sigs::CoreTy;
        match ct {
            CoreTy::Int => Some(self.tcx.int()),
            CoreTy::Float => Some(self.tcx.float()),
            CoreTy::Str => Some(self.tcx.str()),
            CoreTy::Bool => Some(self.tcx.bool()),
            CoreTy::None => Some(self.tcx.none()),
            // No concrete scalar representation — never enforce as a positive
            // scalar. Buffer-ish values still reject impossible concrete scalar
            // actuals in the call hook.
            // `Typed` is handled by the bare-class branch in the hook, and
            // `List` by a negative scalar wall, not here.
            CoreTy::Bytes
            | CoreTy::MemoryView
            | CoreTy::Complex
            | CoreTy::List
            | CoreTy::Tuple
            | CoreTy::Dict
            | CoreTy::Typed
            | CoreTy::Unknown => None,
        }
    }

    /// ① Type-wall PoC: true only when `actual` is a *concrete scalar* type we
    /// are confident about (Int/Float/Str/Bool/None). Any/Error/Union/typevar/
    /// collection/class -> not concrete -> the hook skips (zero false positives).
    pub(crate) fn is_concrete_scalar(&self, actual: TypeId) -> bool {
        matches!(
            self.tcx.get(actual),
            Ty::Int | Ty::Float | Ty::Str | Ty::Bool | Ty::None
        )
    }

    pub(crate) fn types_compatible(&self, expected: TypeId, actual: TypeId) -> bool {
        if expected == actual {
            return true;
        }
        let e = self.tcx.get(expected);
        let a = self.tcx.get(actual);
        // Error types are always compatible (to avoid cascading errors)
        if e.is_error() || a.is_error() {
            return true;
        }
        // #240: Any is compatible with everything (both directions)
        if e.is_any() || a.is_any() {
            return true;
        }
        // #314: TypeVar is compatible with any type (unified during inference)
        if matches!(e, Ty::TypeVar(_)) || matches!(a, Ty::TypeVar(_)) {
            return true;
        }
        // SelfType (the `self` parameter's type) is compatible with any Class type.
        // `return self` in a method whose return type is the class name is always valid.
        if matches!(e, Ty::SelfType) || matches!(a, Ty::SelfType) {
            return true;
        }
        // #314: Parameterized class compatible with bare base class
        // (e.g., Box[T] ≈ Box, Container[int] ≈ Container)
        // but NOT differently parameterized (Box[int] ≠ Box[str])
        if let (Ty::Class { name: n1, .. }, Ty::Class { name: n2, .. }) = (e, a) {
            let base1 = n1.split('[').next().unwrap_or(n1);
            let base2 = n2.split('[').next().unwrap_or(n2);
            if base1 == base2 {
                let has1 = n1.contains('[');
                let has2 = n2.contains('[');
                if !has1 || !has2 || n1 == n2 {
                    return true;
                }
            }
            // Exception class hierarchy: all exception types are compatible
            // with each other (they all derive from BaseException).
            if is_exception_class_name(n1) && is_exception_class_name(n2) {
                return true;
            }
        }
        // PEP 589: class-form TypedDict is a structural schema at type-check
        // time but its runtime values are plain dicts.
        if let (Ty::Class { name, .. }, Ty::Dict(_, _)) = (e, a) {
            if self.typed_dict_classes.contains(name) {
                return true;
            }
        }
        // #314: Protocol structural subtyping — if expected is a protocol class,
        // check if actual class structurally satisfies it
        if let Ty::Class {
            name: proto_name, ..
        } = e
        {
            if self.protocol_registry.get(proto_name).is_some() {
                if let Ty::Class {
                    name: class_name, ..
                } = a
                {
                    let class_methods = self
                        .class_methods
                        .get(class_name)
                        .cloned()
                        .unwrap_or_default();
                    let class_attrs: HashMap<String, TypeId> = if let Ty::Class { fields, .. } = a {
                        fields.iter().cloned().collect()
                    } else {
                        HashMap::new()
                    };
                    return self.protocol_registry.satisfies(
                        proto_name,
                        &class_methods,
                        &class_attrs,
                        &self.tcx,
                    );
                }
            }
        }
        // Union compatibility: actual is compatible if it matches any member
        if let Ty::Union(members) = e {
            let members = members.clone();
            return members.iter().any(|m| self.types_compatible(*m, actual));
        }
        if let Ty::Union(members) = a {
            let members = members.clone();
            return members.iter().all(|m| self.types_compatible(expected, *m));
        }
        // Recursive collection compatibility: List[X] ≈ List[Y] when X ≈ Y,
        // similarly for Set and Dict. This handles annotations like
        // `list[dict]` where the inner type resolves to Any and must match
        // concrete types.
        if let (Ty::List(inner_e), Ty::List(inner_a)) = (e, a) {
            let (ie, ia) = (*inner_e, *inner_a);
            return self.types_compatible(ie, ia);
        }
        if let (Ty::Set(inner_e), Ty::Set(inner_a)) = (e, a) {
            let (ie, ia) = (*inner_e, *inner_a);
            return self.types_compatible(ie, ia);
        }
        if let (Ty::Dict(ke, ve), Ty::Dict(ka, va)) = (e, a) {
            let (ke, ve, ka, va) = (*ke, *ve, *ka, *va);
            return self.types_compatible(ke, ka) && self.types_compatible(ve, va);
        }
        // Recursive tuple compatibility, mirroring List/Dict. This removes the
        // param-default false positive on `def f(p: tuple[float, float] = (1, 2))`:
        // the literal `(1, 2)` infers `tuple[int, int]`, and element-wise the
        // same int->float promotion the whole-value rule allows must apply.
        // Element checks recurse with the same (expected, actual) direction, so
        // promotion stays one-way: `tuple[float, ...]` accepts an int element,
        // but `tuple[int, ...]` does NOT accept a float element.
        if let (Ty::Tuple(es), Ty::Tuple(as_)) = (e, a) {
            let es = es.clone();
            let as_ = as_.clone();
            // Bare `tuple` (no type args) imposes no element constraint — it is
            // the unparameterized collection, compatible with any tuple value.
            if es.is_empty() || as_.is_empty() {
                return true;
            }
            // Equal arity: treat as fixed-length and compare element-wise. This
            // also subsumes the case where both sides are homogeneous
            // `tuple[T, ...]` (each a 2-element `[T, Any]`), since `Any`
            // elements are universally compatible.
            if es.len() == as_.len() {
                return es
                    .iter()
                    .zip(as_.iter())
                    .all(|(&elem_e, &elem_a)| self.types_compatible(elem_e, elem_a));
            }
            // Differing arity: the only compatible shape is a homogeneous
            // `tuple[T, ...]`, parsed as a 2-element tuple whose second element
            // is the `...` ellipsis (which resolves to `Any`). The homogeneous
            // side accepts a tuple of any arity whose every element is
            // compatible with `T`.
            if es.len() == 2 && self.tcx.get(es[1]).is_any() {
                let elem_e = es[0];
                return as_
                    .iter()
                    .all(|&elem_a| self.types_compatible(elem_e, elem_a));
            }
            if as_.len() == 2 && self.tcx.get(as_[1]).is_any() {
                let elem_a = as_[0];
                return es
                    .iter()
                    .all(|&elem_e| self.types_compatible(elem_e, elem_a));
            }
            // Differing arity, neither homogeneous: genuine length mismatch.
            return false;
        }
        // Callable compatibility is structural: equal arity, parameters
        // contravariant (checked reversed), return covariant. `Any` on either
        // side of any position dominates, so an `(Any) -> Any` lambda
        // satisfies an `(Any) -> int` parameter (the mypy-accepted shape).
        if let (
            Ty::Fn {
                params: pe,
                ret: re,
                ..
            },
            Ty::Fn {
                params: pa,
                ret: ra,
                ..
            },
        ) = (e, a)
        {
            let (pe, pa, re, ra) = (pe.clone(), pa.clone(), *re, *ra);
            if pe.len() != pa.len() {
                return false;
            }
            return pe
                .iter()
                .zip(pa.iter())
                .all(|(&te, &ta)| self.types_compatible(ta, te))
                && self.types_compatible(re, ra);
        }
        // Bool is a subclass of int in Python (#1680) — `isinstance(True, int) is True`.
        // Accept bool wherever int or float is expected, and int wherever float is
        // expected (Python's numeric promotion is implicit in argument position too;
        // `math.sqrt(4)` accepts an int).
        match (e, a) {
            (Ty::Int, Ty::Bool) => true,
            (Ty::Float, Ty::Bool) | (Ty::Float, Ty::Int) => true,
            _ => false,
        }
    }

    pub(crate) fn ty_name(&self, ty: TypeId) -> String {
        match self.tcx.get(ty) {
            Ty::Never => "Never".into(),
            Ty::None => "None".into(),
            Ty::Bool => "bool".into(),
            Ty::Int => "int".into(),
            Ty::Float => "float".into(),
            Ty::Str => "str".into(),
            Ty::Any => "Any".into(),
            Ty::List(inner) => format!("list[{}]", self.ty_name(*inner)),
            Ty::Set(inner) => format!("set[{}]", self.ty_name(*inner)),
            Ty::Dict(k, v) => format!("dict[{}, {}]", self.ty_name(*k), self.ty_name(*v)),
            Ty::Tuple(ts) => {
                let parts: Vec<_> = ts.iter().map(|t| self.ty_name(*t)).collect();
                format!("tuple[{}]", parts.join(", "))
            }
            Ty::Union(ts) => {
                let parts: Vec<_> = ts.iter().map(|t| self.ty_name(*t)).collect();
                parts.join(" | ")
            }
            Ty::Fn { params, ret, .. } => {
                let ps: Vec<_> = params.iter().map(|p| self.ty_name(*p)).collect();
                format!("({}) -> {}", ps.join(", "), self.ty_name(*ret))
            }
            Ty::Class { name, .. } => name.clone(),
            Ty::Enum { name, .. } => name.clone(),
            Ty::TypeVar(id) => {
                let info = self.tcx.get_type_var(*id);
                info.name.clone()
            }
            Ty::Literal(vals) => {
                let parts: Vec<String> = vals
                    .iter()
                    .map(|v| match v {
                        super::ty::LiteralValue::Int(i) => i.to_string(),
                        super::ty::LiteralValue::Str(s) => format!("\"{s}\""),
                        super::ty::LiteralValue::Bool(b) => b.to_string(),
                    })
                    .collect();
                format!("Literal[{}]", parts.join(", "))
            }
            Ty::SelfType => "Self".into(),
            Ty::Infer(_) => "?".into(),
            Ty::Error => "<error>".into(),
        }
    }

    /// Register a Protocol class in the protocol registry.
    pub(crate) fn register_protocol(&mut self, name: &str, body: &[Spanned<Stmt>]) {
        use super::protocol::{MethodSig, Protocol};

        let mut methods = HashMap::new();
        let mut attrs = HashMap::new();

        for stmt in body {
            match &stmt.node {
                Stmt::FnDef {
                    name: method_name,
                    params,
                    return_ty,
                    ..
                } => {
                    // Skip self parameter for protocol methods
                    let param_types: Vec<TypeId> = params
                        .iter()
                        .filter(|p| p.name != "self")
                        .map(|p| self.resolve_type_expr(&p.ty))
                        .collect();
                    let ret = return_ty
                        .as_ref()
                        .map(|t| self.resolve_type_expr(t))
                        .unwrap_or(self.tcx.any());
                    methods.insert(
                        method_name.clone(),
                        MethodSig {
                            params: param_types,
                            return_type: ret,
                        },
                    );
                }
                Stmt::VarDecl {
                    name: attr_name,
                    ty,
                    ..
                } => {
                    let ty_id = self.resolve_type_expr(ty);
                    attrs.insert(attr_name.clone(), ty_id);
                }
                _ => {}
            }
        }

        self.protocol_registry.register(Protocol {
            name: name.to_string(),
            methods,
            attrs,
            runtime_checkable: false,
        });
    }

    /// Collect method signatures from a class body for protocol conformance.
    pub(crate) fn collect_class_methods(&mut self, class_name: &str, body: &[Spanned<Stmt>]) {
        use super::protocol::MethodSig;

        let mut methods = HashMap::new();
        let mut unbound_methods = HashMap::new();
        let receiver_ty = self
            .symbols
            .lookup(class_name)
            .map(|sym| self.get_sym_type(sym.0))
            .unwrap_or_else(|| {
                self.tcx.intern(Ty::Class {
                    name: class_name.to_string(),
                    fields: vec![],
                    match_args: None,
                })
            });
        for stmt in body {
            if let Stmt::FnDef {
                decorators,
                name,
                type_params,
                params,
                return_ty,
                ..
            } = &stmt.node
            {
                // Generic methods (`def meth[U](...)`) resolve their own
                // type params within the signature (PEP 695).
                let _gp = self.register_type_params(type_params);
                let param_types: Vec<TypeId> = params
                    .iter()
                    .filter(|p| p.name != "self")
                    .map(|p| self.resolve_type_expr(&p.ty))
                    .collect();
                let ret = return_ty
                    .as_ref()
                    .map(|t| self.resolve_type_expr(t))
                    .unwrap_or(self.tcx.any());
                let unbound_param_types = if decorators.is_empty() && !params.is_empty() {
                    let mut unbound_param_types = Vec::with_capacity(params.len());
                    unbound_param_types.push(receiver_ty);
                    unbound_param_types
                        .extend(params.iter().skip(1).map(|p| self.resolve_type_expr(&p.ty)));
                    Some(unbound_param_types)
                } else {
                    None
                };
                self.unregister_type_params(type_params);
                methods.insert(
                    name.clone(),
                    MethodSig {
                        params: param_types,
                        return_type: ret,
                    },
                );
                if let Some(unbound_param_types) = unbound_param_types {
                    unbound_methods.insert(
                        name.clone(),
                        MethodSig {
                            params: unbound_param_types,
                            return_type: ret,
                        },
                    );
                }
            }
        }
        if !methods.is_empty() {
            self.class_methods.insert(class_name.to_string(), methods);
        }
        if !unbound_methods.is_empty() {
            self.class_unbound_methods
                .insert(class_name.to_string(), unbound_methods);
        }
    }

    fn base_is_typed_dict(&self, expr: &Expr) -> bool {
        let name = match expr {
            Expr::Ident(name) => Some(name.as_str()),
            Expr::Attr { attr, .. } => Some(attr.as_str()),
            _ => None,
        };
        name.is_some_and(|name| name == "TypedDict" || self.typed_dict_classes.contains(name))
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Best-effort conversion of a PEP 695 type-alias *value expression* back
/// into a `TypeExpr` for compile-time annotation resolution (`x: Alias`).
///
/// Only the type-shaped subset converts (`int`, `int | str`, `list[T]`,
/// `(A, B)`, dotted names). Anything else — lambdas, calls, literals —
/// returns `None` and the alias exists purely as a runtime TypeAliasType.
pub(crate) fn expr_to_type_expr(expr: &Spanned<Expr>) -> Option<Spanned<TypeExpr>> {
    fn dotted_name(expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(n) => Some(n.clone()),
            Expr::Attr { object, attr } => Some(format!("{}.{}", dotted_name(&object.node)?, attr)),
            _ => None,
        }
    }
    let node = match &expr.node {
        Expr::Ident(n) => TypeExpr::Named(n.clone()),
        Expr::NoneLit => TypeExpr::Named("None".to_string()),
        Expr::StrLit(s) => TypeExpr::Named(crate::parser::ast::forward_ref_name(s)),
        Expr::Attr { .. } => TypeExpr::Named(dotted_name(&expr.node)?),
        Expr::BinOp {
            op: BinOp::BitOr,
            lhs,
            rhs,
        } => {
            let mut variants = Vec::new();
            match expr_to_type_expr(lhs)?.node {
                TypeExpr::Union(vs) => variants.extend(vs),
                other => variants.push(Spanned::new(other, lhs.span)),
            }
            variants.push(expr_to_type_expr(rhs)?);
            TypeExpr::Union(variants)
        }
        Expr::Index { object, index } => {
            let name = dotted_name(&object.node)?;
            let args = match &index.node {
                Expr::TupleLit(items) => items
                    .iter()
                    .map(expr_to_type_expr)
                    .collect::<Option<Vec<_>>>()?,
                _ => vec![expr_to_type_expr(index)?],
            };
            TypeExpr::Generic { name, args }
        }
        Expr::TupleLit(items) => TypeExpr::Tuple(
            items
                .iter()
                .map(expr_to_type_expr)
                .collect::<Option<Vec<_>>>()?,
        ),
        _ => return None,
    };
    Some(Spanned::new(node, expr.span))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_has_builtins() {
        let tc = TypeChecker::new();
        // print, len, range, etc. should be registered
        assert!(tc.symbols.lookup("print").is_some());
        assert!(tc.symbols.lookup("len").is_some());
        assert!(tc.symbols.lookup("range").is_some());
        assert!(tc.symbols.lookup("isinstance").is_some());
    }

    #[test]
    fn test_new_has_exceptions() {
        let tc = TypeChecker::new();
        assert!(tc.symbols.lookup("ValueError").is_some());
        assert!(tc.symbols.lookup("TypeError").is_some());
        assert!(tc.symbols.lookup("KeyError").is_some());
    }

    #[test]
    fn test_set_get_sym_type() {
        let mut tc = TypeChecker::new();
        let int_ty = tc.tcx.int();
        tc.set_sym_type(0, int_ty);
        assert_eq!(tc.get_sym_type(0), int_ty);
    }

    #[test]
    fn test_get_sym_type_unset_returns_error() {
        let tc = TypeChecker::new();
        // Very high index that was never set
        assert_eq!(tc.get_sym_type(9999), tc.tcx.error());
    }

    #[test]
    fn test_set_sym_type_resizes() {
        let mut tc = TypeChecker::new();
        let str_ty = tc.tcx.str();
        // Set at a high index — should resize internal Vec
        tc.set_sym_type(500, str_ty);
        assert_eq!(tc.get_sym_type(500), str_ty);
        // Slots beyond what builtins set are None → return error
        assert_eq!(tc.get_sym_type(499), tc.tcx.error());
    }

    #[test]
    fn test_types_compatible_same_type() {
        let tc = TypeChecker::new();
        assert!(tc.types_compatible(tc.tcx.int(), tc.tcx.int()));
        assert!(tc.types_compatible(tc.tcx.str(), tc.tcx.str()));
        assert!(tc.types_compatible(tc.tcx.none(), tc.tcx.none()));
    }

    #[test]
    fn test_types_compatible_error_always_compatible() {
        let tc = TypeChecker::new();
        assert!(tc.types_compatible(tc.tcx.error(), tc.tcx.int()));
        assert!(tc.types_compatible(tc.tcx.int(), tc.tcx.error()));
        assert!(tc.types_compatible(tc.tcx.error(), tc.tcx.error()));
    }

    #[test]
    fn test_types_compatible_any_always_compatible() {
        let tc = TypeChecker::new();
        assert!(tc.types_compatible(tc.tcx.any(), tc.tcx.int()));
        assert!(tc.types_compatible(tc.tcx.int(), tc.tcx.any()));
        assert!(tc.types_compatible(tc.tcx.any(), tc.tcx.str()));
    }

    #[test]
    fn test_types_compatible_different_primitives() {
        let tc = TypeChecker::new();
        // Mamba strict typing: int and float NOT compatible
        assert!(!tc.types_compatible(tc.tcx.int(), tc.tcx.float()));
        assert!(!tc.types_compatible(tc.tcx.int(), tc.tcx.str()));
        assert!(!tc.types_compatible(tc.tcx.str(), tc.tcx.bool()));
    }

    #[test]
    fn test_types_compatible_typevar() {
        let mut tc = TypeChecker::new();
        let var_ty = tc.tcx.intern(Ty::TypeVar(super::super::ty::TypeVarId(0)));
        // TypeVar is compatible with any type
        assert!(tc.types_compatible(var_ty, tc.tcx.int()));
        assert!(tc.types_compatible(tc.tcx.int(), var_ty));
    }

    #[test]
    fn test_types_compatible_union() {
        let mut tc = TypeChecker::new();
        let int_ty = tc.tcx.int();
        let str_ty = tc.tcx.str();
        let union_ty = tc.tcx.intern(Ty::Union(vec![int_ty, str_ty]));
        // int matches Union[int, str]
        assert!(tc.types_compatible(union_ty, int_ty));
        // str matches Union[int, str]
        assert!(tc.types_compatible(union_ty, str_ty));
        // float does NOT match Union[int, str]
        assert!(!tc.types_compatible(union_ty, tc.tcx.float()));
    }

    #[test]
    fn test_types_compatible_actual_union_all_match() {
        let mut tc = TypeChecker::new();
        let int_ty = tc.tcx.int();
        let union_int = tc.tcx.intern(Ty::Union(vec![int_ty]));
        // Union[int] as actual, int as expected → compatible
        assert!(tc.types_compatible(int_ty, union_int));
    }

    #[test]
    fn test_types_compatible_actual_union_not_all_match() {
        let mut tc = TypeChecker::new();
        let int_ty = tc.tcx.int();
        let str_ty = tc.tcx.str();
        let union_mixed = tc.tcx.intern(Ty::Union(vec![int_ty, str_ty]));
        // Union[int, str] as actual, int as expected → str not compat with int
        assert!(!tc.types_compatible(int_ty, union_mixed));
    }

    #[test]
    fn test_types_compatible_class_same_base() {
        let mut tc = TypeChecker::new();
        let c1 = tc.tcx.intern(Ty::Class {
            name: "Box".to_string(),
            fields: vec![],
            match_args: None,
        });
        let c2 = tc.tcx.intern(Ty::Class {
            name: "Box[int]".to_string(),
            fields: vec![],
            match_args: None,
        });
        // Box and Box[int] → same base, one unparameterized → compatible
        assert!(tc.types_compatible(c1, c2));
        assert!(tc.types_compatible(c2, c1));
    }

    #[test]
    fn test_types_compatible_class_different_params() {
        let mut tc = TypeChecker::new();
        let c1 = tc.tcx.intern(Ty::Class {
            name: "Box[int]".to_string(),
            fields: vec![],
            match_args: None,
        });
        let c2 = tc.tcx.intern(Ty::Class {
            name: "Box[str]".to_string(),
            fields: vec![],
            match_args: None,
        });
        // Both parameterized differently → NOT compatible
        assert!(!tc.types_compatible(c1, c2));
    }

    #[test]
    fn test_types_compatible_class_different_base() {
        let mut tc = TypeChecker::new();
        let c1 = tc.tcx.intern(Ty::Class {
            name: "Foo".to_string(),
            fields: vec![],
            match_args: None,
        });
        let c2 = tc.tcx.intern(Ty::Class {
            name: "Bar".to_string(),
            fields: vec![],
            match_args: None,
        });
        assert!(!tc.types_compatible(c1, c2));
    }

    #[test]
    fn test_ty_name_primitives() {
        let tc = TypeChecker::new();
        assert_eq!(tc.ty_name(tc.tcx.int()), "int");
        assert_eq!(tc.ty_name(tc.tcx.float()), "float");
        assert_eq!(tc.ty_name(tc.tcx.str()), "str");
        assert_eq!(tc.ty_name(tc.tcx.bool()), "bool");
        assert_eq!(tc.ty_name(tc.tcx.none()), "None");
        assert_eq!(tc.ty_name(tc.tcx.any()), "Any");
        assert_eq!(tc.ty_name(tc.tcx.never()), "Never");
        assert_eq!(tc.ty_name(tc.tcx.error()), "<error>");
    }

    #[test]
    fn test_ty_name_list() {
        let mut tc = TypeChecker::new();
        let list_int = tc.tcx.intern(Ty::List(tc.tcx.int()));
        assert_eq!(tc.ty_name(list_int), "list[int]");
    }

    #[test]
    fn test_ty_name_dict() {
        let mut tc = TypeChecker::new();
        let dict_ty = tc.tcx.intern(Ty::Dict(tc.tcx.str(), tc.tcx.int()));
        assert_eq!(tc.ty_name(dict_ty), "dict[str, int]");
    }

    #[test]
    fn test_ty_name_tuple() {
        let mut tc = TypeChecker::new();
        let tuple_ty = tc.tcx.intern(Ty::Tuple(vec![tc.tcx.int(), tc.tcx.str()]));
        assert_eq!(tc.ty_name(tuple_ty), "tuple[int, str]");
    }

    #[test]
    fn test_ty_name_union() {
        let mut tc = TypeChecker::new();
        let union_ty = tc.tcx.intern(Ty::Union(vec![tc.tcx.int(), tc.tcx.str()]));
        assert_eq!(tc.ty_name(union_ty), "int | str");
    }

    #[test]
    fn test_ty_name_fn() {
        let mut tc = TypeChecker::new();
        let fn_ty = tc.tcx.intern(Ty::Fn {
            params: vec![tc.tcx.int(), tc.tcx.str()],
            ret: tc.tcx.bool(),
            variadic: false,
        });
        assert_eq!(tc.ty_name(fn_ty), "(int, str) -> bool");
    }

    #[test]
    fn test_ty_name_class() {
        let mut tc = TypeChecker::new();
        let class_ty = tc.tcx.intern(Ty::Class {
            name: "MyClass".to_string(),
            fields: vec![],
            match_args: None,
        });
        assert_eq!(tc.ty_name(class_ty), "MyClass");
    }

    #[test]
    fn test_ty_name_enum() {
        let mut tc = TypeChecker::new();
        let enum_ty = tc.tcx.intern(Ty::Enum {
            name: "Color".to_string(),
            variants: vec![],
        });
        assert_eq!(tc.ty_name(enum_ty), "Color");
    }

    #[test]
    fn test_ty_name_self_type() {
        let mut tc = TypeChecker::new();
        let self_ty = tc.tcx.intern(Ty::SelfType);
        assert_eq!(tc.ty_name(self_ty), "Self");
    }

    #[test]
    fn test_ty_name_infer() {
        let mut tc = TypeChecker::new();
        let infer_ty = tc.tcx.intern(Ty::Infer(0));
        assert_eq!(tc.ty_name(infer_ty), "?");
    }

    #[test]
    fn test_register_type_params() {
        let mut tc = TypeChecker::new();
        let gp = tc.register_type_params(&[
            crate::parser::ast::TypeParam::plain("T"),
            crate::parser::ast::TypeParam::plain("U"),
        ]);
        assert_eq!(gp.len(), 2);
        assert_eq!(gp.params[0].name, "T");
        assert_eq!(gp.params[1].name, "U");
        // T and U should be resolvable as type aliases
        assert!(tc.tcx.resolve_alias("T").is_some());
        assert!(tc.tcx.resolve_alias("U").is_some());
    }

    #[test]
    fn test_unregister_type_params() {
        let mut tc = TypeChecker::new();
        tc.register_type_params(&[crate::parser::ast::TypeParam::plain("T")]);
        assert!(tc.tcx.resolve_alias("T").is_some());
        tc.unregister_type_params(&[crate::parser::ast::TypeParam::plain("T")]);
        assert!(tc.tcx.resolve_alias("T").is_none());
    }

    #[test]
    fn test_strict_mode_default_false() {
        let tc = TypeChecker::new();
        assert!(!tc.strict);
        assert!(!tc.no_warn_any);
    }

    #[test]
    fn test_default_impl() {
        let tc = TypeChecker::default();
        assert!(tc.symbols.lookup("print").is_some());
    }

    #[test]
    fn test_diag_level_eq() {
        assert_eq!(DiagLevel::Warning, DiagLevel::Warning);
        assert_eq!(DiagLevel::Error, DiagLevel::Error);
        assert_ne!(DiagLevel::Warning, DiagLevel::Error);
    }
}
