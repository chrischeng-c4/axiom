//! ① Type-wall PoC: a tiny hardcoded table of stdlib call signatures so that a
//! wrong-typed *stdlib* argument is rejected at compile time, reusing the same
//! value-vs-annotation rejection loop that already rejects `x: int = "3"`.
//!
//! This is a proof-of-concept *path*, not a complete typeshed import. The real
//! version would be generated from `vendor/typeshed`. The closed [`CoreTy`] enum
//! is deliberately scalar-only: anything we cannot represent as a concrete
//! scalar (protocols, unions, typevars, overloads, buffers) collapses to
//! [`CoreTy::Unknown`], which the hook *skips* — guaranteeing zero false
//! positives on correct calls. [`CoreTy::Bytes`], [`CoreTy::MemoryView`],
//! [`CoreTy::Complex`], [`CoreTy::List`], [`CoreTy::Tuple`], and
//! [`CoreTy::Dict`] are represented as negative scalar walls: concrete
//! incompatible scalars are never those values, while
//! bytes/memoryview/complex/list/tuple/dict expressions currently infer to
//! dynamic or collection types and therefore remain skip-when-unsure.

/// Closed set of argument types the PoC table can express. Anything richer
/// (Optional, Union, Protocol, TypeVar, overload, ReadableBuffer, SupportsIndex)
/// must be encoded as [`CoreTy::Unknown`] or [`CoreTy::Typed`] so the hook does
/// not over-enforce it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreTy {
    Int,
    Float,
    Str,
    Bytes,
    MemoryView,
    Complex,
    List,
    Tuple,
    Dict,
    Bool,
    None,
    /// A NOMINAL/protocol type contract (a named typeshed type that is neither a
    /// concrete scalar nor `object`/`Any`/`Incomplete`): `os.PathLike`,
    /// `_SupportsFloatOrIndex`, `BaseException`, etc. The hook does not treat it
    /// as a scalar, but it rejects a *bare* user class instance (no bases, no
    /// methods — `class _W: pass`) passed here, since such a value can satisfy
    /// neither a protocol (no dunders) nor a nominal type (no superclass). Any
    /// class with a base or a method, and every non-class value, is skipped — so
    /// it stays false-positive-clean.
    Typed,
    /// Not a concrete scalar — never enforce against this. Catch-all for every
    /// non-scalar typeshed annotation (unions, subscripts, typevars, buffers,
    /// object/Any, etc.).
    Unknown,
}

/// What kind of callee a signature describes, used to disambiguate the lookup
/// against import provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigKind {
    /// Module-level free function: `os.strerror`, `base64.b64encode`.
    ModuleFn,
    /// Instance method: `HTMLParser.handle_entityref` (qualifier = class name).
    Method,
}

/// A single positional parameter's enforceable contract.
#[derive(Debug, Clone, Copy)]
pub struct ParamSig {
    pub name: &'static str,
    pub ty: CoreTy,
    /// `*args` / `*` boundary: the hook stops enforcement at the first star
    /// param and never enforces past it.
    pub star: bool,
}

/// One stdlib callable signature.
#[derive(Debug, Clone, Copy)]
pub struct StdlibSig {
    /// Dotted module path, e.g. `"os"`, `"html.parser"`,
    /// `"multiprocessing.reduction"`.
    pub module: &'static str,
    /// For [`SigKind::Method`], the owning class name (e.g. `"HTMLParser"`).
    /// Empty for module functions.
    pub qualifier: &'static str,
    /// The callable's own name.
    pub name: &'static str,
    pub kind: SigKind,
    pub params: &'static [ParamSig],
    /// Whether this signature has *any* enforceable (concrete-scalar) param. If
    /// false, the hook skips it wholesale (kept in the table as a documented
    /// negative test that it is NOT rejected).
    pub enforceable: bool,
}

const fn p(name: &'static str, ty: CoreTy) -> ParamSig {
    ParamSig {
        name,
        ty,
        star: false,
    }
}

/// The PoC signature table. Hardcoded; the production version regenerates this
/// from typeshed.
pub const STDLIB_SIGS: &[StdlibSig] = &[
    // POSITIVE: os.strerror(code: int) — bare-scalar module fn, enforceable.
    StdlibSig {
        module: "os",
        qualifier: "",
        name: "strerror",
        kind: SigKind::ModuleFn,
        params: &[p("code", CoreTy::Int)],
        enforceable: true,
    },
    // POSITIVE: os.getenv(key: str, default=...) — bare-scalar module fn.
    // Only `key` is concrete (str); `default` is Unknown, so the hook stops
    // enforcing after the first non-scalar param.
    StdlibSig {
        module: "os",
        qualifier: "",
        name: "getenv",
        kind: SigKind::ModuleFn,
        params: &[p("key", CoreTy::Str), p("default", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: multiprocessing.reduction.duplicate(handle: int, ...).
    StdlibSig {
        module: "multiprocessing.reduction",
        qualifier: "",
        name: "duplicate",
        kind: SigKind::ModuleFn,
        params: &[p("handle", CoreTy::Int)],
        enforceable: true,
    },
    // POSITIVE: html.parser.HTMLParser.handle_entityref(name: str) — method.
    StdlibSig {
        module: "html.parser",
        qualifier: "HTMLParser",
        name: "handle_entityref",
        kind: SigKind::Method,
        params: &[p("name", CoreTy::Str)],
        enforceable: true,
    },
    // NEGATIVE: base64.b64encode(s: ReadableBuffer, altchars=...) — `s` is a
    // buffer protocol -> Unknown, so this is NOT enforceable. Kept as a
    // regression guard that `b64encode(123)` is never rejected.
    StdlibSig {
        module: "base64",
        qualifier: "",
        name: "b64encode",
        kind: SigKind::ModuleFn,
        params: &[p("s", CoreTy::Unknown), p("altchars", CoreTy::Unknown)],
        enforceable: false,
    },
    // POSITIVE: generated bdb rows lose tuple/callable precision for these
    // strict wall probes. Tighten only the first argument so valid optional
    // suffix/varargs remain skip-safe.
    StdlibSig {
        module: "bdb",
        qualifier: "Bdb",
        name: "format_stack_entry",
        kind: SigKind::Method,
        params: &[p("frame_lineno", CoreTy::Tuple), p("lprefix", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "bdb",
        qualifier: "Bdb",
        name: "runcall",
        kind: SigKind::Method,
        params: &[p("func", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: bz2 filename arguments are overload/protocol unions
    // (path-like or file-object). A bare user instance cannot satisfy either,
    // while real strings/bytes/path/file-like objects stay skip-safe.
    StdlibSig {
        module: "bz2",
        qualifier: "",
        name: "open",
        kind: SigKind::ModuleFn,
        params: &[p("filename", CoreTy::Typed), p("mode", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "bz2",
        qualifier: "BZ2File",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("filename", CoreTy::Typed), p("mode", CoreTy::Unknown)],
        enforceable: true,
    },
    // NEGATIVE: math.factorial(x: SupportsIndex) — protocol -> Unknown, NOT
    // enforceable. Kept as a regression guard that `factorial(obj)` and
    // `factorial(3.0)` are never rejected by this table.
    StdlibSig {
        module: "math",
        qualifier: "",
        name: "factorial",
        kind: SigKind::ModuleFn,
        params: &[p("x", CoreTy::Unknown)],
        enforceable: false,
    },
    // NEGATIVE: calendar.setfirstweekday(firstweekday) — CPython's body is
    // `if not MONDAY <= firstweekday <= SUNDAY`, so a str argument is a
    // RUNTIME TypeError (from the int/str comparison), not a compile-time
    // reject. The runtime dispatcher raises it; keep the wall out of the way.
    StdlibSig {
        module: "calendar",
        qualifier: "",
        name: "setfirstweekday",
        kind: SigKind::ModuleFn,
        params: &[p("firstweekday", CoreTy::Unknown)],
        enforceable: false,
    },
    // POSITIVE: calendar.timegm(tuple) consumes a typed time tuple. A bare
    // user object is not a tuple/time tuple and must be rejected in strict mode.
    StdlibSig {
        module: "calendar",
        qualifier: "",
        name: "timegm",
        kind: SigKind::ModuleFn,
        params: &[p("tuple", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: generated cgi/cgitb rows lose these first-argument walls to
    // Unknown/Callable collapse. Bare user objects and concrete scalars are
    // safe strict-mode rejects for these contracts.
    StdlibSig {
        module: "cgi",
        qualifier: "",
        name: "print_exception",
        kind: SigKind::ModuleFn,
        params: &[p("type", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cgi",
        qualifier: "",
        name: "print_form",
        kind: SigKind::ModuleFn,
        params: &[p("form", CoreTy::Dict)],
        enforceable: true,
    },
    StdlibSig {
        module: "cgitb",
        qualifier: "",
        name: "scanvars",
        kind: SigKind::ModuleFn,
        params: &[p("reader", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: cmath accepts complex-like `_C` / float-like `_F` protocols.
    // Use Typed, not Complex/Float, so valid int/float/complex values and
    // richer protocol objects stay skip-safe while bare user objects are
    // rejected in strict mode.
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "acos",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "acosh",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "asin",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "asinh",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "atan",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "atanh",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "cos",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "cosh",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "exp",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "isclose",
        kind: SigKind::ModuleFn,
        params: &[p("a", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "isfinite",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "isinf",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "isnan",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "log",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "log10",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "phase",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "polar",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "rect",
        kind: SigKind::ModuleFn,
        params: &[p("r", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "sin",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "sinh",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "sqrt",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "tan",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmath",
        qualifier: "",
        name: "tanh",
        kind: SigKind::ModuleFn,
        params: &[p("z", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: cmd/code constructors and methods where generated signatures
    // either collapse the first argument to Unknown or skip at a star-param
    // boundary. These fixtures exercise the first positional wall only.
    StdlibSig {
        module: "cmd",
        qualifier: "Cmd",
        name: "cmdloop",
        kind: SigKind::Method,
        params: &[p("intro", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmd",
        qualifier: "Cmd",
        name: "columnize",
        kind: SigKind::Method,
        params: &[p("list", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmd",
        qualifier: "Cmd",
        name: "completenames",
        kind: SigKind::Method,
        params: &[p("text", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "cmd",
        qualifier: "Cmd",
        name: "postcmd",
        kind: SigKind::Method,
        params: &[p("stop", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "code",
        qualifier: "InteractiveConsole",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("locals", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "code",
        qualifier: "InteractiveInterpreter",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("locals", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: collections.ChainMap methods are mostly typevar/protocol
    // shaped in typeshed, so generated rows conservatively collapse them to
    // Unknown. A bare user object satisfies none of these contracts; keep real
    // mappings/iterables/dynamic values skip-safe through `Typed`.
    StdlibSig {
        module: "collections",
        qualifier: "ChainMap",
        name: "__delitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "ChainMap",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "ChainMap",
        name: "__ior__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "ChainMap",
        name: "__missing__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "ChainMap",
        name: "__or__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "ChainMap",
        name: "__ror__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "ChainMap",
        name: "__setitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed), p("value", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "ChainMap",
        name: "fromkeys",
        kind: SigKind::Method,
        params: &[p("iterable", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "ChainMap",
        name: "get",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed), p("default", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "ChainMap",
        name: "new_child",
        kind: SigKind::Method,
        params: &[p("m", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "ChainMap",
        name: "pop",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "ChainMap",
        name: "setdefault",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed), p("default", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: collections.Counter multiset operations and mutators are
    // nominal/protocol-shaped (`Counter`, `Mapping`, `Iterable`, typevars).
    // The generated rows keep these Unknown unless typeshed collapses the
    // annotation to `typed`; curate the fixture-backed first argument walls.
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "__add__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "__and__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "__ge__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "__gt__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("iterable", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "__ixor__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "__le__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "__lt__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "__missing__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "__or__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "__sub__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "__xor__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "subtract",
        kind: SigKind::Method,
        params: &[p("iterable", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "Counter",
        name: "update",
        kind: SigKind::Method,
        params: &[p("m", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: OrderedDict key/value operations are typevar/protocol-shaped
    // in typeshed. Use `Typed` for key/iterable contracts, `Bool` for
    // `popitem(last)`, and `Dict` for merge operands so both scalar non-dicts
    // and bare user objects are rejected while real mapping-like values remain
    // skip-safe.
    StdlibSig {
        module: "collections",
        qualifier: "OrderedDict",
        name: "__or__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Dict)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "OrderedDict",
        name: "__ror__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Dict)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "OrderedDict",
        name: "fromkeys",
        kind: SigKind::Method,
        params: &[p("iterable", CoreTy::Typed), p("value", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "OrderedDict",
        name: "move_to_end",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed), p("last", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "OrderedDict",
        name: "pop",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "OrderedDict",
        name: "popitem",
        kind: SigKind::Method,
        params: &[p("last", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "OrderedDict",
        name: "setdefault",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed), p("default", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: UserDict methods similarly collapse to Unknown in generated
    // rows. Bare user objects satisfy neither mapping/iterable protocols nor
    // typevar key contracts, so `Typed` gives fixture-backed strict walls
    // without hard-coding full protocol semantics.
    StdlibSig {
        module: "collections",
        qualifier: "UserDict",
        name: "__delitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserDict",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserDict",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("dict", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserDict",
        name: "__ior__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserDict",
        name: "__or__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserDict",
        name: "__ror__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserDict",
        name: "__setitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed), p("item", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserDict",
        name: "fromkeys",
        kind: SigKind::Method,
        params: &[p("iterable", CoreTy::Typed), p("value", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserDict",
        name: "get",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed), p("default", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: UserList has several typevar/protocol/slice overloads that
    // generated rows collapse to Unknown. The strict fixtures probe those with
    // a bare user object, which cannot satisfy an item typevar, SupportsIndex,
    // slice, Iterable, or comparable-list contract.
    StdlibSig {
        module: "collections",
        qualifier: "UserList",
        name: "__delitem__",
        kind: SigKind::Method,
        params: &[p("i", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserList",
        name: "__ge__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserList",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("i", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserList",
        name: "__gt__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserList",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("initlist", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserList",
        name: "__le__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserList",
        name: "__lt__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserList",
        name: "__setitem__",
        kind: SigKind::Method,
        params: &[p("i", CoreTy::Typed), p("item", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserList",
        name: "append",
        kind: SigKind::Method,
        params: &[p("item", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserList",
        name: "count",
        kind: SigKind::Method,
        params: &[p("item", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserList",
        name: "index",
        kind: SigKind::Method,
        params: &[
            p("item", CoreTy::Typed),
            p("start", CoreTy::Unknown),
            p("stop", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserList",
        name: "remove",
        kind: SigKind::Method,
        params: &[p("item", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: UserString overloads collapse several first-argument walls to
    // Unknown/Typed generated rows. Restore the fixture-backed strict walls
    // without modeling the full string overload matrix.
    StdlibSig {
        module: "collections",
        qualifier: "UserString",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("index", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserString",
        name: "center",
        kind: SigKind::Method,
        params: &[p("width", CoreTy::Int), p("fillchar", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserString",
        name: "endswith",
        kind: SigKind::Method,
        params: &[
            p("suffix", CoreTy::Typed),
            p("start", CoreTy::Unknown),
            p("end", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserString",
        name: "format_map",
        kind: SigKind::Method,
        params: &[p("mapping", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserString",
        name: "ljust",
        kind: SigKind::Method,
        params: &[p("width", CoreTy::Int), p("fillchar", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserString",
        name: "rjust",
        kind: SigKind::Method,
        params: &[p("width", CoreTy::Int), p("fillchar", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserString",
        name: "splitlines",
        kind: SigKind::Method,
        params: &[p("keepends", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "UserString",
        name: "startswith",
        kind: SigKind::Method,
        params: &[
            p("prefix", CoreTy::Typed),
            p("start", CoreTy::Unknown),
            p("end", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    // NEGATIVE: fnmatch.translate(pat) — `translate(123)` is a RUNTIME
    // TypeError (normcase raises it); the dispatcher models that contract.
    StdlibSig {
        module: "fnmatch",
        qualifier: "",
        name: "translate",
        kind: SigKind::ModuleFn,
        params: &[p("pat", CoreTy::Unknown)],
        enforceable: false,
    },
    // NEGATIVE: hashlib.new(name, data=b'') — `new(1)` is a RUNTIME
    // TypeError raised by the dispatcher (CPython: 'name must be a string'),
    // which the fixture catches; keep the type wall from rejecting it early.
    StdlibSig {
        module: "hashlib",
        qualifier: "",
        name: "new",
        kind: SigKind::ModuleFn,
        params: &[p("name", CoreTy::Unknown), p("data", CoreTy::Unknown)],
        enforceable: false,
    },
    // POSITIVE: SyntaxError(msg: str, details=...) -- generated typeshed rows
    // collapse the overloaded constructor to Unknown, but the strict type wall
    // requires rejecting a non-str message argument.
    StdlibSig {
        module: "builtins",
        qualifier: "SyntaxError",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("msg", CoreTy::Str), p("details", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: anext(i, default=...) -- generated overload accounting
    // collapses the first argument to Unknown, but a bare user instance cannot
    // satisfy either async or synchronous __anext__ protocol.
    StdlibSig {
        module: "builtins",
        qualifier: "",
        name: "anext",
        kind: SigKind::ModuleFn,
        params: &[p("i", CoreTy::Typed), p("default", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: asyncio.coroutines.iscoroutinefunction(func: Callable) - the
    // generated Callable row collapses to Unknown. A bare user instance cannot
    // satisfy Callable, so strict fixtures should fail before the runtime helper
    // returns False like CPython.
    StdlibSig {
        module: "asyncio.coroutines",
        qualifier: "",
        name: "iscoroutinefunction",
        kind: SigKind::ModuleFn,
        params: &[p("func", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: asyncio event/future helper signatures lose important strict
    // walls when Callable/Future-like types collapse to Unknown or too-broad
    // Typed rows in the generated table.
    StdlibSig {
        module: "asyncio.events",
        qualifier: "Handle",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("callback", CoreTy::Typed),
            p("args", CoreTy::Unknown),
            p("loop", CoreTy::Typed),
            p("context", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.exceptions",
        qualifier: "IncompleteReadError",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("partial", CoreTy::Bytes), p("expected", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.futures",
        qualifier: "",
        name: "wrap_future",
        kind: SigKind::ModuleFn,
        params: &[p("future", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.runners",
        qualifier: "",
        name: "run",
        kind: SigKind::ModuleFn,
        params: &[p("main", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.subprocess",
        qualifier: "",
        name: "create_subprocess_exec",
        kind: SigKind::ModuleFn,
        params: &[p("program", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.tasks",
        qualifier: "",
        name: "create_eager_task_factory",
        kind: SigKind::ModuleFn,
        params: &[p("custom_task_constructor", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.tasks",
        qualifier: "",
        name: "ensure_future",
        kind: SigKind::ModuleFn,
        params: &[p("coro_or_future", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.tasks",
        qualifier: "",
        name: "gather",
        kind: SigKind::ModuleFn,
        params: &[p("args", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.tasks",
        qualifier: "",
        name: "run_coroutine_threadsafe",
        kind: SigKind::ModuleFn,
        params: &[p("coro", CoreTy::Typed), p("loop", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.threads",
        qualifier: "",
        name: "to_thread",
        kind: SigKind::ModuleFn,
        params: &[p("func", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.transports",
        qualifier: "BaseTransport",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("extra", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: asyncio.trsock.TransportSocket is a private-but-importable
    // Py312 stdlib type used by asyncio transports. Generated rows either lose
    // protocol/bool precision or conservatively mark platform-specific socket
    // helpers unenforceable; tighten only the strict-type fixture walls.
    StdlibSig {
        module: "asyncio.trsock",
        qualifier: "TransportSocket",
        name: "__exit__",
        kind: SigKind::Method,
        params: &[
            p("exc_type", CoreTy::Typed),
            p("exc_val", CoreTy::Typed),
            p("exc_tb", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.trsock",
        qualifier: "TransportSocket",
        name: "ioctl",
        kind: SigKind::Method,
        params: &[p("control", CoreTy::Int), p("option", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.trsock",
        qualifier: "TransportSocket",
        name: "sendmsg_afalg",
        kind: SigKind::Method,
        params: &[p("msg", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.trsock",
        qualifier: "TransportSocket",
        name: "sendto",
        kind: SigKind::Method,
        params: &[p("data", CoreTy::Bytes), p("address", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.trsock",
        qualifier: "TransportSocket",
        name: "set_inheritable",
        kind: SigKind::Method,
        params: &[p("inheritable", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.trsock",
        qualifier: "TransportSocket",
        name: "setblocking",
        kind: SigKind::Method,
        params: &[p("flag", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.trsock",
        qualifier: "TransportSocket",
        name: "share",
        kind: SigKind::Method,
        params: &[p("process_id", CoreTy::Int)],
        enforceable: true,
    },
    // POSITIVE: atexit register/unregister require a callable target. The
    // generated Callable rows collapse to Unknown, but a bare user instance
    // cannot satisfy Callable and must be rejected by force typing.
    StdlibSig {
        module: "atexit",
        qualifier: "",
        name: "register",
        kind: SigKind::ModuleFn,
        params: &[p("func", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "atexit",
        qualifier: "",
        name: "unregister",
        kind: SigKind::ModuleFn,
        params: &[p("func", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: ast's deprecated Py312 literal-node helpers expose legacy
    // constructor/property contracts in typeshed. Generated rows either collapse
    // them to `Typed` or lose the parameter entirely; keep the strict wall exact
    // for the force-typed probes while leaving runtime AST construction separate.
    StdlibSig {
        module: "ast",
        qualifier: "Bytes",
        name: "__new__",
        kind: SigKind::Method,
        params: &[p("s", CoreTy::Bytes)],
        enforceable: true,
    },
    StdlibSig {
        module: "ast",
        qualifier: "Constant",
        name: "n",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "ast",
        qualifier: "Constant",
        name: "s",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "ast",
        qualifier: "Num",
        name: "__new__",
        kind: SigKind::Method,
        params: &[p("n", CoreTy::Complex)],
        enforceable: true,
    },
    // POSITIVE: CPython 3.12 local builds may not expose the internal module,
    // but the typeshed-derived strict wall must still reject a bare user object
    // before import-time behavior is observed.
    StdlibSig {
        module: "_interpchannels",
        qualifier: "",
        name: "create",
        kind: SigKind::ModuleFn,
        params: &[p("unboundop", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "_interpreters",
        qualifier: "",
        name: "capture_exception",
        kind: SigKind::ModuleFn,
        params: &[p("exc", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: generated rows are too conservative for these extension-class
    // contracts. Keep the static wall strict and let the runtime shims cover
    // dynamic execution paths through object.__new__(...).
    StdlibSig {
        module: "_lsprof",
        qualifier: "Profiler",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("timer", CoreTy::Typed),
            p("timeunit", CoreTy::Float),
            p("subcalls", CoreTy::Bool),
            p("builtins", CoreTy::Bool),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "_lsprof",
        qualifier: "Profiler",
        name: "enable",
        kind: SigKind::Method,
        params: &[p("subcalls", CoreTy::Bool), p("builtins", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "_multibytecodec",
        qualifier: "MultibyteIncrementalDecoder",
        name: "setstate",
        kind: SigKind::Method,
        params: &[p("state", CoreTy::Tuple)],
        enforceable: true,
    },
    // POSITIVE: generated typeshed collapses dict parameters to Unknown because
    // dict element types are richer than CoreTy. The container shape itself is
    // still enforceable as a negative scalar wall: concrete scalars such as int
    // are never dicts, while dynamic/container values stay skip-when-unsure.
    StdlibSig {
        module: "_osx_support",
        qualifier: "",
        name: "customize_compiler",
        kind: SigKind::ModuleFn,
        params: &[p("_config_vars", CoreTy::Dict)],
        enforceable: true,
    },
    StdlibSig {
        module: "_osx_support",
        qualifier: "",
        name: "customize_config_vars",
        kind: SigKind::ModuleFn,
        params: &[p("_config_vars", CoreTy::Dict)],
        enforceable: true,
    },
    StdlibSig {
        module: "_osx_support",
        qualifier: "",
        name: "get_platform_osx",
        kind: SigKind::ModuleFn,
        params: &[
            p("_config_vars", CoreTy::Dict),
            p("osname", CoreTy::Unknown),
            p("release", CoreTy::Unknown),
            p("machine", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    // POSITIVE: `_operator` has many protocol/typevar rows that generated
    // typeshed keeps Unknown-skipped. A bare user instance cannot satisfy these
    // operator protocols, so reject the first operand/callable object statically
    // while leaving the right-hand side dynamic where overloads vary.
    StdlibSig {
        module: "_operator",
        qualifier: "",
        name: "add",
        kind: SigKind::ModuleFn,
        params: &[p("a", CoreTy::Typed), p("b", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "_operator",
        qualifier: "",
        name: "call",
        kind: SigKind::ModuleFn,
        params: &[p("obj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "_operator",
        qualifier: "",
        name: "concat",
        kind: SigKind::ModuleFn,
        params: &[p("a", CoreTy::Typed), p("b", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "_operator",
        qualifier: "",
        name: "delitem",
        kind: SigKind::ModuleFn,
        params: &[p("a", CoreTy::Typed), p("b", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "_operator",
        qualifier: "",
        name: "getitem",
        kind: SigKind::ModuleFn,
        params: &[p("a", CoreTy::Typed), p("b", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "_operator",
        qualifier: "",
        name: "is_not_none",
        kind: SigKind::ModuleFn,
        params: &[p("a", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "_operator",
        qualifier: "",
        name: "mod",
        kind: SigKind::ModuleFn,
        params: &[p("a", CoreTy::Typed), p("b", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "_operator",
        qualifier: "",
        name: "mul",
        kind: SigKind::ModuleFn,
        params: &[p("a", CoreTy::Typed), p("b", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "_operator",
        qualifier: "",
        name: "setitem",
        kind: SigKind::ModuleFn,
        params: &[
            p("a", CoreTy::Typed),
            p("b", CoreTy::Unknown),
            p("c", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "_operator",
        qualifier: "",
        name: "sub",
        kind: SigKind::ModuleFn,
        params: &[p("a", CoreTy::Typed), p("b", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: `_posixsubprocess.fork_exec` accepts argv-like typed values for
    // `args`; generated typeshed collapses that first parameter to Unknown.
    // Keep the rest of the generated shape so this curated override does not
    // weaken the existing scalar walls for descriptor/fd parameters.
    StdlibSig {
        module: "_posixsubprocess",
        qualifier: "",
        name: "fork_exec",
        kind: SigKind::ModuleFn,
        params: &[
            p("args", CoreTy::Typed),
            p("executable_list", CoreTy::Unknown),
            p("close_fds", CoreTy::Typed),
            p("pass_fds", CoreTy::Unknown),
            p("cwd", CoreTy::Str),
            p("env", CoreTy::Unknown),
            p("p2cread", CoreTy::Int),
            p("p2cwrite", CoreTy::Int),
            p("c2pread", CoreTy::Int),
            p("c2pwrite", CoreTy::Int),
            p("errread", CoreTy::Int),
            p("errwrite", CoreTy::Int),
            p("errpipe_read", CoreTy::Int),
            p("errpipe_write", CoreTy::Int),
            p("restore_signals", CoreTy::Typed),
            p("call_setsid", CoreTy::Typed),
            p("pgid_to_set", CoreTy::Int),
            p("gid", CoreTy::Typed),
            p("extra_groups", CoreTy::Unknown),
            p("uid", CoreTy::Typed),
            p("child_umask", CoreTy::Int),
            p("preexec_fn", CoreTy::Unknown),
            p("allow_vfork", CoreTy::Typed),
        ],
        enforceable: true,
    },
    // POSITIVE: generated `_queue.SimpleQueue` rows keep `_T` and bool-like
    // extension parameters too loose for force-typed fixtures. Tighten only the
    // parameters represented by the current probes while preserving the rest of
    // the generated shape.
    StdlibSig {
        module: "_queue",
        qualifier: "SimpleQueue",
        name: "get",
        kind: SigKind::Method,
        params: &[p("block", CoreTy::Bool), p("timeout", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "_queue",
        qualifier: "SimpleQueue",
        name: "put",
        kind: SigKind::Method,
        params: &[
            p("item", CoreTy::Typed),
            p("block", CoreTy::Typed),
            p("timeout", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "_queue",
        qualifier: "SimpleQueue",
        name: "put_nowait",
        kind: SigKind::Method,
        params: &[p("item", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: generated `_remote_debugging` rows collapse callback/list/bool
    // contracts to Unknown/Typed. Tighten only the force-typed probes that the
    // current fixtures exercise.
    StdlibSig {
        module: "_remote_debugging",
        qualifier: "BinaryReader",
        name: "replay",
        kind: SigKind::Method,
        params: &[
            p("collector", CoreTy::Unknown),
            p("progress_callback", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "_remote_debugging",
        qualifier: "BinaryWriter",
        name: "write_sample",
        kind: SigKind::Method,
        params: &[
            p("stack_frames", CoreTy::List),
            p("timestamp_us", CoreTy::Int),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "_remote_debugging",
        qualifier: "GCMonitor",
        name: "get_gc_stats",
        kind: SigKind::Method,
        params: &[p("all_interpreters", CoreTy::Bool)],
        enforceable: true,
    },
    // POSITIVE: generated `_socket.getnameinfo` keeps sockaddr as Unknown, so
    // the force-typed probe leaks to the runtime _socket surface gap. Tighten
    // only sockaddr while preserving the generated int flags contract.
    StdlibSig {
        module: "_socket",
        qualifier: "",
        name: "getnameinfo",
        kind: SigKind::ModuleFn,
        params: &[p("sockaddr", CoreTy::Typed), p("flags", CoreTy::Int)],
        enforceable: true,
    },
    // POSITIVE: generated `_sqlite3` rows either omit later typevar params or
    // collapse path/type/bool contracts. Tighten only the currently promoted
    // force-typed probes; the `_sqlite3` runtime import surface is separate
    // Py312 behavior work.
    StdlibSig {
        module: "_sqlite3",
        qualifier: "",
        name: "adapt",
        kind: SigKind::ModuleFn,
        params: &[
            p("obj", CoreTy::Unknown),
            p("proto", CoreTy::Unknown),
            p("alt", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "_sqlite3",
        qualifier: "",
        name: "connect",
        kind: SigKind::ModuleFn,
        params: &[
            p("database", CoreTy::Typed),
            p("timeout", CoreTy::Float),
            p("detect_types", CoreTy::Int),
            p("isolation_level", CoreTy::Unknown),
            p("check_same_thread", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "_sqlite3",
        qualifier: "",
        name: "enable_callback_tracebacks",
        kind: SigKind::ModuleFn,
        params: &[p("enable", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "_sqlite3",
        qualifier: "",
        name: "register_adapter",
        kind: SigKind::ModuleFn,
        params: &[p("type", CoreTy::Typed), p("adapter", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: generated `_ssl.Certificate.public_bytes` currently loses its
    // `format` enum/int parameter. Enforce the scalar wall here; the
    // `_ssl.Certificate` export/runtime surface remains separate Py312 work.
    StdlibSig {
        module: "_ssl",
        qualifier: "Certificate",
        name: "public_bytes",
        kind: SigKind::Method,
        params: &[p("format", CoreTy::Int)],
        enforceable: true,
    },
    // POSITIVE: generated `_struct.pack`/`pack_into` rows are variadic and
    // therefore skipped wholesale. Enforce only the fixed prefix; extra values
    // remain skip-safe because the call hook stops when params are exhausted.
    StdlibSig {
        module: "_struct",
        qualifier: "",
        name: "pack",
        kind: SigKind::ModuleFn,
        params: &[p("fmt", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "_struct",
        qualifier: "",
        name: "pack_into",
        kind: SigKind::ModuleFn,
        params: &[
            p("fmt", CoreTy::Typed),
            p("buffer", CoreTy::Typed),
            p("offset", CoreTy::Int),
        ],
        enforceable: true,
    },
    // POSITIVE: generated `_thread` rows lose Callable/bool/context-manager
    // precision, and some CPython-private aliases are not runtime importable on
    // every platform. Enforce only the fixed first parameter for these strict
    // type-wall probes; variadic args, timeout, and traceback slots stay out of
    // scope.
    StdlibSig {
        module: "_thread",
        qualifier: "",
        name: "start_joinable_thread",
        kind: SigKind::ModuleFn,
        params: &[p("function", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "_thread",
        qualifier: "",
        name: "start_new",
        kind: SigKind::ModuleFn,
        params: &[p("function", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "_thread",
        qualifier: "",
        name: "start_new_thread",
        kind: SigKind::ModuleFn,
        params: &[p("function", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "_thread",
        qualifier: "LockType",
        name: "__exit__",
        kind: SigKind::Method,
        params: &[p("type", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "_thread",
        qualifier: "LockType",
        name: "acquire",
        kind: SigKind::Method,
        params: &[p("blocking", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "_thread",
        qualifier: "LockType",
        name: "acquire_lock",
        kind: SigKind::Method,
        params: &[p("blocking", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "_thread",
        qualifier: "RLock",
        name: "__exit__",
        kind: SigKind::Method,
        params: &[p("t", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "_thread",
        qualifier: "RLock",
        name: "acquire",
        kind: SigKind::Method,
        params: &[p("blocking", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "_thread",
        qualifier: "lock",
        name: "__exit__",
        kind: SigKind::Method,
        params: &[p("type", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "_thread",
        qualifier: "lock",
        name: "acquire",
        kind: SigKind::Method,
        params: &[p("blocking", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "_thread",
        qualifier: "lock",
        name: "acquire_lock",
        kind: SigKind::Method,
        params: &[p("blocking", CoreTy::Bool)],
        enforceable: true,
    },
    // POSITIVE: generated `_tkinter.TkappType.wantobjects` loses the optional
    // setter argument, and local CPython builds may omit `_tkinter` entirely.
    // Enforce only the strict-type wall for the explicit setter probe.
    StdlibSig {
        module: "_tkinter",
        qualifier: "TkappType",
        name: "wantobjects",
        kind: SigKind::Method,
        params: &[p("wantobjects", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: typeshed exposes `_warnings.warn` / `warn_explicit` overloads
    // for `str` and `Warning` messages, while CPython accepts arbitrary runtime
    // objects and stringifies them. Use a scalar `str` wall for strict-type
    // fixture probes: it rejects obviously wrong scalars and bare user objects,
    // while warning instances stay skip-when-unsure instead of becoming false
    // positives.
    StdlibSig {
        module: "_warnings",
        qualifier: "",
        name: "warn",
        kind: SigKind::ModuleFn,
        params: &[
            p("message", CoreTy::Str),
            p("category", CoreTy::Unknown),
            p("stacklevel", CoreTy::Int),
            p("source", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "_warnings",
        qualifier: "",
        name: "warn_explicit",
        kind: SigKind::ModuleFn,
        params: &[
            p("message", CoreTy::Str),
            p("category", CoreTy::Unknown),
            p("filename", CoreTy::Str),
            p("lineno", CoreTy::Int),
            p("module", CoreTy::Unknown),
            p("registry", CoreTy::Unknown),
            p("module_globals", CoreTy::Unknown),
            p("source", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    // POSITIVE: `_weakrefset.WeakSet.__init__(data: Iterable | None)` is richer
    // than CoreTy can represent. A `Typed` wall rejects the generated bare
    // user-instance probes while runtime validation handles scalar
    // non-iterables for Py312 behavior.
    StdlibSig {
        module: "_weakrefset",
        qualifier: "WeakSet",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("data", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: complex(real=0, imag=0) accepts string/numeric/dynamic values.
    // `Typed` only rejects a provably bare user instance and leaves scalar
    // overload candidates skip-safe.
    StdlibSig {
        module: "builtins",
        qualifier: "",
        name: "complex",
        kind: SigKind::ModuleFn,
        params: &[p("real", CoreTy::Typed), p("imag", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: bytes/bytearray constructors are overload-heavy. The first
    // argument may be a size int, text str with encoding, bytes-like object, or
    // iterable of ints. A Typed source only rejects a provably bare `_W()` probe
    // and leaves all scalar overload candidates untouched; later scalar params
    // still enforce when present.
    StdlibSig {
        module: "builtins",
        qualifier: "",
        name: "bytes",
        kind: SigKind::ModuleFn,
        params: &[
            p("source", CoreTy::Typed),
            p("encoding", CoreTy::Str),
            p("errors", CoreTy::Str),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "",
        name: "bytearray",
        kind: SigKind::ModuleFn,
        params: &[
            p("source", CoreTy::Typed),
            p("encoding", CoreTy::Str),
            p("errors", CoreTy::Str),
        ],
        enforceable: true,
    },
    // POSITIVE: filter(function, iterable) routes through filter.__new__ in
    // CPython. A bare user instance cannot satisfy Callable/None; the second
    // iterable argument remains Unknown until protocol modeling is richer.
    StdlibSig {
        module: "builtins",
        qualifier: "",
        name: "filter",
        kind: SigKind::ModuleFn,
        params: &[p("function", CoreTy::Typed), p("iterable", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: map.__new__(cls, func, iterable, ...) requires a callable
    // function. Model the first iterable as Unknown and keep additional
    // iterables skip-safe until variadic protocol rows are representable.
    StdlibSig {
        module: "builtins",
        qualifier: "map",
        name: "__new__",
        kind: SigKind::Method,
        params: &[
            p("cls", CoreTy::Typed),
            p("func", CoreTy::Typed),
            p("iterable", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    // POSITIVE: isinstance's second argument must be a class or tuple of
    // classes. A bare `_W()` instance cannot satisfy that classinfo contract.
    StdlibSig {
        module: "builtins",
        qualifier: "",
        name: "isinstance",
        kind: SigKind::ModuleFn,
        params: &[
            p("obj", CoreTy::Unknown),
            p("class_or_tuple", CoreTy::Typed),
        ],
        enforceable: true,
    },
    // POSITIVE: iter(object, sentinel) requires a callable object in CPython;
    // one-arg iter(object) similarly requires an iterable/getitem-capable object.
    // A bare `_W()` instance satisfies neither contract, while callable values
    // and dynamic iterable values stay skip-safe.
    StdlibSig {
        module: "builtins",
        qualifier: "",
        name: "iter",
        kind: SigKind::ModuleFn,
        params: &[p("object", CoreTy::Typed), p("sentinel", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytearray",
        name: "__release_buffer__",
        kind: SigKind::Method,
        params: &[p("buffer", CoreTy::MemoryView)],
        enforceable: true,
    },
    // POSITIVE: memoryview method contracts. Key/exception/order contracts are
    // represented as Typed so bare `_W()` probes are rejected while dynamic and
    // scalar-valid values remain skip-safe. The private release hook requires a
    // memoryview object, so concrete scalars are rejected by the MemoryView
    // negative wall.
    StdlibSig {
        module: "builtins",
        qualifier: "memoryview",
        name: "__exit__",
        kind: SigKind::Method,
        params: &[
            p("exc_type", CoreTy::Typed),
            p("exc_value", CoreTy::Unknown),
            p("traceback", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "memoryview",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "memoryview",
        name: "__release_buffer__",
        kind: SigKind::Method,
        params: &[p("buffer", CoreTy::MemoryView)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "memoryview",
        name: "__setitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed), p("value", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "memoryview",
        name: "tobytes",
        kind: SigKind::Method,
        params: &[p("order", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: complex arithmetic dunders accept complex-compatible numeric
    // values. A dedicated negative wall rejects impossible concrete scalars such
    // as str while allowing int/float/bool and dynamic complex-like values.
    StdlibSig {
        module: "builtins",
        qualifier: "complex",
        name: "__add__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "complex",
        name: "__mul__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "complex",
        name: "__pow__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Complex), p("mod", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "complex",
        name: "__radd__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "complex",
        name: "__rmul__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "complex",
        name: "__rpow__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Complex), p("mod", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "complex",
        name: "__rsub__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "complex",
        name: "__rtruediv__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "complex",
        name: "__sub__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "complex",
        name: "__truediv__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Complex)],
        enforceable: true,
    },
    // POSITIVE: generated float pow/round rows collapse overload/protocol
    // details to Unknown. The numeric dunders accept bool/int/float under
    // Python numeric promotion, which CoreTy::Float already models, and reject
    // impossible concrete scalars. round(ndigits) uses SupportsIndex; Int is the
    // conservative scalar wall and still accepts bool through bool-is-int.
    StdlibSig {
        module: "builtins",
        qualifier: "float",
        name: "__pow__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Float), p("mod", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "float",
        name: "__round__",
        kind: SigKind::Method,
        params: &[p("ndigits", CoreTy::Int)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "float",
        name: "__rpow__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Float), p("mod", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: bytes/bytearray bytes-like methods accept bytes-like values or
    // tuples thereof. Concrete scalars such as int/str/bool are never bytes,
    // while actual bytes literals infer to Any today and stay skip-when-unsure.
    StdlibSig {
        module: "builtins",
        qualifier: "bytes",
        name: "__ge__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Bytes)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytes",
        name: "__gt__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Bytes)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytes",
        name: "__le__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Bytes)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytes",
        name: "__lt__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Bytes)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytes",
        name: "endswith",
        kind: SigKind::Method,
        params: &[
            p("suffix", CoreTy::Bytes),
            p("start", CoreTy::Typed),
            p("end", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytes",
        name: "startswith",
        kind: SigKind::Method,
        params: &[
            p("prefix", CoreTy::Bytes),
            p("start", CoreTy::Typed),
            p("end", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytearray",
        name: "endswith",
        kind: SigKind::Method,
        params: &[
            p("suffix", CoreTy::Bytes),
            p("start", CoreTy::Typed),
            p("end", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytearray",
        name: "startswith",
        kind: SigKind::Method,
        params: &[
            p("prefix", CoreTy::Bytes),
            p("start", CoreTy::Typed),
            p("end", CoreTy::Typed),
        ],
        enforceable: true,
    },
    // POSITIVE: index/slice overloads are represented as Typed so a bare user
    // object is rejected without claiming a full slice/SupportsIndex model.
    StdlibSig {
        module: "builtins",
        qualifier: "bytes",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytearray",
        name: "__delitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytearray",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytearray",
        name: "__setitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed), p("value", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytes",
        name: "splitlines",
        kind: SigKind::Method,
        params: &[p("keepends", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bytearray",
        name: "splitlines",
        kind: SigKind::Method,
        params: &[p("keepends", CoreTy::Bool)],
        enforceable: true,
    },
    // POSITIVE: str operator and string-method overloads collapse to Unknown in
    // the generated table because they involve LiteralString, SupportsIndex,
    // tuple[str, ...], or printf-style formatting. Keep them as guarded
    // negative walls: concrete wrong scalars are rejected for str-only
    // parameters, and bare `_W()` probes are rejected for protocol/typed slots
    // while valid strings, ints, slices, and dynamic values stay skip-safe.
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "__add__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "__mod__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "__mul__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "__new__",
        kind: SigKind::Method,
        params: &[
            p("cls", CoreTy::Typed),
            p("object", CoreTy::Typed),
            p("encoding", CoreTy::Unknown),
            p("errors", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "__rmul__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "center",
        kind: SigKind::Method,
        params: &[p("width", CoreTy::Typed), p("fillchar", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "endswith",
        kind: SigKind::Method,
        params: &[
            p("suffix", CoreTy::Typed),
            p("start", CoreTy::Typed),
            p("end", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "expandtabs",
        kind: SigKind::Method,
        params: &[p("tabsize", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "ljust",
        kind: SigKind::Method,
        params: &[p("width", CoreTy::Typed), p("fillchar", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "lstrip",
        kind: SigKind::Method,
        params: &[p("chars", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "maketrans",
        kind: SigKind::Method,
        params: &[p("x", CoreTy::Str), p("y", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "partition",
        kind: SigKind::Method,
        params: &[p("sep", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "removeprefix",
        kind: SigKind::Method,
        params: &[p("prefix", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "removesuffix",
        kind: SigKind::Method,
        params: &[p("suffix", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "replace",
        kind: SigKind::Method,
        params: &[p("old", CoreTy::Str), p("new", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "rjust",
        kind: SigKind::Method,
        params: &[p("width", CoreTy::Typed), p("fillchar", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "rpartition",
        kind: SigKind::Method,
        params: &[p("sep", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "rsplit",
        kind: SigKind::Method,
        params: &[p("sep", CoreTy::Str), p("maxsplit", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "rstrip",
        kind: SigKind::Method,
        params: &[p("chars", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "split",
        kind: SigKind::Method,
        params: &[p("sep", CoreTy::Str), p("maxsplit", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "splitlines",
        kind: SigKind::Method,
        params: &[p("keepends", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "startswith",
        kind: SigKind::Method,
        params: &[
            p("prefix", CoreTy::Typed),
            p("start", CoreTy::Typed),
            p("end", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "strip",
        kind: SigKind::Method,
        params: &[p("chars", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "str",
        name: "zfill",
        kind: SigKind::Method,
        params: &[p("width", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: list dunders have overloads for list-vs-list operations and
    // index/slice operations. Use a dedicated List negative scalar wall for
    // list-valued operands, and Typed for index/slice protocol operands where a
    // bare `_W()` cannot satisfy either SupportsIndex or slice.
    StdlibSig {
        module: "builtins",
        qualifier: "list",
        name: "__add__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::List)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "list",
        name: "__ge__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::List)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "list",
        name: "__gt__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::List)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "list",
        name: "__le__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::List)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "list",
        name: "__lt__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::List)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "list",
        name: "__delitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "list",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "list",
        name: "__setitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed), p("value", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: tuple dunders mirror list's value/key contracts. Use a
    // dedicated Tuple negative scalar wall for tuple-valued operands and Typed
    // for the SupportsIndex/slice key overloads.
    StdlibSig {
        module: "builtins",
        qualifier: "tuple",
        name: "__add__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Tuple)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "tuple",
        name: "__ge__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Tuple)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "tuple",
        name: "__gt__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Tuple)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "tuple",
        name: "__le__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Tuple)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "tuple",
        name: "__lt__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Tuple)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "tuple",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: type object constructors/descriptors. The strict fixtures call
    // `obj = object.__new__(type)` then bound methods on that object, so these
    // rows start at the instance-bound argument rather than the classmethod
    // `cls` slot.
    StdlibSig {
        module: "builtins",
        qualifier: "type",
        name: "__new__",
        kind: SigKind::Method,
        params: &[
            p("name", CoreTy::Str),
            p("bases", CoreTy::Unknown),
            p("namespace", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "type",
        name: "__subclasscheck__",
        kind: SigKind::Method,
        params: &[p("subclass", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: zip.__new__(iter1, *iter2) requires iterable inputs. Model the
    // first iterable as a Typed negative wall so a provably bare `_W()` is
    // rejected while real/dynamic iterable values remain skip-safe.
    StdlibSig {
        module: "builtins",
        qualifier: "zip",
        name: "__new__",
        kind: SigKind::Method,
        params: &[p("iter1", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: range index/new overloads use SupportsIndex/slice protocols.
    // Typed rejects only a provably bare `_W()` while accepting ints, slices,
    // classes with __index__, and dynamic values.
    StdlibSig {
        module: "builtins",
        qualifier: "range",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "range",
        name: "__new__",
        kind: SigKind::Method,
        params: &[
            p("cls", CoreTy::Typed),
            p("start_or_stop", CoreTy::Typed),
            p("stop", CoreTy::Typed),
            p("step", CoreTy::Typed),
        ],
        enforceable: true,
    },
    // POSITIVE: property descriptors accept callable/dynamic/None values at
    // runtime. The strict wall only rejects a provably bare `_W()` for the
    // descriptor target/callable protocol slots.
    StdlibSig {
        module: "builtins",
        qualifier: "property",
        name: "__get__",
        kind: SigKind::Method,
        params: &[p("instance", CoreTy::Typed), p("owner", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "property",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("fget", CoreTy::Typed),
            p("fset", CoreTy::Typed),
            p("fdel", CoreTy::Typed),
            p("doc", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "property",
        name: "deleter",
        kind: SigKind::Method,
        params: &[p("fdel", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "property",
        name: "getter",
        kind: SigKind::Method,
        params: &[p("fget", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "property",
        name: "setter",
        kind: SigKind::Method,
        params: &[p("fset", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: object.__subclasshook__(subclass) expects a type object. Reject
    // only a direct bare instance probe; class objects remain accepted by the
    // expression-shape guard in the stdlib hook.
    StdlibSig {
        module: "builtins",
        qualifier: "object",
        name: "__subclasshook__",
        kind: SigKind::Method,
        params: &[p("subclass", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: reversed.__new__(sequence) accepts Reversible /
    // SupportsLenAndGetItem protocol values. Model that protocol contract as a
    // Typed negative wall so a bare instance is rejected while concrete
    // sequence/protocol-shaped values stay skip-safe.
    StdlibSig {
        module: "builtins",
        qualifier: "reversed",
        name: "__new__",
        kind: SigKind::Method,
        params: &[p("cls", CoreTy::Typed), p("sequence", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: classmethod is descriptor-shaped. These curated rows only
    // reject a provably bare `_W()` for Callable/type-variable contracts; real
    // callables and dynamic descriptor uses stay skip-safe.
    StdlibSig {
        module: "builtins",
        qualifier: "classmethod",
        name: "__get__",
        kind: SigKind::Method,
        params: &[p("instance", CoreTy::Typed), p("owner", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "classmethod",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("f", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: staticmethod mirrors classmethod for force-typed descriptor
    // contracts: a bare user instance cannot satisfy Callable or a concrete
    // descriptor instance type, while real callables and None/dynamic owner
    // values remain skip-safe.
    StdlibSig {
        module: "builtins",
        qualifier: "staticmethod",
        name: "__get__",
        kind: SigKind::Method,
        params: &[p("instance", CoreTy::Typed), p("owner", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "staticmethod",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("f", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: Python function objects are not importable as
    // `builtins.function`, but every `def f(): ...` value is an instance of
    // that internal type. Keep instance permissive for descriptor access while
    // rejecting a provably bare user object as `owner`.
    StdlibSig {
        module: "builtins",
        qualifier: "function",
        name: "__get__",
        kind: SigKind::Method,
        params: &[p("instance", CoreTy::Unknown), p("owner", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: `int.__new__` is normally reached as a classmethod-style call,
    // so the explicit `cls` argument must be consumed before enforcing `x`.
    // `base` is intentionally Unknown: CPython validates it at runtime and
    // typeshed overloads make it unsafe to scalar-wall here.
    StdlibSig {
        module: "builtins",
        qualifier: "int",
        name: "__new__",
        kind: SigKind::Method,
        params: &[
            p("cls", CoreTy::Typed),
            p("x", CoreTy::Typed),
            p("base", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    // POSITIVE: generated int.__pow__ collapses overloads/Literal aliases to
    // Unknown. The receiver method still requires an int exponent; `mod` stays
    // runtime-validated because `None` and omitted-mod forms are both common.
    StdlibSig {
        module: "builtins",
        qualifier: "int",
        name: "__pow__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Int), p("mod", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: bool bitwise dunders accept bool/int operands. A single int
    // contract covers both overloads because bool is int-compatible in the type
    // checker, while wrong scalar operands such as str must be rejected.
    StdlibSig {
        module: "builtins",
        qualifier: "bool",
        name: "__and__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Int)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bool",
        name: "__or__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Int)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bool",
        name: "__xor__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Int)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bool",
        name: "__rand__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Int)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bool",
        name: "__ror__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Int)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "bool",
        name: "__rxor__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Int)],
        enforceable: true,
    },
    // POSITIVE: set rich/in-place operators mirror frozenset's AbstractSet
    // contract. Use a Typed negative wall until the type model has dedicated
    // set/frozenset protocol inference.
    StdlibSig {
        module: "builtins",
        qualifier: "set",
        name: "__and__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "set",
        name: "__ge__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "set",
        name: "__gt__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "set",
        name: "__iand__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "set",
        name: "__ior__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "set",
        name: "__isub__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "set",
        name: "__ixor__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "set",
        name: "__le__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "set",
        name: "__lt__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "set",
        name: "__or__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "set",
        name: "__sub__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "set",
        name: "__xor__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: slice.__new__ carries type-variable start/stop contracts in
    // typeshed. Treat those as Typed negative walls so direct bare instances are
    // rejected while ints, None, class objects, and dynamic values stay accepted.
    StdlibSig {
        module: "builtins",
        qualifier: "slice",
        name: "__new__",
        kind: SigKind::Method,
        params: &[
            p("cls", CoreTy::Typed),
            p("start", CoreTy::Typed),
            p("stop", CoreTy::Typed),
            p("step", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    // POSITIVE: frozenset rich/set operators accept AbstractSet-like values.
    // There is no dedicated Ty::Set/FrozenSet yet, so model the protocol as a
    // Typed negative wall: a bare user instance satisfies neither AbstractSet
    // nor the nominal/protocol contract, while modeled/dynamic operands stay
    // skip-when-unsure.
    StdlibSig {
        module: "builtins",
        qualifier: "frozenset",
        name: "__and__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "frozenset",
        name: "__ge__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "frozenset",
        name: "__gt__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "frozenset",
        name: "__le__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "frozenset",
        name: "__lt__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "frozenset",
        name: "__new__",
        kind: SigKind::Method,
        params: &[p("cls", CoreTy::Typed), p("iterable", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "frozenset",
        name: "__or__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "frozenset",
        name: "__sub__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "frozenset",
        name: "__xor__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: ExceptionGroup matcher/sequence methods use non-scalar
    // contracts in typeshed. A bare user instance cannot satisfy Callable,
    // exception type, tuple-of-types, or Sequence, so reject it through the
    // Typed bare-class path before the runtime method surface is reached.
    StdlibSig {
        module: "builtins",
        qualifier: "BaseExceptionGroup",
        name: "derive",
        kind: SigKind::Method,
        params: &[p("excs", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "BaseExceptionGroup",
        name: "split",
        kind: SigKind::Method,
        params: &[p("matcher_value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "BaseExceptionGroup",
        name: "subgroup",
        kind: SigKind::Method,
        params: &[p("matcher_value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "ExceptionGroup",
        name: "split",
        kind: SigKind::Method,
        params: &[p("matcher_value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "builtins",
        qualifier: "ExceptionGroup",
        name: "subgroup",
        kind: SigKind::Method,
        params: &[p("matcher_value", CoreTy::Typed)],
        enforceable: true,
    },
    // NEGATIVE: unicodedata.name(chr[, default]) / category(chr) — a non-str
    // or multi-character argument is a RUNTIME TypeError (the dispatcher
    // requires a single unicode character: `name(123)`), which the fixture
    // catches; keep the type wall from rejecting it at compile time.
    StdlibSig {
        module: "unicodedata",
        qualifier: "",
        name: "name",
        kind: SigKind::ModuleFn,
        params: &[p("chr", CoreTy::Unknown), p("default", CoreTy::Unknown)],
        enforceable: false,
    },
    StdlibSig {
        module: "unicodedata",
        qualifier: "",
        name: "category",
        kind: SigKind::ModuleFn,
        params: &[p("chr", CoreTy::Unknown)],
        enforceable: false,
    },
    // NEGATIVE: colorsys conversions take three real numbers; a non-numeric
    // channel (`rgb_to_hsv("x", 0, 0)`) is a RUNTIME TypeError raised by the
    // dispatcher, so keep the type wall from rejecting it at compile time.
    StdlibSig {
        module: "colorsys",
        qualifier: "",
        name: "rgb_to_hsv",
        kind: SigKind::ModuleFn,
        params: &[
            p("r", CoreTy::Unknown),
            p("g", CoreTy::Unknown),
            p("b", CoreTy::Unknown),
        ],
        enforceable: false,
    },
    StdlibSig {
        module: "colorsys",
        qualifier: "",
        name: "hsv_to_rgb",
        kind: SigKind::ModuleFn,
        params: &[
            p("h", CoreTy::Unknown),
            p("s", CoreTy::Unknown),
            p("v", CoreTy::Unknown),
        ],
        enforceable: false,
    },
    StdlibSig {
        module: "colorsys",
        qualifier: "",
        name: "rgb_to_hls",
        kind: SigKind::ModuleFn,
        params: &[
            p("r", CoreTy::Unknown),
            p("g", CoreTy::Unknown),
            p("b", CoreTy::Unknown),
        ],
        enforceable: false,
    },
    StdlibSig {
        module: "colorsys",
        qualifier: "",
        name: "hls_to_rgb",
        kind: SigKind::ModuleFn,
        params: &[
            p("h", CoreTy::Unknown),
            p("l", CoreTy::Unknown),
            p("s", CoreTy::Unknown),
        ],
        enforceable: false,
    },
    StdlibSig {
        module: "colorsys",
        qualifier: "",
        name: "rgb_to_yiq",
        kind: SigKind::ModuleFn,
        params: &[
            p("r", CoreTy::Unknown),
            p("g", CoreTy::Unknown),
            p("b", CoreTy::Unknown),
        ],
        enforceable: false,
    },
    StdlibSig {
        module: "colorsys",
        qualifier: "",
        name: "yiq_to_rgb",
        kind: SigKind::ModuleFn,
        params: &[
            p("y", CoreTy::Unknown),
            p("i", CoreTy::Unknown),
            p("q", CoreTy::Unknown),
        ],
        enforceable: false,
    },
    // NEGATIVE: textwrap.dedent(text) — `dedent(123)` is a RUNTIME TypeError
    // (CPython runs `_whitespace_only_re.sub` over a non-str → "expected string
    // or bytes-like object"); the dispatcher raises it. Keep the type wall from
    // rejecting it at compile time.
    StdlibSig {
        module: "textwrap",
        qualifier: "",
        name: "dedent",
        kind: SigKind::ModuleFn,
        params: &[p("text", CoreTy::Unknown)],
        enforceable: false,
    },
    // NOTE: textwrap.indent is deliberately NOT overridden. Its runtime raises
    // AttributeError on a non-str (CPython's `text.splitlines(True)`), but the
    // `type/std-libs/textwrap/indent__text_as_str_wrong` STRICT_TYPE fixture
    // requires the compile-time wall to raise TypeError. Those two contracts
    // conflict for `indent(<int>, …)`, and the type-dimension enforcement wins,
    // so the wall stays and errors/indent_non_str_raises remains unmet.
    // NEGATIVE: shlex.quote(s) — `quote(42)` is a RUNTIME TypeError (CPython's
    // `_find_unsafe(s)` regex over a non-str → "expected string or bytes-like
    // object"); the dispatcher raises it. Keep the type wall out of the way.
    StdlibSig {
        module: "shlex",
        qualifier: "",
        name: "quote",
        kind: SigKind::ModuleFn,
        params: &[p("s", CoreTy::Unknown)],
        enforceable: false,
    },
    // NEGATIVE: os.umask(mask) — `umask("x")` is a RUNTIME TypeError ("'str'
    // object cannot be interpreted as an integer"); the dispatcher raises it.
    StdlibSig {
        module: "os",
        qualifier: "",
        name: "umask",
        kind: SigKind::ModuleFn,
        params: &[p("mask", CoreTy::Unknown)],
        enforceable: false,
    },
    // NEGATIVE: locale.setlocale(category, locale=None) — a non-int category
    // (`setlocale("not_a_category", ...)`) is a RUNTIME TypeError ("an integer
    // is required (got type str)"); the dispatcher raises it.
    StdlibSig {
        module: "locale",
        qualifier: "",
        name: "setlocale",
        kind: SigKind::ModuleFn,
        params: &[p("category", CoreTy::Unknown), p("locale", CoreTy::Unknown)],
        enforceable: false,
    },
    // NEGATIVE: signal.setitimer(which, seconds, interval=0.0) — a non-int
    // `which` (`setitimer("not_int", 1.0)`) is a RUNTIME TypeError; the
    // dispatcher raises it.
    StdlibSig {
        module: "signal",
        qualifier: "",
        name: "setitimer",
        kind: SigKind::ModuleFn,
        params: &[
            p("which", CoreTy::Unknown),
            p("seconds", CoreTy::Unknown),
            p("interval", CoreTy::Unknown),
        ],
        enforceable: false,
    },
    // keyword.iskeyword/issoftkeyword are force-typed as `s: str` for
    // strict-type fixtures. Runtime behavior stays CPython-compatible:
    // non-str values compare unequal to every keyword and return False.
    StdlibSig {
        module: "keyword",
        qualifier: "",
        name: "iskeyword",
        kind: SigKind::ModuleFn,
        params: &[p("s", CoreTy::Str)],
        enforceable: false,
    },
    StdlibSig {
        module: "keyword",
        qualifier: "",
        name: "issoftkeyword",
        kind: SigKind::ModuleFn,
        params: &[p("s", CoreTy::Str)],
        enforceable: false,
    },
];

/// Look up a signature by `(module, qualifier, name)`. `qualifier` is `""` for
/// module functions and the class name for methods.
///
/// The curated [`STDLIB_SIGS`] table takes precedence (it is an explicit,
/// human-verified override — including documented negative guards). On a miss
/// we fall back to the typeshed-derived [`STDLIB_SIGS_GENERATED`] table, whose
/// rows are conservatively `enforceable=false` for anything non-scalar /
/// overloaded / variadic. Either way the call-site hook only ever enforces
/// when `enforceable=true` AND the actual argument is a concrete scalar, so a
/// fallback row never introduces a false positive on a correct call.
pub fn get(module: &str, qualifier: &str, name: &str) -> Option<&'static StdlibSig> {
    STDLIB_SIGS
        .iter()
        .find(|s| s.module == module && s.qualifier == qualifier && s.name == name)
        .or_else(|| {
            super::stdlib_sigs_generated::STDLIB_SIGS_GENERATED
                .iter()
                .find(|s| s.module == module && s.qualifier == qualifier && s.name == name)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_module_fn() {
        let s = get("os", "", "strerror").expect("strerror present");
        assert!(s.enforceable);
        assert_eq!(s.params[0].ty, CoreTy::Int);
        assert_eq!(s.kind, SigKind::ModuleFn);
    }

    #[test]
    fn lookup_method() {
        let s = get("html.parser", "HTMLParser", "handle_entityref").expect("method present");
        assert_eq!(s.kind, SigKind::Method);
        assert_eq!(s.params[0].ty, CoreTy::Str);
    }

    #[test]
    fn negative_not_enforceable() {
        assert!(!get("base64", "", "b64encode").unwrap().enforceable);
        assert!(!get("math", "", "factorial").unwrap().enforceable);
    }

    #[test]
    fn qualifier_disambiguates() {
        // Method lookup with empty qualifier must miss.
        assert!(get("html.parser", "", "handle_entityref").is_none());
        // Module-fn lookup with a qualifier must miss.
        assert!(get("os", "HTMLParser", "strerror").is_none());
    }

    #[test]
    fn unknown_misses() {
        assert!(get("os", "", "nonexistent").is_none());
        assert!(get("nope", "", "strerror").is_none());
    }

    // --- Generated typeshed table -----------------------------------------

    #[test]
    fn generated_table_is_nonempty_and_consulted() {
        use super::super::stdlib_sigs_generated::STDLIB_SIGS_GENERATED;
        assert!(
            STDLIB_SIGS_GENERATED.len() > 1000,
            "generated table should hold thousands of typeshed sigs, got {}",
            STDLIB_SIGS_GENERATED.len(),
        );
        // At least some rows must be enforceable scalars; most are Unknown-skipped.
        let enf = STDLIB_SIGS_GENERATED
            .iter()
            .filter(|s| s.enforceable)
            .count();
        assert!(
            enf > 100,
            "expected hundreds of enforceable scalar sigs, got {enf}"
        );
    }

    #[test]
    fn generated_enforceable_rows_have_a_scalar_and_no_star() {
        // The invariant: every row the hook will ENFORCE must (a) carry at least
        // one checkable param (Int/Float/Str/Typed/Bytes/List/Tuple/Dict) and (b) have no
        // star param (positional alignment past `*args` is uncertain).
        // Unknown/None params are skipped, while Bytes/MemoryView/Complex/List/Tuple/Dict
        // are negative scalar walls that reject impossible concrete scalars and leave
        // dynamic/buffer/object values as Any-skips.
        // This is what lets a scalar param sitting BEHIND an Unknown param
        // enforce at its real position; a full uncapped 28k-fixture ② FP scan
        // confirms 0 false positives from the skipped params. (Earlier the
        // generator truncated enforceable rows to their leading scalar prefix to
        // satisfy a stricter all-scalar invariant.)
        use super::super::stdlib_sigs_generated::STDLIB_SIGS_GENERATED;
        for s in STDLIB_SIGS_GENERATED.iter().filter(|s| s.enforceable) {
            assert!(
                !s.params.is_empty(),
                "{}.{} enforceable but no params",
                s.module,
                s.name
            );
            assert!(
                s.params.iter().any(|p| matches!(
                    p.ty,
                    CoreTy::Int
                        | CoreTy::Float
                        | CoreTy::Str
                        | CoreTy::Typed
                        | CoreTy::Bytes
                        | CoreTy::MemoryView
                        | CoreTy::Complex
                        | CoreTy::List
                        | CoreTy::Tuple
                        | CoreTy::Dict
                )),
                "{}.{} enforceable with no checkable (scalar/Typed) param",
                s.module,
                s.name,
            );
            for prm in s.params {
                assert!(
                    !prm.star,
                    "{}.{} enforceable with a star param",
                    s.module, s.name
                );
            }
        }
    }

    #[test]
    fn curated_overrides_generated() {
        // Curated rows win over any generated row of the same key, and the
        // generated table is reachable on a curated miss.
        let s = get("os", "", "strerror").unwrap();
        assert!(
            s.enforceable,
            "curated os.strerror override must stay enforceable"
        );
        // A purely generated lookup (not in the curated 6) must resolve.
        assert!(
            super::super::stdlib_sigs_generated::STDLIB_SIGS_GENERATED
                .iter()
                .any(|s| s.module == "os" && s.qualifier.is_empty()),
            "generated table should contain os module fns",
        );
    }

    #[test]
    fn curated_warnings_message_wall_overrides_unknown_generated_rows() {
        for name in ["warn", "warn_explicit"] {
            let sig = get("_warnings", "", name).expect("_warnings row present");
            assert!(sig.enforceable);
            assert_eq!(sig.params[0].name, "message");
            assert_eq!(sig.params[0].ty, CoreTy::Str);
        }
    }

    #[test]
    fn curated_weakrefset_constructor_wall_overrides_unknown_generated_row() {
        let sig = get("_weakrefset", "WeakSet", "__init__").expect("WeakSet.__init__ present");
        assert!(sig.enforceable);
        assert_eq!(sig.params[0].name, "data");
        assert_eq!(sig.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_chainmap_walls_override_unknown_generated_rows() {
        for (name, first_param) in [
            ("__delitem__", "key"),
            ("__getitem__", "key"),
            ("__ior__", "other"),
            ("__missing__", "key"),
            ("__or__", "other"),
            ("__ror__", "other"),
            ("__setitem__", "key"),
            ("fromkeys", "iterable"),
            ("get", "key"),
            ("new_child", "m"),
            ("pop", "key"),
            ("setdefault", "key"),
        ] {
            let sig = get("collections", "ChainMap", name).expect("ChainMap row present");
            assert!(sig.enforceable, "ChainMap.{name} must stay enforceable");
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }
    }

    #[test]
    fn curated_counter_walls_override_unknown_generated_rows() {
        for (name, first_param) in [
            ("__add__", "other"),
            ("__and__", "other"),
            ("__ge__", "other"),
            ("__gt__", "other"),
            ("__init__", "iterable"),
            ("__ixor__", "other"),
            ("__le__", "other"),
            ("__lt__", "other"),
            ("__missing__", "key"),
            ("__or__", "other"),
            ("__sub__", "other"),
            ("__xor__", "other"),
            ("subtract", "iterable"),
            ("update", "m"),
        ] {
            let sig = get("collections", "Counter", name).expect("Counter row present");
            assert!(sig.enforceable, "Counter.{name} must stay enforceable");
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }
    }

    #[test]
    fn curated_ordereddict_walls_override_unknown_generated_rows() {
        for (name, first_param, first_ty) in [
            ("__or__", "value", CoreTy::Dict),
            ("__ror__", "value", CoreTy::Dict),
            ("fromkeys", "iterable", CoreTy::Typed),
            ("move_to_end", "key", CoreTy::Typed),
            ("pop", "key", CoreTy::Typed),
            ("popitem", "last", CoreTy::Bool),
            ("setdefault", "key", CoreTy::Typed),
        ] {
            let sig = get("collections", "OrderedDict", name).expect("OrderedDict row present");
            assert!(sig.enforceable, "OrderedDict.{name} must stay enforceable");
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, first_ty);
        }
    }

    #[test]
    fn curated_userdict_walls_override_unknown_generated_rows() {
        for (name, first_param) in [
            ("__delitem__", "key"),
            ("__getitem__", "key"),
            ("__init__", "dict"),
            ("__ior__", "other"),
            ("__or__", "other"),
            ("__ror__", "other"),
            ("__setitem__", "key"),
            ("fromkeys", "iterable"),
            ("get", "key"),
        ] {
            let sig = get("collections", "UserDict", name).expect("UserDict row present");
            assert!(sig.enforceable, "UserDict.{name} must stay enforceable");
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }
    }

    #[test]
    fn curated_userlist_walls_override_unknown_generated_rows() {
        for (name, first_param) in [
            ("__delitem__", "i"),
            ("__ge__", "other"),
            ("__getitem__", "i"),
            ("__gt__", "other"),
            ("__init__", "initlist"),
            ("__le__", "other"),
            ("__lt__", "other"),
            ("__setitem__", "i"),
            ("append", "item"),
            ("count", "item"),
            ("index", "item"),
            ("remove", "item"),
        ] {
            let sig = get("collections", "UserList", name).expect("UserList row present");
            assert!(sig.enforceable, "UserList.{name} must stay enforceable");
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }
    }

    #[test]
    fn curated_userstring_walls_override_unknown_generated_rows() {
        for (name, first_param, first_ty) in [
            ("__getitem__", "index", CoreTy::Typed),
            ("center", "width", CoreTy::Int),
            ("endswith", "suffix", CoreTy::Typed),
            ("format_map", "mapping", CoreTy::Typed),
            ("ljust", "width", CoreTy::Int),
            ("rjust", "width", CoreTy::Int),
            ("splitlines", "keepends", CoreTy::Bool),
            ("startswith", "prefix", CoreTy::Typed),
        ] {
            let sig = get("collections", "UserString", name).expect("UserString row present");
            assert!(sig.enforceable, "UserString.{name} must stay enforceable");
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, first_ty);
        }
    }

    /// Regenerable contract (fixture_lint-style): the checked-in
    /// `stdlib_sigs_generated.rs` must be byte-for-byte reproducible by
    /// `type_wall_gen.py --emit-rust`. Skips gracefully if `python3.12` or the
    /// vendored typeshed is unavailable (CI without the harness toolchain).
    #[test]
    fn generated_table_is_regenerable() {
        use std::path::Path;
        use std::process::Command;
        let manifest = env!("CARGO_MANIFEST_DIR");
        let gen = Path::new(manifest).join("tests/harness/cpython/tools/type_wall_gen.py");
        let typeshed = Path::new(manifest).join("vendor/typeshed/stdlib");
        if !gen.exists() || !typeshed.exists() {
            eprintln!("skip: harness generator / typeshed not present");
            return;
        }
        let out = match Command::new("python3.12")
            .arg(&gen)
            .arg("--check-rust")
            .output()
        {
            Ok(o) => o,
            Err(_) => {
                eprintln!("skip: python3.12 not available");
                return;
            }
        };
        assert!(
            out.status.success(),
            "stdlib_sigs_generated.rs is stale — re-run \
             `python3.12 tests/harness/cpython/tools/type_wall_gen.py --emit-rust`.\n\
             stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
}
