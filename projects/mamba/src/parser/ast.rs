use crate::source::span::{Span, Spanned};

pub type Name = String;

/// PEP 695 type-parameter kind: `T`, `*Ts`, `**P`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeParamKind {
    TypeVar,
    TypeVarTuple,
    ParamSpec,
}

/// PEP 695 type parameter: `T`, `T: bound`, `T: (c1, c2)`, `*Ts`, `**P`.
///
/// The bound / constraint expressions are kept as ordinary expressions so the
/// runtime can evaluate them lazily (CPython semantics).
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: Name,
    pub kind: TypeParamKind,
    /// `T: bound` — single (non-tuple) bound expression.
    pub bound: Option<Spanned<Expr>>,
    /// `T: (c1, c2, ...)` — tuple of constraint expressions.
    pub constraints: Option<Vec<Spanned<Expr>>>,
}

impl TypeParam {
    /// Plain unbounded type parameter (used by tests / desugared nodes).
    pub fn plain(name: impl Into<Name>) -> Self {
        TypeParam {
            name: name.into(),
            kind: TypeParamKind::TypeVar,
            bound: None,
            constraints: None,
        }
    }
}

/// Top-level module: a sequence of statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub stmts: Vec<Spanned<Stmt>>,
}

/// Statement node.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `x: int = 42`
    VarDecl {
        name: Name,
        ty: Spanned<TypeExpr>,
        value: Spanned<Expr>,
    },
    /// `x = expr` (reassignment, no type annotation)
    Assign {
        target: Spanned<Expr>,
        value: Spanned<Expr>,
    },
    /// `x += expr` (augmented assignment, #221)
    AugAssign {
        target: Spanned<Expr>,
        op: AugOp,
        value: Spanned<Expr>,
    },
    /// `def foo(x: int) -> int: ...`
    FnDef {
        decorators: Vec<Spanned<Expr>>,
        name: Name,
        type_params: Vec<TypeParam>,
        params: Vec<Param>,
        return_ty: Option<Spanned<TypeExpr>>,
        body: Vec<Spanned<Stmt>>,
    },
    /// `async def foo(x: int) -> int: ...` (#227)
    AsyncFnDef {
        decorators: Vec<Spanned<Expr>>,
        name: Name,
        type_params: Vec<TypeParam>,
        params: Vec<Param>,
        return_ty: Option<Spanned<TypeExpr>>,
        body: Vec<Spanned<Stmt>>,
    },
    /// `class Foo(Base1, Base2, metaclass=Meta, name="alpha"): ...`
    ClassDef {
        decorators: Vec<Spanned<Expr>>,
        name: Name,
        type_params: Vec<TypeParam>,
        bases: Vec<Spanned<Expr>>,
        /// Keyword arguments in the class declaration (e.g. `metaclass=Meta`, `name="alpha"`).
        keyword_args: Vec<(Name, Spanned<Expr>)>,
        body: Vec<Spanned<Stmt>>,
    },
    /// `enum Shape: Circle(r: float) | Rectangle(w: float, h: float) | Point`
    EnumDef {
        name: Name,
        type_params: Vec<TypeParam>,
        variants: Vec<Variant>,
    },
    /// `if cond: ... elif cond: ... else: ...`
    If {
        condition: Spanned<Expr>,
        body: Vec<Spanned<Stmt>>,
        elif_clauses: Vec<(Spanned<Expr>, Vec<Spanned<Stmt>>)>,
        else_body: Option<Vec<Spanned<Stmt>>>,
    },
    /// `while cond: ... else: ...`
    While {
        condition: Spanned<Expr>,
        body: Vec<Spanned<Stmt>>,
        else_body: Option<Vec<Spanned<Stmt>>>,
    },
    /// `for x in iterable: ... else: ...`
    For {
        targets: Vec<Name>,
        var_ty: Option<Spanned<TypeExpr>>,
        iter: Spanned<Expr>,
        body: Vec<Spanned<Stmt>>,
        else_body: Option<Vec<Spanned<Stmt>>>,
    },
    /// `async for x in iterable: ... else: ...` (#227)
    AsyncFor {
        targets: Vec<Name>,
        var_ty: Option<Spanned<TypeExpr>>,
        iter: Spanned<Expr>,
        body: Vec<Spanned<Stmt>>,
        else_body: Option<Vec<Spanned<Stmt>>>,
    },
    /// `match expr: case Pattern: ...`
    Match {
        expr: Spanned<Expr>,
        arms: Vec<MatchArm>,
    },
    /// `return expr`
    Return(Option<Spanned<Expr>>),
    /// `pass`
    Pass,
    /// `break`
    Break,
    /// `continue`
    Continue,
    /// `import module` or `from module import name` or `import module as alias`
    Import {
        module: Vec<Name>,
        /// For `from X import Y as Z`: Some([("Y", Some("Z"))]).
        /// For `import X`: None.
        names: Option<Vec<(Name, Option<Name>)>>,
        /// For `import X as alias`: Some("alias").  None for plain `import X`.
        module_alias: Option<Name>,
    },
    /// Bare type annotation without a value: `x: int` (#1014).
    ///
    /// Used in class bodies and module scope (e.g. ORM field declarations).
    BareAnnotation { name: Name, ty: Spanned<TypeExpr> },
    /// `try: ... except E as e: ... finally: ...` (#225)
    Try {
        body: Vec<Spanned<Stmt>>,
        handlers: Vec<ExceptHandler>,
        else_body: Option<Vec<Spanned<Stmt>>>,
        finally_body: Option<Vec<Spanned<Stmt>>>,
    },
    /// `raise expr` or `raise expr from expr` (#225)
    Raise {
        value: Option<Spanned<Expr>>,
        from: Option<Spanned<Expr>>,
    },
    /// `with expr as name: ...` (#226)
    With {
        items: Vec<WithItem>,
        body: Vec<Spanned<Stmt>>,
    },
    /// `async with expr as name: ...` (#227)
    AsyncWith {
        items: Vec<WithItem>,
        body: Vec<Spanned<Stmt>>,
    },
    /// `assert expr` or `assert expr, msg` (#229)
    Assert {
        test: Spanned<Expr>,
        msg: Option<Spanned<Expr>>,
    },
    /// `del target` (#230)
    Del(Spanned<Expr>),
    /// `global x, y` (#231)
    Global(Vec<Name>),
    /// `nonlocal x, y` (#231)
    Nonlocal(Vec<Name>),
    /// `type X = int | str` (PEP 695, #233)
    ///
    /// The value is kept as a general expression (CPython allows any
    /// expression, e.g. `type Lazy[T] = lambda: T`) and is evaluated lazily
    /// at runtime by the TypeAliasType object built during desugaring.
    TypeAlias {
        name: Name,
        type_params: Vec<TypeParam>,
        value: Spanned<Expr>,
    },
    /// Expression used as statement
    ExprStmt(Spanned<Expr>),
}

/// Augmented assignment operator (#221).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AugOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    MatMul,
}

/// Exception handler in try/except (#225).
/// `is_star` is true for `except*` (PEP 654) handlers.
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptHandler {
    pub exc_type: Option<Spanned<Expr>>,
    pub name: Option<Name>,
    pub body: Vec<Spanned<Stmt>>,
    pub is_star: bool,
    pub span: Span,
}

/// `with expr as name` item (#226).
#[derive(Debug, Clone, PartialEq)]
pub struct WithItem {
    pub context: Spanned<Expr>,
    pub alias: Option<Name>,
}

/// Function parameter with mandatory type annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Name,
    pub ty: Spanned<TypeExpr>,
    pub default: Option<Spanned<Expr>>,
    pub kind: ParamKind,
    /// Declared before a `/` separator (PEP 570 positional-only).
    /// Introspection metadata only — call binding is unaffected.
    pub pos_only: bool,
    /// Declared after a bare `*` separator or `*args` (keyword-only).
    /// Introspection metadata only — call binding is unaffected.
    pub kw_only: bool,
    pub span: Span,
}

/// Parameter kind (#218).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Regular,
    Star,       // *args
    DoubleStar, // **kwargs
}

/// Enum variant definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: Name,
    pub fields: Vec<Param>,
    pub span: Span,
}

/// Match arm: `case Pattern: body`
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Spanned<Pattern>,
    pub guard: Option<Spanned<Expr>>,
    pub body: Vec<Spanned<Stmt>>,
    pub span: Span,
}

/// Match pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// `_`
    Wildcard,
    /// Binding: `x`
    Binding(Name),
    /// Enum constructor: `Shape.Circle(r)`
    Constructor { path: Vec<Name>, fields: Vec<Name> },
    /// Literal pattern
    Literal(Expr),
    /// OR pattern: `p1 | p2` (#237)
    Or(Vec<Spanned<Pattern>>),
    /// Sequence pattern: `[a, b, *rest]` (#235)
    Sequence(Vec<Spanned<Pattern>>),
    /// Mapping pattern: `{"key": value, **rest}` (#236, #827)
    /// `rest` captures unmatched key-value pairs when `**rest` is present.
    Mapping {
        pairs: Vec<(Spanned<Expr>, Spanned<Pattern>)>,
        rest: Option<Name>,
    },
    /// Class pattern: `ClassName(x=1, y=2)` (#238)
    ClassPattern {
        cls: Vec<Name>,
        patterns: Vec<(Option<Name>, Spanned<Pattern>)>,
    },
    /// Star pattern in sequence: `*rest`
    Star(Option<Name>),
    /// AS pattern: `<pattern> as <name>` (#827)
    /// Matches the inner pattern and binds the matched value to `name`.
    As {
        pattern: Box<Spanned<Pattern>>,
        name: Name,
    },
}

/// Expression node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal
    IntLit(i64),
    /// Integer literal larger than the compiler's i64 literal path.
    BigIntLit(String),
    /// Float literal
    FloatLit(f64),
    /// Complex literal (imaginary part, e.g. 2j)
    ComplexLit(f64),
    /// String literal
    StrLit(String),
    /// Bytes literal: `b"hello"`
    BytesLit(Vec<u8>),
    /// Boolean literal
    BoolLit(bool),
    /// None literal
    NoneLit,
    /// Ellipsis literal `...` (#212)
    Ellipsis,
    /// Identifier
    Ident(Name),
    /// Binary operation: `a + b`
    BinOp {
        op: BinOp,
        lhs: Box<Spanned<Expr>>,
        rhs: Box<Spanned<Expr>>,
    },
    /// Unary operation: `-x`, `not x`, `~x`
    UnaryOp {
        op: UnaryOp,
        operand: Box<Spanned<Expr>>,
    },
    /// Function call: `f(args)` with keyword args support
    Call {
        func: Box<Spanned<Expr>>,
        args: Vec<CallArg>,
    },
    /// Attribute access: `obj.attr`
    Attr {
        object: Box<Spanned<Expr>>,
        attr: Name,
    },
    /// Index access: `obj[idx]`
    Index {
        object: Box<Spanned<Expr>>,
        index: Box<Spanned<Expr>>,
    },
    /// Slice: `obj[start:stop:step]` (#219)
    Slice {
        start: Option<Box<Spanned<Expr>>>,
        stop: Option<Box<Spanned<Expr>>>,
        step: Option<Box<Spanned<Expr>>>,
    },
    /// List literal: `[1, 2, 3]`
    ListLit(Vec<Spanned<Expr>>),
    /// Dict literal: `{k: v, ...}` or dict-unpack `{**d, ...}` (PEP 448 / #1014).
    ///
    /// Each entry is `(Option<key>, value)`.  When the key is `None` the value
    /// expression is unpacked via `**` (i.e. `{**defaults, **overrides}`).
    DictLit(Vec<(Option<Spanned<Expr>>, Spanned<Expr>)>),
    /// Set literal: `{1, 2, 3}` (#224)
    SetLit(Vec<Spanned<Expr>>),
    /// Tuple literal: `(1, 2)`
    TupleLit(Vec<Spanned<Expr>>),
    /// Ternary: `a if cond else b` (#213)
    IfExpr {
        body: Box<Spanned<Expr>>,
        condition: Box<Spanned<Expr>>,
        else_body: Box<Spanned<Expr>>,
    },
    /// Lambda: `lambda x: int, y: int -> int: x + y` (#214)
    Lambda {
        params: Vec<Param>,
        body: Box<Spanned<Expr>>,
    },
    /// List comprehension: `[expr for x in iter if cond]` (#215)
    ListComp {
        element: Box<Spanned<Expr>>,
        generators: Vec<Comprehension>,
    },
    /// Set comprehension: `{expr for x in iter}` (#215)
    SetComp {
        element: Box<Spanned<Expr>>,
        generators: Vec<Comprehension>,
    },
    /// Dict comprehension: `{k: v for k, v in items}` (#215)
    DictComp {
        key: Box<Spanned<Expr>>,
        value: Box<Spanned<Expr>>,
        generators: Vec<Comprehension>,
    },
    /// Generator expression: `(expr for x in iter)` (#216)
    GeneratorExpr {
        element: Box<Spanned<Expr>>,
        generators: Vec<Comprehension>,
    },
    /// F-string: `f"hello {name}"` (#217)
    FString(Vec<FStringPart>),
    /// Yield: `yield expr` (#222)
    Yield(Option<Box<Spanned<Expr>>>),
    /// Yield from: `yield from expr` (#222)
    YieldFrom(Box<Spanned<Expr>>),
    /// Await: `await expr` (#223)
    Await(Box<Spanned<Expr>>),
    /// Walrus: `x := expr` (#210)
    Walrus {
        target: Name,
        value: Box<Spanned<Expr>>,
    },
    /// Chained comparison: `a < b < c` → `(a < b) and (b < c)` with `b` evaluated once.
    ChainedCompare {
        operands: Vec<Spanned<Expr>>,
        ops: Vec<BinOp>,
    },
    /// Star expression: `*expr` (in function calls, assignments)
    Starred(Box<Spanned<Expr>>),
    /// Tuple unpacking target (for assignments, #232)
    UnpackTarget(Vec<Spanned<Expr>>),
}

/// Call argument — positional or keyword.
#[derive(Debug, Clone, PartialEq)]
pub enum CallArg {
    Positional(Spanned<Expr>),
    Keyword { name: Name, value: Spanned<Expr> },
    StarArg(Spanned<Expr>),
    DoubleStarArg(Spanned<Expr>),
}

/// Comprehension clause: `for x in iter if cond` (#215).
///
/// `targets` holds the loop variable name(s).  For a simple `for x in …`
/// clause there is exactly one element.  For a tuple target (`for k, v in …`)
/// each name is stored separately.
#[derive(Debug, Clone, PartialEq)]
pub struct Comprehension {
    pub targets: Vec<Name>,
    pub unpack_target: bool,
    pub iter: Spanned<Expr>,
    pub conditions: Vec<Spanned<Expr>>,
    pub is_async: bool,
}

/// F-string part (#217).
#[derive(Debug, Clone, PartialEq)]
pub enum FStringPart {
    Literal(String),
    /// Expression with optional format spec. The spec is itself a part list
    /// so nested replacement fields (`{value:{width}}`) evaluate at runtime;
    /// a static spec is a single `Literal` part (e.g. `{x:.2f}` →
    /// `Some(vec![Literal(".2f")])`).
    Expr(Spanned<Expr>, Option<Vec<FStringPart>>),
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    MatMul,
    // Comparison
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    // Logical
    And,
    Or,
    // Bitwise (#209)
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    // Identity / Membership (#212)
    Is,
    IsNot,
    In,
    NotIn,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Pos, // + (unary positive)
    Neg,
    Not,
    BitNot, // ~ (#209)
}

/// Type expression for annotations.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// Simple type name: `int`, `float`, `MyClass`
    Named(Name),
    /// Generic type: `list[int]`, `dict[str, int]`
    Generic {
        name: Name,
        args: Vec<Spanned<TypeExpr>>,
    },
    /// Optional shorthand: `T?` => `T | None`
    Optional(Box<Spanned<TypeExpr>>),
    /// Union type: `int | str`
    Union(Vec<Spanned<TypeExpr>>),
    /// Function type: `(int, int) -> int`
    Fn {
        params: Vec<Spanned<TypeExpr>>,
        ret: Box<Spanned<TypeExpr>>,
    },
    /// Tuple type: `tuple[int, str]`
    Tuple(Vec<Spanned<TypeExpr>>),
}

pub const FORWARD_REF_PREFIX: &str = "__mamba_forward_ref__:";

pub fn forward_ref_name(name: &str) -> String {
    format!("{FORWARD_REF_PREFIX}{name}")
}

pub fn strip_forward_ref_name(name: &str) -> Option<&str> {
    name.strip_prefix(FORWARD_REF_PREFIX)
}
