use crate::resolve::SymbolId;
use crate::source::span::Span;
use crate::types::TypeId;
use std::collections::HashMap;

/// HIR Module — desugared, all names resolved to SymbolId, types resolved to TypeId.
#[derive(Debug, Clone)]
pub struct HirModule {
    pub functions: Vec<HirFunction>,
    pub classes: Vec<HirClass>,
    pub top_level: Vec<HirStmt>,
    pub imports: Vec<HirImport>,
    /// Reverse mapping from synthetic SymbolId → variable name (for REPL persistence).
    pub sym_names: HashMap<SymbolId, String>,
    /// SymbolId → TypeId mapping for REPL type-preserving global restoration.
    pub sym_types: HashMap<SymbolId, TypeId>,
    /// Module-scope variable annotations in source order: `(name, type_repr)`.
    /// Populated for `x: int` / `x: int = 1` at module scope so the runtime can
    /// auto-create and populate the module `__annotations__` dict (CPython
    /// semantics). `type_repr` is the textual annotation (e.g. "int").
    pub module_annotations: Vec<(String, String)>,
    /// SymbolId.0 → introspection signature metadata for each user-defined
    /// `def` (parameter names/kinds/defaults/annotations + return annotation).
    /// Primed into the runtime FUNC_PARAMS registry at module init so
    /// `inspect.signature(f)` reflects the real declared signature.
    pub func_sigs: HashMap<u32, HirFuncSig>,
}

/// Introspection signature metadata for one user-defined function.
#[derive(Debug, Clone, Default)]
pub struct HirFuncSig {
    pub params: Vec<HirParamSig>,
    /// Textual return annotation (`"int"`, `"42"`), None when undeclared.
    pub return_annotation: Option<String>,
}

/// One parameter's introspection metadata.
#[derive(Debug, Clone)]
pub struct HirParamSig {
    pub name: String,
    /// CPython `inspect.Parameter` kind ordinal: 0 POSITIONAL_ONLY,
    /// 1 POSITIONAL_OR_KEYWORD, 2 VAR_POSITIONAL, 3 KEYWORD_ONLY,
    /// 4 VAR_KEYWORD.
    pub kind: u8,
    /// Literal default value when one is declared and representable.
    pub default: Option<HirSigDefault>,
    /// True when a default is declared but not a representable literal —
    /// the runtime records "has a default" with a None placeholder.
    pub default_opaque: bool,
    /// Textual annotation (`"int"`), None when un-annotated.
    pub annotation: Option<String>,
}

/// Literal default values representable without evaluating module code.
#[derive(Debug, Clone)]
pub enum HirSigDefault {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    None,
}

#[derive(Debug, Clone)]
pub struct HirFunction {
    pub name: SymbolId,
    pub params: Vec<(SymbolId, TypeId)>,
    pub return_ty: TypeId,
    pub body: Vec<HirStmt>,
    pub span: Span,
    /// Captured variable symbols (for closures)
    pub captures: Vec<SymbolId>,
    /// Whether this is an async function
    pub is_async: bool,
    /// Whether this is a generator (contains yield)
    pub is_generator: bool,
    /// Decorator expressions (applied outer-to-inner)
    pub decorators: Vec<HirExpr>,
    /// Whether this function has a `*args` parameter.
    /// When true, the last-but-one param (before **kwargs if present) receives a packed list.
    pub has_star_args: bool,
    /// Index into `params` of the `*args` parameter, when present. Recorded
    /// explicitly because keyword-only params after `*args` make the position
    /// non-derivable from `has_star_args`/`has_kwargs` alone.
    pub star_param_pos: Option<usize>,
    /// Whether this function has a `**kwargs` parameter.
    /// When true, the last param receives a packed dict.
    pub has_kwargs: bool,
}

#[derive(Debug, Clone)]
pub struct NamedTupleBaseSpec {
    pub tuple_name: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HirClass {
    pub name: SymbolId,
    pub base: Option<SymbolId>,
    /// All base classes for multiple inheritance (P1 OOP conformance).
    /// When non-empty, takes priority over `base` for MRO computation.
    pub all_bases: Vec<SymbolId>,
    /// Literal `namedtuple("T", [...])` base metadata for namedtuple subclasses.
    pub namedtuple_base: Option<NamedTupleBaseSpec>,
    pub fields: Vec<(SymbolId, TypeId)>,
    pub methods: Vec<HirFunction>,
    pub span: Span,
    pub decorators: Vec<HirExpr>,
    /// Explicit `__match_args__` tuple from the class body, if present (#827).
    /// When set, takes priority over `__init__`-derived match args in lowering.
    pub explicit_match_args: Option<Vec<String>>,
    /// Metaclass name from `class Foo(metaclass=Meta)`, if specified.
    pub metaclass: Option<String>,
    /// Class-level attribute assignments from class body (P2-R3).
    /// e.g., `attr = Verbose()` in class body → (attr_name, value_expr).
    /// Used for descriptor protocol support.
    pub class_attr_assigns: Vec<(String, HirExpr)>,
    /// `__slots__` declaration from class body (R14).
    /// e.g., `__slots__ = ['x', 'y']` → Some(vec!["x", "y"])
    pub slots: Option<Vec<String>>,
    /// Keyword arguments from class statement, excluding `metaclass=` (R10).
    /// e.g., `class Child(Base, registry="users")` → vec![("registry", expr)]
    pub class_kwargs: Vec<(String, HirExpr)>,
    /// Ordered dataclass field facts collected from the class body when the
    /// class carries a `@dataclass`-shaped decorator (PEP 557):
    /// `(field_name, annotation_repr, default_expr)`. `default_expr` is None
    /// for bare annotations (`x: int`); annotated assignments (`y: int = 0`,
    /// `z: list = field(default_factory=list)`) carry the lowered value
    /// expression. Emitted at the ClassDefPlaceholder position (before the
    /// decorator call) so the runtime `@dataclass` synthesizer sees ordered
    /// field facts with class-definition-time default values.
    pub dataclass_fields: Vec<(String, String, Option<HirExpr>)>,
    /// Class-body docstring (first bare string statement), for
    /// `inspect.getdoc` / `Cls.__doc__`.
    pub doc: Option<String>,
}

/// Import statement.
#[derive(Debug, Clone)]
pub struct HirImport {
    pub module: Vec<String>,
    pub names: Option<Vec<(String, Option<String>)>>,
    /// Alias for `import X as Y` (#1014).
    pub module_alias: Option<String>,
    pub span: Span,
}

/// Exception handler in try/except.
/// `is_star` is true for `except*` (PEP 654) handlers.
#[derive(Debug, Clone)]
pub struct HirExceptHandler {
    pub exc_type: Option<HirExpr>,
    pub name: Option<SymbolId>,
    pub body: Vec<HirStmt>,
    pub is_star: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum HirStmt {
    Let {
        target: SymbolId,
        ty: TypeId,
        value: HirExpr,
        span: Span,
    },
    Assign {
        target: HirLValue,
        value: HirExpr,
        span: Span,
    },
    Return {
        value: Option<HirExpr>,
        span: Span,
    },
    Expr {
        expr: HirExpr,
        span: Span,
    },
    If {
        cond: HirExpr,
        then_body: Vec<HirStmt>,
        else_body: Vec<HirStmt>,
        span: Span,
    },
    While {
        cond: HirExpr,
        body: Vec<HirStmt>,
        else_body: Vec<HirStmt>,
        span: Span,
    },
    For {
        var: SymbolId,
        iter: HirExpr,
        body: Vec<HirStmt>,
        else_body: Vec<HirStmt>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    /// try/except/finally (#283)
    Try {
        body: Vec<HirStmt>,
        handlers: Vec<HirExceptHandler>,
        else_body: Vec<HirStmt>,
        finally_body: Vec<HirStmt>,
        span: Span,
    },
    /// raise expression (#283)
    Raise {
        value: Option<HirExpr>,
        from: Option<HirExpr>,
        span: Span,
    },
    /// import statement (#292)
    Import {
        import: HirImport,
        span: Span,
    },
    /// with statement (context manager). `is_async` is true for `async with`,
    /// which dispatches `__aenter__`/`__aexit__` and awaits the coroutine
    /// returned by each.
    With {
        items: Vec<(HirExpr, Option<SymbolId>)>,
        body: Vec<HirStmt>,
        is_async: bool,
        span: Span,
    },
    /// assert statement
    Assert {
        test: HirExpr,
        msg: Option<HirExpr>,
        span: Span,
    },
    /// del statement
    Del {
        target: HirLValue,
        span: Span,
    },
    /// global/nonlocal declarations
    Global {
        names: Vec<SymbolId>,
        span: Span,
    },
    Nonlocal {
        names: Vec<SymbolId>,
        span: Span,
    },
    /// Placeholder for a function definition in the top-level order.
    /// Used to emit decorator applications at the correct position (#decorder).
    FuncDefPlaceholder {
        name: SymbolId,
        span: Span,
    },
    /// Placeholder for a decorated class definition in the top-level order.
    /// Class registration itself happens at the top of the module body
    /// regardless, but decorator application runs at the textual position so
    /// (a) decorator expressions can see imports / other module bindings
    /// declared above, and (b) later top-level statements observe the
    /// post-decorator class. Only emitted for classes with decorators
    /// (#1690 — proper fix for #1686 trade-off).
    ClassDefPlaceholder {
        name: SymbolId,
        span: Span,
    },
    /// match/case statement (#309)
    Match {
        subject: HirExpr,
        cases: Vec<HirMatchCase>,
        span: Span,
    },
}

/// L-value for assignments (variable, attribute, index, unpack).
#[derive(Debug, Clone)]
pub enum HirLValue {
    Var(SymbolId),
    Attr {
        object: Box<HirExpr>,
        attr: String,
    },
    Index {
        object: Box<HirExpr>,
        index: Box<HirExpr>,
    },
    /// Tuple unpacking: `a, b, c = ...` or `a, *rest, b = ...` (#409).
    /// `star_index` is the position of the `*rest` element, if any.
    Unpack {
        targets: Vec<HirLValue>,
        star_index: Option<usize>,
    },
}

#[derive(Debug, Clone)]
pub enum HirExpr {
    IntLit(i64, TypeId),
    FloatLit(f64, TypeId),
    StrLit(String, TypeId),
    BytesLit(Vec<u8>, TypeId),
    BoolLit(bool, TypeId),
    NoneLit(TypeId),
    Var(SymbolId, TypeId),
    BinOp {
        op: HirBinOp,
        lhs: Box<HirExpr>,
        rhs: Box<HirExpr>,
        ty: TypeId,
    },
    UnaryOp {
        op: HirUnaryOp,
        operand: Box<HirExpr>,
        ty: TypeId,
    },
    Call {
        func: Box<HirExpr>,
        args: Vec<HirExpr>,
        ty: TypeId,
    },
    Attr {
        object: Box<HirExpr>,
        attr: String,
        ty: TypeId,
    },
    Index {
        object: Box<HirExpr>,
        index: Box<HirExpr>,
        ty: TypeId,
    },
    List {
        elements: Vec<HirExpr>,
        ty: TypeId,
    },
    Set {
        elements: Vec<HirExpr>,
        ty: TypeId,
    },
    Tuple {
        elements: Vec<HirExpr>,
        ty: TypeId,
    },
    Dict {
        entries: Vec<(HirExpr, HirExpr)>,
        ty: TypeId,
    },
    /// Slice expression: obj[start:stop:step]
    Slice {
        start: Option<Box<HirExpr>>,
        stop: Option<Box<HirExpr>>,
        step: Option<Box<HirExpr>>,
        ty: TypeId,
    },
    /// Ternary: value if cond else other
    IfExpr {
        cond: Box<HirExpr>,
        then_val: Box<HirExpr>,
        else_val: Box<HirExpr>,
        ty: TypeId,
    },
    /// Lambda expression. `defaults` carries the default-arg expressions
    /// (one slot per parameter, `None` when the parameter has no default).
    /// Defaults are evaluated at closure creation time per Python semantics.
    Lambda {
        params: Vec<(SymbolId, TypeId)>,
        param_kinds: Vec<u8>,
        defaults: Vec<Option<Box<HirExpr>>>,
        body: Box<HirExpr>,
        ty: TypeId,
        span: Span,
    },
    /// Yield expression (#290)
    Yield {
        value: Option<Box<HirExpr>>,
        ty: TypeId,
    },
    /// Yield from expression (#290)
    YieldFrom {
        iter: Box<HirExpr>,
        ty: TypeId,
    },
    /// Await expression (#293)
    Await {
        value: Box<HirExpr>,
        ty: TypeId,
    },
    /// List comprehension (#291)
    ListComp {
        element: Box<HirExpr>,
        generators: Vec<HirComprehension>,
        ty: TypeId,
    },
    /// Lazy all()/any() over a generator expression.
    AnyAllComp {
        is_all: bool,
        element: Box<HirExpr>,
        generators: Vec<HirComprehension>,
        ty: TypeId,
    },
    /// Set comprehension (#291)
    SetComp {
        element: Box<HirExpr>,
        generators: Vec<HirComprehension>,
        ty: TypeId,
    },
    /// Dict comprehension (#291)
    DictComp {
        key: Box<HirExpr>,
        value: Box<HirExpr>,
        generators: Vec<HirComprehension>,
        ty: TypeId,
    },
    /// F-string
    FString {
        parts: Vec<HirFStringPart>,
        ty: TypeId,
    },
    /// Walrus operator := (PEP 572)
    Walrus {
        target: SymbolId,
        value: Box<HirExpr>,
        ty: TypeId,
    },
}

/// Comprehension clause: for var in iter if cond
///
/// For tuple-unpacking targets (e.g. `for k, v in pairs`), `var` holds the
/// first target and `extra_vars` holds the rest. The lowering unpacks the
/// iterator's next value into (var, extra_vars...) via tuple indexing.
#[derive(Debug, Clone)]
pub struct HirComprehension {
    pub var: SymbolId,
    pub extra_vars: Vec<SymbolId>,
    pub unpack_target: bool,
    pub iter: HirExpr,
    pub conditions: Vec<HirExpr>,
    pub is_async: bool,
}

/// A match/case arm.
#[derive(Debug, Clone)]
pub struct HirMatchCase {
    pub pattern: HirPattern,
    pub guard: Option<HirExpr>,
    pub body: Vec<HirStmt>,
    pub span: Span,
}

/// Pattern for match/case (#309, #827).
#[derive(Debug, Clone)]
pub enum HirPattern {
    /// Wildcard `_` — matches anything
    Wildcard,
    /// Literal value pattern: `case 42:`, `case "hello":`
    Literal(HirExpr),
    /// Capture pattern: `case x:` — binds subject to variable
    Capture(SymbolId),
    /// OR pattern: `case 1 | 2 | 3:`
    Or(Vec<HirPattern>),
    /// Sequence pattern: `case [a, b, c]:`
    Sequence(Vec<HirPattern>),
    /// Class pattern: `case Point(x=0, y=0):`
    Class {
        class: SymbolId,
        class_name: String,
        args: Vec<(String, HirPattern)>,
    },
    /// Mapping pattern: `case {"key": val, **rest}:` (#827)
    /// Checks for each key's presence and matches the associated value pattern.
    /// `rest` binds remaining key-value pairs to a new dict variable.
    Mapping {
        pairs: Vec<(HirExpr, HirPattern)>,
        rest: Option<SymbolId>,
    },
    /// AS pattern: `case <pattern> as <name>:` (#827)
    /// Matches the inner pattern and additionally binds the subject to `name`.
    As {
        pattern: Box<HirPattern>,
        name: SymbolId,
    },
    /// Star capture in sequence pattern: `[a, *rest, b]` (#827)
    Star(Option<SymbolId>),
}

/// F-string part
#[derive(Debug, Clone)]
pub enum HirFStringPart {
    Literal(String),
    /// Field expression with an optional format spec; the spec is a part
    /// list so nested replacement fields evaluate at runtime. A static spec
    /// is a single Literal part.
    Expr(HirExpr, Option<Vec<HirFStringPart>>),
}

impl HirExpr {
    pub fn ty(&self) -> TypeId {
        match self {
            HirExpr::IntLit(_, t)
            | HirExpr::FloatLit(_, t)
            | HirExpr::StrLit(_, t)
            | HirExpr::BytesLit(_, t)
            | HirExpr::BoolLit(_, t)
            | HirExpr::NoneLit(t)
            | HirExpr::Var(_, t)
            | HirExpr::BinOp { ty: t, .. }
            | HirExpr::UnaryOp { ty: t, .. }
            | HirExpr::Call { ty: t, .. }
            | HirExpr::Attr { ty: t, .. }
            | HirExpr::Index { ty: t, .. }
            | HirExpr::List { ty: t, .. }
            | HirExpr::Set { ty: t, .. }
            | HirExpr::Tuple { ty: t, .. }
            | HirExpr::Dict { ty: t, .. }
            | HirExpr::Slice { ty: t, .. }
            | HirExpr::IfExpr { ty: t, .. }
            | HirExpr::Lambda { ty: t, .. }
            | HirExpr::Yield { ty: t, .. }
            | HirExpr::YieldFrom { ty: t, .. }
            | HirExpr::Await { ty: t, .. }
            | HirExpr::ListComp { ty: t, .. }
            | HirExpr::AnyAllComp { ty: t, .. }
            | HirExpr::SetComp { ty: t, .. }
            | HirExpr::DictComp { ty: t, .. }
            | HirExpr::FString { ty: t, .. }
            | HirExpr::Walrus { ty: t, .. } => *t,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirBinOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    Is,
    IsNot,
    In,
    NotIn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirUnaryOp {
    Pos,
    Neg,
    Not,
    BitNot,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::SymbolId;
    use crate::source::span::Span;
    use crate::types::TypeContext;

    fn make_tcx() -> TypeContext {
        TypeContext::new()
    }

    #[test]
    fn test_hir_expr_ty_int_lit() {
        let tcx = make_tcx();
        let int_ty = tcx.int();
        let expr = HirExpr::IntLit(42, int_ty);
        assert_eq!(expr.ty(), int_ty);
    }

    #[test]
    fn test_hir_expr_ty_all_variants() {
        let tcx = make_tcx();
        let int_ty = tcx.int();
        let float_ty = tcx.float();
        let str_ty = tcx.str();
        let bool_ty = tcx.bool();
        let none_ty = tcx.none();
        let any_ty = tcx.any();

        assert_eq!(HirExpr::IntLit(1, int_ty).ty(), int_ty);
        assert_eq!(HirExpr::FloatLit(1.0, float_ty).ty(), float_ty);
        assert_eq!(HirExpr::StrLit("x".into(), str_ty).ty(), str_ty);
        assert_eq!(HirExpr::BoolLit(true, bool_ty).ty(), bool_ty);
        assert_eq!(HirExpr::NoneLit(none_ty).ty(), none_ty);
        assert_eq!(HirExpr::Var(SymbolId(0), int_ty).ty(), int_ty);
        assert_eq!(
            HirExpr::List {
                elements: vec![],
                ty: any_ty
            }
            .ty(),
            any_ty
        );
        assert_eq!(
            HirExpr::Tuple {
                elements: vec![],
                ty: any_ty
            }
            .ty(),
            any_ty
        );
        assert_eq!(
            HirExpr::Dict {
                entries: vec![],
                ty: any_ty
            }
            .ty(),
            any_ty
        );
    }

    #[test]
    fn test_hir_expr_ty_binop() {
        let tcx = make_tcx();
        let int_ty = tcx.int();
        let expr = HirExpr::BinOp {
            op: HirBinOp::Add,
            lhs: Box::new(HirExpr::IntLit(1, int_ty)),
            rhs: Box::new(HirExpr::IntLit(2, int_ty)),
            ty: int_ty,
        };
        assert_eq!(expr.ty(), int_ty);
    }

    #[test]
    fn test_hir_expr_ty_unaryop() {
        let tcx = make_tcx();
        let int_ty = tcx.int();
        let expr = HirExpr::UnaryOp {
            op: HirUnaryOp::Neg,
            operand: Box::new(HirExpr::IntLit(5, int_ty)),
            ty: int_ty,
        };
        assert_eq!(expr.ty(), int_ty);
    }

    #[test]
    fn test_hir_expr_ty_call() {
        let tcx = make_tcx();
        let any_ty = tcx.any();
        let int_ty = tcx.int();
        let expr = HirExpr::Call {
            func: Box::new(HirExpr::Var(SymbolId(0), any_ty)),
            args: vec![HirExpr::IntLit(1, int_ty)],
            ty: any_ty,
        };
        assert_eq!(expr.ty(), any_ty);
    }

    #[test]
    fn test_hir_expr_ty_if_expr() {
        let tcx = make_tcx();
        let int_ty = tcx.int();
        let bool_ty = tcx.bool();
        let expr = HirExpr::IfExpr {
            cond: Box::new(HirExpr::BoolLit(true, bool_ty)),
            then_val: Box::new(HirExpr::IntLit(1, int_ty)),
            else_val: Box::new(HirExpr::IntLit(0, int_ty)),
            ty: int_ty,
        };
        assert_eq!(expr.ty(), int_ty);
    }

    #[test]
    fn test_hir_expr_ty_fstring() {
        let tcx = make_tcx();
        let str_ty = tcx.str();
        let expr = HirExpr::FString {
            parts: vec![HirFStringPart::Literal("hello".into())],
            ty: str_ty,
        };
        assert_eq!(expr.ty(), str_ty);
    }

    #[test]
    fn test_hir_binop_eq() {
        assert_eq!(HirBinOp::Add, HirBinOp::Add);
        assert_ne!(HirBinOp::Add, HirBinOp::Sub);
        assert_ne!(HirBinOp::Eq, HirBinOp::NotEq);
        assert_eq!(HirBinOp::Is, HirBinOp::Is);
    }

    #[test]
    fn test_hir_unaryop_eq() {
        assert_eq!(HirUnaryOp::Neg, HirUnaryOp::Neg);
        assert_ne!(HirUnaryOp::Neg, HirUnaryOp::Pos);
        assert_ne!(HirUnaryOp::Not, HirUnaryOp::BitNot);
    }

    #[test]
    fn test_hir_module_empty() {
        let module = HirModule {
            functions: vec![],
            classes: vec![],
            top_level: vec![],
            imports: vec![],
            sym_names: HashMap::new(),
            sym_types: HashMap::new(),
            module_annotations: Vec::new(),
            func_sigs: HashMap::new(),
        };
        assert!(module.functions.is_empty());
        assert!(module.classes.is_empty());
        assert!(module.top_level.is_empty());
    }

    #[test]
    fn test_hir_lvalue_var() {
        let lv = HirLValue::Var(SymbolId(42));
        assert!(matches!(lv, HirLValue::Var(SymbolId(42))));
    }

    #[test]
    fn test_hir_lvalue_unpack() {
        let lv = HirLValue::Unpack {
            targets: vec![
                HirLValue::Var(SymbolId(1)),
                HirLValue::Var(SymbolId(2)),
                HirLValue::Var(SymbolId(3)),
            ],
            star_index: Some(1),
        };
        if let HirLValue::Unpack {
            targets,
            star_index,
        } = &lv
        {
            assert_eq!(targets.len(), 3);
            assert_eq!(*star_index, Some(1));
        } else {
            panic!("expected Unpack");
        }
    }

    #[test]
    fn test_hir_pattern_variants() {
        assert!(matches!(HirPattern::Wildcard, HirPattern::Wildcard));
        let cap = HirPattern::Capture(SymbolId(5));
        assert!(matches!(cap, HirPattern::Capture(SymbolId(5))));
        let or = HirPattern::Or(vec![HirPattern::Wildcard, HirPattern::Wildcard]);
        if let HirPattern::Or(pats) = &or {
            assert_eq!(pats.len(), 2);
        }
        let seq = HirPattern::Sequence(vec![HirPattern::Wildcard]);
        assert!(matches!(seq, HirPattern::Sequence(_)));
    }

    #[test]
    fn test_hir_pattern_mapping() {
        let tcx = make_tcx();
        let str_ty = tcx.str();
        let key = HirExpr::StrLit("type".into(), str_ty);
        let mapping = HirPattern::Mapping {
            pairs: vec![(key, HirPattern::Wildcard)],
            rest: Some(SymbolId(10)),
        };
        if let HirPattern::Mapping { pairs, rest } = &mapping {
            assert_eq!(pairs.len(), 1);
            assert_eq!(*rest, Some(SymbolId(10)));
        } else {
            panic!("expected Mapping");
        }
    }

    #[test]
    fn test_hir_pattern_as() {
        let as_pat = HirPattern::As {
            pattern: Box::new(HirPattern::Wildcard),
            name: SymbolId(7),
        };
        if let HirPattern::As { pattern, name } = &as_pat {
            assert!(matches!(pattern.as_ref(), HirPattern::Wildcard));
            assert_eq!(*name, SymbolId(7));
        } else {
            panic!("expected As");
        }
    }

    #[test]
    fn test_hir_function_construction() {
        let tcx = make_tcx();
        let int_ty = tcx.int();
        let func = HirFunction {
            name: SymbolId(0),
            params: vec![(SymbolId(1), int_ty), (SymbolId(2), int_ty)],
            return_ty: int_ty,
            body: vec![],
            span: Span::dummy(),
            captures: vec![SymbolId(10)],
            is_async: true,
            is_generator: false,
            decorators: vec![],
            has_star_args: false,
            star_param_pos: None,
            has_kwargs: false,
        };
        assert_eq!(func.params.len(), 2);
        assert!(func.is_async);
        assert!(!func.is_generator);
        assert_eq!(func.captures.len(), 1);
    }

    #[test]
    fn test_hir_class_construction() {
        let tcx = make_tcx();
        let int_ty = tcx.int();
        let cls = HirClass {
            name: SymbolId(0),
            base: Some(SymbolId(1)),
            all_bases: vec![SymbolId(1)],
            namedtuple_base: None,
            fields: vec![(SymbolId(2), int_ty)],
            methods: vec![],
            span: Span::dummy(),
            decorators: vec![],
            explicit_match_args: None,
            metaclass: None,
            class_attr_assigns: vec![],
            slots: None,
            class_kwargs: vec![],
            dataclass_fields: vec![],
            doc: None,
        };
        assert_eq!(cls.base, Some(SymbolId(1)));
        assert_eq!(cls.fields.len(), 1);
    }
}
