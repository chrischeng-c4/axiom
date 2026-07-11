use super::context::AliasIdentity;
use super::generic::{
    bind_explicit_type_args, bind_explicit_type_args_without_bounds, complete_type_args,
    GenericParams, Substitution,
};
use super::protocol::ProtocolRegistry;
use super::stdlib_typespec::{TypeParamSpecId, TypeSpecId};
use super::ty::{
    AliasInstanceId, CallableParam, CallableParamKind, ClassRole, ExternalCallable,
    ExternalCallableAccess, ExternalCallableRuntimeKind, ExternalClass, ExternalValue,
    TypeParamDefault, TypeVarId, TypeVarKind, UserClass,
};
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

fn is_dataclass_decorator(expr: &Expr) -> bool {
    match expr {
        Expr::Ident(n) => n == "dataclass",
        Expr::Attr { attr, .. } => attr == "dataclass",
        Expr::Call { func, .. } => is_dataclass_decorator(&func.node),
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

/// Numeric builtin reached by a user class's base chain (#1031). `bool`
/// cannot be subclassed in Python (`TypeError: type 'bool' is not an
/// acceptable base type`), so the only reachable roots are `int` and
/// `float`; kept distinct because int-only contexts (`~`, `<<`, `>>`) must
/// still reject a `float`-derived class exactly like a bare `float` would.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NumericRoot {
    Int,
    Float,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClassPatternTarget {
    Instance(TypeId),
    Unknown,
    Invalid,
}

const MAX_TYPE_COMPATIBILITY_DEPTH: usize = 128;

/// Type checker: walks the AST, resolves names, and checks types.
#[derive(Debug, Clone)]
pub(crate) struct FunctionParamSig {
    pub(crate) name: String,
    pub(crate) ty: TypeId,
    pub(crate) kind: ParamKind,
    pub(crate) pos_only: bool,
    pub(crate) kw_only: bool,
    pub(crate) has_default: bool,
}

impl FunctionParamSig {
    pub(crate) fn into_callable_param(self) -> CallableParam {
        let kind = match self.kind {
            ParamKind::Star => CallableParamKind::VarPos,
            ParamKind::DoubleStar => CallableParamKind::VarKw,
            ParamKind::Regular if self.pos_only => CallableParamKind::PosOnly,
            ParamKind::Regular if self.kw_only => CallableParamKind::KwOnly,
            ParamKind::Regular => CallableParamKind::PosOrKw,
        };
        CallableParam {
            name: Some(self.name),
            ty: self.ty,
            kind,
            has_default: self.has_default,
        }
    }

    pub(crate) fn from_callable_param(param: CallableParam) -> Self {
        // An empty name is not a valid Python identifier. It preserves an
        // unnamed positional contract without making keyword binding guess.
        let name = param.name.unwrap_or_default();
        let (kind, pos_only, kw_only) = match param.kind {
            CallableParamKind::PosOnly => (ParamKind::Regular, true, false),
            CallableParamKind::PosOrKw => (ParamKind::Regular, false, false),
            CallableParamKind::VarPos => (ParamKind::Star, false, false),
            CallableParamKind::KwOnly => (ParamKind::Regular, false, true),
            CallableParamKind::VarKw => (ParamKind::DoubleStar, false, false),
        };
        Self {
            name,
            ty: param.ty,
            kind,
            pos_only,
            kw_only,
            has_default: param.has_default,
        }
    }
}

#[derive(Debug, Clone)]
struct TypeAliasDef {
    name: String,
    params: GenericParams,
    captures: Vec<TypeVarId>,
    value: Spanned<TypeExpr>,
    template: Option<TypeId>,
    resolving: bool,
}

fn parse_forward_ref_type_expr(source: &str, span: Span) -> Option<Spanned<TypeExpr>> {
    if source.contains(['\n', '\r']) {
        return None;
    }
    let wrapper = format!("def __mamba_forward_ref(value: {source}) -> None:\n    pass\n");
    let module = crate::parser::parse(&wrapper, span.file).ok()?;
    let Stmt::FnDef { params, .. } = &module.stmts.first()?.node else {
        return None;
    };
    let mut ty = params.first()?.ty.clone();

    fn respan(ty: &mut Spanned<TypeExpr>, span: Span) {
        ty.span = span;
        match &mut ty.node {
            TypeExpr::Generic { args, .. } | TypeExpr::Union(args) | TypeExpr::Tuple(args) => {
                for arg in args {
                    respan(arg, span);
                }
            }
            TypeExpr::Optional(inner) => respan(inner, span),
            TypeExpr::Fn { params, ret } => {
                for param in params {
                    respan(param, span);
                }
                respan(ret, span);
            }
            TypeExpr::Named(_) => {}
        }
    }

    respan(&mut ty, span);
    Some(ty)
}

fn collect_same_scope_stmts<'a>(
    stmts: &'a [Spanned<Stmt>],
    collected: &mut Vec<&'a Spanned<Stmt>>,
) {
    for stmt in stmts {
        collected.push(stmt);
        match &stmt.node {
            Stmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
            } => {
                collect_same_scope_stmts(body, collected);
                for handler in handlers {
                    collect_same_scope_stmts(&handler.body, collected);
                }
                if let Some(body) = else_body {
                    collect_same_scope_stmts(body, collected);
                }
                if let Some(body) = finally_body {
                    collect_same_scope_stmts(body, collected);
                }
            }
            Stmt::If {
                body,
                elif_clauses,
                else_body,
                ..
            } => {
                collect_same_scope_stmts(body, collected);
                for (_, body) in elif_clauses {
                    collect_same_scope_stmts(body, collected);
                }
                if let Some(body) = else_body {
                    collect_same_scope_stmts(body, collected);
                }
            }
            Stmt::While {
                body, else_body, ..
            }
            | Stmt::For {
                body, else_body, ..
            }
            | Stmt::AsyncFor {
                body, else_body, ..
            } => {
                collect_same_scope_stmts(body, collected);
                if let Some(body) = else_body {
                    collect_same_scope_stmts(body, collected);
                }
            }
            Stmt::With { body, .. } | Stmt::AsyncWith { body, .. } => {
                collect_same_scope_stmts(body, collected)
            }
            Stmt::Match { arms, .. } => {
                for arm in arms {
                    collect_same_scope_stmts(&arm.body, collected);
                }
            }
            _ => {}
        }
    }
}

fn record_binding_event(bindings: &mut HashMap<String, usize>, name: &str) {
    *bindings.entry(name.to_string()).or_default() += 1;
}

fn record_target_binding_events(expr: &Expr, bindings: &mut HashMap<String, usize>) {
    match expr {
        Expr::Ident(name) => record_binding_event(bindings, name),
        Expr::TupleLit(items) | Expr::UnpackTarget(items) => {
            for item in items {
                record_target_binding_events(&item.node, bindings);
            }
        }
        Expr::Starred(inner) => record_target_binding_events(&inner.node, bindings),
        _ => {}
    }
}

fn record_pattern_binding_events(pattern: &Pattern, bindings: &mut HashMap<String, usize>) {
    match pattern {
        Pattern::Binding(name) => record_binding_event(bindings, name),
        Pattern::Constructor { fields, .. } => {
            for field in fields {
                record_binding_event(bindings, field);
            }
        }
        Pattern::Or(patterns) | Pattern::Sequence(patterns) => {
            for pattern in patterns {
                record_pattern_binding_events(&pattern.node, bindings);
            }
        }
        Pattern::Mapping { pairs, rest } => {
            for (_, pattern) in pairs {
                record_pattern_binding_events(&pattern.node, bindings);
            }
            if let Some(rest) = rest {
                record_binding_event(bindings, rest);
            }
        }
        Pattern::ClassPattern { patterns, .. } => {
            for (_, pattern) in patterns {
                record_pattern_binding_events(&pattern.node, bindings);
            }
        }
        Pattern::Star(Some(name)) => record_binding_event(bindings, name),
        Pattern::As { pattern, name } => {
            record_pattern_binding_events(&pattern.node, bindings);
            record_binding_event(bindings, name);
        }
        Pattern::Wildcard | Pattern::Literal(_) | Pattern::Star(None) => {}
    }
}

fn collect_same_scope_binding_events(
    stmts: &[Spanned<Stmt>],
    bindings: &mut HashMap<String, usize>,
) {
    for stmt in stmts {
        match &stmt.node {
            Stmt::VarDecl { name, .. }
            | Stmt::BareAnnotation { name, .. }
            | Stmt::TypeAlias { name, .. }
            | Stmt::FnDef { name, .. }
            | Stmt::AsyncFnDef { name, .. }
            | Stmt::ClassDef { name, .. }
            | Stmt::EnumDef { name, .. } => record_binding_event(bindings, name),
            Stmt::Assign { target, .. } | Stmt::AugAssign { target, .. } | Stmt::Del(target) => {
                record_target_binding_events(&target.node, bindings);
            }
            Stmt::If {
                body,
                elif_clauses,
                else_body,
                ..
            } => {
                collect_same_scope_binding_events(body, bindings);
                for (_, body) in elif_clauses {
                    collect_same_scope_binding_events(body, bindings);
                }
                if let Some(body) = else_body {
                    collect_same_scope_binding_events(body, bindings);
                }
            }
            Stmt::While {
                body, else_body, ..
            } => {
                collect_same_scope_binding_events(body, bindings);
                if let Some(body) = else_body {
                    collect_same_scope_binding_events(body, bindings);
                }
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
                for target in targets {
                    record_binding_event(bindings, target);
                }
                collect_same_scope_binding_events(body, bindings);
                if let Some(body) = else_body {
                    collect_same_scope_binding_events(body, bindings);
                }
            }
            Stmt::Match { arms, .. } => {
                for arm in arms {
                    record_pattern_binding_events(&arm.pattern.node, bindings);
                    collect_same_scope_binding_events(&arm.body, bindings);
                }
            }
            Stmt::Import {
                module,
                names,
                module_alias,
            } => {
                if let Some(names) = names {
                    for (name, alias) in names {
                        record_binding_event(bindings, alias.as_ref().unwrap_or(name));
                    }
                    if names.is_empty() {
                        if let Some(name) = module_alias.as_ref().or_else(|| module.first()) {
                            record_binding_event(bindings, name);
                        }
                    }
                } else if let Some(name) = module_alias.as_ref().or_else(|| module.first()) {
                    record_binding_event(bindings, name);
                }
            }
            Stmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
            } => {
                collect_same_scope_binding_events(body, bindings);
                for handler in handlers {
                    if let Some(name) = &handler.name {
                        record_binding_event(bindings, name);
                    }
                    collect_same_scope_binding_events(&handler.body, bindings);
                }
                if let Some(body) = else_body {
                    collect_same_scope_binding_events(body, bindings);
                }
                if let Some(body) = finally_body {
                    collect_same_scope_binding_events(body, bindings);
                }
            }
            Stmt::With { items, body } | Stmt::AsyncWith { items, body } => {
                for item in items {
                    if let Some(name) = &item.alias {
                        record_binding_event(bindings, name);
                    }
                }
                collect_same_scope_binding_events(body, bindings);
            }
            Stmt::Pass
            | Stmt::Break
            | Stmt::Continue
            | Stmt::Return(_)
            | Stmt::Raise { .. }
            | Stmt::Assert { .. }
            | Stmt::Global(_)
            | Stmt::Nonlocal(_)
            | Stmt::ExprStmt(_) => {}
        }
    }
}

pub(crate) fn same_scope_binding_events(stmts: &[Spanned<Stmt>]) -> HashMap<String, usize> {
    let mut bindings = HashMap::new();
    collect_same_scope_binding_events(stmts, &mut bindings);
    let mut walrus_bindings = Vec::new();
    crate::resolve::pass::collect_walrus_targets_in_stmts(stmts, &mut walrus_bindings);
    for name in walrus_bindings {
        record_binding_event(&mut bindings, &name);
    }
    bindings
}

fn direct_assignment_counts(stmts: &[Spanned<Stmt>]) -> HashMap<String, usize> {
    let mut assignments = HashMap::new();
    for stmt in stmts {
        let Stmt::Assign { target, .. } = &stmt.node else {
            continue;
        };
        let Expr::Ident(name) = &target.node else {
            continue;
        };
        *assignments.entry(name.clone()).or_default() += 1;
    }
    assignments
}

#[derive(Clone)]
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
    /// Runtime/JIT execution mode: unresolved names should become runtime
    /// NameError instead of compile-time undefined-name diagnostics.
    pub allow_runtime_unresolved_names: bool,
    errors: Vec<MambaError>,
    pub diagnostics: Vec<Diagnostic>,
    /// Generic parameter lists for functions/classes (#314).
    pub(crate) generic_defs: HashMap<SymbolId, GenericParams>,
    /// PEP 695 aliases keyed by their lexical declaration identity.
    type_alias_defs: HashMap<SymbolId, TypeAliasDef>,
    /// Source identity for idempotent header scans and duplicate diagnostics.
    type_alias_declarations: HashMap<SymbolId, Span>,
    /// Non-zero while a scope's declaration pass is still incomplete.
    preregister_depth: u32,
    /// Binding-event counts for the current preregistration scope. Recursive
    /// compound bodies reuse this map because they do not introduce a scope.
    preregister_binding_events: HashMap<String, usize>,
    /// Bare assignments that occur directly in the current statement sequence.
    /// When these account for every event of a name, aliases can be replayed in
    /// source order instead of being collapsed as control-flow-ambiguous.
    preregister_direct_assignments: HashMap<String, usize>,
    /// Names whose assignments target an outer scope via global/nonlocal.
    preregister_declared_bindings: HashSet<String>,
    /// Aliases shadowed by nested PEP 695 type-parameter scopes. Each
    /// `register_type_params` call pushes one frame and cleanup restores it.
    type_param_alias_scopes: Vec<Vec<(String, TypeVarId, Option<TypeId>)>>,
    /// Semantic annotation results keyed by source span. Lowering consumes
    /// these instead of independently re-resolving class/generic annotations.
    resolved_type_exprs: HashMap<Span, TypeId>,
    /// Declaration lookup retained while call sites migrate to intrinsic
    /// `Ty::Fn::signature` metadata.
    pub(crate) function_param_sigs: HashMap<SymbolId, Vec<FunctionParamSig>>,
    /// Canonical preregistered function types keyed by declaration binding.
    /// Body checking allocates temporary TypeVars, so declaration reassertion
    /// must restore this type rather than rebuilding from those fresh ids.
    pub(crate) function_declaration_types: HashMap<SymbolId, TypeId>,
    /// Stable symbol identity for each declaration occurrence. Python names are
    /// rebound in source order, so same-named declarations cannot share the
    /// symbol that owns their generic, nominal, and signature metadata.
    pub(crate) declaration_symbols: HashMap<usize, SymbolId>,
    /// Active function scopes. Class and comprehension scopes also use the
    /// symbol-table stack, but they are not valid `nonlocal` targets.
    pub(crate) function_scope_stack: Vec<usize>,
    /// Active executable class namespaces. Methods skip these when choosing
    /// their lexical parent, while nested class bodies still use them locally.
    pub(crate) class_scope_stack: Vec<usize>,
    /// Protocol registry for structural subtyping (#314).
    pub(crate) protocol_registry: ProtocolRegistry,
    /// Protocol definitions keyed by nominal class identity. The name-keyed
    /// registry remains as a fallback for non-user/native class types.
    pub(crate) protocols_by_symbol: HashMap<SymbolId, super::protocol::Protocol>,
    /// Class method signatures for protocol conformance checking (#314).
    pub(crate) class_methods: HashMap<String, HashMap<String, super::protocol::MethodSig>>,
    /// User-class method signatures keyed by the owning class symbol.
    pub(crate) class_methods_by_symbol:
        HashMap<SymbolId, HashMap<String, super::protocol::MethodSig>>,
    /// Named/kinded method parameters retained for keyword argument matching.
    pub(crate) class_method_param_sigs: HashMap<SymbolId, HashMap<String, Vec<FunctionParamSig>>>,
    /// Decorated method declarations whose runtime call shape is not faithfully
    /// represented by the single-signature protocol map.
    pub(crate) protocol_indeterminate_methods: HashMap<SymbolId, HashSet<String>>,
    /// Method-local PEP 695 parameters keyed by owning class and method name.
    pub(crate) class_method_generic_defs: HashMap<(SymbolId, String), GenericParams>,
    /// Class method signatures for bare-class unbound calls such as
    /// `Box.get(obj, arg)`. These include the explicit receiver parameter.
    pub(crate) class_unbound_methods:
        HashMap<SymbolId, HashMap<String, super::protocol::MethodSig>>,
    pub(crate) class_unbound_method_param_sigs:
        HashMap<SymbolId, HashMap<String, Vec<FunctionParamSig>>>,
    /// Property access contracts keyed by nominal class identity. Getter and
    /// setter declarations intentionally share a Python name, so the ordinary
    /// last-definition-wins method map cannot represent both contracts.
    pub(crate) class_property_getters: HashMap<SymbolId, HashMap<String, TypeId>>,
    pub(crate) class_property_setters: HashMap<SymbolId, HashMap<String, TypeId>>,
    /// User classes declared with `TypedDict` in their base chain. Runtime
    /// instances of these classes are plain dict values, so a variable annotated
    /// as the TypedDict class accepts dict literals/values.
    pub(crate) typed_dict_classes: HashSet<String>,
    pub(crate) typed_dict_class_symbols: HashSet<SymbolId>,
    /// User classes that are BARE: no base class (other than `object`) and no
    /// methods. A bare class instance (`class _W: pass` → `_W()`) can satisfy
    /// neither a protocol (it has no dunders) nor a nominal type (it has no
    /// superclass), so the ① hook rejects it against a `CoreTy::Typed` param.
    /// Classes with any base or any method are NOT recorded here, so they are
    /// always skipped — keeping the bare-class rejection false-positive-clean.
    pub(crate) user_bare_classes: std::collections::HashSet<String>,
    pub(crate) user_bare_class_symbols: HashSet<SymbolId>,
    /// User classes whose base chain reaches a numeric builtin (#1031),
    /// mapped to which builtin (`int` or `float`) they ultimately derive
    /// from. Lets numeric-only compile checks (unary `-`/`+`/`~`, shifts,
    /// ...) accept `class P(int): pass; -P(7)` without loosening rejection
    /// of genuinely non-numeric classes. See [`NumericRoot`] and
    /// [`TypeChecker::numeric_root`].
    pub(crate) numeric_derived_classes: std::collections::HashMap<String, NumericRoot>,
    pub(crate) numeric_derived_class_symbols: HashMap<SymbolId, NumericRoot>,
    /// #1041: maps a user class name to the identifier bases it declares
    /// (`class W(V): pass` -> `"W" -> ["V"]`), letting the unary/shift dunder
    /// wall (`check_expr.rs::class_defines_dunder`) walk the inheritance
    /// chain to find a dunder override several levels up, not just on the
    /// immediate class. Sibling of `numeric_derived_classes` above, which
    /// only tracks chains reaching `int`/`float`; this map is general-purpose
    /// and unfiltered (any identifier base, not just numeric ones).
    pub(crate) class_bases: HashMap<String, Vec<String>>,
    /// Nominal base graph for user-class semantics that cannot tolerate
    /// same-named nested declarations sharing the legacy name map.
    pub(crate) class_base_symbols: HashMap<SymbolId, Vec<SymbolId>>,
    /// Classes whose base graph contains an external or unresolved base.
    pub(crate) class_inheritance_open: HashSet<SymbolId>,
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
    /// Import provenance for stdlib call resolution. Maps a bound symbol to
    /// its `(dotted-module, qualifier)` origin so generated and compact
    /// signatures can recover the canonical callable.
    ///
    /// Symbol identity prevents a parameter or nested local with the same text
    /// from inheriting an outer import's contract.
    pub(crate) import_origins: HashMap<SymbolId, (String, String)>,
    /// Imported class origin of the stdlib instance a local binding holds,
    /// populated when a var is assigned
    /// `object.__new__(Cls)` or `Cls(...)` where `Cls` is a known imported
    /// stdlib class. The snapshot survives later rebinding of `Cls`.
    pub(crate) instance_origins: HashMap<SymbolId, (String, String)>,
    /// #1021: maps a local binding to the native-class name it aliases
    /// when the var is assigned a bare *class reference* (not a call) to a
    /// `NATIVE_CTOR_CLASSES` entry — e.g. `_Queue = queue.Queue` (the
    /// perf-tier "hoist convention (#2097)" that binds `module.Cls` locally
    /// to avoid a per-iteration module-attribute lookup before calling it as
    /// `_Queue()`). Lets `native_ctor_class_call` recognize the *aliased*
    /// constructor call the same way it recognizes the direct
    /// `queue.Queue()` form. Cleared on any reassignment to something else,
    /// mirroring `instance_origins`'s discipline.
    pub(crate) class_ref_origins: HashMap<SymbolId, &'static str>,
    /// Primitive builtin class-object aliases keyed by the active binding.
    /// Builtin constructors currently use `Ty::Fn`, so this preserves their
    /// class identity without treating ordinary function aliases as classes.
    pub(crate) builtin_class_aliases: HashMap<SymbolId, TypeId>,
    /// Function-local names predeclared as `Any` solely to enforce Python's
    /// lexical-local lookup rule. Their first direct assignment replaces the
    /// placeholder with the inferred RHS type.
    pub(crate) inferred_local_placeholders: HashSet<SymbolId>,
    /// Original symbol ids for builtin functions and classes registered during
    /// TypeChecker construction. If the current lookup no longer matches this
    /// id, user code has shadowed the builtin.
    pub(crate) builtin_symbols: HashMap<String, SymbolId>,
    /// Context-local materialization caches for generated typeshed contracts.
    /// Generated ids are stable data identities; TypeId/TypeVarId values belong
    /// to this checker and must never be stored in the generated manifest.
    pub(crate) stdlib_spec_types: HashMap<TypeSpecId, TypeId>,
    pub(crate) stdlib_spec_type_params: HashMap<TypeParamSpecId, TypeVarId>,
    pub(crate) stdlib_spec_type_param_initialized: HashSet<TypeParamSpecId>,
    pub(crate) stdlib_spec_type_param_initializing: HashSet<TypeParamSpecId>,
    pub(crate) stdlib_spec_type_param_failed: HashSet<TypeParamSpecId>,
    /// One top-level generated TypeSpec materialization is transactional. A
    /// failed recursive alias must not leave unresolved AliasRefs in the cache
    /// or targets committed by another member of the failed alias cycle.
    pub(crate) stdlib_spec_materialization_depth: usize,
    pub(crate) stdlib_spec_materialization_nodes: Vec<TypeSpecId>,
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
            allow_runtime_unresolved_names: false,
            errors: Vec::new(),
            diagnostics: Vec::new(),
            generic_defs: HashMap::new(),
            type_alias_defs: HashMap::new(),
            type_alias_declarations: HashMap::new(),
            preregister_depth: 0,
            preregister_binding_events: HashMap::new(),
            preregister_direct_assignments: HashMap::new(),
            preregister_declared_bindings: HashSet::new(),
            type_param_alias_scopes: Vec::new(),
            resolved_type_exprs: HashMap::new(),
            function_param_sigs: HashMap::new(),
            function_declaration_types: HashMap::new(),
            declaration_symbols: HashMap::new(),
            function_scope_stack: Vec::new(),
            class_scope_stack: Vec::new(),
            protocol_registry: ProtocolRegistry::new(),
            protocols_by_symbol: HashMap::new(),
            class_methods: HashMap::new(),
            class_methods_by_symbol: HashMap::new(),
            class_method_param_sigs: HashMap::new(),
            protocol_indeterminate_methods: HashMap::new(),
            class_method_generic_defs: HashMap::new(),
            class_unbound_methods: HashMap::new(),
            class_unbound_method_param_sigs: HashMap::new(),
            class_property_getters: HashMap::new(),
            class_property_setters: HashMap::new(),
            typed_dict_classes: HashSet::new(),
            typed_dict_class_symbols: HashSet::new(),
            user_bare_classes: std::collections::HashSet::new(),
            user_bare_class_symbols: HashSet::new(),
            numeric_derived_classes: std::collections::HashMap::new(),
            numeric_derived_class_symbols: HashMap::new(),
            class_bases: HashMap::new(),
            class_base_symbols: HashMap::new(),
            class_inheritance_open: HashSet::new(),
            current_match_subject_ty: None,
            comprehension_depth: 0,
            import_origins: HashMap::new(),
            instance_origins: HashMap::new(),
            class_ref_origins: HashMap::new(),
            builtin_class_aliases: HashMap::new(),
            inferred_local_placeholders: HashSet::new(),
            builtin_symbols: HashMap::new(),
            stdlib_spec_types: HashMap::new(),
            stdlib_spec_type_params: HashMap::new(),
            stdlib_spec_type_param_initialized: HashSet::new(),
            stdlib_spec_type_param_initializing: HashSet::new(),
            stdlib_spec_type_param_failed: HashSet::new(),
            stdlib_spec_materialization_depth: 0,
            stdlib_spec_materialization_nodes: Vec::new(),
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

    fn declaration_key(stmt: &Spanned<Stmt>) -> usize {
        stmt as *const Spanned<Stmt> as usize
    }

    pub(crate) fn declaration_symbol(&self, stmt: &Spanned<Stmt>) -> Option<SymbolId> {
        self.declaration_symbols
            .get(&Self::declaration_key(stmt))
            .copied()
    }

    pub(crate) fn class_base_symbol_named(
        &self,
        class_symbol: SymbolId,
        base_name: &str,
    ) -> Option<SymbolId> {
        self.class_base_symbols
            .get(&class_symbol)?
            .iter()
            .copied()
            .find(|base_symbol| {
                self.get_symbol_type(*base_symbol).is_some_and(|ty| {
                    matches!(
                        self.tcx.get(ty),
                        Ty::Class { name, .. } if name == base_name
                    )
                })
            })
    }

    fn user_class_is_subclass(&self, child: SymbolId, parent: SymbolId) -> bool {
        let mut pending = vec![child];
        let mut seen = HashSet::new();
        while let Some(symbol) = pending.pop() {
            if !seen.insert(symbol) {
                continue;
            }
            if symbol == parent {
                return true;
            }
            if let Some(bases) = self.class_base_symbols.get(&symbol) {
                pending.extend(bases.iter().copied());
            }
        }
        false
    }

    pub(crate) fn is_unshadowed_builtin(&self, name: &str) -> bool {
        let Some(builtin_id) = self.builtin_symbols.get(name).copied() else {
            return false;
        };
        self.symbols.lookup(name) == Some(builtin_id)
    }

    pub(crate) fn builtin_class_pattern_instance(&mut self, name: &str) -> Option<TypeId> {
        match name {
            "bool" => Some(self.tcx.bool()),
            "float" => Some(self.tcx.float()),
            "int" => Some(self.tcx.int()),
            "str" => Some(self.tcx.str()),
            "list" => Some(self.tcx.intern(Ty::List(self.tcx.any()))),
            "set" => Some(self.tcx.intern(Ty::Set(self.tcx.any()))),
            "dict" => Some(self.tcx.intern(Ty::Dict(self.tcx.any(), self.tcx.any()))),
            "tuple" => Some(self.tcx.intern(Ty::Tuple(Vec::new()))),
            "bytearray" | "bytes" | "complex" | "frozenset" | "memoryview" | "range"
            | "slice" | "type" => {
                Some(self.external_class_instance("builtins", name, Vec::new()))
            }
            "object" => Some(self.tcx.any()),
            _ => None,
        }
    }

    pub(crate) fn builtin_class_alias_value(&mut self, value: &Spanned<Expr>) -> Option<TypeId> {
        let Expr::Ident(name) = &value.node else {
            return None;
        };
        if self.is_unshadowed_builtin(name) {
            return self.builtin_class_pattern_instance(name);
        }
        let symbol = self.symbols.lookup(name)?;
        self.builtin_class_aliases.get(&symbol).copied()
    }

    pub(crate) fn class_object_instance_type(
        &mut self,
        value: &Spanned<Expr>,
        actual: TypeId,
    ) -> Option<TypeId> {
        if matches!(
            self.tcx.get(actual),
            Ty::Class {
                role: ClassRole::Object,
                ..
            }
        ) {
            return Some(self.with_class_role(actual, ClassRole::Instance));
        }
        match &value.node {
            Expr::Ident(name) => {
                if self.is_unshadowed_builtin(name) {
                    return self.builtin_class_pattern_instance(name);
                }
                let symbol = self.symbols.lookup(name)?;
                if let Some(instance) = self.builtin_class_aliases.get(&symbol).copied() {
                    return Some(instance);
                }
                let (module, qualifier) = self.import_origins.get(&symbol)?.clone();
                let (module, qualifier) =
                    super::stdlib_typespec::exported_class(&module, &qualifier)?;
                Some(self.external_class_instance(module, qualifier, Vec::new()))
            }
            Expr::Attr { object, attr } => {
                let Expr::Ident(module_alias) = &object.node else {
                    return None;
                };
                let symbol = self.symbols.lookup(module_alias)?;
                let (module, qualifier) = self.import_origins.get(&symbol)?.clone();
                if !qualifier.is_empty() {
                    return None;
                }
                let (module, qualifier) = super::stdlib_typespec::exported_class(&module, attr)?;
                Some(self.external_class_instance(module, qualifier, Vec::new()))
            }
            _ => None,
        }
    }

    pub(crate) fn refine_class_object_actual(
        &mut self,
        expected: TypeId,
        actual: TypeId,
        value: &Spanned<Expr>,
    ) -> TypeId {
        let expects_class_info = matches!(
            self.tcx.get(expected),
            Ty::Class {
                external: Some(external),
                ..
            } if (external.module.as_str(), external.name.as_str())
                == ("builtins", "_ClassInfo")
        );
        if expects_class_info {
            return self.refine_class_info_actual(value, actual);
        }
        if !matches!(self.tcx.get(expected), Ty::TypeObject(_)) {
            return actual;
        }
        let Some(instance) = self.class_object_instance_type(value, actual) else {
            return actual;
        };
        self.tcx.intern(Ty::TypeObject(instance))
    }

    fn refine_class_info_actual(&mut self, value: &Spanned<Expr>, actual: TypeId) -> TypeId {
        if let Some(instance) = self.class_object_instance_type(value, actual) {
            return self.tcx.intern(Ty::TypeObject(instance));
        }
        let (Expr::TupleLit(values), Ty::Tuple(items)) =
            (&value.node, self.tcx.get(actual).clone())
        else {
            return actual;
        };
        if values.len() != items.len() {
            return actual;
        }
        let items = values
            .iter()
            .zip(items)
            .map(|(value, item)| self.refine_class_info_actual(value, item))
            .collect();
        self.tcx.intern(Ty::Tuple(items))
    }

    pub(crate) fn set_builtin_class_alias(
        &mut self,
        symbol: SymbolId,
        instance_ty: Option<TypeId>,
    ) {
        if let Some(instance_ty) = instance_ty {
            self.builtin_class_aliases.insert(symbol, instance_ty);
        } else {
            self.builtin_class_aliases.remove(&symbol);
        }
    }

    fn shallow_assignment_type(&mut self, value: &Spanned<Expr>) -> Option<TypeId> {
        match &value.node {
            Expr::IntLit(_) => Some(self.tcx.int()),
            Expr::FloatLit(_) => Some(self.tcx.float()),
            Expr::BoolLit(_) => Some(self.tcx.bool()),
            Expr::StrLit(_) => Some(self.tcx.str()),
            Expr::NoneLit => Some(self.tcx.none()),
            Expr::Ident(name) => {
                let symbol = self.symbols.lookup(name)?;
                let ty = self.get_sym_type(symbol.0);
                (!matches!(self.tcx.get(ty), Ty::Any | Ty::Error)).then_some(ty)
            }
            Expr::Call { func, .. } => {
                if let Some(class_name) = self.native_ctor_class_call(func) {
                    return Some(self.tcx.intern(Ty::Class {
                        name: class_name.to_string(),
                        role: ClassRole::Instance,
                        user: None,
                        external: None,
                        fields: Vec::new(),
                        match_args: None,
                    }));
                }
                let callee_ty = match &func.node {
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
                    _ => return None,
                };
                match self.tcx.get(callee_ty).clone() {
                    Ty::Class {
                        role: ClassRole::Object,
                        ..
                    } => Some(self.with_class_role(callee_ty, ClassRole::Instance)),
                    Ty::Fn { ret, .. } if !matches!(self.tcx.get(ret), Ty::Any | Ty::Error) => {
                        Some(ret)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub(crate) fn seed_inferred_local_placeholders(&mut self, stmts: &[Spanned<Stmt>]) {
        let binding_events = same_scope_binding_events(stmts);

        let mut statements = Vec::new();
        collect_same_scope_stmts(stmts, &mut statements);
        let mut candidates: HashMap<SymbolId, (String, usize, Option<TypeId>, Option<TypeId>)> =
            HashMap::new();
        for stmt in statements {
            let Stmt::Assign { target, value } = &stmt.node else {
                continue;
            };
            let Expr::Ident(name) = &target.node else {
                continue;
            };
            let Some(symbol) = self
                .symbols
                .lookup_in_scope(self.symbols.current_scope_idx(), name)
            else {
                continue;
            };
            if !self.inferred_local_placeholders.contains(&symbol) {
                continue;
            }
            let inferred = self.shallow_assignment_type(value);
            let builtin_alias = self.builtin_class_alias_value(value);
            let entry = candidates
                .entry(symbol)
                .or_insert_with(|| (name.clone(), 0, None, None));
            entry.1 += 1;
            if entry.1 == 1 {
                entry.2 = inferred;
                entry.3 = builtin_alias;
            } else {
                entry.2 = None;
                entry.3 = None;
            }
        }
        for (symbol, (name, assignments, inferred, builtin_alias)) in candidates {
            if assignments != 1 || binding_events.get(&name) != Some(&1) {
                continue;
            }
            if let Some(inferred) = inferred {
                self.set_sym_type(symbol.0, inferred);
            }
            self.set_builtin_class_alias(symbol, builtin_alias);
        }
    }

    pub(crate) fn get_sym_type(&self, sym_idx: u32) -> TypeId {
        self.sym_types
            .get(sym_idx as usize)
            .and_then(|t| *t)
            .unwrap_or(self.tcx.error())
    }

    pub(crate) fn record_function_param_sigs(
        &mut self,
        symbol: SymbolId,
        params: &[Param],
        overload_decorated: bool,
    ) {
        if overload_decorated {
            self.function_param_sigs.remove(&symbol);
            return;
        }

        let sigs = params
            .iter()
            .map(|param| FunctionParamSig {
                name: param.name.clone(),
                ty: self.resolve_type_expr(&param.ty),
                kind: param.kind,
                pos_only: param.pos_only,
                kw_only: param.kw_only,
                has_default: param.default.is_some(),
            })
            .collect();
        self.function_param_sigs.insert(symbol, sigs);
    }

    pub(crate) fn function_callable_signature(
        &self,
        symbol: SymbolId,
    ) -> Option<Vec<CallableParam>> {
        self.function_param_sigs.get(&symbol).cloned().map(|params| {
            params
                .into_iter()
                .map(FunctionParamSig::into_callable_param)
                .collect()
        })
    }

    pub(crate) fn error(&mut self, span: Span, msg: impl Into<String>) {
        let message = msg.into();
        if self.errors.iter().any(|error| {
            matches!(
                error,
                MambaError::Type {
                    span: existing_span,
                    message: existing_message,
                } if *existing_span == span && existing_message == &message
            )
        }) {
            return;
        }
        self.errors.push(MambaError::type_err(span, message));
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

        // Allocate every parameter first so lazy metadata can keep stable
        // identities while the declaration-local aliases are in scope.
        let allocated: Vec<_> = type_params
            .iter()
            .map(|param| {
                let kind = match param.kind {
                    TypeParamKind::TypeVar => TypeVarKind::TypeVar,
                    TypeParamKind::TypeVarTuple => TypeVarKind::TypeVarTuple,
                    TypeParamKind::ParamSpec => TypeVarKind::ParamSpec,
                };
                let var_id = self.tcx.new_type_param(
                    param.name.clone(),
                    kind,
                    None,
                    Vec::new(),
                    TypeParamDefault::None,
                );
                (param, var_id, kind)
            })
            .collect();
        let aliases: Vec<_> = allocated
            .iter()
            .map(|(param, var_id, _)| (param.name.clone(), *var_id))
            .collect();
        self.register_type_param_aliases(&aliases);

        // Resolve metadata only after the complete alias scope exists. A
        // lazy forward reference that is not resolvable during preregistration
        // is intentionally omitted without leaking an `unknown type` error.
        for (param, var_id, kind) in allocated {
            let bound = param
                .bound
                .as_ref()
                .and_then(|expr| self.resolve_type_param_metadata_expr(expr));
            let constraints = param
                .constraints
                .as_ref()
                .and_then(|items| {
                    items
                        .iter()
                        .map(|expr| self.resolve_type_param_metadata_expr(expr))
                        .collect::<Option<Vec<_>>>()
                })
                .unwrap_or_default();
            let default = match &param.default {
                None => TypeParamDefault::None,
                Some(expr) => self
                    .resolve_type_param_metadata_expr(expr)
                    .map(TypeParamDefault::Resolved)
                    .unwrap_or(TypeParamDefault::Unresolved),
            };

            self.tcx
                .set_type_var_metadata(var_id, bound, constraints.clone(), default);
            gp.add_param(&param.name, var_id, kind, bound, constraints, default);
        }

        gp
    }

    pub(crate) fn register_type_param_aliases(&mut self, aliases: &[(String, TypeVarId)]) {
        let shadowed = aliases
            .iter()
            .map(|(name, id)| (name.clone(), *id, self.tcx.resolve_alias(name)))
            .collect();
        self.type_param_alias_scopes.push(shadowed);

        for (name, var_id) in aliases {
            let tv_ty = self.tcx.intern(Ty::TypeVar(*var_id));
            self.tcx.register_alias(name.clone(), tv_ty);
        }
    }

    fn resolve_type_param_metadata_expr(&mut self, expr: &Spanned<Expr>) -> Option<TypeId> {
        let type_expr = expr_to_type_expr(expr)?;
        let error_mark = self.errors_mark();
        let resolved = self.resolve_type_expr(&type_expr);
        if self.errors.len() != error_mark || resolved == self.tcx.error() {
            self.truncate_errors(error_mark);
            None
        } else {
            Some(resolved)
        }
    }

    /// Remove type parameter aliases to prevent leaking outside scope.
    pub(crate) fn unregister_type_params(&mut self, type_params: &[crate::parser::ast::TypeParam]) {
        let names: Vec<_> = type_params.iter().map(|param| param.name.clone()).collect();
        self.unregister_type_param_aliases(&names);
    }

    pub(crate) fn unregister_type_param_aliases(&mut self, names: &[String]) {
        for name in names {
            self.tcx.unregister_alias(name);
        }

        let shadowed = self
            .type_param_alias_scopes
            .pop()
            .expect("type parameter alias scopes must be balanced");
        for (name, _, prior) in shadowed {
            if let Some(ty) = prior {
                self.tcx.register_alias(name, ty);
            }
        }
    }

    fn active_type_param_ids(&self) -> Vec<TypeVarId> {
        let mut names = HashSet::new();
        let mut ids = Vec::new();
        for scope in self.type_param_alias_scopes.iter().rev() {
            for (name, id, _) in scope.iter().rev() {
                if names.insert(name.as_str()) {
                    ids.push(*id);
                }
            }
        }
        ids.reverse();
        ids
    }

    fn resolve_active_type_param_alias(&self, name: &str) -> Option<TypeId> {
        self.type_param_alias_scopes
            .iter()
            .rev()
            .any(|scope| scope.iter().any(|(param, _, _)| param == name))
            .then(|| self.tcx.resolve_alias(name))
            .flatten()
    }

    /// Re-resolve lazy PEP 695 metadata after the first pass has registered
    /// every definition. This keeps the original TypeVar ids embedded in
    /// function/class types while allowing forward bounds to become concrete.
    fn finalize_generic_param_metadata(
        &mut self,
        symbol: SymbolId,
        type_params: &[crate::parser::ast::TypeParam],
    ) {
        let Some(gp) = self.generic_defs.get(&symbol).cloned() else {
            return;
        };
        let Some(gp) = self.finalize_generic_params(gp, type_params) else {
            return;
        };
        self.generic_defs.insert(symbol, gp);
    }

    fn finalize_type_alias_generic_metadata(
        &mut self,
        name: &str,
        type_params: &[crate::parser::ast::TypeParam],
    ) {
        let Some(symbol) = self.lookup_type_alias_symbol(name) else {
            return;
        };
        let Some(gp) = self
            .type_alias_defs
            .get(&symbol)
            .map(|definition| definition.params.clone())
        else {
            return;
        };
        let Some(gp) = self.finalize_generic_params(gp, type_params) else {
            return;
        };
        self.type_alias_defs
            .get_mut(&symbol)
            .expect("type alias definition disappeared")
            .params = gp;
        self.resolve_type_alias_template(symbol);
    }

    fn refresh_function_signature(
        &mut self,
        symbol: SymbolId,
        params: &[Param],
        return_ty: Option<&Spanned<TypeExpr>>,
        decorators: &[Spanned<Expr>],
    ) {
        let aliases: Vec<_> = self
            .generic_defs
            .get(&symbol)
            .map(|generic_params| {
                generic_params
                    .params
                    .iter()
                    .map(|param| (param.name.clone(), param.id))
                    .collect()
            })
            .unwrap_or_default();
        if !aliases.is_empty() {
            self.register_type_param_aliases(&aliases);
        }

        let overload_decorated = decorators
            .iter()
            .any(|decorator| is_typing_overload_decorator(&decorator.node));
        self.record_function_param_sigs(symbol, params, overload_decorated);
        let (param_types, ret, variadic) = if overload_decorated {
            (Vec::new(), self.tcx.any(), true)
        } else {
            let star_pos = params
                .iter()
                .position(|param| param.kind == crate::parser::ast::ParamKind::Star);
            let variadic = star_pos.is_some()
                || params
                    .iter()
                    .any(|param| param.kind == crate::parser::ast::ParamKind::DoubleStar);
            let effective_params = star_pos.map_or(params, |position| &params[..position]);
            let param_types = effective_params
                .iter()
                .filter(|param| param.kind == crate::parser::ast::ParamKind::Regular)
                .map(|param| self.resolve_type_expr(&param.ty))
                .collect();
            let ret = return_ty
                .map(|return_ty| self.resolve_type_expr(return_ty))
                .unwrap_or(self.tcx.any());
            (param_types, ret, variadic)
        };
        let function_ty = self.tcx.intern(Ty::Fn {
            params: param_types,
            ret,
            variadic,
            signature: self.function_callable_signature(symbol),
            param_spec: None,
        });
        self.set_sym_type(symbol.0, function_ty);
        self.function_declaration_types
            .insert(symbol, function_ty);

        if !aliases.is_empty() {
            let names: Vec<_> = aliases.into_iter().map(|(name, _)| name).collect();
            self.unregister_type_param_aliases(&names);
        }
    }

    fn finalize_type_alias_metadata_in(&mut self, stmts: &[Spanned<Stmt>]) {
        let mut statements = Vec::new();
        collect_same_scope_stmts(stmts, &mut statements);
        for stmt in statements {
            if let Stmt::TypeAlias {
                name, type_params, ..
            } = &stmt.node
            {
                self.finalize_type_alias_generic_metadata(name, type_params);
            }
        }
    }

    pub(crate) fn refresh_function_signatures_in(&mut self, stmts: &[Spanned<Stmt>]) {
        let mut statements = Vec::new();
        collect_same_scope_stmts(stmts, &mut statements);
        for stmt in statements {
            match &stmt.node {
                Stmt::FnDef {
                    name,
                    params,
                    return_ty,
                    decorators,
                    ..
                }
                | Stmt::AsyncFnDef {
                    name,
                    params,
                    return_ty,
                    decorators,
                    ..
                } => {
                    if let Some(symbol) = self.declaration_symbol(stmt) {
                        self.refresh_function_signature(
                            symbol,
                            params,
                            return_ty.as_ref(),
                            decorators,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn rebuild_class_metadata(
        &mut self,
        class_symbol: SymbolId,
        name: &str,
        body: &[Spanned<Stmt>],
        is_protocol: bool,
    ) {
        let fields = self.collect_class_fields(body);
        let match_args = self.collect_match_args(body);
        let type_args = self
            .generic_defs
            .get(&class_symbol)
            .map(|params| {
                params
                    .params
                    .iter()
                    .map(|param| param.id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
            .into_iter()
            .map(|id| self.tcx.intern(Ty::TypeVar(id)))
            .collect();
        let class_ty = self.tcx.intern(Ty::Class {
            name: name.to_string(),
            role: ClassRole::Object,
            user: Some(UserClass {
                symbol: class_symbol,
                args: type_args,
            }),
            external: None,
            fields,
            match_args,
        });
        self.set_builtin_class_alias(class_symbol, None);
        self.set_binding_origins(class_symbol, None, None, None);
        self.set_sym_type(class_symbol.0, class_ty);
        self.collect_class_methods(class_symbol, name, body);
        if is_protocol {
            self.register_protocol(class_symbol, name, body);
        }
    }

    fn refresh_class_declaration_metadata(
        &mut self,
        class_symbol: SymbolId,
        name: &str,
        body: &[Spanned<Stmt>],
        bases: &[Spanned<Expr>],
    ) {
        let aliases: Vec<_> = self
            .generic_defs
            .get(&class_symbol)
            .map(|generic_params| {
                generic_params
                    .params
                    .iter()
                    .map(|param| (param.name.clone(), param.id))
                    .collect()
            })
            .unwrap_or_default();
        if !aliases.is_empty() {
            self.register_type_param_aliases(&aliases);
        }
        let previous_class = self.current_class.replace(name.to_string());
        self.symbols.push_scope();
        self.preregister_type_alias_headers(body);
        self.finalize_type_alias_metadata_in(body);
        let is_protocol = bases
            .iter()
            .any(|base| matches!(&base.node, Expr::Ident(name) if name == "Protocol"));
        self.rebuild_class_metadata(class_symbol, name, body, is_protocol);
        self.symbols.pop_scope();
        self.current_class = previous_class;
        if !aliases.is_empty() {
            let names: Vec<_> = aliases.into_iter().map(|(name, _)| name).collect();
            self.unregister_type_param_aliases(&names);
        }
    }

    fn finalize_class_method_generic_metadata(
        &mut self,
        class_symbol: SymbolId,
        method_name: &str,
        type_params: &[crate::parser::ast::TypeParam],
    ) {
        let key = (class_symbol, method_name.to_string());
        let Some(gp) = self.class_method_generic_defs.get(&key).cloned() else {
            return;
        };
        let Some(gp) = self.finalize_generic_params(gp, type_params) else {
            return;
        };
        self.class_method_generic_defs.insert(key, gp);
    }

    fn finalize_generic_params(
        &mut self,
        mut gp: GenericParams,
        type_params: &[crate::parser::ast::TypeParam],
    ) -> Option<GenericParams> {
        if gp.params.len() != type_params.len() {
            return None;
        }

        let aliases: Vec<_> = gp
            .params
            .iter()
            .map(|tv| (tv.name.clone(), tv.id))
            .collect();
        let param_positions: HashMap<_, _> = gp
            .params
            .iter()
            .enumerate()
            .map(|(index, tv)| (tv.id, (index, tv.kind)))
            .collect();
        self.register_type_param_aliases(&aliases);

        for (index, (param, tv)) in type_params.iter().zip(gp.params.iter_mut()).enumerate() {
            if let Some(expr) = &param.bound {
                if let Some(bound) = self.resolve_type_param_metadata_expr(expr) {
                    if self.tcx.contains_type_var(bound) {
                        self.error(expr.span, "type parameter bound must be concrete");
                        tv.bound = None;
                    } else {
                        tv.bound = Some(bound);
                    }
                }
            }
            if let Some(items) = &param.constraints {
                if let Some(constraints) = items
                    .iter()
                    .map(|expr| self.resolve_type_param_metadata_expr(expr))
                    .collect::<Option<Vec<_>>>()
                {
                    if constraints
                        .iter()
                        .any(|constraint| self.tcx.contains_type_var(*constraint))
                    {
                        if let Some(expr) = items.first() {
                            self.error(expr.span, "type parameter constraints must be concrete");
                        }
                        tv.constraints.clear();
                    } else {
                        tv.constraints = constraints;
                    }
                }
            }
            if let Some(expr) = &param.default {
                tv.default = self
                    .resolve_type_param_metadata_expr(expr)
                    .map(TypeParamDefault::Resolved)
                    .unwrap_or(TypeParamDefault::Unresolved);
            }
            if let Some(default) = tv.default.resolved() {
                let references = self.tcx.type_vars_in(default);
                let invalid_scope = references.iter().any(|referenced| {
                    param_positions
                        .get(referenced)
                        .is_none_or(|(position, _)| *position >= index)
                });
                let invalid_kind = match self.tcx.get(default) {
                    Ty::TypeVar(referenced) => param_positions
                        .get(referenced)
                        .is_some_and(|(_, kind)| *kind != tv.kind),
                    _ => false,
                };
                if invalid_scope || invalid_kind {
                    if let Some(expr) = &param.default {
                        self.error(
                            expr.span,
                            format!(
                                "default for type parameter '{}' may only reference earlier parameters of the same kind",
                                tv.name
                            ),
                        );
                    }
                    tv.default = TypeParamDefault::Unresolved;
                }
            }
            if let Some(default) = tv.default.resolved() {
                if !self.tcx.contains_type_var(default) {
                    if let Some(bound) = tv.bound {
                        if !self.tcx.is_subtype(default, bound) {
                            if let Some(expr) = &param.default {
                                self.error(
                                    expr.span,
                                    format!(
                                        "default for type parameter '{}' violates its bound",
                                        tv.name
                                    ),
                                );
                            }
                        }
                    }
                    if !tv.constraints.is_empty()
                        && !tv
                            .constraints
                            .iter()
                            .any(|constraint| default == *constraint)
                    {
                        if let Some(expr) = &param.default {
                            self.error(
                                expr.span,
                                format!(
                                    "default for type parameter '{}' violates its constraints",
                                    tv.name
                                ),
                            );
                        }
                    }
                }
            }
            self.tcx
                .set_type_var_metadata(tv.id, tv.bound, tv.constraints.clone(), tv.default);
        }

        self.unregister_type_params(type_params);
        Some(gp)
    }

    pub(crate) fn finalize_generic_metadata_in(&mut self, stmts: &[Spanned<Stmt>]) {
        self.finalize_type_alias_metadata_in(stmts);
        let mut statements = Vec::new();
        collect_same_scope_stmts(stmts, &mut statements);
        for stmt in statements {
            match &stmt.node {
                Stmt::FnDef {
                    name, type_params, ..
                }
                | Stmt::AsyncFnDef {
                    name, type_params, ..
                } => {
                    if let Some(symbol) = self.declaration_symbol(stmt) {
                        self.finalize_generic_param_metadata(symbol, type_params);
                    }
                }
                Stmt::TypeAlias { .. } => {}
                Stmt::ClassDef {
                    name,
                    type_params,
                    bases,
                    body,
                    ..
                } => {
                    if let Some(class_symbol) = self.declaration_symbol(stmt) {
                        self.finalize_generic_param_metadata(class_symbol, type_params);
                        self.refresh_class_declaration_metadata(class_symbol, name, body, bases);
                        for method in body {
                            match &method.node {
                                Stmt::FnDef {
                                    name, type_params, ..
                                }
                                | Stmt::AsyncFnDef {
                                    name, type_params, ..
                                } => self.finalize_class_method_generic_metadata(
                                    class_symbol,
                                    name,
                                    type_params,
                                ),
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Get the TypeId for a SymbolId, if known (#1190).
    pub fn get_symbol_type(&self, sym: crate::resolve::SymbolId) -> Option<crate::types::TypeId> {
        self.sym_types.get(sym.0 as usize).and_then(|t| *t)
    }

    fn preregister_type_alias_headers(&mut self, stmts: &[Spanned<Stmt>]) {
        let mut statements = Vec::new();
        collect_same_scope_stmts(stmts, &mut statements);
        for stmt in statements {
            if let Stmt::TypeAlias {
                name,
                type_params,
                value,
            } = &stmt.node
            {
                self.preregister_type_alias_header(name, type_params, value);
            }
        }
    }

    fn preregister_type_alias_header(
        &mut self,
        name: &str,
        type_params: &[crate::parser::ast::TypeParam],
        value: &Spanned<Expr>,
    ) {
        let scope = self.symbols.current_scope_idx();
        let symbol = self
            .symbols
            .lookup_in_scope(scope, name)
            .unwrap_or_else(|| self.symbols.define(name.to_string(), SymbolKind::Variable));
        self.set_sym_type(symbol.0, self.tcx.any());

        if let Some(previous) = self.type_alias_declarations.get(&symbol).copied() {
            if previous != value.span {
                self.error(
                    value.span,
                    format!("type alias '{name}' is already defined in this scope"),
                );
            }
            return;
        }
        self.type_alias_declarations.insert(symbol, value.span);
        let Some(value) = expr_to_type_expr(value) else {
            return;
        };
        let mut captures = self.active_type_param_ids();
        captures.retain(|id| {
            type_params
                .iter()
                .all(|param| param.name != self.tcx.get_type_var(*id).name)
        });
        let params = self.register_type_params(type_params);
        self.unregister_type_params(type_params);
        self.type_alias_defs.insert(
            symbol,
            TypeAliasDef {
                name: name.to_string(),
                params,
                captures,
                value,
                template: None,
                resolving: false,
            },
        );
    }

    fn lookup_type_alias_symbol(&self, name: &str) -> Option<SymbolId> {
        let symbol = self.symbols.lookup(name)?;
        self.type_alias_defs.contains_key(&symbol).then_some(symbol)
    }

    fn resolve_type_alias_template(&mut self, symbol: SymbolId) -> TypeId {
        let Some(definition) = self.type_alias_defs.get(&symbol).cloned() else {
            return self.tcx.error();
        };
        let declaration_args: Vec<_> = definition
            .params
            .params
            .iter()
            .map(|param| param.id)
            .chain(definition.captures.iter().copied())
            .map(|id| self.tcx.intern(Ty::TypeVar(id)))
            .collect();
        let (instance, alias_ref) = self.tcx.intern_alias_instance(
            AliasIdentity::Source(symbol),
            definition.name.clone(),
            declaration_args,
            definition.params.len(),
        );
        if let Some(template) = definition.template {
            return template;
        }
        if definition.resolving {
            return alias_ref;
        }

        self.type_alias_defs
            .get_mut(&symbol)
            .expect("type alias definition disappeared")
            .resolving = true;
        let aliases: Vec<_> = definition
            .params
            .params
            .iter()
            .map(|param| (param.name.clone(), param.id))
            .collect();
        self.register_type_param_aliases(&aliases);
        let error_mark = self.errors_mark();
        let mut template = self.resolve_type_expr(&definition.value);
        let alias_names: Vec<_> = aliases.into_iter().map(|(name, _)| name).collect();
        self.unregister_type_param_aliases(&alias_names);
        if self.preregister_depth > 0 && self.errors.len() != error_mark {
            self.truncate_errors(error_mark);
            self.type_alias_defs
                .get_mut(&symbol)
                .expect("type alias definition disappeared")
                .resolving = false;
            return self.tcx.any();
        }
        if self.tcx.alias_has_unguarded_cycle(instance, template) {
            self.error(
                definition.value.span,
                format!("unproductive recursive type alias '{}'", definition.name),
            );
            template = self.tcx.error();
        }
        let definition = self
            .type_alias_defs
            .get_mut(&symbol)
            .expect("type alias definition disappeared");
        definition.template = Some(template);
        definition.resolving = false;
        self.tcx.set_alias_target(instance, template);
        template
    }

    fn resolve_type_alias(
        &mut self,
        name: &str,
        symbol: SymbolId,
        supplied: Option<&[TypeId]>,
        span: Span,
    ) -> TypeId {
        let template = self.resolve_type_alias_template(symbol);
        let Some(definition) = self.type_alias_defs.get(&symbol).cloned() else {
            return template;
        };
        let recursive_edge = definition.resolving && definition.template.is_none();
        if definition.params.is_empty() {
            if supplied.is_some() {
                self.error(span, format!("type '{name}' is not generic"));
            }
            return self.tcx.semantic_head_id(template).unwrap_or(template);
        }

        let (subst, resolved, errors) = if let Some(args) = supplied {
            bind_explicit_type_args(&definition.params, args, &mut self.tcx)
        } else {
            let Some((subst, resolved)) =
                complete_type_args(&definition.params, Substitution::new(), &mut self.tcx)
            else {
                return template;
            };
            (subst, resolved, Vec::new())
        };
        for error in errors {
            self.error(span, error);
        }
        if recursive_edge {
            return subst.apply(template, &mut self.tcx);
        }

        let mut identity_args = resolved;
        identity_args.extend(
            definition
                .captures
                .iter()
                .map(|id| self.tcx.intern(Ty::TypeVar(*id))),
        );
        let (instance, _) = self.tcx.intern_alias_instance(
            AliasIdentity::Source(symbol),
            name.to_string(),
            identity_args,
            definition.params.len(),
        );
        let owns_target = self.tcx.begin_alias_target(instance);
        let specialized = subst.apply(template, &mut self.tcx);
        if owns_target {
            self.tcx.set_alias_target(instance, specialized);
        }
        self.tcx
            .semantic_head_id(specialized)
            .unwrap_or(specialized)
    }

    fn materialize_alias_instance(&mut self, id: AliasInstanceId) -> Option<TypeId> {
        if let Some(target) = self.tcx.alias_target(id) {
            return Some(target);
        }
        if self.tcx.alias_target_is_resolving(id) {
            return None;
        }

        let instance = self.tcx.alias_instance(id).clone();
        if let Some(deferred) = self.tcx.deferred_alias_target(id).cloned() {
            let checkpoint = self.tcx.begin_alias_target_transaction();
            let materialized = (|| {
                let template = self.materialize_alias_instance(deferred.template)?;
                if !self.tcx.begin_alias_target(id) {
                    return self.tcx.alias_target(id);
                }
                let substitution = Substitution::from_bindings(
                    &deferred.substitutions,
                    &deferred.param_packs,
                );
                let target = substitution.apply(template, &mut self.tcx);
                if self.tcx.alias_has_unguarded_cycle(id, target)
                    || self
                        .tcx
                        .alias_target_has_invalid_generated_edge(id, target)
                {
                    return None;
                }
                self.tcx.set_alias_target(id, target);
                Some(target)
            })();
            self.tcx
                .finish_alias_target_transaction(checkpoint, materialized.is_some());
            if materialized.is_none() {
                self.tcx.reject_alias_target(id);
            }
            return materialized.or_else(|| self.tcx.alias_target(id));
        }

        let AliasIdentity::Source(symbol) = instance.identity else {
            return None;
        };
        let definition = self.type_alias_defs.get(&symbol)?.clone();
        let template = definition.template?;
        let identity_params: Vec<_> = definition
            .params
            .params
            .iter()
            .map(|param| param.id)
            .chain(definition.captures.iter().copied())
            .collect();
        if identity_params.len() != instance.args.len() {
            return None;
        }

        let mut subst = Substitution::new();
        for (param, arg) in identity_params.iter().zip(&instance.args) {
            subst.insert(*param, *arg);
        }
        if !self.tcx.begin_alias_target(id) {
            return self.tcx.alias_target(id);
        }
        let target = subst.apply(template, &mut self.tcx);
        self.tcx.set_alias_target(id, target);
        Some(target)
    }

    fn materialize_alias_head(&mut self, ty: TypeId) -> TypeId {
        let mut current = ty;
        let mut seen = HashSet::new();
        loop {
            let Ty::AliasRef(id) = self.tcx.get(current) else {
                return current;
            };
            let id = *id;
            if self.tcx.alias_target_is_rejected(id) {
                return current;
            }
            if !seen.insert(id) {
                return self.tcx.error();
            }
            let target = self
                .tcx
                .alias_target(id)
                .or_else(|| self.materialize_alias_instance(id));
            let Some(target) = target else {
                return self.tcx.error();
            };
            if self.tcx.alias_target_is_rejected(id) {
                return current;
            }
            current = target;
        }
    }

    pub(crate) fn semantic_ty(&mut self, ty: TypeId) -> Ty {
        let head = self.materialize_alias_head(ty);
        self.tcx.get(head).clone()
    }

    fn preregister_class_object_alias(&mut self, value: &Spanned<Expr>) -> Option<TypeId> {
        match &value.node {
            Expr::Ident(source) => {
                let symbol = self.symbols.lookup(source)?;
                let ty = self.get_sym_type(symbol.0);
                matches!(
                    self.tcx.get(ty),
                    Ty::Class {
                        role: ClassRole::Object,
                        ..
                    }
                )
                .then_some(ty)
            }
            Expr::Index { object, .. } => {
                let Expr::Ident(source) = &object.node else {
                    return None;
                };
                let source_symbol = self.symbols.lookup(source)?;
                let source_ty = self.get_sym_type(source_symbol.0);
                if !matches!(
                    self.tcx.get(source_ty),
                    Ty::Class {
                        role: ClassRole::Object,
                        user: Some(_),
                        ..
                    }
                ) {
                    return None;
                }
                let type_expr = expr_to_type_expr(value)?;
                let error_len = self.errors.len();
                let diagnostic_len = self.diagnostics.len();
                let resolved = self.resolve_type_expr(&type_expr);
                self.errors.truncate(error_len);
                self.diagnostics.truncate(diagnostic_len);
                match self.tcx.get(resolved) {
                    Ty::Class {
                        role: ClassRole::Instance,
                        user: Some(_),
                        ..
                    } => Some(self.with_class_role(resolved, ClassRole::Object)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub(crate) fn invalidate_conditional_class_object_aliases(&mut self, stmts: &[Spanned<Stmt>]) {
        let mut assigned = Vec::new();
        let mut declared = Vec::new();
        crate::resolve::pass::collect_assignment_targets(stmts, &mut assigned, &mut declared);
        crate::resolve::pass::collect_walrus_targets_in_stmts(stmts, &mut assigned);
        self.invalidate_conditional_binding_names(assigned);
    }

    pub(crate) fn invalidate_conditional_binding_names(&mut self, assigned: Vec<String>) {
        for name in assigned {
            let Some(symbol) = self
                .symbols
                .lookup_in_scope(self.symbols.current_scope_idx(), &name)
            else {
                continue;
            };
            let had_origin = self.import_origins.contains_key(&symbol)
                || self.instance_origins.contains_key(&symbol)
                || self.class_ref_origins.contains_key(&symbol);
            self.builtin_class_aliases.remove(&symbol);
            self.set_binding_origins(symbol, None, None, None);
            if had_origin
                || matches!(
                    self.tcx.get(self.get_sym_type(symbol.0)),
                    Ty::Class { .. } | Ty::External(_)
                )
            {
                self.set_sym_type(symbol.0, self.tcx.any());
            }
        }
    }

    /// Check a module. Returns accumulated errors.
    /// First-pass def/class/enum/alias pre-registration, descending into
    /// compound-statement bodies (try/if/while/for/with): a class defined in
    /// a module-level `try:` is still a module-scope binding.
    pub(crate) fn preregister_defs(&mut self, stmts: &[Spanned<Stmt>]) {
        if self.preregister_depth == 0 {
            self.preregister_binding_events = same_scope_binding_events(stmts);
            self.preregister_direct_assignments = direct_assignment_counts(stmts);
            let mut assigned = Vec::new();
            let mut declared = Vec::new();
            crate::resolve::pass::collect_assignment_targets(stmts, &mut assigned, &mut declared);
            self.preregister_declared_bindings = declared.into_iter().collect();
        }
        self.preregister_depth += 1;
        self.preregister_type_alias_headers(stmts);
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

                    let sym = if let Some(symbol) = self.declaration_symbol(stmt) {
                        self.symbols.bind_symbol_in_scope(
                            self.symbols.current_scope_idx(),
                            name.clone(),
                            symbol,
                        );
                        symbol
                    } else {
                        let symbol = self.symbols.define(name.clone(), SymbolKind::Function);
                        self.declaration_symbols
                            .insert(Self::declaration_key(stmt), symbol);
                        symbol
                    };
                    let overload_decorated = decorators
                        .iter()
                        .any(|d| is_typing_overload_decorator(&d.node));
                    self.record_function_param_sigs(sym, params, overload_decorated);
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
                        signature: self.function_callable_signature(sym),
                        param_spec: None,
                    });
                    self.set_sym_type(sym.0, fn_ty);
                    self.function_declaration_types.insert(sym, fn_ty);

                    if !gp.is_empty() {
                        self.generic_defs.insert(sym, gp);
                    }
                    // Clean up type parameter aliases to prevent leaking
                    self.unregister_type_params(type_params);
                }
                Stmt::ClassDef {
                    decorators,
                    name,
                    type_params,
                    bases,
                    body,
                    ..
                } => {
                    // Register generic type params for the class
                    let gp = self.register_type_params(type_params);
                    let is_typed_dict = bases.iter().any(|b| self.base_is_typed_dict(&b.node));
                    let base_symbols: Vec<SymbolId> = bases
                        .iter()
                        .filter_map(|base| {
                            let base_name = match &base.node {
                                Expr::Ident(base_name) => base_name,
                                Expr::Index { object, .. } => {
                                    let Expr::Ident(base_name) = &object.node else {
                                        return None;
                                    };
                                    base_name
                                }
                                _ => return None,
                            };
                            let binding = self.symbols.lookup(base_name)?;
                            match self.tcx.get(self.get_sym_type(binding.0)) {
                                Ty::Class {
                                    role: ClassRole::Object,
                                    user: Some(user),
                                    ..
                                } => Some(user.symbol),
                                _ => None,
                            }
                        })
                        .collect();
                    let numeric_root = bases.iter().find_map(|base| {
                        let Expr::Ident(base_name) = &base.node else {
                            return None;
                        };
                        match base_name.as_str() {
                            "int" | "bool" => Some(NumericRoot::Int),
                            "float" => Some(NumericRoot::Float),
                            _ => self
                                .symbols
                                .lookup(base_name)
                                .and_then(|symbol| match self.tcx.get(self.get_sym_type(symbol.0)) {
                                    Ty::Class {
                                        role: ClassRole::Object,
                                        user: Some(user),
                                        ..
                                    } => self
                                        .numeric_derived_class_symbols
                                        .get(&user.symbol)
                                        .copied(),
                                    _ => self.numeric_derived_classes.get(base_name).copied(),
                                }),
                        }
                    });
                    self.typed_dict_classes.remove(name);
                    self.user_bare_classes.remove(name);
                    self.numeric_derived_classes.remove(name);
                    self.class_bases.remove(name);
                    let sym = if let Some(symbol) = self.declaration_symbol(stmt) {
                        self.symbols.bind_symbol_in_scope(
                            self.symbols.current_scope_idx(),
                            name.clone(),
                            symbol,
                        );
                        symbol
                    } else {
                        let symbol = self.symbols.define(name.clone(), SymbolKind::Class);
                        self.declaration_symbols
                            .insert(Self::declaration_key(stmt), symbol);
                        symbol
                    };
                    let non_object_base_count = bases
                        .iter()
                        .filter(|base| {
                            !matches!(&base.node, Expr::Ident(name) if name == "object")
                        })
                        .count();
                    let inheritance_open = base_symbols.len() != non_object_base_count
                        || base_symbols
                            .iter()
                            .any(|base| self.class_inheritance_open.contains(base));
                    if inheritance_open {
                        self.class_inheritance_open.insert(sym);
                    } else {
                        self.class_inheritance_open.remove(&sym);
                    }
                    self.symbols.push_scope();
                    let class_metadata_error_mark = self.errors_mark();
                    self.preregister_type_alias_headers(body);
                    let fields = self.collect_class_fields(body);
                    let match_args = self.collect_match_args(body);
                    let mut type_args = Vec::with_capacity(gp.len());
                    for param in &gp.params {
                        type_args.push(self.tcx.intern(Ty::TypeVar(param.id)));
                    }
                    let class_ty = self.tcx.intern(Ty::Class {
                        name: name.clone(),
                        role: ClassRole::Object,
                        user: Some(UserClass {
                            symbol: sym,
                            args: type_args,
                        }),
                        external: None,
                        fields,
                        match_args,
                    });
                    self.set_sym_type(sym.0, class_ty);
                    if is_typed_dict {
                        self.typed_dict_classes.insert(name.clone());
                        self.typed_dict_class_symbols.insert(sym);
                    }

                    if !gp.is_empty() {
                        self.generic_defs.insert(sym, gp);
                    }

                    // Collect class methods for protocol conformance
                    self.collect_class_methods(sym, name, body);

                    let is_protocol = bases
                        .iter()
                        .any(|b| matches!(&b.node, Expr::Ident(n) if n == "Protocol"));
                    if is_protocol {
                        self.register_protocol(sym, name, body);
                    }
                    if self.preregister_depth > 0 {
                        self.truncate_errors(class_metadata_error_mark);
                    }
                    self.symbols.pop_scope();

                    // #1041: record this class's identifier bases (unfiltered,
                    // not just numeric ones) so `class_defines_dunder`
                    // (check_expr.rs) can walk the inheritance chain to find
                    // a unary/shift dunder override declared on an ancestor
                    // rather than the immediate class.
                    let base_names: Vec<String> = bases
                        .iter()
                        .filter_map(|b| match &b.node {
                            Expr::Ident(n) => Some(n.clone()),
                            _ => None,
                        })
                        .collect();
                    self.class_bases.insert(name.clone(), base_names);
                    self.class_base_symbols.insert(sym, base_symbols);

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
                    let dataclass_decorated =
                        decorators.iter().any(|d| is_dataclass_decorator(&d.node));
                    if only_object_base && !has_method && !dataclass_decorated {
                        self.user_bare_classes.insert(name.clone());
                        self.user_bare_class_symbols.insert(sym);
                    }

                    // #1031: record classes whose base chain reaches a
                    // numeric builtin (`int`/`float`) so numeric-only
                    // compile checks accept their instances. Python requires
                    // a base class to already be defined when a `ClassDef`'s
                    // bases are evaluated, so classes are visited in an order
                    // where `numeric_derived_classes` already holds any base
                    // that itself derives a numeric builtin — a single
                    // forward lookup (no explicit recursion) resolves
                    // multi-level chains like `class Q(P): pass` where
                    // `P(int)`.
                    if let Some(root) = numeric_root {
                        self.numeric_derived_classes.insert(name.clone(), root);
                        self.numeric_derived_class_symbols.insert(sym, root);
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
                    let sym = if let Some(symbol) = self.declaration_symbol(stmt) {
                        self.symbols.bind_symbol_in_scope(
                            self.symbols.current_scope_idx(),
                            name.clone(),
                            symbol,
                        );
                        symbol
                    } else {
                        let symbol = self.symbols.define(name.clone(), SymbolKind::Enum);
                        self.declaration_symbols
                            .insert(Self::declaration_key(stmt), symbol);
                        symbol
                    };
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
                        self.record_function_param_sigs(sym, &fn_def.params, false);
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
                            signature: self.function_callable_signature(sym),
                            param_spec: None,
                        });
                        self.set_sym_type(sym.0, fn_ty);
                        self.function_declaration_types.insert(sym, fn_ty);
                    }
                }
                Stmt::TypeAlias { .. } => {}
                // Register generated stdlib identities in the first pass so
                // class signatures and aliases see the same canonical value
                // types as the statement pass. Unknown imports remain Any.
                Stmt::Import {
                    names,
                    module_alias,
                    module,
                } => {
                    let dotted = module.join(".");
                    if let Some(import_names) = names {
                        for (name, alias) in import_names {
                            if name == "*" {
                                continue;
                            }
                            let effective = alias.as_ref().unwrap_or(name);
                            if self
                                .symbols
                                .lookup_in_scope(self.symbols.current_scope_idx(), effective)
                                .is_none()
                            {
                                let sym =
                                    self.symbols.define(effective.clone(), SymbolKind::Variable);
                                let imported_ty =
                                    self.stdlib_imported_member_type(&dotted, name);
                                self.set_sym_type(sym.0, imported_ty);
                                self.import_origins
                                    .insert(sym, (dotted.clone(), name.clone()));
                            }
                        }
                    } else if let Some(alias) = module_alias {
                        let existing = self
                            .symbols
                            .lookup_in_scope(self.symbols.current_scope_idx(), alias);
                        let previous = existing.map(|symbol| self.get_sym_type(symbol.0));
                        let imported_ty =
                            self.stdlib_module_import_type(&dotted, &dotted, previous);
                        let sym = existing.unwrap_or_else(|| {
                            self.symbols.define(alias.clone(), SymbolKind::Variable)
                        });
                        self.set_sym_type(sym.0, imported_ty);
                        self.import_origins
                            .insert(sym, (dotted.clone(), String::new()));
                    } else if let Some(root) = module.first() {
                        let existing = self
                            .symbols
                            .lookup_in_scope(self.symbols.current_scope_idx(), root);
                        let previous = existing.map(|symbol| self.get_sym_type(symbol.0));
                        let imported_ty =
                            self.stdlib_module_import_type(root, &dotted, previous);
                        let sym = existing.unwrap_or_else(|| {
                            self.symbols.define(root.clone(), SymbolKind::Variable)
                        });
                        self.set_sym_type(sym.0, imported_ty);
                        self.import_origins
                            .insert(sym, (root.clone(), String::new()));
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
                            let var_id = self.tcx.new_type_var(name.clone(), None, Vec::new());
                            let tv_ty = self.tcx.intern(Ty::TypeVar(var_id));
                            self.tcx.register_alias(name.clone(), tv_ty);
                        } else {
                            if self.preregister_declared_bindings.contains(name) {
                                continue;
                            }
                            let existing = self
                                .symbols
                                .lookup_in_scope(self.symbols.current_scope_idx(), name);
                            let shadows_builtin = existing.is_some_and(|symbol| {
                                self.builtin_symbols.get(name).copied() == Some(symbol)
                            });
                            if shadows_builtin {
                                continue;
                            }
                            if existing.is_some_and(|symbol| {
                                matches!(
                                    &self.symbols.get_symbol(symbol).kind,
                                    SymbolKind::Parameter
                                )
                            }) {
                                continue;
                            }
                            let binding_events = self
                                .preregister_binding_events
                                .get(name)
                                .copied()
                                .unwrap_or_default();
                            let direct_assignments = self
                                .preregister_direct_assignments
                                .get(name)
                                .copied()
                                .unwrap_or_default();
                            let only_direct_assignments = binding_events > 0
                                && binding_events == direct_assignments;
                            if binding_events != 1 && !only_direct_assignments {
                                let symbol = existing.unwrap_or_else(|| {
                                    self.symbols.define(name.clone(), SymbolKind::Variable)
                                });
                                self.builtin_class_aliases.remove(&symbol);
                                self.set_binding_origins(symbol, None, None, None);
                                let any_ty = self.tcx.any();
                                self.set_sym_type(symbol.0, any_ty);
                                continue;
                            }
                            match (existing, self.preregister_class_object_alias(value)) {
                                (Some(symbol), Some(alias_ty)) => {
                                    self.set_sym_type(symbol.0, alias_ty);
                                }
                                (None, Some(alias_ty)) => {
                                    let symbol =
                                        self.symbols.define(name.clone(), SymbolKind::Variable);
                                    self.set_sym_type(symbol.0, alias_ty);
                                }
                                (Some(symbol), None)
                                    if matches!(
                                        self.tcx.get(self.get_sym_type(symbol.0)),
                                        Ty::Class {
                                            role: ClassRole::Object,
                                            ..
                                        }
                                    ) =>
                                {
                                    self.set_sym_type(symbol.0, self.tcx.any());
                                }
                                _ => {}
                            }
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
                    self.invalidate_conditional_class_object_aliases(std::slice::from_ref(stmt));
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
                    self.invalidate_conditional_class_object_aliases(std::slice::from_ref(stmt));
                }
                Stmt::While {
                    body, else_body, ..
                }
                | Stmt::For {
                    body, else_body, ..
                }
                | Stmt::AsyncFor {
                    body, else_body, ..
                } => {
                    self.preregister_defs(body);
                    if let Some(eb) = else_body {
                        self.preregister_defs(eb);
                    }
                    self.invalidate_conditional_class_object_aliases(std::slice::from_ref(stmt));
                }
                Stmt::With { body, .. } | Stmt::AsyncWith { body, .. } => {
                    self.preregister_defs(body);
                }
                Stmt::Match { arms, .. } => {
                    for arm in arms {
                        self.preregister_defs(&arm.body);
                    }
                    self.invalidate_conditional_class_object_aliases(std::slice::from_ref(stmt));
                }
                _ => {}
            }
        }
        self.preregister_depth -= 1;
        if self.preregister_depth == 0 {
            self.preregister_binding_events.clear();
            self.preregister_direct_assignments.clear();
            self.preregister_declared_bindings.clear();
        }
    }

    pub fn check_module(&mut self, module: &Module) -> Vec<MambaError> {
        // Declaration keys are AST addresses and are valid only for this
        // module object. A persistent REPL checker can otherwise observe an
        // allocator-reused address as an old declaration identity.
        self.declaration_symbols.clear();
        // First pass: register all top-level function/class/enum/alias names
        self.preregister_defs(&module.stmts);
        self.finalize_generic_metadata_in(&module.stmts);
        self.refresh_function_signatures_in(&module.stmts);

        // Second pass: check bodies
        for stmt in &module.stmts {
            self.check_stmt(stmt);
        }

        std::mem::take(&mut self.errors)
    }

    fn resolve_named_class_annotation(
        &mut self,
        name: &str,
        binding_symbol: SymbolId,
        base_ty: TypeId,
        supplied: Option<&[TypeId]>,
        span: Span,
    ) -> TypeId {
        if let Ty::Class {
            external: Some(external),
            ..
        } = self.tcx.get(base_ty).clone()
        {
            return self.resolve_external_class_annotation(name, &external, supplied, span);
        }
        self.specialize_user_class_as(
            name,
            binding_symbol,
            base_ty,
            supplied,
            span,
            ClassRole::Instance,
        )
    }

    fn resolve_external_class_annotation(
        &mut self,
        display_name: &str,
        external: &ExternalClass,
        supplied: Option<&[TypeId]>,
        span: Span,
    ) -> TypeId {
        let Some(supplied) = supplied else {
            return self.external_class_instance(
                &external.module,
                &external.name,
                external.args.clone(),
            );
        };
        if !external.args.is_empty() {
            self.error(span, format!("type '{display_name}' is already specialized"));
            return self.external_class_instance(
                &external.module,
                &external.name,
                external.args.clone(),
            );
        }
        let Some((_class_id, class)) =
            super::stdlib_typespec::class_spec_any_name(&external.module, &external.name)
        else {
            return self.external_class_instance(
                &external.module,
                &external.name,
                supplied.to_vec(),
            );
        };
        let mut generic_params = GenericParams::new();
        for spec_id in super::stdlib_typespec::class_type_params(class) {
            let Some(ty) = self.materialize_stdlib_type_param(*spec_id) else {
                return self.external_class_instance(
                    &external.module,
                    &external.name,
                    supplied.to_vec(),
                );
            };
            let Ty::TypeVar(var_id) = self.tcx.get(ty) else {
                return self.external_class_instance(
                    &external.module,
                    &external.name,
                    supplied.to_vec(),
                );
            };
            let info = self.tcx.get_type_var(*var_id).clone();
            generic_params.add_param(
                &info.name,
                *var_id,
                info.kind,
                info.bound,
                info.constraints,
                info.default,
            );
        }
        if generic_params.is_empty() {
            if !supplied.is_empty() {
                self.error(span, format!("type '{display_name}' is not generic"));
            }
            return self.external_class_instance(
                &external.module,
                &external.name,
                Vec::new(),
            );
        }
        let (substitution, resolved, errors) =
            bind_explicit_type_args_without_bounds(&generic_params, supplied, &mut self.tcx);
        let shape_valid = errors.is_empty();
        for error in errors {
            self.error(span, error);
        }
        if shape_valid {
            if let Some(error) =
                self.stdlib_generic_bounds_error(&substitution, &generic_params)
            {
                self.error(span, error);
            }
        }
        self.external_class_instance(&external.module, &external.name, resolved)
    }

    fn resolve_dotted_external_class_annotation(
        &mut self,
        display_name: &str,
        supplied: Option<&[TypeId]>,
        span: Span,
    ) -> Option<TypeId> {
        let mut parts = display_name.split('.');
        let local_root = parts.next()?;
        let mut members: Vec<_> = parts.collect();
        let class_name = members.pop()?;
        let symbol = self.symbols.lookup(local_root)?;
        let root_ty = self.get_sym_type(symbol.0);
        let Ty::External(ExternalValue::Module { path, loaded }) = self.tcx.get(root_ty).clone()
        else {
            return None;
        };
        let mut module = path;
        for member in members {
            let child = format!("{module}.{member}");
            if !loaded
                .iter()
                .any(|loaded| loaded == &child || loaded.starts_with(&format!("{child}.")))
            {
                return None;
            }
            module = child;
        }
        let (module, qualifier) =
            super::stdlib_typespec::exported_class(&module, class_name)?;
        let external = ExternalClass {
            module: module.to_string(),
            name: qualifier.to_string(),
            args: Vec::new(),
        };
        Some(self.resolve_external_class_annotation(
            display_name,
            &external,
            supplied,
            span,
        ))
    }

    pub(crate) fn specialize_user_class_as(
        &mut self,
        name: &str,
        binding_symbol: SymbolId,
        base_ty: TypeId,
        supplied: Option<&[TypeId]>,
        span: Span,
        role: ClassRole,
    ) -> TypeId {
        let (symbol, base_user) = match self.tcx.get(base_ty) {
            Ty::Class {
                user: Some(user), ..
            } => (user.symbol, Some(user.clone())),
            Ty::Class { user: None, .. } => (binding_symbol, None),
            _ => return base_ty,
        };
        let Some(generic_params) = self.generic_defs.get(&symbol).cloned() else {
            if supplied.is_some() {
                self.error(span, format!("type '{name}' is not generic"));
            }
            return self.with_class_role(base_ty, role);
        };

        let is_open = base_user.as_ref().is_some_and(|user| {
            generic_params.params.len() == user.args.len()
                && generic_params.params.iter().zip(&user.args).all(
                    |(param, arg)| matches!(self.tcx.get(*arg), Ty::TypeVar(id) if *id == param.id),
                )
        });
        if !is_open {
            if supplied.is_some() {
                self.error(span, format!("type '{name}' is already specialized"));
            }
            return self.with_class_role(base_ty, role);
        }

        let (subst, resolved_args, errors) = if let Some(args) = supplied {
            bind_explicit_type_args(&generic_params, args, &mut self.tcx)
        } else {
            let Some((subst, resolved)) =
                complete_type_args(&generic_params, Substitution::new(), &mut self.tcx)
            else {
                return self.with_class_role(base_ty, role);
            };
            (subst, resolved, Vec::new())
        };
        for error in errors {
            self.error(span, error);
        }

        let declaration_ty = self.get_sym_type(symbol.0);
        self.apply_user_class_specialization(symbol, declaration_ty, &subst, &resolved_args, role)
    }

    pub(crate) fn apply_user_class_specialization(
        &mut self,
        symbol: SymbolId,
        base_ty: TypeId,
        subst: &Substitution,
        resolved_args: &[TypeId],
        role: ClassRole,
    ) -> TypeId {
        let Ty::Class {
            name: base_name,
            fields,
            match_args,
            ..
        } = self.tcx.get(base_ty).clone()
        else {
            return base_ty;
        };
        let fields = fields
            .iter()
            .map(|(field_name, field_ty)| {
                (field_name.clone(), subst.apply(*field_ty, &mut self.tcx))
            })
            .collect();
        let specialized = self.tcx.intern(Ty::Class {
            name: base_name,
            role,
            user: Some(UserClass {
                symbol,
                args: resolved_args.to_vec(),
            }),
            external: None,
            fields,
            match_args,
        });
        specialized
    }

    pub(crate) fn with_class_role(&mut self, ty: TypeId, role: ClassRole) -> TypeId {
        let Ty::Class {
            name,
            role: current_role,
            user,
            external,
            fields,
            match_args,
        } = self.tcx.get(ty).clone()
        else {
            return ty;
        };
        if current_role == role {
            return ty;
        }
        self.tcx.intern(Ty::Class {
            name,
            role,
            user,
            external,
            fields,
            match_args,
        })
    }

    pub(crate) fn external_class_instance(
        &mut self,
        module: &str,
        name: &str,
        args: Vec<TypeId>,
    ) -> TypeId {
        self.tcx.intern(Ty::Class {
            name: name.to_string(),
            role: ClassRole::Instance,
            user: None,
            external: Some(ExternalClass {
                module: module.to_string(),
                name: name.to_string(),
                args,
            }),
            fields: Vec::new(),
            match_args: None,
        })
    }

    pub(crate) fn external_class_object(
        &mut self,
        module: &str,
        name: &str,
        args: Vec<TypeId>,
    ) -> TypeId {
        let instance = self.external_class_instance(module, name, args);
        self.with_class_role(instance, ClassRole::Object)
    }

    pub(crate) fn stdlib_module_import_type(
        &mut self,
        path: &str,
        loaded: &str,
        previous: Option<TypeId>,
    ) -> TypeId {
        if !super::stdlib_typespec::module_exists(loaded) {
            return self.tcx.any();
        }
        let mut loaded_modules = vec![loaded.to_string()];
        if let Some(previous) = previous {
            if let Ty::External(ExternalValue::Module {
                path: previous_path,
                loaded: previous_loaded,
            }) = self.tcx.get(previous)
            {
                if previous_path == path {
                    loaded_modules.extend(previous_loaded.iter().cloned());
                }
            }
        }
        loaded_modules.sort();
        loaded_modules.dedup();
        self.tcx.intern(Ty::External(ExternalValue::Module {
            path: path.to_string(),
            loaded: loaded_modules,
        }))
    }

    pub(crate) fn stdlib_imported_member_type(
        &mut self,
        module: &str,
        member: &str,
    ) -> TypeId {
        if let Some((class_module, class_name)) =
            super::stdlib_typespec::exported_class(module, member)
        {
            return self.external_class_object(class_module, class_name, Vec::new());
        }
        if super::stdlib_typespec::module_callable_exists(module, member) {
            return self
                .tcx
                .intern(Ty::External(ExternalValue::Callable(ExternalCallable {
                    module: module.to_string(),
                    qualifier: String::new(),
                    name: member.to_string(),
                    access: ExternalCallableAccess::Module,
                    runtime_kind: ExternalCallableRuntimeKind::Unknown,
                    receiver: None,
                })));
        }
        let child = format!("{module}.{member}");
        if super::stdlib_typespec::module_exists(&child) {
            return self.stdlib_module_import_type(&child, &child, None);
        }
        self.tcx.any()
    }

    pub(crate) fn class_pattern_target(&mut self, path: &[Name]) -> ClassPatternTarget {
        let [name] = path else {
            return ClassPatternTarget::Unknown;
        };
        let unshadowed_builtin = self.is_unshadowed_builtin(name);
        if unshadowed_builtin {
            if let Some(instance_ty) = self.builtin_class_pattern_instance(name) {
                return ClassPatternTarget::Instance(instance_ty);
            }
        }

        let Some(symbol) = self.symbols.lookup(name) else {
            return ClassPatternTarget::Unknown;
        };
        if let Some(instance_ty) = self.builtin_class_aliases.get(&symbol).copied() {
            return ClassPatternTarget::Instance(instance_ty);
        }
        let ty = self.get_sym_type(symbol.0);
        match self.tcx.get(ty) {
            Ty::Class {
                role: ClassRole::Object,
                ..
            } => ClassPatternTarget::Instance(self.with_class_role(ty, ClassRole::Instance)),
            Ty::Any | Ty::Error => ClassPatternTarget::Unknown,
            Ty::Fn { .. } if unshadowed_builtin => ClassPatternTarget::Unknown,
            Ty::Fn { .. } if self.symbols.get_symbol(symbol).kind == SymbolKind::Variable => {
                ClassPatternTarget::Unknown
            }
            _ => ClassPatternTarget::Invalid,
        }
    }

    pub(crate) fn resolve_type_expr(&mut self, ty: &Spanned<TypeExpr>) -> TypeId {
        let resolved = self.resolve_type_expr_inner(ty);
        self.resolved_type_exprs.insert(ty.span, resolved);
        resolved
    }

    pub(crate) fn resolved_type_expr(&self, span: Span) -> Option<TypeId> {
        self.resolved_type_exprs.get(&span).copied()
    }

    fn resolve_type_expr_inner(&mut self, ty: &Spanned<TypeExpr>) -> TypeId {
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
                "set" => {
                    let a = self.tcx.any();
                    self.tcx.intern(Ty::Set(a))
                }
                "frozenset" => self.external_class_instance(
                    "builtins",
                    "frozenset",
                    vec![self.tcx.any()],
                ),
                "bytes" | "bytearray" | "memoryview" | "complex" | "range" | "slice" => {
                    self.external_class_instance("builtins", name, Vec::new())
                }
                "type" => self.tcx.intern(Ty::TypeObject(self.tcx.any())),
                "object" => self.tcx.any(),
                n if crate::parser::ast::strip_forward_ref_name(n).is_some() => {
                    let forwarded = crate::parser::ast::strip_forward_ref_name(n)
                        .expect("forward-reference prefix disappeared");
                    if !self
                        .type_alias_defs
                        .values()
                        .any(|definition| definition.resolving)
                    {
                        self.tcx.any()
                    } else if let Some(forwarded) = parse_forward_ref_type_expr(forwarded, ty.span)
                    {
                        self.resolve_type_expr_inner(&forwarded)
                    } else {
                        self.error(ty.span, "invalid type expression in forward reference");
                        self.tcx.error()
                    }
                }
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
                    if let Some(param_ty) = self.resolve_active_type_param_alias(name) {
                        return param_ty;
                    }
                    if let Some(symbol) = self.lookup_type_alias_symbol(name) {
                        return self.resolve_type_alias(name, symbol, None, ty.span);
                    }
                    // Legacy `T = TypeVar(...)` aliases remain name-based, but
                    // a lexical PEP 695 declaration shadows them.
                    if let Some(alias_ty) = self.tcx.resolve_alias(name) {
                        return alias_ty;
                    }
                    // User-defined type — look up in symbols
                    if let Some(sym) = self.symbols.lookup(name) {
                        if let Some((module, qualifier)) = self.import_origins.get(&sym).cloned() {
                            if let Some((module, qualifier)) =
                                super::stdlib_typespec::exported_class(&module, &qualifier)
                            {
                                return self.external_class_instance(
                                    module,
                                    qualifier,
                                    Vec::new(),
                                );
                            }
                        }
                        let base_ty = self.get_sym_type(sym.0);
                        self.resolve_named_class_annotation(name, sym, base_ty, None, ty.span)
                    } else if name.contains('.') {
                        if let Some(resolved) = self.resolve_dotted_external_class_annotation(
                            name,
                            None,
                            ty.span,
                        ) {
                            return resolved;
                        }
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
                    } else if self.allow_runtime_unresolved_names {
                        // Runtime execution keeps CPython's dynamic behavior:
                        // an unresolved annotation name must not abort module
                        // execution during static checking. Defer it to runtime
                        // annotation consumers and type the slot as Any.
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
                    "set" if inner.len() == 1 => self.tcx.intern(Ty::Set(inner[0])),
                    "frozenset" if inner.len() == 1 => self.external_class_instance(
                        "builtins",
                        "frozenset",
                        inner,
                    ),
                    "list" | "set" | "frozenset" => {
                        self.error(
                            ty.span,
                            format!("expected 1 type argument, got {}", inner.len()),
                        );
                        self.tcx.error()
                    }
                    "dict" => {
                        self.error(
                            ty.span,
                            format!("expected 2 type arguments, got {}", inner.len()),
                        );
                        self.tcx.error()
                    }
                    "Callable" if inner.len() >= 2 => {
                        // #243: Callable[[params...], ret] → Fn type
                        let ret = inner[inner.len() - 1];
                        let params = inner[..inner.len() - 1].to_vec();
                        self.tcx.intern(Ty::Fn {
                            params,
                            ret,
                            variadic: false,
                            signature: None,
                            param_spec: None,
                        })
                    }
                    "type" if inner.len() == 1 => self.tcx.intern(Ty::TypeObject(inner[0])),
                    "type" => {
                        self.error(
                            ty.span,
                            format!("expected 1 type argument, got {}", inner.len()),
                        );
                        self.tcx.error()
                    }
                    _ => {
                        if let Some(symbol) = self.lookup_type_alias_symbol(name) {
                            self.resolve_type_alias(name, symbol, Some(&inner), ty.span)
                        } else if self.is_unshadowed_builtin(name) {
                            if let Some((module, qualifier)) =
                                super::stdlib_typespec::exported_class("builtins", name)
                            {
                                let external = ExternalClass {
                                    module: module.to_string(),
                                    name: qualifier.to_string(),
                                    args: Vec::new(),
                                };
                                self.resolve_external_class_annotation(
                                    name,
                                    &external,
                                    Some(&inner),
                                    ty.span,
                                )
                            } else {
                                self.tcx.any()
                            }
                        // Support user-defined generic types like Box[int]
                        } else if let Some(sym) = self.symbols.lookup(name) {
                            let base_ty = self.get_sym_type(sym.0);
                            self.resolve_named_class_annotation(
                                name,
                                sym,
                                base_ty,
                                Some(&inner),
                                ty.span,
                            )
                        } else if let Some(alias_ty) = self.tcx.resolve_alias(name) {
                            alias_ty
                        } else if name.contains('.') {
                            self.resolve_dotted_external_class_annotation(
                                name,
                                Some(&inner),
                                ty.span,
                            )
                            .unwrap_or_else(|| self.tcx.any())
                        } else if self.allow_runtime_unresolved_names {
                            // Same run-mode carve-out as bare names above:
                            // unresolved generic heads in annotations should
                            // not fail the whole file before runtime.
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
                    signature: None,
                    param_spec: None,
                })
            }
            TypeExpr::Tuple(types) => {
                let inner: Vec<TypeId> = types.iter().map(|t| self.resolve_type_expr(t)).collect();
                self.tcx.intern(Ty::Tuple(inner))
            }
        }
    }

    /// Map a compact signature [`CoreTy`] to a concrete scalar [`TypeId`], or
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
            // `Type`/`List` by negative scalar walls, not here.
            CoreTy::Bytes
            | CoreTy::MemoryView
            | CoreTy::Complex
            | CoreTy::IntOrStr
            | CoreTy::PathOrFd
            | CoreTy::List
            | CoreTy::Tuple
            | CoreTy::Dict
            | CoreTy::Typed
            | CoreTy::TypedNamed(_)
            | CoreTy::Type
            | CoreTy::Unknown => None,
        }
    }

    /// True only when `actual` is a concrete scalar type we
    /// are confident about (Int/Float/Str/Bool/None). Any/Error/Union/typevar/
    /// collection/class -> not concrete -> the hook skips (zero false positives).
    pub(crate) fn is_concrete_scalar(&self, actual: TypeId) -> bool {
        matches!(
            self.tcx.get(actual),
            Ty::Int | Ty::Float | Ty::Str | Ty::Bool | Ty::None
        )
    }

    pub(crate) fn types_compatible(&mut self, expected: TypeId, actual: TypeId) -> bool {
        let mut visiting = HashSet::new();
        self.types_compatible_inner(expected, actual, &mut visiting)
    }

    fn types_compatible_inner(
        &mut self,
        expected: TypeId,
        actual: TypeId,
        visiting: &mut HashSet<(TypeId, TypeId)>,
    ) -> bool {
        if self.tcx.alias_ref_is_rejected(expected)
            || self.tcx.alias_ref_is_rejected(actual)
        {
            return false;
        }
        if expected == actual {
            return true;
        }
        if visiting.len() >= MAX_TYPE_COMPATIBILITY_DEPTH {
            return false;
        }
        if !visiting.insert((expected, actual)) {
            return true;
        }
        let compatible = self.types_compatible_step(expected, actual, visiting);
        visiting.remove(&(expected, actual));
        compatible
    }

    fn types_compatible_step(
        &mut self,
        expected: TypeId,
        actual: TypeId,
        visiting: &mut HashSet<(TypeId, TypeId)>,
    ) -> bool {
        let expected_head = self.materialize_alias_head(expected);
        let actual_head = self.materialize_alias_head(actual);
        if expected_head != expected || actual_head != actual {
            return self.types_compatible_inner(expected_head, actual_head, visiting);
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
        // SelfType denotes an instance receiver, never the class object.
        if matches!(e, Ty::SelfType) {
            return !matches!(
                a,
                Ty::Class {
                    role: ClassRole::Object,
                    ..
                }
            );
        }
        if matches!(a, Ty::SelfType) {
            return !matches!(
                e,
                Ty::Class {
                    role: ClassRole::Object,
                    ..
                }
            );
        }
        if let (Ty::TypeObject(expected_instance), Ty::TypeObject(actual_instance)) =
            (e.clone(), a.clone())
        {
            return self.types_compatible_inner(expected_instance, actual_instance, visiting);
        }
        if let (Ty::TypeObject(expected_instance), Ty::Class { role, .. }) =
            (e.clone(), a.clone())
        {
            if role != ClassRole::Object {
                return false;
            }
            let actual_instance = self.with_class_role(actual, ClassRole::Instance);
            return self.types_compatible_inner(expected_instance, actual_instance, visiting);
        }
        if let Ty::Class {
            role: ClassRole::Instance,
            external: Some(external),
            ..
        } = e.clone()
        {
            if let Ty::External(value) = a {
                let runtime_value_match = match value {
                    ExternalValue::Module { .. } => {
                        external.module == "types" && external.name == "ModuleType"
                    }
                    ExternalValue::Callable(callable) if external.module == "types" => matches!(
                        (callable.runtime_kind, external.name.as_str()),
                        (
                            ExternalCallableRuntimeKind::PythonFunction,
                            "FunctionType" | "LambdaType"
                        ) | (ExternalCallableRuntimeKind::PythonMethod, "MethodType")
                            | (
                                ExternalCallableRuntimeKind::BuiltinFunction,
                                "BuiltinFunctionType"
                            )
                            | (
                                ExternalCallableRuntimeKind::BuiltinMethod,
                                "BuiltinFunctionType" | "BuiltinMethodType"
                            )
                            | (
                                ExternalCallableRuntimeKind::WrapperDescriptor,
                                "WrapperDescriptorType"
                            )
                            | (
                                ExternalCallableRuntimeKind::MethodWrapper,
                                "MethodWrapperType"
                            )
                            | (
                                ExternalCallableRuntimeKind::MethodDescriptor,
                                "MethodDescriptorType"
                            )
                            | (
                                ExternalCallableRuntimeKind::ClassMethodDescriptor,
                                "ClassMethodDescriptorType"
                            )
                    ),
                    ExternalValue::Callable(_) => false,
                };
                if runtime_value_match {
                    return true;
                }
            }
            if external.module == "builtins"
                && external.name == "type"
                && external.args.is_empty()
                && matches!(
                    a,
                    Ty::Class {
                        role: ClassRole::Object,
                        ..
                    }
                )
            {
                return true;
            }
            if external.module == "builtins"
                && external.name == "complex"
                && matches!(a, Ty::Bool | Ty::Int | Ty::Float)
            {
                return true;
            }

            if external.module == "typing" && external.name == "MutableSequence" {
                let mutable_items = match a.clone() {
                    Ty::List(item) => Some(vec![item]),
                    Ty::Class {
                        external: Some(actual),
                        ..
                    } if actual.module == "builtins" && actual.name == "bytearray" => {
                        Some(vec![self.tcx.int()])
                    }
                    _ => None,
                };
                if let (Some(expected_item), Some(actual_items)) =
                    (external.args.first().copied(), mutable_items)
                {
                    return actual_items.into_iter().all(|actual_item| {
                        self.types_compatible_inner(expected_item, actual_item, visiting)
                    });
                }
            }

            let collection_item = match a.clone() {
                Ty::List(item) | Ty::Set(item) => Some(vec![item]),
                Ty::Tuple(items) => Some(items),
                Ty::Dict(key, _) => Some(vec![key]),
                Ty::Str => Some(vec![self.tcx.str()]),
                Ty::Class {
                    external: Some(actual),
                    ..
                } if actual.module == "builtins"
                    && matches!(
                        actual.name.as_str(),
                        "bytes" | "bytearray" | "range"
                    ) => Some(vec![self.tcx.int()]),
                Ty::Class {
                    external: Some(actual),
                    ..
                } if actual.module == "builtins" && actual.name == "frozenset" => {
                    Some(actual.args)
                }
                _ => None,
            };
            if matches!(
                (external.module.as_str(), external.name.as_str()),
                ("typing", "Iterable" | "Collection" | "Sequence")
            ) {
                if let (Some(expected_item), Some(actual_items)) =
                    (external.args.first().copied(), collection_item)
                {
                    return actual_items.into_iter().all(|actual_item| {
                        self.types_compatible_inner(expected_item, actual_item, visiting)
                    });
                }
            }
            if matches!(
                (external.module.as_str(), external.name.as_str()),
                ("typing", "Mapping" | "MutableMapping")
            ) {
                if let (Some(expected_key), Some(expected_value), Ty::Dict(key, value)) =
                    (external.args.first(), external.args.get(1), a.clone())
                {
                    return self.types_compatible_inner(*expected_key, key, visiting)
                        && self.types_compatible_inner(*expected_value, value, visiting);
                }
            }
        }
        // User-class compatibility is nominal by declaration symbol. Generic
        // arguments are invariant unless either side is still gradual.
        if let (
            Ty::Class {
                name: n1,
                role: role1,
                user: user1,
                external: external1,
                ..
            },
            Ty::Class {
                name: n2,
                role: role2,
                user: user2,
                external: external2,
                ..
            },
        ) = (e, a)
        {
            if role1 != role2 {
                return false;
            }
            match (external1, external2) {
                (Some(left), Some(right)) => {
                    return left.module == right.module
                        && left.name == right.name
                        && (left.args.is_empty()
                            || right.args.is_empty()
                            || (left.args.len() == right.args.len()
                                && left.args.iter().zip(&right.args).all(|(left, right)| {
                                    left == right
                                        || matches!(
                                            self.tcx.get(*left),
                                            Ty::Any | Ty::TypeVar(_)
                                        )
                                        || matches!(
                                            self.tcx.get(*right),
                                            Ty::Any | Ty::TypeVar(_)
                                        )
                                })));
                }
                (Some(left), None)
                    if left.module == "builtins"
                        && user2.is_none()
                        && is_exception_class_name(&left.name)
                        && is_exception_class_name(n2) =>
                {
                    return true;
                }
                (None, Some(right))
                    if right.module == "builtins"
                        && user1.is_none()
                        && is_exception_class_name(n1)
                        && is_exception_class_name(&right.name) =>
                {
                    return true;
                }
                (Some(_), None) | (None, Some(_)) => return false,
                (None, None) => {}
            }
            match (user1, user2) {
                (Some(left), Some(right)) if left.symbol == right.symbol => {
                    let args_compatible = left.args.len() == right.args.len()
                        && left.args.iter().zip(&right.args).all(|(left, right)| {
                            left == right
                                || matches!(self.tcx.get(*left), Ty::Any | Ty::TypeVar(_))
                                || matches!(self.tcx.get(*right), Ty::Any | Ty::TypeVar(_))
                        });
                    return args_compatible;
                }
                (Some(left), Some(right))
                    if left.args.is_empty()
                        && self.user_class_is_subclass(right.symbol, left.symbol) =>
                {
                    return true;
                }
                (None, None) if n1 == n2 => return true,
                _ => {}
            }
            // Exception class hierarchy: all exception types are compatible
            // with each other (they all derive from BaseException).
            if user1.is_none()
                && user2.is_none()
                && is_exception_class_name(n1)
                && is_exception_class_name(n2)
            {
                return true;
            }
        }
        // PEP 589: class-form TypedDict is a structural schema at type-check
        // time but its runtime values are plain dicts.
        if let (
            Ty::Class {
                name,
                role: ClassRole::Instance,
                user,
                ..
            },
            Ty::Dict(_, _),
        ) = (e, a)
        {
            let is_typed_dict = user
                .as_ref()
                .is_some_and(|user| self.typed_dict_class_symbols.contains(&user.symbol))
                || (user.is_none() && self.typed_dict_classes.contains(name));
            if is_typed_dict {
                return true;
            }
        }
        // #314: Protocol structural subtyping — if expected is a protocol class,
        // check if actual class structurally satisfies it
        if let Ty::Class {
            name: proto_name,
            role: expected_role,
            user: expected_user,
            ..
        } = e
        {
            let protocol = expected_user
                .as_ref()
                .and_then(|user| self.protocols_by_symbol.get(&user.symbol))
                .or_else(|| {
                    expected_user
                        .is_none()
                        .then(|| self.protocol_registry.get(proto_name))
                        .flatten()
                });
            if *expected_role == ClassRole::Instance && protocol.is_some() {
                if let Ty::Class {
                    name: class_name,
                    role: actual_role,
                    user: actual_user,
                    ..
                } = a
                {
                    if *actual_role == ClassRole::Object {
                        return false;
                    }
                    let class_methods = actual_user
                        .as_ref()
                        .and_then(|user| self.class_methods_by_symbol.get(&user.symbol))
                        .or_else(|| {
                            actual_user
                                .is_none()
                                .then(|| self.class_methods.get(class_name))
                                .flatten()
                        })
                        .cloned()
                        .unwrap_or_default();
                    let class_attrs: HashMap<String, TypeId> = if let Ty::Class { fields, .. } = a {
                        fields.iter().cloned().collect()
                    } else {
                        HashMap::new()
                    };
                    return ProtocolRegistry::satisfies_definition(
                        protocol.expect("checked above"),
                        &class_methods,
                        &class_attrs,
                        &self.tcx,
                    );
                }
            }
        }
        // Union-to-union compatibility maps every actual branch to at least
        // one expected branch. This must run before the one-sided rules so
        // structurally equivalent recursive aliases compare coinductively.
        if let (Ty::Union(expected), Ty::Union(actual)) = (e, a) {
            let expected = expected.clone();
            let actual = actual.clone();
            return actual.iter().all(|actual| {
                expected
                    .iter()
                    .any(|expected| self.types_compatible_inner(*expected, *actual, visiting))
            });
        }
        // Union compatibility: actual is compatible if it matches any member
        if let Ty::Union(members) = e {
            let members = members.clone();
            return members
                .iter()
                .any(|m| self.types_compatible_inner(*m, actual, visiting));
        }
        if let Ty::Union(members) = a {
            let members = members.clone();
            return members
                .iter()
                .all(|m| self.types_compatible_inner(expected, *m, visiting));
        }
        // Recursive collection compatibility: List[X] ≈ List[Y] when X ≈ Y,
        // similarly for Set and Dict. This handles annotations like
        // `list[dict]` where the inner type resolves to Any and must match
        // concrete types.
        if let (Ty::List(inner_e), Ty::List(inner_a)) = (e, a) {
            let (ie, ia) = (*inner_e, *inner_a);
            return self.types_compatible_inner(ie, ia, visiting);
        }
        if let (Ty::Set(inner_e), Ty::Set(inner_a)) = (e, a) {
            let (ie, ia) = (*inner_e, *inner_a);
            return self.types_compatible_inner(ie, ia, visiting);
        }
        if let (Ty::Dict(ke, ve), Ty::Dict(ka, va)) = (e, a) {
            let (ke, ve, ka, va) = (*ke, *ve, *ka, *va);
            return self.types_compatible_inner(ke, ka, visiting)
                && self.types_compatible_inner(ve, va, visiting);
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
                return es.iter().zip(as_.iter()).all(|(&elem_e, &elem_a)| {
                    self.types_compatible_inner(elem_e, elem_a, visiting)
                });
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
                    .all(|&elem_a| self.types_compatible_inner(elem_e, elem_a, visiting));
            }
            if as_.len() == 2 && self.tcx.get(as_[1]).is_any() {
                let elem_a = as_[0];
                return es
                    .iter()
                    .all(|&elem_e| self.types_compatible_inner(elem_e, elem_a, visiting));
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
                variadic: expected_variadic,
                param_spec: expected_param_spec,
                ..
            },
            Ty::Fn {
                params: pa,
                ret: ra,
                variadic: actual_variadic,
                param_spec: actual_param_spec,
                ..
            },
        ) = (e, a)
        {
            let (pe, pa, re, ra, expected_variadic, actual_variadic) = (
                pe.clone(),
                pa.clone(),
                *re,
                *ra,
                *expected_variadic,
                *actual_variadic,
            );
            let (expected_param_spec, actual_param_spec) =
                (*expected_param_spec, *actual_param_spec);
            if expected_param_spec.is_some() || actual_param_spec.is_some() {
                return false;
            }
            if !self.types_compatible_inner(re, ra, visiting) {
                return false;
            }
            if expected_variadic && pe.is_empty() {
                return true;
            }
            let compare_prefix = |this: &mut Self,
                                  expected: &[TypeId],
                                  actual: &[TypeId],
                                  visiting: &mut HashSet<(TypeId, TypeId)>| {
                expected
                    .iter()
                    .zip(actual)
                    .all(|(&te, &ta)| this.types_compatible_inner(ta, te, visiting))
            };
            if expected_variadic {
                if !actual_variadic && pa.len() < pe.len() {
                    return false;
                }
                return compare_prefix(self, &pe, &pa, visiting);
            }
            if actual_variadic {
                if pa.len() > pe.len() {
                    return false;
                }
                return compare_prefix(self, &pe, &pa, visiting);
            }
            return pe.len() == pa.len() && compare_prefix(self, &pe, &pa, visiting);
        }
        if matches!(e, Ty::Fn { .. })
            && matches!(a, Ty::External(ExternalValue::Callable(_)))
        {
            return true;
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
        self.ty_name_inner(ty, &mut HashSet::new())
    }

    fn ty_name_inner(&self, ty: TypeId, visiting: &mut HashSet<AliasInstanceId>) -> String {
        match self.tcx.get(ty) {
            Ty::Never => "Never".into(),
            Ty::None => "None".into(),
            Ty::Bool => "bool".into(),
            Ty::Int => "int".into(),
            Ty::Float => "float".into(),
            Ty::Str => "str".into(),
            Ty::Any => "Any".into(),
            Ty::External(ExternalValue::Module { path, .. }) => path.clone(),
            Ty::External(ExternalValue::Callable(callable)) => {
                if callable.qualifier.is_empty() {
                    format!("{}.{}", callable.module, callable.name)
                } else {
                    format!(
                        "{}.{}.{}",
                        callable.module, callable.qualifier, callable.name
                    )
                }
            }
            Ty::List(inner) => format!("list[{}]", self.ty_name_inner(*inner, visiting)),
            Ty::Set(inner) => format!("set[{}]", self.ty_name_inner(*inner, visiting)),
            Ty::Dict(k, v) => format!(
                "dict[{}, {}]",
                self.ty_name_inner(*k, visiting),
                self.ty_name_inner(*v, visiting)
            ),
            Ty::Tuple(ts) => {
                let parts: Vec<_> = ts
                    .iter()
                    .map(|t| self.ty_name_inner(*t, visiting))
                    .collect();
                format!("tuple[{}]", parts.join(", "))
            }
            Ty::Union(ts) => {
                let parts: Vec<_> = ts
                    .iter()
                    .map(|t| self.ty_name_inner(*t, visiting))
                    .collect();
                parts.join(" | ")
            }
            Ty::Fn {
                params,
                ret,
                variadic,
                param_spec,
                ..
            } => {
                let mut ps: Vec<_> = params
                    .iter()
                    .map(|p| self.ty_name_inner(*p, visiting))
                    .collect();
                if let Some(param_spec) = param_spec {
                    ps.push(format!("{}...", self.tcx.get_type_var(*param_spec).name));
                } else if *variadic {
                    ps.push("...".to_string());
                }
                format!(
                    "({}) -> {}",
                    ps.join(", "),
                    self.ty_name_inner(*ret, visiting)
                )
            }
            Ty::TypeObject(instance) => {
                format!("type[{}]", self.ty_name_inner(*instance, visiting))
            }
            Ty::Class {
                external: Some(external),
                ..
            } => {
                let name = if external.module == "builtins" {
                    external.name.clone()
                } else {
                    format!("{}.{}", external.module, external.name)
                };
                if external.args.is_empty() {
                    name
                } else {
                    let args: Vec<_> = external
                        .args
                        .iter()
                        .map(|arg| self.ty_name_inner(*arg, visiting))
                        .collect();
                    format!("{name}[{}]", args.join(", "))
                }
            }
            Ty::Class {
                name,
                user: Some(user),
                ..
            } if !user.args.is_empty() => {
                let args: Vec<_> = user
                    .args
                    .iter()
                    .map(|arg| self.ty_name_inner(*arg, visiting))
                    .collect();
                format!("{name}[{}]", args.join(", "))
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
            Ty::AliasRef(id) => {
                let instance = self.tcx.alias_instance(*id);
                let name = instance.name.clone();
                let args = instance.args[..instance.display_arg_count].to_vec();
                if args.is_empty() || !visiting.insert(*id) {
                    return name;
                }
                let args: Vec<_> = args
                    .iter()
                    .map(|arg| self.ty_name_inner(*arg, visiting))
                    .collect();
                visiting.remove(id);
                format!("{name}[{}]", args.join(", "))
            }
            Ty::Infer(_) => "?".into(),
            Ty::Error => "<error>".into(),
        }
    }

    /// Register a Protocol class in the protocol registry.
    pub(crate) fn register_protocol(
        &mut self,
        class_symbol: SymbolId,
        name: &str,
        body: &[Spanned<Stmt>],
    ) {
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

        let protocol = Protocol {
            name: name.to_string(),
            methods,
            attrs,
            runtime_checkable: false,
        };
        self.protocols_by_symbol
            .insert(class_symbol, protocol.clone());
        self.protocol_registry.register(protocol);
    }

    /// Collect method signatures from a class body for protocol conformance.
    pub(crate) fn collect_class_methods(
        &mut self,
        class_symbol: SymbolId,
        class_name: &str,
        body: &[Spanned<Stmt>],
    ) {
        use super::protocol::MethodSig;

        let mut methods = HashMap::new();
        let mut method_param_sigs = HashMap::new();
        let mut protocol_indeterminate_methods = HashSet::new();
        let mut unbound_methods = HashMap::new();
        let mut unbound_method_param_sigs = HashMap::new();
        let mut property_getters = HashMap::new();
        let mut property_setters = HashMap::new();
        self.class_method_generic_defs
            .retain(|(owner, _), _| *owner != class_symbol);
        let receiver_ty = self.get_symbol_type(class_symbol).unwrap_or_else(|| {
                self.tcx.intern(Ty::Class {
                    name: class_name.to_string(),
                    role: ClassRole::Instance,
                    user: Some(UserClass {
                        symbol: class_symbol,
                        args: Vec::new(),
                    }),
                    external: None,
                    fields: vec![],
                    match_args: None,
                })
            });
        let receiver_ty = self.with_class_role(receiver_ty, ClassRole::Instance);
        for stmt in body {
            if let Stmt::FnDef {
                decorators,
                name,
                type_params,
                params,
                return_ty,
                ..
            }
            | Stmt::AsyncFnDef {
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
                let gp = self.register_type_params(type_params);
                let is_staticmethod = decorators.iter().any(|decorator| {
                    matches!(&decorator.node, Expr::Ident(name) if name == "staticmethod")
                        || matches!(
                            &decorator.node,
                            Expr::Attr { attr, .. } if attr == "staticmethod"
                        )
                });
                let protocol_signature_unfaithful = decorators.iter().any(|decorator| {
                    matches!(
                        &decorator.node,
                        Expr::Ident(name) if matches!(name.as_str(), "overload" | "property")
                    ) || matches!(
                        &decorator.node,
                        Expr::Attr { attr, .. }
                            if matches!(attr.as_str(), "overload" | "property" | "setter")
                    )
                });
                let all_param_sigs: Vec<FunctionParamSig> = params
                    .iter()
                    .map(|p| FunctionParamSig {
                        name: p.name.clone(),
                        ty: self.resolve_type_expr(&p.ty),
                        kind: p.kind,
                        pos_only: p.pos_only,
                        kw_only: p.kw_only,
                        has_default: p.default.is_some(),
                    })
                    .collect();
                let param_sigs: Vec<FunctionParamSig> = if is_staticmethod {
                    all_param_sigs.clone()
                } else {
                    all_param_sigs.iter().skip(1).cloned().collect()
                };
                let param_types = param_sigs.iter().map(|p| p.ty).collect();
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
                if !gp.is_empty() {
                    self.class_method_generic_defs
                        .insert((class_symbol, name.clone()), gp);
                }
                let is_property_getter = decorators.iter().any(|decorator| {
                    matches!(&decorator.node, Expr::Ident(name) if name == "property")
                        || matches!(&decorator.node, Expr::Attr { attr, .. } if attr == "property")
                });
                let is_property_setter = decorators.iter().any(|decorator| {
                    matches!(&decorator.node, Expr::Attr { attr, .. } if attr == "setter")
                });
                if is_property_getter {
                    property_getters.insert(name.clone(), ret);
                }
                if is_property_setter {
                    property_setters.insert(
                        name.clone(),
                        param_sigs
                            .first()
                            .map(|param| param.ty)
                            .unwrap_or_else(|| self.tcx.any()),
                    );
                }
                if protocol_signature_unfaithful {
                    protocol_indeterminate_methods.insert(name.clone());
                }
                methods.insert(
                    name.clone(),
                    MethodSig {
                        params: param_types,
                        return_type: ret,
                    },
                );
                method_param_sigs.insert(name.clone(), param_sigs);
                if let Some(unbound_param_types) = unbound_param_types {
                    let mut unbound_param_sigs = all_param_sigs.clone();
                    unbound_param_sigs[0].ty = receiver_ty;
                    unbound_method_param_sigs.insert(name.clone(), unbound_param_sigs);
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
        self.class_methods_by_symbol
            .insert(class_symbol, methods.clone());
        self.class_methods.insert(class_name.to_string(), methods);
        self.class_method_param_sigs
            .insert(class_symbol, method_param_sigs);
        self.protocol_indeterminate_methods
            .insert(class_symbol, protocol_indeterminate_methods);
        self.class_unbound_methods
            .insert(class_symbol, unbound_methods);
        self.class_unbound_method_param_sigs
            .insert(class_symbol, unbound_method_param_sigs);
        self.class_property_getters
            .insert(class_symbol, property_getters);
        self.class_property_setters
            .insert(class_symbol, property_setters);
    }

    fn base_is_typed_dict(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(name) if name == "TypedDict" => true,
            Expr::Ident(name) => self
                .symbols
                .lookup(name)
                .and_then(|symbol| match self.tcx.get(self.get_sym_type(symbol.0)) {
                    Ty::Class {
                        role: ClassRole::Object,
                        user: Some(user),
                        ..
                    } => Some(user.symbol),
                    _ => None,
                })
                .is_some_and(|symbol| self.typed_dict_class_symbols.contains(&symbol)),
            Expr::Attr { attr, .. } => attr == "TypedDict",
            _ => false,
        }
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
    fn generated_transformed_recursive_alias_backfills_transactionally() {
        use crate::types::context::AliasIdentity;
        use crate::types::generic::Substitution;
        use crate::types::stdlib_typespec::StrSpecId;

        let mut checker = TypeChecker::new();
        let var = checker.tcx.new_type_var("T".to_string(), None, Vec::new());
        let var_ty = checker.tcx.intern(Ty::TypeVar(var));
        let (template, template_ref) = checker.tcx.intern_alias_instance(
            AliasIdentity::Generated(StrSpecId(1), StrSpecId(2)),
            "example.A".to_string(),
            vec![var_ty],
            1,
        );
        assert!(checker.tcx.begin_alias_target(template));
        let mut subst = Substitution::new();
        let list_of_var = checker.tcx.intern(Ty::List(var_ty));
        subst.insert(var, list_of_var);
        let specialized_ref = subst.apply(template_ref, &mut checker.tcx);
        let Ty::AliasRef(specialized) = checker.tcx.get(specialized_ref) else {
            panic!("transformed recursive specialization lost its AliasRef");
        };
        let specialized = *specialized;
        assert!(checker.tcx.alias_target(specialized).is_none());
        assert!(checker.tcx.deferred_alias_target(specialized).is_some());

        let target = checker.tcx.intern(Ty::List(template_ref));
        checker.tcx.set_alias_target(template, target);

        let checkpoint = checker.tcx.begin_alias_target_transaction();
        let rolled_back = checker
            .materialize_alias_instance(specialized)
            .expect("productive transformed alias must materialize");
        assert!(matches!(checker.tcx.get(rolled_back), Ty::List(_)));
        checker
            .tcx
            .finish_alias_target_transaction(checkpoint, false);
        assert!(checker.tcx.alias_target(specialized).is_none());
        assert!(checker.tcx.deferred_alias_target(specialized).is_some());

        let first = checker.materialize_alias_instance(specialized);
        let second = checker.materialize_alias_instance(specialized);
        assert_eq!(
            first, second,
            "repeated lookup must retain the backfilled target"
        );
        let target = first.expect("productive transformed alias must materialize");
        let Ty::List(edge) = checker.tcx.get(target) else {
            panic!("transformed recursive alias lost its productive list head");
        };
        assert_eq!(*edge, specialized_ref);
        assert!(checker.tcx.deferred_alias_target(specialized).is_none());
        assert!(!checker.tcx.alias_target_is_rejected(specialized));
        assert!(!checker
            .tcx
            .alias_target_has_invalid_generated_edge(specialized, target));
        assert!(!matches!(checker.tcx.get(target), Ty::Any | Ty::Error));
        let str_ty = checker.tcx.str();
        assert!(!checker.types_compatible(specialized_ref, str_ty));
    }

    #[test]
    fn generated_parameter_changing_recursive_alias_fails_closed() {
        use crate::types::context::AliasIdentity;
        use crate::types::generic::Substitution;
        use crate::types::stdlib_typespec::StrSpecId;

        let mut checker = TypeChecker::new();
        let var = checker.tcx.new_type_var("T".to_string(), None, Vec::new());
        let var_ty = checker.tcx.intern(Ty::TypeVar(var));
        let (template, template_ref) = checker.tcx.intern_alias_instance(
            AliasIdentity::Generated(StrSpecId(3), StrSpecId(4)),
            "example.NonRegular".to_string(),
            vec![var_ty],
            1,
        );
        assert!(checker.tcx.begin_alias_target(template));
        let mut subst = Substitution::new();
        let list_of_var = checker.tcx.intern(Ty::List(var_ty));
        subst.insert(var, list_of_var);
        let specialized_ref = subst.apply(template_ref, &mut checker.tcx);
        let Ty::AliasRef(specialized) = checker.tcx.get(specialized_ref) else {
            panic!("parameter-changing specialization lost its AliasRef");
        };
        let specialized = *specialized;
        let target = checker.tcx.intern(Ty::List(specialized_ref));
        checker.tcx.set_alias_target(template, target);

        let checkpoint = checker.tcx.begin_alias_target_transaction();
        let never = checker.tcx.never();
        assert_eq!(
            checker.materialize_alias_instance(specialized),
            Some(never)
        );
        assert!(checker.tcx.deferred_alias_target(specialized).is_none());
        assert!(checker.tcx.alias_target_is_rejected(specialized));
        checker
            .tcx
            .finish_alias_target_transaction(checkpoint, false);
        assert!(checker.tcx.alias_target(specialized).is_none());
        assert!(checker.tcx.deferred_alias_target(specialized).is_some());
        assert!(!checker.tcx.alias_target_is_rejected(specialized));

        let first = checker.materialize_alias_instance(specialized);
        let second = checker.materialize_alias_instance(specialized);
        assert_eq!(first, Some(checker.tcx.never()));
        assert_eq!(second, first);
        assert!(checker.tcx.deferred_alias_target(specialized).is_none());
        assert!(checker.tcx.alias_target_is_rejected(specialized));
        assert_eq!(
            checker.tcx.semantic_head_id(specialized_ref),
            Err(crate::types::context::AliasHeadError::Rejected(specialized))
        );
        let str_ty = checker.tcx.str();
        assert!(!checker.types_compatible(specialized_ref, specialized_ref));
        assert!(!checker.types_compatible(specialized_ref, str_ty));
        assert!(!checker.types_compatible(str_ty, specialized_ref));
    }

    #[test]
    fn generated_unguarded_transformed_alias_fails_closed() {
        use crate::types::context::AliasIdentity;
        use crate::types::generic::Substitution;
        use crate::types::stdlib_typespec::StrSpecId;

        let mut checker = TypeChecker::new();
        let var = checker.tcx.new_type_var("T".to_string(), None, Vec::new());
        let var_ty = checker.tcx.intern(Ty::TypeVar(var));
        let (template, template_ref) = checker.tcx.intern_alias_instance(
            AliasIdentity::Generated(StrSpecId(5), StrSpecId(6)),
            "example.Direct".to_string(),
            vec![var_ty],
            1,
        );
        assert!(checker.tcx.begin_alias_target(template));
        let mut subst = Substitution::new();
        let list_of_var = checker.tcx.intern(Ty::List(var_ty));
        subst.insert(var, list_of_var);
        let specialized_ref = subst.apply(template_ref, &mut checker.tcx);
        let Ty::AliasRef(specialized) = checker.tcx.get(specialized_ref) else {
            panic!("transformed specialization lost its AliasRef");
        };
        let specialized = *specialized;
        checker.tcx.set_alias_target(template, specialized_ref);

        let first = checker.materialize_alias_instance(specialized);
        let second = checker.materialize_alias_instance(specialized);
        assert_eq!(first, Some(checker.tcx.never()));
        assert_eq!(second, first);
        assert!(checker.tcx.deferred_alias_target(specialized).is_none());
        assert!(checker.tcx.alias_target_is_rejected(specialized));
        let str_ty = checker.tcx.str();
        assert!(!checker.types_compatible(specialized_ref, specialized_ref));
        assert!(!checker.types_compatible(specialized_ref, str_ty));
        assert!(!checker.types_compatible(str_ty, specialized_ref));
    }

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
        let mut tc = TypeChecker::new();
        assert!(tc.types_compatible(tc.tcx.int(), tc.tcx.int()));
        assert!(tc.types_compatible(tc.tcx.str(), tc.tcx.str()));
        assert!(tc.types_compatible(tc.tcx.none(), tc.tcx.none()));
    }

    #[test]
    fn test_types_compatible_error_always_compatible() {
        let mut tc = TypeChecker::new();
        assert!(tc.types_compatible(tc.tcx.error(), tc.tcx.int()));
        assert!(tc.types_compatible(tc.tcx.int(), tc.tcx.error()));
        assert!(tc.types_compatible(tc.tcx.error(), tc.tcx.error()));
    }

    #[test]
    fn test_types_compatible_any_always_compatible() {
        let mut tc = TypeChecker::new();
        assert!(tc.types_compatible(tc.tcx.any(), tc.tcx.int()));
        assert!(tc.types_compatible(tc.tcx.int(), tc.tcx.any()));
        assert!(tc.types_compatible(tc.tcx.any(), tc.tcx.str()));
    }

    #[test]
    fn test_types_compatible_different_primitives() {
        let mut tc = TypeChecker::new();
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
            role: ClassRole::Instance,
            user: Some(UserClass {
                symbol: SymbolId(100),
                args: vec![tc.tcx.any()],
            }),
            external: None,
            fields: vec![],
            match_args: None,
        });
        let c2 = tc.tcx.intern(Ty::Class {
            name: "Box".to_string(),
            role: ClassRole::Instance,
            user: Some(UserClass {
                symbol: SymbolId(100),
                args: vec![tc.tcx.int()],
            }),
            external: None,
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
            name: "Box".to_string(),
            role: ClassRole::Instance,
            user: Some(UserClass {
                symbol: SymbolId(100),
                args: vec![tc.tcx.int()],
            }),
            external: None,
            fields: vec![],
            match_args: None,
        });
        let c2 = tc.tcx.intern(Ty::Class {
            name: "Box".to_string(),
            role: ClassRole::Instance,
            user: Some(UserClass {
                symbol: SymbolId(100),
                args: vec![tc.tcx.str()],
            }),
            external: None,
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
            role: ClassRole::Instance,
            user: Some(UserClass {
                symbol: SymbolId(100),
                args: vec![],
            }),
            external: None,
            fields: vec![],
            match_args: None,
        });
        let c2 = tc.tcx.intern(Ty::Class {
            name: "Bar".to_string(),
            role: ClassRole::Instance,
            user: Some(UserClass {
                symbol: SymbolId(101),
                args: vec![],
            }),
            external: None,
            fields: vec![],
            match_args: None,
        });
        assert!(!tc.types_compatible(c1, c2));
    }

    #[test]
    fn test_external_class_identity_is_module_qualified() {
        let mut tc = TypeChecker::new();
        let external = |tc: &mut TypeChecker, module: &str| {
            tc.tcx.intern(Ty::Class {
                name: "Path".to_string(),
                role: ClassRole::Instance,
                user: None,
                external: Some(super::super::ty::ExternalClass {
                    module: module.to_string(),
                    name: "Path".to_string(),
                    args: Vec::new(),
                }),
                fields: Vec::new(),
                match_args: None,
            })
        };
        let pathlib = external(&mut tc, "pathlib");
        let zipfile = external(&mut tc, "zipfile");
        assert!(tc.types_compatible(pathlib, pathlib));
        assert!(!tc.types_compatible(pathlib, zipfile));
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
            signature: None,
            param_spec: None,
        });
        assert_eq!(tc.ty_name(fn_ty), "(int, str) -> bool");
    }

    #[test]
    fn test_ty_name_class() {
        let mut tc = TypeChecker::new();
        let class_ty = tc.tcx.intern(Ty::Class {
            name: "MyClass".to_string(),
            role: ClassRole::Instance,
            user: None,
            external: None,
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
    fn test_nested_type_param_scopes_restore_shadowed_aliases() {
        let mut tc = TypeChecker::new();
        let outer_ty = tc.tcx.int();
        tc.tcx.register_alias("T".to_string(), outer_ty);
        let params = [crate::parser::ast::TypeParam::plain("T")];

        tc.register_type_params(&params);
        let first_scope_ty = tc.tcx.resolve_alias("T").unwrap();
        assert_ne!(first_scope_ty, outer_ty);

        tc.register_type_params(&params);
        let second_scope_ty = tc.tcx.resolve_alias("T").unwrap();
        assert_ne!(second_scope_ty, first_scope_ty);

        tc.unregister_type_params(&params);
        assert_eq!(tc.tcx.resolve_alias("T"), Some(first_scope_ty));
        tc.unregister_type_params(&params);
        assert_eq!(tc.tcx.resolve_alias("T"), Some(outer_ty));
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
