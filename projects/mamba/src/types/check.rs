use super::generic::{GenericParams, Substitution};
use super::protocol::ProtocolRegistry;
use super::ty::TypeVarId;
use super::{Ty, TypeContext, TypeId};
use crate::error::MambaError;
use crate::parser::ast::*;
use crate::resolve::{SymbolKind, SymbolTable};
use crate::source::span::{Span, Spanned};
use std::collections::HashMap;

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
            | "AssertionError"
            | "BufferError"
            | "EOFError"
            | "MemoryError"
            | "ConnectionError"
            | "TimeoutError"
            | "ExceptionGroup"
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
    /// Suppress Any-inference warnings (#244).
    pub no_warn_any: bool,
    errors: Vec<MambaError>,
    pub diagnostics: Vec<Diagnostic>,
    /// Generic parameter lists for functions/classes (#314).
    pub(crate) generic_defs: HashMap<String, GenericParams>,
    /// Protocol registry for structural subtyping (#314).
    pub(crate) protocol_registry: ProtocolRegistry,
    /// Counter for TypeVarId allocation.
    pub(crate) next_type_var_id: u32,
    /// Class method signatures for protocol conformance checking (#314).
    pub(crate) class_methods: HashMap<String, HashMap<String, super::protocol::MethodSig>>,
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
            no_warn_any: false,
            errors: Vec::new(),
            diagnostics: Vec::new(),
            generic_defs: HashMap::new(),
            protocol_registry: ProtocolRegistry::new(),
            next_type_var_id: 0,
            class_methods: HashMap::new(),
            current_match_subject_ty: None,
            comprehension_depth: 0,
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

    pub(crate) fn get_sym_type(&self, sym_idx: u32) -> TypeId {
        self.sym_types
            .get(sym_idx as usize)
            .and_then(|t| *t)
            .unwrap_or(self.tcx.error())
    }

    pub(crate) fn error(&mut self, span: Span, msg: impl Into<String>) {
        self.errors.push(MambaError::type_err(span, msg));
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
    pub(crate) fn register_type_params(&mut self, type_params: &[String]) -> GenericParams {
        let mut gp = GenericParams::new();
        for name in type_params {
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
    pub(crate) fn unregister_type_params(&mut self, type_params: &[String]) {
        for name in type_params {
            self.tcx.unregister_alias(name);
        }
    }

    /// Get the TypeId for a SymbolId, if known (#1190).
    pub fn get_symbol_type(&self, sym: crate::resolve::SymbolId) -> Option<crate::types::TypeId> {
        self.sym_types.get(sym.0 as usize).and_then(|t| *t)
    }

    /// Check a module. Returns accumulated errors.
    pub fn check_module(&mut self, module: &Module) -> Vec<MambaError> {
        // First pass: register all top-level function/class/enum/alias names
        for stmt in &module.stmts {
            match &stmt.node {
                Stmt::FnDef {
                    name,
                    type_params,
                    params,
                    return_ty,
                    ..
                }
                | Stmt::AsyncFnDef {
                    name,
                    type_params,
                    params,
                    return_ty,
                    ..
                } => {
                    // Register generic type params before resolving param/ret types
                    let gp = self.register_type_params(type_params);

                    let sym = self.symbols.define(name.clone(), SymbolKind::Function);
                    // Detect *args variadic parameter and exclude it from param_types.
                    // Only positional params before the *args are required at call sites.
                    let star_pos = params
                        .iter()
                        .position(|p| p.kind == crate::parser::ast::ParamKind::Star);
                    let is_variadic = star_pos.is_some()
                        || params
                            .iter()
                            .any(|p| p.kind == crate::parser::ast::ParamKind::DoubleStar);
                    let effective_params = star_pos.map_or(params.as_slice(), |pos| &params[..pos]);
                    let param_types: Vec<TypeId> = effective_params
                        .iter()
                        .filter(|p| p.kind == crate::parser::ast::ParamKind::Regular)
                        .map(|p| self.resolve_type_expr(&p.ty))
                        .collect();
                    let ret = return_ty
                        .as_ref()
                        .map(|t| self.resolve_type_expr(t))
                        .unwrap_or(self.tcx.any());
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
                    let class_ty = self.tcx.intern(Ty::Class {
                        name: name.clone(),
                        fields,
                        match_args,
                    });
                    let sym = self.symbols.define(name.clone(), SymbolKind::Class);
                    self.set_sym_type(sym.0, class_ty);

                    if !gp.is_empty() {
                        self.generic_defs.insert(name.clone(), gp);
                    }

                    // Collect class methods for protocol conformance
                    self.collect_class_methods(name, body);

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
                Stmt::TypeAlias { name, value, .. } => {
                    let resolved = self.resolve_type_expr(value);
                    self.tcx.register_alias(name.clone(), resolved);
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
                    if let Some(import_names) = names {
                        for (name, alias) in import_names {
                            let effective = alias.as_ref().unwrap_or(name);
                            if self.symbols.lookup(effective).is_none() {
                                let sym =
                                    self.symbols.define(effective.clone(), SymbolKind::Variable);
                                self.set_sym_type(sym.0, any_ty);
                            }
                        }
                    } else if let Some(alias) = module_alias {
                        if self.symbols.lookup(alias).is_none() {
                            let sym = self.symbols.define(alias.clone(), SymbolKind::Variable);
                            self.set_sym_type(sym.0, any_ty);
                        }
                    } else if let Some(root) = module.first() {
                        if self.symbols.lookup(root).is_none() {
                            let sym = self.symbols.define(root.clone(), SymbolKind::Variable);
                            self.set_sym_type(sym.0, any_ty);
                        }
                    }
                }
                _ => {}
            }
        }

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
                "set" | "frozenset" => self.tcx.any(),
                // `type` as a type expression (e.g. `type[BaseModel]` bare name):
                // the class-object type is represented as Any for now.
                "type" | "object" => self.tcx.any(),
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
        // and similarly for Dict. This handles annotations like `list[dict]`
        // where the inner type resolves to Any and must match concrete types.
        if let (Ty::List(inner_e), Ty::List(inner_a)) = (e, a) {
            let (ie, ia) = (*inner_e, *inner_a);
            return self.types_compatible(ie, ia);
        }
        if let (Ty::Dict(ke, ve), Ty::Dict(ka, va)) = (e, a) {
            let (ke, ve, ka, va) = (*ke, *ve, *ka, *va);
            return self.types_compatible(ke, ka) && self.types_compatible(ve, va);
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
        for stmt in body {
            if let Stmt::FnDef {
                name,
                params,
                return_ty,
                ..
            } = &stmt.node
            {
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
                    name.clone(),
                    MethodSig {
                        params: param_types,
                        return_type: ret,
                    },
                );
            }
        }
        if !methods.is_empty() {
            self.class_methods.insert(class_name.to_string(), methods);
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
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
        let gp = tc.register_type_params(&["T".to_string(), "U".to_string()]);
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
        tc.register_type_params(&["T".to_string()]);
        assert!(tc.tcx.resolve_alias("T").is_some());
        tc.unregister_type_params(&["T".to_string()]);
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
