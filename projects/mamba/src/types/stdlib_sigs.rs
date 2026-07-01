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
    /// A small explicit union used for stdlib overloads whose accepted scalar
    /// set is still false-positive-clean to enforce.
    IntOrStr,
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
    /// A type/class object contract. Reject bare user instances and concrete
    /// scalar values; keep real class-like expressions skip-safe until the type
    /// model can distinguish class objects from instances everywhere.
    Type,
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
    // POSITIVE: typeshed models compile_path(skip_curdir) as bool, but the
    // generated row collapses it to Typed and therefore skips a wrong concrete
    // string. Keep the remaining generated scalar walls intact.
    StdlibSig {
        module: "compileall",
        qualifier: "",
        name: "compile_path",
        kind: SigKind::ModuleFn,
        params: &[
            p("skip_curdir", CoreTy::Bool),
            p("maxlevels", CoreTy::Int),
            p("force", CoreTy::Typed),
            p("quiet", CoreTy::Int),
            p("legacy", CoreTy::Typed),
            p("optimize", CoreTy::Int),
            p("invalidation_mode", CoreTy::Typed),
        ],
        enforceable: true,
    },
    // POSITIVE: concurrent.futures executor `max_workers` is `int | None`.
    // The generated rows either collapse it to Unknown or mark the row
    // unenforceable; a bare user instance cannot satisfy the integer branch,
    // while checker policy keeps `None` skip-safe for optional sentinels.
    StdlibSig {
        module: "concurrent.futures.interpreter",
        qualifier: "InterpreterPoolExecutor",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("max_workers", CoreTy::Int),
            p("thread_name_prefix", CoreTy::Str),
            p("initializer", CoreTy::Unknown),
            p("initargs", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "concurrent.futures.interpreter",
        qualifier: "InterpreterPoolExecutor",
        name: "prepare_context",
        kind: SigKind::Method,
        params: &[
            p("initializer", CoreTy::Typed),
            p("initargs", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "concurrent.futures.interpreter",
        qualifier: "WorkerContext",
        name: "prepare",
        kind: SigKind::Method,
        params: &[
            p("initializer", CoreTy::Typed),
            p("initargs", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "concurrent.futures.process",
        qualifier: "ProcessPoolExecutor",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("max_workers", CoreTy::Int),
            p("mp_context", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "concurrent.futures.thread",
        qualifier: "ThreadPoolExecutor",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("max_workers", CoreTy::Int),
            p("thread_name_prefix", CoreTy::Str),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "concurrent.futures.thread",
        qualifier: "ThreadPoolExecutor",
        name: "prepare_context",
        kind: SigKind::Method,
        params: &[
            p("initializer", CoreTy::Typed),
            p("initargs", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "concurrent.futures.thread",
        qualifier: "WorkerContext",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("initializer", CoreTy::Typed),
            p("initargs", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "concurrent.futures.thread",
        qualifier: "WorkerContext",
        name: "prepare",
        kind: SigKind::Method,
        params: &[
            p("initializer", CoreTy::Typed),
            p("initargs", CoreTy::Unknown),
        ],
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
    // POSITIVE: defaultdict typevar/default-factory and merge operands collapse
    // to Unknown or an empty generated row. The fixtures probe bare user objects
    // and scalar non-dicts, so these rows restore strict walls while leaving the
    // full mapping/callable overload matrix to future protocol modeling.
    StdlibSig {
        module: "collections",
        qualifier: "defaultdict",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("default_factory", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "defaultdict",
        name: "__missing__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "defaultdict",
        name: "__or__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Dict)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "defaultdict",
        name: "__ror__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Dict)],
        enforceable: true,
    },
    // POSITIVE: deque's typevar, Self, nominal-deque, and iterable contracts
    // collapse to Unknown/empty generated rows. The strict fixtures use bare
    // user objects, which satisfy none of those contracts, so Typed restores the
    // wall without pretending the scalar table can model deque generics.
    StdlibSig {
        module: "collections",
        qualifier: "deque",
        name: "__add__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "deque",
        name: "__ge__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "deque",
        name: "__gt__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "deque",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("iterable", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "deque",
        name: "__le__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "deque",
        name: "__lt__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "deque",
        name: "append",
        kind: SigKind::Method,
        params: &[p("x", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "deque",
        name: "appendleft",
        kind: SigKind::Method,
        params: &[p("x", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "deque",
        name: "count",
        kind: SigKind::Method,
        params: &[p("x", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "deque",
        name: "index",
        kind: SigKind::Method,
        params: &[
            p("x", CoreTy::Typed),
            p("start", CoreTy::Int),
            p("stop", CoreTy::Int),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "collections",
        qualifier: "deque",
        name: "remove",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
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
    StdlibSig {
        module: "ast",
        qualifier: "arguments",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("posonlyargs", CoreTy::List)],
        enforceable: true,
    },
    // POSITIVE: configparser's `_SectionName` alias is a string-shaped section
    // key in CPython's public API. The generated row keeps it Unknown, which
    // lets a bare user object fall through to runtime NoSectionError instead
    // of the force-typed TypeError required by the strict fixture.
    StdlibSig {
        module: "configparser",
        qualifier: "ConfigParser",
        name: "get",
        kind: SigKind::Method,
        params: &[p("section", CoreTy::Str), p("option", CoreTy::Str)],
        enforceable: true,
    },
    // POSITIVE: the RawConfigParser generated rows keep several protocol-ish
    // arguments Unknown. For force typing, a bare user object cannot satisfy the
    // public section-name, mapping, iterable, or path-like contracts; reject it
    // before the runtime falls through to NoSectionError/AttributeError.
    StdlibSig {
        module: "configparser",
        qualifier: "RawConfigParser",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("defaults", CoreTy::Typed),
            p("dict_type", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "configparser",
        qualifier: "RawConfigParser",
        name: "get",
        kind: SigKind::Method,
        params: &[p("section", CoreTy::Str), p("option", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "configparser",
        qualifier: "RawConfigParser",
        name: "getboolean",
        kind: SigKind::Method,
        params: &[p("section", CoreTy::Str), p("option", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "configparser",
        qualifier: "RawConfigParser",
        name: "getfloat",
        kind: SigKind::Method,
        params: &[p("section", CoreTy::Str), p("option", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "configparser",
        qualifier: "RawConfigParser",
        name: "getint",
        kind: SigKind::Method,
        params: &[p("section", CoreTy::Str), p("option", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "configparser",
        qualifier: "RawConfigParser",
        name: "items",
        kind: SigKind::Method,
        params: &[p("section", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "configparser",
        qualifier: "RawConfigParser",
        name: "read",
        kind: SigKind::Method,
        params: &[
            p("filenames", CoreTy::Typed),
            p("encoding", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "configparser",
        qualifier: "RawConfigParser",
        name: "read_dict",
        kind: SigKind::Method,
        params: &[p("dictionary", CoreTy::Typed), p("source", CoreTy::Str)],
        enforceable: true,
    },
    // POSITIVE: contextlib's decorator/context-manager contracts include
    // Callable, path-like, type-variable, and exception-type shapes that the
    // generated scalar table collapses to Unknown. In strict mode, a bare user
    // object cannot satisfy those contracts, while real callable/path/exception
    // objects stay skip-safe through the Typed wall.
    StdlibSig {
        module: "contextlib",
        qualifier: "",
        name: "asynccontextmanager",
        kind: SigKind::ModuleFn,
        params: &[p("func", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "contextlib",
        qualifier: "",
        name: "contextmanager",
        kind: SigKind::ModuleFn,
        params: &[p("func", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "contextlib",
        qualifier: "AbstractAsyncContextManager",
        name: "__aexit__",
        kind: SigKind::Method,
        params: &[
            p("exc_type", CoreTy::Typed),
            p("exc_value", CoreTy::Typed),
            p("traceback", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "contextlib",
        qualifier: "AbstractContextManager",
        name: "__exit__",
        kind: SigKind::Method,
        params: &[
            p("exc_type", CoreTy::Typed),
            p("exc_value", CoreTy::Typed),
            p("traceback", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "contextlib",
        qualifier: "chdir",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("path", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "contextlib",
        qualifier: "nullcontext",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("enter_result", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: copy/copyreg expose TypeVar, Hashable, Callable, and type
    // contracts that are not scalar rows in the generated table. A bare user
    // object cannot satisfy those contracts in strict mode; valid callables,
    // types, hashable builtins, and richer protocol objects remain skip-safe.
    StdlibSig {
        module: "copy",
        qualifier: "",
        name: "copy",
        kind: SigKind::ModuleFn,
        params: &[p("x", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "copy",
        qualifier: "",
        name: "deepcopy",
        kind: SigKind::ModuleFn,
        params: &[p("x", CoreTy::Typed), p("memo", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "copyreg",
        qualifier: "",
        name: "add_extension",
        kind: SigKind::ModuleFn,
        params: &[
            p("module", CoreTy::Typed),
            p("name", CoreTy::Unknown),
            p("code", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "copyreg",
        qualifier: "",
        name: "constructor",
        kind: SigKind::ModuleFn,
        params: &[p("object", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "copyreg",
        qualifier: "",
        name: "pickle",
        kind: SigKind::ModuleFn,
        params: &[
            p("ob_type", CoreTy::Typed),
            p("pickle_function", CoreTy::Unknown),
            p("constructor_ob", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "copyreg",
        qualifier: "",
        name: "remove_extension",
        kind: SigKind::ModuleFn,
        params: &[
            p("module", CoreTy::Typed),
            p("name", CoreTy::Unknown),
            p("code", CoreTy::Int),
        ],
        enforceable: true,
    },
    // POSITIVE: csv.DictReader consumes an iterable row source. The generated
    // row collapses the iterable protocol to Unknown; a bare user object has no
    // iterator protocol and should be rejected by the strict wall.
    StdlibSig {
        module: "csv",
        qualifier: "DictReader",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("f", CoreTy::Typed),
            p("fieldnames", CoreTy::Unknown),
            p("restkey", CoreTy::Unknown),
            p("restval", CoreTy::Unknown),
            p("dialect", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    // POSITIVE: curses.wrapper(func: Callable, *args) collapses to an
    // unenforceable generated row because Callable is non-scalar and the row is
    // variadic. The strict fixture probes only a bare user instance for `func`;
    // reject that impossible Callable while keeping `*args` skip-safe.
    StdlibSig {
        module: "curses",
        qualifier: "",
        name: "wrapper",
        kind: SigKind::ModuleFn,
        params: &[
            p("func", CoreTy::Typed),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    // POSITIVE: curses.ascii's `_CharT` helpers are int-or-1-char-string
    // contracts. A bare user instance can satisfy neither branch, while real
    // scalar and dynamic values remain handled by the runtime behavior path.
    StdlibSig {
        module: "curses.ascii",
        qualifier: "",
        name: "alt",
        kind: SigKind::ModuleFn,
        params: &[p("c", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "curses.ascii",
        qualifier: "",
        name: "ascii",
        kind: SigKind::ModuleFn,
        params: &[p("c", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "curses.ascii",
        qualifier: "",
        name: "ctrl",
        kind: SigKind::ModuleFn,
        params: &[p("c", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: Textbox.edit(validate: Callable | None) loses its callable
    // contract in the generated row. A bare `_W()` is neither callable nor the
    // `None` sentinel, so reject it without changing runtime Textbox behavior.
    StdlibSig {
        module: "curses.textpad",
        qualifier: "Textbox",
        name: "edit",
        kind: SigKind::Method,
        params: &[p("validate", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: dataclasses generated rows collapse dataclass-instance,
    // type-object, TypeVar, and overload-only contracts to Unknown. The mamba
    // strict wall rejects the inert `_W()` fixture shape while the checker
    // keeps real `@dataclass` instances out of the bare-instance reject path.
    StdlibSig {
        module: "dataclasses",
        qualifier: "",
        name: "asdict",
        kind: SigKind::ModuleFn,
        params: &[p("obj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "dataclasses",
        qualifier: "",
        name: "astuple",
        kind: SigKind::ModuleFn,
        params: &[p("obj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "dataclasses",
        qualifier: "",
        name: "dataclass",
        kind: SigKind::ModuleFn,
        params: &[p("cls", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "dataclasses",
        qualifier: "",
        name: "replace",
        kind: SigKind::ModuleFn,
        params: &[p("obj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "dataclasses",
        qualifier: "Field",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("default", CoreTy::Typed),
            p("default_factory", CoreTy::Unknown),
            p("init", CoreTy::Typed),
            p("repr", CoreTy::Typed),
            p("hash", CoreTy::Typed),
            p("compare", CoreTy::Typed),
            p("metadata", CoreTy::Unknown),
            p("kw_only", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "dataclasses",
        qualifier: "Field",
        name: "__set_name__",
        kind: SigKind::Method,
        params: &[p("owner", CoreTy::Type), p("name", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "dataclasses",
        qualifier: "InitVar",
        name: "__class_getitem__",
        kind: SigKind::Method,
        params: &[p("type", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "dataclasses",
        qualifier: "InitVar",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("type", CoreTy::Type)],
        enforceable: true,
    },
    // POSITIVE: datetime date/datetime subtraction accepts only compatible
    // date/datetime/timedelta operands. The generated overload union collapses
    // to Unknown; reject inert strict-wall objects while keeping real stdlib
    // operands skip-safe.
    StdlibSig {
        module: "datetime",
        qualifier: "date",
        name: "__sub__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "datetime",
        qualifier: "datetime",
        name: "__sub__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: decimal.Decimal.__round__ has a no-argument overload and an
    // `ndigits: SupportsIndex` overload. The generated overload row collapses
    // to an empty unenforceable signature; keep zero-arg calls skip-safe while
    // rejecting concrete non-index scalars when ndigits is present.
    StdlibSig {
        module: "decimal",
        qualifier: "Decimal",
        name: "__round__",
        kind: SigKind::Method,
        params: &[p("ndigits", CoreTy::Int)],
        enforceable: true,
    },
    // POSITIVE: difflib callbacks/sequences are protocol/callable-heavy in
    // typeshed, so generated rows collapse them to Unknown or an empty
    // unenforceable constructor row. The strict fixtures probe bare `_W()`
    // values, which satisfy neither callable nor sequence contracts.
    StdlibSig {
        module: "difflib",
        qualifier: "Differ",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("linejunk", CoreTy::Typed),
            p("charjunk", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "difflib",
        qualifier: "SequenceMatcher",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("isjunk", CoreTy::Typed),
            p("a", CoreTy::Typed),
            p("b", CoreTy::Typed),
            p("autojunk", CoreTy::Bool),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "difflib",
        qualifier: "",
        name: "diff_bytes",
        kind: SigKind::ModuleFn,
        params: &[
            p("dfunc", CoreTy::Typed),
            p("a", CoreTy::Typed),
            p("b", CoreTy::Typed),
            p("fromfile", CoreTy::Typed),
            p("tofile", CoreTy::Typed),
            p("fromfiledate", CoreTy::Typed),
            p("tofiledate", CoreTy::Typed),
            p("n", CoreTy::Int),
            p("lineterm", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "difflib",
        qualifier: "",
        name: "ndiff",
        kind: SigKind::ModuleFn,
        params: &[
            p("a", CoreTy::Typed),
            p("b", CoreTy::Typed),
            p("linejunk", CoreTy::Typed),
            p("charjunk", CoreTy::Typed),
        ],
        enforceable: true,
    },
    // POSITIVE: dis accepts code/function/code-like inputs represented by
    // private protocols in typeshed. Generated rows collapse those to Unknown,
    // so bare `_W()` values currently leak through strict fixtures.
    StdlibSig {
        module: "dis",
        qualifier: "",
        name: "code_info",
        kind: SigKind::ModuleFn,
        params: &[p("x", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "dis",
        qualifier: "",
        name: "dis",
        kind: SigKind::ModuleFn,
        params: &[p("x", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "dis",
        qualifier: "",
        name: "disassemble",
        kind: SigKind::ModuleFn,
        params: &[p("co", CoreTy::Typed), p("lasti", CoreTy::Int)],
        enforceable: true,
    },
    StdlibSig {
        module: "dis",
        qualifier: "",
        name: "findlabels",
        kind: SigKind::ModuleFn,
        params: &[p("code", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "dis",
        qualifier: "",
        name: "findlinestarts",
        kind: SigKind::ModuleFn,
        params: &[p("code", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "dis",
        qualifier: "",
        name: "get_instructions",
        kind: SigKind::ModuleFn,
        params: &[p("x", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "dis",
        qualifier: "",
        name: "show_code",
        kind: SigKind::ModuleFn,
        params: &[p("co", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "dis",
        qualifier: "Bytecode",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("x", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: typeshed exposes distutils.archive_util.make_archive through a
    // path-like alias that the generator collapses to Unknown. Use a str wall
    // for scalar rejection; non-scalar path-like objects stay skip-safe.
    StdlibSig {
        module: "distutils.archive_util",
        qualifier: "",
        name: "make_archive",
        kind: SigKind::ModuleFn,
        params: &[
            p("base_name", CoreTy::Str),
            p("format", CoreTy::Str),
            p("root_dir", CoreTy::Typed),
            p("base_dir", CoreTy::Typed),
            p("verbose", CoreTy::Typed),
            p("dry_run", CoreTy::Typed),
            p("owner", CoreTy::Typed),
            p("group", CoreTy::Typed),
        ],
        enforceable: true,
    },
    // POSITIVE: distutils.ccompiler has long-tail aliases and optional flags
    // that generated rows collapse to Unknown. Curate the first strict walls
    // needed by the corpus while preserving skip-safe Typed params elsewhere.
    StdlibSig {
        module: "distutils.ccompiler",
        qualifier: "",
        name: "gen_preprocess_options",
        kind: SigKind::ModuleFn,
        params: &[p("macros", CoreTy::List), p("include_dirs", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.ccompiler",
        qualifier: "CCompiler",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("verbose", CoreTy::Typed),
            p("dry_run", CoreTy::Typed),
            p("force", CoreTy::Typed),
        ],
        enforceable: true,
    },
    // POSITIVE: distutils.command submodules are Py312-deprecated but still
    // importable, and typeshed preserves strict argument walls for command
    // helpers. Generated rows lose the target params to Unknown; curate only
    // the probed walls so valid dynamic command state stays skip-safe.
    StdlibSig {
        module: "distutils.command.build_py",
        qualifier: "build_py",
        name: "get_outputs",
        kind: SigKind::Method,
        params: &[p("include_bytecode", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.command.check",
        qualifier: "SilentReporter",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("source", CoreTy::Typed),
            p("report_level", CoreTy::Typed),
            p("halt_level", CoreTy::Typed),
            p("stream", CoreTy::Typed),
            p("debug", CoreTy::Typed),
            p("encoding", CoreTy::Str),
            p("error_handler", CoreTy::Str),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.command.config",
        qualifier: "config",
        name: "search_cpp",
        kind: SigKind::Method,
        params: &[
            p("pattern", CoreTy::Typed),
            p("body", CoreTy::Typed),
            p("headers", CoreTy::Typed),
            p("include_dirs", CoreTy::Typed),
            p("lang", CoreTy::Str),
        ],
        enforceable: true,
    },
    // POSITIVE: Distribution(attrs) accepts a mapping-shaped attrs object in
    // CPython's distutils contract. A bare user instance is not a mapping and
    // must not leak through the force-typed strict fixture.
    StdlibSig {
        module: "distutils.dist",
        qualifier: "Distribution",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("attrs", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: distutils.file_util accepts str/bytes/path-like sources. The
    // generator sees only Unknown; reject bare user instances while keeping
    // real non-scalar path-like values skip-safe.
    StdlibSig {
        module: "distutils.file_util",
        qualifier: "",
        name: "copy_file",
        kind: SigKind::ModuleFn,
        params: &[p("src", CoreTy::Typed), p("dst", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.file_util",
        qualifier: "",
        name: "move_file",
        kind: SigKind::ModuleFn,
        params: &[p("src", CoreTy::Typed), p("dst", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: filelist pattern helpers are string-pattern boundaries in
    // distutils. Generated rows collapse translate/include/exclude to Unknown;
    // keep strict mode from leaking impossible scalar or bare-object patterns
    // into runtime import/setup errors.
    StdlibSig {
        module: "distutils.filelist",
        qualifier: "",
        name: "translate_pattern",
        kind: SigKind::ModuleFn,
        params: &[p("pattern", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.filelist",
        qualifier: "FileList",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("warn", CoreTy::Typed), p("debug_print", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.filelist",
        qualifier: "FileList",
        name: "exclude_pattern",
        kind: SigKind::Method,
        params: &[p("pattern", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.filelist",
        qualifier: "FileList",
        name: "include_pattern",
        kind: SigKind::Method,
        params: &[p("pattern", CoreTy::Str)],
        enforceable: true,
    },
    // POSITIVE: distutils.log formatting helpers are variadic, which makes the
    // generated rows skip wholesale. Enforce the required `msg`/`level` prefix
    // parameters while preserving the `*args` boundary as Unknown.
    StdlibSig {
        module: "distutils.log",
        qualifier: "",
        name: "debug",
        kind: SigKind::ModuleFn,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.log",
        qualifier: "",
        name: "error",
        kind: SigKind::ModuleFn,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.log",
        qualifier: "",
        name: "fatal",
        kind: SigKind::ModuleFn,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.log",
        qualifier: "",
        name: "info",
        kind: SigKind::ModuleFn,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.log",
        qualifier: "",
        name: "log",
        kind: SigKind::ModuleFn,
        params: &[
            p("level", CoreTy::Int),
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.log",
        qualifier: "",
        name: "warn",
        kind: SigKind::ModuleFn,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.log",
        qualifier: "Log",
        name: "debug",
        kind: SigKind::Method,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.log",
        qualifier: "Log",
        name: "error",
        kind: SigKind::Method,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.log",
        qualifier: "Log",
        name: "fatal",
        kind: SigKind::Method,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.log",
        qualifier: "Log",
        name: "info",
        kind: SigKind::Method,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.log",
        qualifier: "Log",
        name: "log",
        kind: SigKind::Method,
        params: &[
            p("level", CoreTy::Int),
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.log",
        qualifier: "Log",
        name: "warn",
        kind: SigKind::Method,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    // POSITIVE: distutils.sysconfig generated rows lose Literal/string and
    // bool-ish flag walls. Restore the required prefix checks while leaving
    // optional prefix/path-like values skip-safe through Typed.
    StdlibSig {
        module: "distutils.sysconfig",
        qualifier: "",
        name: "get_config_var",
        kind: SigKind::ModuleFn,
        params: &[p("name", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.sysconfig",
        qualifier: "",
        name: "get_config_vars",
        kind: SigKind::ModuleFn,
        params: &[p("arg", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.sysconfig",
        qualifier: "",
        name: "get_python_inc",
        kind: SigKind::ModuleFn,
        params: &[p("plat_specific", CoreTy::Typed), p("prefix", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.sysconfig",
        qualifier: "",
        name: "get_python_lib",
        kind: SigKind::ModuleFn,
        params: &[
            p("plat_specific", CoreTy::Typed),
            p("standard_lib", CoreTy::Typed),
            p("prefix", CoreTy::Typed),
        ],
        enforceable: true,
    },
    // POSITIVE: distutils.util generated rows lose the concrete list/callable
    // prefix walls. Use Typed for Callable until the PoC table has a dedicated
    // callable core type; it rejects the bare user object used by the fixture.
    StdlibSig {
        module: "distutils.util",
        qualifier: "",
        name: "byte_compile",
        kind: SigKind::ModuleFn,
        params: &[p("py_files", CoreTy::List)],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.util",
        qualifier: "",
        name: "execute",
        kind: SigKind::ModuleFn,
        params: &[p("func", CoreTy::Typed), p("args", CoreTy::Unknown)],
        enforceable: true,
    },
    // POSITIVE: doctest generated rows collapse list/dict/bool constructor and
    // helper walls. Keep unknown leading values skip-safe while enforcing the
    // concrete positional parameters that the CPython fixture corpus exercises.
    StdlibSig {
        module: "doctest",
        qualifier: "",
        name: "run_docstring_examples",
        kind: SigKind::ModuleFn,
        params: &[p("f", CoreTy::Unknown), p("globs", CoreTy::Dict)],
        enforceable: true,
    },
    StdlibSig {
        module: "doctest",
        qualifier: "DocTest",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("examples", CoreTy::List)],
        enforceable: true,
    },
    StdlibSig {
        module: "doctest",
        qualifier: "DocTestFinder",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("verbose", CoreTy::Bool)],
        enforceable: true,
    },
    // POSITIVE: email top-level parser helpers take IO/bytes-like protocol
    // values. Generated rows collapse these to Unknown; Typed rejects the bare
    // user object probes while staying skip-safe for real file/bytes-like values.
    StdlibSig {
        module: "email",
        qualifier: "",
        name: "message_from_binary_file",
        kind: SigKind::ModuleFn,
        params: &[p("fp", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email",
        qualifier: "",
        name: "message_from_bytes",
        kind: SigKind::ModuleFn,
        params: &[p("s", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email",
        qualifier: "",
        name: "message_from_file",
        kind: SigKind::ModuleFn,
        params: &[p("fp", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.charset",
        qualifier: "Charset",
        name: "body_encode",
        kind: SigKind::Method,
        params: &[p("string", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.feedparser",
        qualifier: "BytesFeedParser",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("_factory", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.feedparser",
        qualifier: "FeedParser",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("_factory", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.generator",
        qualifier: "BytesGenerator",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("outfp", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.generator",
        qualifier: "DecodedGenerator",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("outfp", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.generator",
        qualifier: "Generator",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("outfp", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.headerregistry",
        qualifier: "HeaderRegistry",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("base_class", CoreTy::Type),
            p("default_class", CoreTy::Unknown),
            p("use_default_map", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "email.iterators",
        qualifier: "",
        name: "typed_subpart_iterator",
        kind: SigKind::ModuleFn,
        params: &[
            p("msg", CoreTy::Typed),
            p("maintype", CoreTy::Str),
            p("subtype", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "MIMEPart",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("policy", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "MIMEPart",
        name: "attach",
        kind: SigKind::Method,
        params: &[p("payload", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "MIMEPart",
        name: "get_body",
        kind: SigKind::Method,
        params: &[p("preferencelist", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "MIMEPart",
        name: "as_string",
        kind: SigKind::Method,
        params: &[
            p("unixfrom", CoreTy::Bool),
            p("maxheaderlen", CoreTy::Typed),
            p("policy", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "Message",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("policy", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "Message",
        name: "as_bytes",
        kind: SigKind::Method,
        params: &[
            p("unixfrom", CoreTy::Bool),
            p("policy", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "Message",
        name: "set_payload",
        kind: SigKind::Method,
        params: &[
            p("payload", CoreTy::Typed),
            p("charset", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "Message",
        name: "get_payload",
        kind: SigKind::Method,
        params: &[
            p("i", CoreTy::Int),
            p("decode", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "Message",
        name: "get_boundary",
        kind: SigKind::Method,
        params: &[p("failobj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "Message",
        name: "get_charsets",
        kind: SigKind::Method,
        params: &[p("failobj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "Message",
        name: "get_content_charset",
        kind: SigKind::Method,
        params: &[p("failobj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "Message",
        name: "get_filename",
        kind: SigKind::Method,
        params: &[p("failobj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "Message",
        name: "get_params",
        kind: SigKind::Method,
        params: &[p("failobj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.message",
        qualifier: "Message",
        name: "as_string",
        kind: SigKind::Method,
        params: &[
            p("unixfrom", CoreTy::Bool),
            p("maxheaderlen", CoreTy::Int),
            p("policy", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "email.mime.message",
        qualifier: "MIMEMessage",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("_msg", CoreTy::Typed),
            p("_subtype", CoreTy::Str),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "email.policy",
        qualifier: "EmailPolicy",
        name: "header_source_parse",
        kind: SigKind::Method,
        params: &[p("sourcelines", CoreTy::List)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.utils",
        qualifier: "",
        name: "decode_params",
        kind: SigKind::ModuleFn,
        params: &[p("params", CoreTy::List)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.utils",
        qualifier: "",
        name: "formataddr",
        kind: SigKind::ModuleFn,
        params: &[
            p("pair", CoreTy::Tuple),
            p("charset", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "email.utils",
        qualifier: "",
        name: "localtime",
        kind: SigKind::ModuleFn,
        params: &[p("dt", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.utils",
        qualifier: "",
        name: "parseaddr",
        kind: SigKind::ModuleFn,
        params: &[p("addr", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.utils",
        qualifier: "",
        name: "parsedate",
        kind: SigKind::ModuleFn,
        params: &[p("data", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.utils",
        qualifier: "",
        name: "parsedate_tz",
        kind: SigKind::ModuleFn,
        params: &[p("data", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.parser",
        qualifier: "BytesParser",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("_class", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "email.parser",
        qualifier: "Parser",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("_class", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: enum's public typeshed surface uses several Self/typevar/
    // decorator contracts that the generated table keeps Unknown-skipped. These
    // rows restore fixture-backed force-typed walls while rejecting only
    // provably wrong scalars or bare `_W()` instances.
    StdlibSig {
        module: "enum",
        qualifier: "EnumMeta",
        name: "__call__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Str), p("names", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "enum",
        qualifier: "Enum",
        name: "_generate_next_value_",
        kind: SigKind::Method,
        params: &[
            p("name", CoreTy::Str),
            p("start", CoreTy::Int),
            p("count", CoreTy::Int),
            p("last_values", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "enum",
        qualifier: "StrEnum",
        name: "_generate_next_value_",
        kind: SigKind::Method,
        params: &[
            p("name", CoreTy::Str),
            p("start", CoreTy::Int),
            p("count", CoreTy::Int),
            p("last_values", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "enum",
        qualifier: "Flag",
        name: "__and__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "enum",
        qualifier: "Flag",
        name: "__contains__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "enum",
        qualifier: "Flag",
        name: "__or__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "enum",
        qualifier: "Flag",
        name: "__xor__",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "enum",
        qualifier: "",
        name: "global_enum",
        kind: SigKind::ModuleFn,
        params: &[p("cls", CoreTy::Typed), p("update_str", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "enum",
        qualifier: "member",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "enum",
        qualifier: "nonmember",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "enum",
        qualifier: "",
        name: "pickle_by_enum_name",
        kind: SigKind::ModuleFn,
        params: &[p("self", CoreTy::Typed), p("proto", CoreTy::Int)],
        enforceable: true,
    },
    StdlibSig {
        module: "enum",
        qualifier: "",
        name: "unique",
        kind: SigKind::ModuleFn,
        params: &[p("enumeration", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: fancy_getopt uses list/sequence-shaped option tables and arg
    // lists. Generated rows collapse the key parameters to Unknown; curate the
    // strict walls that prove bare objects/scalars cannot cross this boundary.
    StdlibSig {
        module: "distutils.fancy_getopt",
        qualifier: "",
        name: "fancy_getopt",
        kind: SigKind::ModuleFn,
        params: &[p("options", CoreTy::List)],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.fancy_getopt",
        qualifier: "FancyGetopt",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("option_table", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "distutils.fancy_getopt",
        qualifier: "FancyGetopt",
        name: "getopt",
        kind: SigKind::Method,
        params: &[p("args", CoreTy::Typed), p("object", CoreTy::Typed)],
        enforceable: true,
    },
    // POSITIVE: ctypes public factory helpers take ctypes type/class-like
    // values that generated rows collapse to Unknown or mark unenforceable. A
    // bare user instance and impossible concrete scalar cannot satisfy those
    // contracts, while real ctypes classes and `None` sentinels stay skip-safe.
    StdlibSig {
        module: "ctypes",
        qualifier: "",
        name: "ARRAY",
        kind: SigKind::ModuleFn,
        params: &[p("typ", CoreTy::Type), p("len", CoreTy::Int)],
        enforceable: true,
    },
    StdlibSig {
        module: "ctypes",
        qualifier: "",
        name: "CFUNCTYPE",
        kind: SigKind::ModuleFn,
        params: &[p("restype", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "ctypes",
        qualifier: "LibraryLoader",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("dlltype", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "ctypes",
        qualifier: "",
        name: "POINTER",
        kind: SigKind::ModuleFn,
        params: &[p("cls", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "ctypes",
        qualifier: "",
        name: "PYFUNCTYPE",
        kind: SigKind::ModuleFn,
        params: &[p("restype", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "ctypes",
        qualifier: "",
        name: "SetPointerType",
        kind: SigKind::ModuleFn,
        params: &[p("pointer", CoreTy::Type), p("cls", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "ctypes",
        qualifier: "",
        name: "WINFUNCTYPE",
        kind: SigKind::ModuleFn,
        params: &[p("restype", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "ctypes",
        qualifier: "",
        name: "pointer",
        kind: SigKind::ModuleFn,
        params: &[p("obj", CoreTy::Typed)],
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
    // POSITIVE: locale has several generated rows weakened by aliases,
    // deprecated helpers, or callable stubs. Keep strict fixtures in front of
    // those stubs without changing the broader runtime surface yet.
    StdlibSig {
        module: "locale",
        qualifier: "",
        name: "format",
        kind: SigKind::ModuleFn,
        params: &[p("percent", CoreTy::Typed), p("value", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "locale",
        qualifier: "",
        name: "getdefaultlocale",
        kind: SigKind::ModuleFn,
        params: &[p("envvars", CoreTy::Tuple)],
        enforceable: true,
    },
    StdlibSig {
        module: "locale",
        qualifier: "",
        name: "getpreferredencoding",
        kind: SigKind::ModuleFn,
        params: &[p("do_setlocale", CoreTy::Bool)],
        enforceable: true,
    },
    // POSITIVE: logging has a large alias/protocol-heavy surface where the
    // generated table correctly collapses many rows to Unknown. The strict wall
    // fixtures below are still enforceable with conservative single-parameter
    // walls: scalar bool/int contracts stay scalar, while Sequence/Mapping/
    // Callable/typevar contracts reject only bare invalid user objects.
    StdlibSig {
        module: "logging",
        qualifier: "",
        name: "captureWarnings",
        kind: SigKind::ModuleFn,
        params: &[p("capture", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "",
        name: "getLevelName",
        kind: SigKind::ModuleFn,
        params: &[p("level", CoreTy::IntOrStr)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "",
        name: "log",
        kind: SigKind::ModuleFn,
        params: &[
            p("level", CoreTy::Int),
            p("msg", CoreTy::Unknown),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "",
        name: "makeLogRecord",
        kind: SigKind::ModuleFn,
        params: &[p("dict", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "",
        name: "setLogRecordFactory",
        kind: SigKind::ModuleFn,
        params: &[p("factory", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "",
        name: "shutdown",
        kind: SigKind::ModuleFn,
        params: &[p("handlerList", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "BufferingFormatter",
        name: "format",
        kind: SigKind::Method,
        params: &[p("records", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "BufferingFormatter",
        name: "formatFooter",
        kind: SigKind::Method,
        params: &[p("records", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "BufferingFormatter",
        name: "formatHeader",
        kind: SigKind::Method,
        params: &[p("records", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "Logger",
        name: "findCaller",
        kind: SigKind::Method,
        params: &[p("stack_info", CoreTy::Bool), p("stacklevel", CoreTy::Int)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "Logger",
        name: "log",
        kind: SigKind::Method,
        params: &[
            p("level", CoreTy::Int),
            p("msg", CoreTy::Unknown),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "LoggerAdapter",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("logger", CoreTy::Typed), p("extra", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "LoggerAdapter",
        name: "log",
        kind: SigKind::Method,
        params: &[
            p("level", CoreTy::Int),
            p("msg", CoreTy::Unknown),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "LoggerAdapter",
        name: "process",
        kind: SigKind::Method,
        params: &[p("msg", CoreTy::Unknown), p("kwargs", CoreTy::Dict)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "Manager",
        name: "setLogRecordFactory",
        kind: SigKind::Method,
        params: &[p("factory", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "StreamHandler",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("stream", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging",
        qualifier: "StreamHandler",
        name: "setStream",
        kind: SigKind::Method,
        params: &[p("stream", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging.config",
        qualifier: "",
        name: "dictConfig",
        kind: SigKind::ModuleFn,
        params: &[p("config", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging.config",
        qualifier: "BaseConfigurator",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("config", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging.config",
        qualifier: "BaseConfigurator",
        name: "as_tuple",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging.config",
        qualifier: "BaseConfigurator",
        name: "configure_custom",
        kind: SigKind::Method,
        params: &[p("config", CoreTy::Dict)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging.config",
        qualifier: "ConvertingDict",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging.config",
        qualifier: "ConvertingDict",
        name: "get",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed), p("default", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging.config",
        qualifier: "ConvertingDict",
        name: "pop",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed), p("default", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging.config",
        qualifier: "ConvertingList",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "logging.config",
        qualifier: "ConvertingMixin",
        name: "convert_with_key",
        kind: SigKind::Method,
        params: &[
            p("key", CoreTy::Unknown),
            p("value", CoreTy::Unknown),
            p("replace", CoreTy::Bool),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "logging.config",
        qualifier: "ConvertingTuple",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
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
    // ipaddress network constructors have overload-heavy address parameters,
    // but `strict` is always a bool wall in typeshed.
    StdlibSig {
        module: "ipaddress",
        qualifier: "IPv4Network",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("address", CoreTy::Unknown), p("strict", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "ipaddress",
        qualifier: "IPv6Network",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("address", CoreTy::Unknown), p("strict", CoreTy::Bool)],
        enforceable: true,
    },
    // ipaddress.get_mixed_type_key accepts IP address/network nominal types;
    // reject bare user instances in strict fixtures while keeping concrete
    // runtime values skip-safe.
    StdlibSig {
        module: "ipaddress",
        qualifier: "",
        name: "get_mixed_type_key",
        kind: SigKind::ModuleFn,
        params: &[p("obj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "ipaddress",
        qualifier: "",
        name: "ip_interface",
        kind: SigKind::ModuleFn,
        params: &[p("address", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "ipaddress",
        qualifier: "",
        name: "ip_network",
        kind: SigKind::ModuleFn,
        params: &[p("address", CoreTy::Typed), p("strict", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "ipaddress",
        qualifier: "",
        name: "summarize_address_range",
        kind: SigKind::ModuleFn,
        params: &[p("first", CoreTy::Typed), p("last", CoreTy::Unknown)],
        enforceable: true,
    },
    // importlib.metadata._meta.SimplePath.read_text has optional encoding in
    // typeshed, but the generated row collapses to no params.
    StdlibSig {
        module: "importlib.metadata._meta",
        qualifier: "SimplePath",
        name: "read_text",
        kind: SigKind::Method,
        params: &[p("encoding", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.readers",
        qualifier: "MultiplexedPath",
        name: "joinpath",
        kind: SigKind::Method,
        params: &[p("child", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.resources._common",
        qualifier: "",
        name: "files",
        kind: SigKind::ModuleFn,
        params: &[p("anchor", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.resources._common",
        qualifier: "",
        name: "package_to_anchor",
        kind: SigKind::ModuleFn,
        params: &[p("func", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.resources._functional",
        qualifier: "",
        name: "contents",
        kind: SigKind::ModuleFn,
        params: &[p("anchor", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.resources._functional",
        qualifier: "",
        name: "is_resource",
        kind: SigKind::ModuleFn,
        params: &[p("anchor", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.resources._functional",
        qualifier: "",
        name: "open_binary",
        kind: SigKind::ModuleFn,
        params: &[p("anchor", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.resources._functional",
        qualifier: "",
        name: "open_text",
        kind: SigKind::ModuleFn,
        params: &[p("anchor", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.resources._functional",
        qualifier: "",
        name: "path",
        kind: SigKind::ModuleFn,
        params: &[p("anchor", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.resources._functional",
        qualifier: "",
        name: "read_binary",
        kind: SigKind::ModuleFn,
        params: &[p("anchor", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.resources._functional",
        qualifier: "",
        name: "read_text",
        kind: SigKind::ModuleFn,
        params: &[p("anchor", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.resources.abc",
        qualifier: "Traversable",
        name: "open",
        kind: SigKind::Method,
        params: &[p("mode", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.resources.simple",
        qualifier: "ResourceHandle",
        name: "joinpath",
        kind: SigKind::Method,
        params: &[p("name", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.resources.simple",
        qualifier: "ResourceHandle",
        name: "open",
        kind: SigKind::Method,
        params: &[p("mode", CoreTy::Str)],
        enforceable: true,
    },
    // io.Writer.write is a contravariant typevar in typeshed, which generated
    // rows collapse to Unknown. Preserve the strict wall for bare invalid user
    // objects without over-specializing the accepted buffer/text shape yet.
    StdlibSig {
        module: "io",
        qualifier: "Writer",
        name: "write",
        kind: SigKind::Method,
        params: &[p("data", CoreTy::Typed)],
        enforceable: true,
    },
    // lib2to3 is mostly a runtime stub in mamba today, but its typeshed surface
    // still participates in strict type walls. Keep these generated-Unknown
    // rows enforceable so wrong-typed probes are rejected before ImportError or
    // partially stubbed runtime behavior is reached.
    StdlibSig {
        module: "lib2to3.fixer_base",
        qualifier: "BaseFix",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("options", CoreTy::Typed), p("log", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.fixer_base",
        qualifier: "BaseFix",
        name: "match",
        kind: SigKind::Method,
        params: &[p("node", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.main",
        qualifier: "StdoutRefactoringTool",
        name: "log_error",
        kind: SigKind::Method,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.pgen2.literals",
        qualifier: "",
        name: "escape",
        kind: SigKind::ModuleFn,
        params: &[p("m", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.pgen2.pgen",
        qualifier: "DFAState",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("nfaset", CoreTy::Dict), p("final", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.pgen2.pgen",
        qualifier: "ParserGenerator",
        name: "raise_error",
        kind: SigKind::Method,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.pgen2.pgen",
        qualifier: "ParserGenerator",
        name: "simplify_dfa",
        kind: SigKind::Method,
        params: &[p("dfa", CoreTy::List)],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.pgen2.tokenize",
        qualifier: "",
        name: "generate_tokens",
        kind: SigKind::ModuleFn,
        params: &[p("readline", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.pgen2.tokenize",
        qualifier: "",
        name: "tokenize",
        kind: SigKind::ModuleFn,
        params: &[p("readline", CoreTy::Typed), p("tokeneater", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.pgen2.tokenize",
        qualifier: "Untokenizer",
        name: "compat",
        kind: SigKind::Method,
        params: &[p("token", CoreTy::Tuple), p("iterable", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.pytree",
        qualifier: "Base",
        name: "replace",
        kind: SigKind::Method,
        params: &[p("new", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.refactor",
        qualifier: "RefactoringTool",
        name: "log_debug",
        kind: SigKind::Method,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.refactor",
        qualifier: "RefactoringTool",
        name: "log_error",
        kind: SigKind::Method,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.refactor",
        qualifier: "RefactoringTool",
        name: "log_message",
        kind: SigKind::Method,
        params: &[
            p("msg", CoreTy::Str),
            ParamSig {
                name: "args",
                ty: CoreTy::Unknown,
                star: true,
            },
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.refactor",
        qualifier: "RefactoringTool",
        name: "refactor_doctest",
        kind: SigKind::Method,
        params: &[
            p("block", CoreTy::List),
            p("lineno", CoreTy::Int),
            p("indent", CoreTy::Int),
            p("filename", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "lib2to3.refactor",
        qualifier: "RefactoringTool",
        name: "refactor_stdin",
        kind: SigKind::Method,
        params: &[p("doctests_only", CoreTy::Bool)],
        enforceable: true,
    },
    // fractions.Fraction arithmetic partners are generated from overloaded
    // numeric aliases and collapse to Unknown. For strict-type fixtures, use
    // Complex as the broad numeric wall: int/float/bool remain compatible, while
    // wrong strings and bare user objects are rejected.
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__add__",
        kind: SigKind::Method,
        params: &[p("b", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__radd__",
        kind: SigKind::Method,
        params: &[p("a", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__sub__",
        kind: SigKind::Method,
        params: &[p("b", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__rsub__",
        kind: SigKind::Method,
        params: &[p("a", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__mul__",
        kind: SigKind::Method,
        params: &[p("b", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__rmul__",
        kind: SigKind::Method,
        params: &[p("a", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__truediv__",
        kind: SigKind::Method,
        params: &[p("b", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__rtruediv__",
        kind: SigKind::Method,
        params: &[p("a", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__floordiv__",
        kind: SigKind::Method,
        params: &[p("b", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__rfloordiv__",
        kind: SigKind::Method,
        params: &[p("a", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__mod__",
        kind: SigKind::Method,
        params: &[p("b", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__rmod__",
        kind: SigKind::Method,
        params: &[p("a", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__divmod__",
        kind: SigKind::Method,
        params: &[p("b", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__rdivmod__",
        kind: SigKind::Method,
        params: &[p("b", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__pow__",
        kind: SigKind::Method,
        params: &[p("b", CoreTy::Int)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__rpow__",
        kind: SigKind::Method,
        params: &[p("a", CoreTy::Complex)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__round__",
        kind: SigKind::Method,
        params: &[p("ndigits", CoreTy::Int)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__int__",
        kind: SigKind::Method,
        params: &[p("_index", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "fractions",
        qualifier: "Fraction",
        name: "__new__",
        kind: SigKind::Method,
        params: &[p("numerator", CoreTy::Typed)],
        enforceable: true,
    },
    // Misc stdlib strict-wall probes where generated rows collapse protocols,
    // literals, or overload-heavy constructor shapes to Unknown/Typed. Keep the
    // probed leading wall precise and leave the rest skip-safe.
    StdlibSig {
        module: "genericpath",
        qualifier: "",
        name: "commonprefix",
        kind: SigKind::ModuleFn,
        params: &[p("m", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "glob",
        qualifier: "",
        name: "glob0",
        kind: SigKind::ModuleFn,
        params: &[p("dirname", CoreTy::Typed), p("pattern", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "glob",
        qualifier: "",
        name: "glob1",
        kind: SigKind::ModuleFn,
        params: &[p("dirname", CoreTy::Typed), p("pattern", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "graphlib",
        qualifier: "TopologicalSorter",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("graph", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "gzip",
        qualifier: "",
        name: "open",
        kind: SigKind::ModuleFn,
        params: &[p("filename", CoreTy::Typed), p("mode", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "gzip",
        qualifier: "GzipFile",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("filename", CoreTy::Typed), p("mode", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "imghdr",
        qualifier: "",
        name: "what",
        kind: SigKind::ModuleFn,
        params: &[p("file", CoreTy::Typed), p("h", CoreTy::Bytes)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.util",
        qualifier: "",
        name: "module_for_loader",
        kind: SigKind::ModuleFn,
        params: &[p("fxn", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.util",
        qualifier: "",
        name: "set_loader",
        kind: SigKind::ModuleFn,
        params: &[p("fxn", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "importlib.util",
        qualifier: "",
        name: "set_package",
        kind: SigKind::ModuleFn,
        params: &[p("fxn", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "classify_class_attrs",
        kind: SigKind::ModuleFn,
        params: &[p("cls", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "Signature",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("parameters", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "formatargspec",
        kind: SigKind::ModuleFn,
        params: &[
            p("args", CoreTy::List),
            p("varargs", CoreTy::Typed),
            p("varkw", CoreTy::Typed),
            p("defaults", CoreTy::Unknown),
            p("kwonlyargs", CoreTy::Unknown),
            p("kwonlydefaults", CoreTy::Unknown),
            p("annotations", CoreTy::Unknown),
            p("formatarg", CoreTy::Unknown),
            p("formatvarargs", CoreTy::Unknown),
            p("formatvarkw", CoreTy::Unknown),
            p("formatvalue", CoreTy::Unknown),
            p("formatreturns", CoreTy::Unknown),
            p("formatannotation", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "formatargvalues",
        kind: SigKind::ModuleFn,
        params: &[
            p("args", CoreTy::List),
            p("varargs", CoreTy::Typed),
            p("varkw", CoreTy::Typed),
            p("locals", CoreTy::Unknown),
            p("formatarg", CoreTy::Unknown),
            p("formatvarargs", CoreTy::Unknown),
            p("formatvarkw", CoreTy::Unknown),
            p("formatvalue", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "get_annotations",
        kind: SigKind::ModuleFn,
        params: &[p("obj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "getasyncgenlocals",
        kind: SigKind::ModuleFn,
        params: &[p("agen", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "getasyncgenstate",
        kind: SigKind::ModuleFn,
        params: &[p("agen", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "getblock",
        kind: SigKind::ModuleFn,
        params: &[p("lines", CoreTy::List)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "getcallargs",
        kind: SigKind::ModuleFn,
        params: &[p("func", CoreTy::Typed), p("args", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "getcoroutinelocals",
        kind: SigKind::ModuleFn,
        params: &[p("coroutine", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "getcoroutinestate",
        kind: SigKind::ModuleFn,
        params: &[p("coroutine", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "getgeneratorstate",
        kind: SigKind::ModuleFn,
        params: &[p("generator", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "getmembers",
        kind: SigKind::ModuleFn,
        params: &[p("object", CoreTy::Unknown), p("predicate", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "getmembers_static",
        kind: SigKind::ModuleFn,
        params: &[p("object", CoreTy::Unknown), p("predicate", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "getmro",
        kind: SigKind::ModuleFn,
        params: &[p("cls", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "isasyncgenfunction",
        kind: SigKind::ModuleFn,
        params: &[p("obj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "iscoroutinefunction",
        kind: SigKind::ModuleFn,
        params: &[p("obj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "isgeneratorfunction",
        kind: SigKind::ModuleFn,
        params: &[p("obj", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "markcoroutinefunction",
        kind: SigKind::ModuleFn,
        params: &[p("func", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "unwrap",
        kind: SigKind::ModuleFn,
        params: &[p("func", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "inspect",
        qualifier: "",
        name: "walktree",
        kind: SigKind::ModuleFn,
        params: &[
            p("classes", CoreTy::List),
            p("children", CoreTy::Unknown),
            p("parent", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "http.client",
        qualifier: "",
        name: "parse_headers",
        kind: SigKind::ModuleFn,
        params: &[p("fp", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "http.client",
        qualifier: "HTTPConnection",
        name: "putheader",
        kind: SigKind::Method,
        params: &[p("header", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "http.client",
        qualifier: "IncompleteRead",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("partial", CoreTy::Bytes), p("expected", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "http.cookiejar",
        qualifier: "DefaultCookiePolicy",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("blocked_domains", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "http.server",
        qualifier: "BaseHTTPRequestHandler",
        name: "log_error",
        kind: SigKind::Method,
        params: &[p("format", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "http.server",
        qualifier: "BaseHTTPRequestHandler",
        name: "log_message",
        kind: SigKind::Method,
        params: &[p("format", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "html.parser",
        qualifier: "HTMLParser",
        name: "goahead",
        kind: SigKind::Method,
        params: &[p("end", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "pkgutil",
        qualifier: "",
        name: "extend_path",
        kind: SigKind::ModuleFn,
        params: &[p("path", CoreTy::Typed), p("name", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "platform",
        qualifier: "",
        name: "platform",
        kind: SigKind::ModuleFn,
        params: &[p("aliased", CoreTy::Bool), p("terse", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "pprint",
        qualifier: "PrettyPrinter",
        name: "format",
        kind: SigKind::Method,
        params: &[
            p("object", CoreTy::Unknown),
            p("context", CoreTy::Dict),
            p("maxlevels", CoreTy::Int),
            p("level", CoreTy::Int),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "py_compile",
        qualifier: "PyCompileError",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("exc_type", CoreTy::Type),
            p("exc_value", CoreTy::Typed),
            p("file", CoreTy::Str),
            p("msg", CoreTy::Str),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "time",
        qualifier: "",
        name: "get_clock_info",
        kind: SigKind::ModuleFn,
        params: &[p("name", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "asyncio.coroutines",
        qualifier: "",
        name: "coroutine",
        kind: SigKind::ModuleFn,
        params: &[p("func", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "http.cookies",
        qualifier: "BaseCookie",
        name: "value_encode",
        kind: SigKind::Method,
        params: &[p("val", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "multiprocessing.process",
        qualifier: "BaseProcess",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("group", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "rlcompleter",
        qualifier: "Completer",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("namespace", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "unittest.runner",
        qualifier: "TextTestResult",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("stream", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "uuid",
        qualifier: "UUID",
        name: "__setattr__",
        kind: SigKind::Method,
        params: &[p("name", CoreTy::Typed), p("value", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "webbrowser",
        qualifier: "GenericBrowser",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("name", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "xml.etree.ElementInclude",
        qualifier: "",
        name: "default_loader",
        kind: SigKind::ModuleFn,
        params: &[p("href", CoreTy::Typed), p("parse", CoreTy::Unknown)],
        enforceable: true,
    },
    // imaplib.Idler.__exit__ uses Unused/Optional exception-state overload
    // pieces that the generated table collapses to Unknown. Keep the probed
    // exception-value slot as a strict typed wall while skipping None sentinels.
    StdlibSig {
        module: "imaplib",
        qualifier: "Idler",
        name: "__exit__",
        kind: SigKind::Method,
        params: &[
            p("exc_type", CoreTy::Unknown),
            p("exc_val", CoreTy::Typed),
            p("exc_tb", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    // mailcap.findmatch(caps) is a Mapping in typeshed. The generated row
    // collapses it to Unknown, but the strict fixture's bare object must be
    // rejected before the long-tail runtime shell returns a placeholder value.
    StdlibSig {
        module: "mailcap",
        qualifier: "",
        name: "findmatch",
        kind: SigKind::ModuleFn,
        params: &[
            p("caps", CoreTy::Typed),
            p("MIMEtype", CoreTy::Str),
            p("key", CoreTy::Str),
            p("filename", CoreTy::Str),
            p("plist", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    // mailbox generated rows collapse StrPath/Literal/Mapping contracts that
    // matter for strict fixture walls. Keep these constructor and mutator walls
    // enforceable without restoring the heavyweight filesystem mailbox classes.
    StdlibSig {
        module: "mailbox",
        qualifier: "Mailbox",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("path", CoreTy::Typed),
            p("factory", CoreTy::Unknown),
            p("create", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "mailbox",
        qualifier: "MaildirMessage",
        name: "set_subdir",
        kind: SigKind::Method,
        params: &[p("subdir", CoreTy::Str)],
        enforceable: true,
    },
    StdlibSig {
        module: "mailbox",
        qualifier: "MH",
        name: "set_sequences",
        kind: SigKind::Method,
        params: &[p("sequences", CoreTy::Typed)],
        enforceable: true,
    },
    // optparse generated rows collapse class objects and defaults mappings to
    // Unknown. Keep strict fixture walls enforceable while still skipping richer
    // runtime objects once the type model cannot prove them wrong.
    StdlibSig {
        module: "optparse",
        qualifier: "OptionContainer",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("option_class", CoreTy::Type),
            p("conflict_handler", CoreTy::Unknown),
            p("description", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "optparse",
        qualifier: "Values",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("defaults", CoreTy::Typed)],
        enforceable: true,
    },
    // statistics sequence/protocol parameters collapse to Unknown in the
    // generated table. Keep strict-mode wrong-type probes from falling through
    // to runtime StatisticsError or missing-function ImportError.
    StdlibSig {
        module: "statistics",
        qualifier: "",
        name: "correlation",
        kind: SigKind::ModuleFn,
        params: &[p("x", CoreTy::Typed), p("y", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "statistics",
        qualifier: "",
        name: "covariance",
        kind: SigKind::ModuleFn,
        params: &[p("x", CoreTy::Typed), p("y", CoreTy::Unknown)],
        enforceable: true,
    },
    StdlibSig {
        module: "statistics",
        qualifier: "",
        name: "kde",
        kind: SigKind::ModuleFn,
        params: &[
            p("data", CoreTy::Typed),
            p("h", CoreTy::Float),
            p("kernel", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "statistics",
        qualifier: "",
        name: "kde_random",
        kind: SigKind::ModuleFn,
        params: &[
            p("data", CoreTy::Typed),
            p("h", CoreTy::Float),
            p("kernel", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "statistics",
        qualifier: "",
        name: "linear_regression",
        kind: SigKind::ModuleFn,
        params: &[
            p("regressor", CoreTy::Typed),
            p("dependent_variable", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    // tkinter.dialog.Dialog.__init__(cnf) is Mapping-shaped. Keep master
    // skip-safe while checking cnf so the strict fixture does not depend on
    // GUI/Tk initialization being available.
    StdlibSig {
        module: "tkinter.dialog",
        qualifier: "Dialog",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("master", CoreTy::Unknown), p("cnf", CoreTy::Typed)],
        enforceable: true,
    },
    // unittest.case callable/context-manager/typevar parameters collapse to
    // Unknown. Treat bare user instances as strict wrong-type probes while
    // leaving callable/protocol-shaped objects skip-safe.
    StdlibSig {
        module: "unittest.case",
        qualifier: "",
        name: "addModuleCleanup",
        kind: SigKind::ModuleFn,
        params: &[p("function", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "unittest.case",
        qualifier: "",
        name: "enterModuleContext",
        kind: SigKind::ModuleFn,
        params: &[p("cm", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "unittest.case",
        qualifier: "",
        name: "expectedFailure",
        kind: SigKind::ModuleFn,
        params: &[p("test_item", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "unittest.case",
        qualifier: "FunctionTestCase",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("testFunc", CoreTy::Typed),
            p("setUp", CoreTy::Unknown),
            p("tearDown", CoreTy::Unknown),
            p("description", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "unittest.case",
        qualifier: "TestCase",
        name: "addClassCleanup",
        kind: SigKind::Method,
        params: &[p("function", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "unittest.case",
        qualifier: "TestCase",
        name: "enterClassContext",
        kind: SigKind::Method,
        params: &[p("cm", CoreTy::Typed)],
        enforceable: true,
    },
    // zipfile has several private/helper classes and Literal/protocol params
    // that the generated table leaves uncheckable. Keep strict type walls in
    // front of runtime ImportError/OSError fallthroughs.
    StdlibSig {
        module: "zipfile",
        qualifier: "CompleteDirs",
        name: "make",
        kind: SigKind::Method,
        params: &[p("source", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "zipfile",
        qualifier: "Path",
        name: "open",
        kind: SigKind::Method,
        params: &[
            p("mode", CoreTy::Str),
            p("encoding", CoreTy::Typed),
            p("errors", CoreTy::Typed),
            p("newline", CoreTy::Typed),
            p("line_buffering", CoreTy::Typed),
            p("write_through", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "zipfile",
        qualifier: "ZipExtFile",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("fileobj", CoreTy::Typed),
            p("mode", CoreTy::Unknown),
            p("zipinfo", CoreTy::Unknown),
            p("pwd", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "zipfile",
        qualifier: "ZipFile",
        name: "__exit__",
        kind: SigKind::Method,
        params: &[
            p("type", CoreTy::Type),
            p("value", CoreTy::Typed),
            p("traceback", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "zipfile",
        qualifier: "ZipFile",
        name: "__init__",
        kind: SigKind::Method,
        params: &[
            p("file", CoreTy::Typed),
            p("mode", CoreTy::Unknown),
            p("compression", CoreTy::Int),
            p("allowZip64", CoreTy::Unknown),
            p("compresslevel", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "zipfile",
        qualifier: "ZipFile",
        name: "setpassword",
        kind: SigKind::Method,
        params: &[p("pwd", CoreTy::Bytes)],
        enforceable: true,
    },
    // pathlib generated rows intentionally collapse type variables, overload
    // literals, and Optional exception state to Unknown/Typed. Keep strict-mode
    // fixture walls enforceable while still skipping correct None sentinels and
    // richer PathLike objects.
    StdlibSig {
        module: "pathlib",
        qualifier: "Path",
        name: "__exit__",
        kind: SigKind::Method,
        params: &[
            p("t", CoreTy::Type),
            p("v", CoreTy::Typed),
            p("tb", CoreTy::Typed),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "pathlib",
        qualifier: "Path",
        name: "copy",
        kind: SigKind::Method,
        params: &[p("target", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "pathlib",
        qualifier: "Path",
        name: "copy_into",
        kind: SigKind::Method,
        params: &[p("target_dir", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "pathlib",
        qualifier: "Path",
        name: "move",
        kind: SigKind::Method,
        params: &[p("target", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "pathlib",
        qualifier: "Path",
        name: "move_into",
        kind: SigKind::Method,
        params: &[p("target_dir", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "pathlib",
        qualifier: "Path",
        name: "open",
        kind: SigKind::Method,
        params: &[
            p("mode", CoreTy::Str),
            p("buffering", CoreTy::Unknown),
            p("encoding", CoreTy::Unknown),
            p("errors", CoreTy::Unknown),
            p("newline", CoreTy::Unknown),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "pathlib",
        qualifier: "Path",
        name: "resolve",
        kind: SigKind::Method,
        params: &[p("strict", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "pathlib",
        qualifier: "Path",
        name: "unlink",
        kind: SigKind::Method,
        params: &[p("missing_ok", CoreTy::Bool)],
        enforceable: true,
    },
    StdlibSig {
        module: "pathlib",
        qualifier: "Path",
        name: "walk",
        kind: SigKind::Method,
        params: &[
            p("top_down", CoreTy::Bool),
            p("on_error", CoreTy::Unknown),
            p("follow_symlinks", CoreTy::Bool),
        ],
        enforceable: true,
    },
    StdlibSig {
        module: "pathlib",
        qualifier: "PurePath",
        name: "is_relative_to",
        kind: SigKind::Method,
        params: &[p("other", CoreTy::Typed)],
        enforceable: true,
    },
    // types module runtime exposes many descriptor/generator objects as stubs.
    // Generated rows are Unknown for these protocol-heavy contracts, so strict
    // type-wall fixtures need curated nominal/callable/type walls here.
    StdlibSig {
        module: "types",
        qualifier: "AsyncGeneratorType",
        name: "asend",
        kind: SigKind::Method,
        params: &[p("val", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "AsyncGeneratorType",
        name: "athrow",
        kind: SigKind::Method,
        params: &[p("typ", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "CoroutineType",
        name: "send",
        kind: SigKind::Method,
        params: &[p("arg", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "CoroutineType",
        name: "throw",
        kind: SigKind::Method,
        params: &[p("typ", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "GeneratorType",
        name: "send",
        kind: SigKind::Method,
        params: &[p("arg", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "GeneratorType",
        name: "throw",
        kind: SigKind::Method,
        params: &[p("typ", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "DynamicClassAttribute",
        name: "__init__",
        kind: SigKind::Method,
        params: &[p("fget", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "DynamicClassAttribute",
        name: "__get__",
        kind: SigKind::Method,
        params: &[p("instance", CoreTy::Unknown), p("ownerclass", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "DynamicClassAttribute",
        name: "getter",
        kind: SigKind::Method,
        params: &[p("fget", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "DynamicClassAttribute",
        name: "setter",
        kind: SigKind::Method,
        params: &[p("fset", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "DynamicClassAttribute",
        name: "deleter",
        kind: SigKind::Method,
        params: &[p("fdel", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "FunctionType",
        name: "__get__",
        kind: SigKind::Method,
        params: &[p("instance", CoreTy::Typed), p("owner", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "ClassMethodDescriptorType",
        name: "__get__",
        kind: SigKind::Method,
        params: &[p("instance", CoreTy::Unknown), p("owner", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "GetSetDescriptorType",
        name: "__get__",
        kind: SigKind::Method,
        params: &[p("instance", CoreTy::Unknown), p("owner", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "MemberDescriptorType",
        name: "__get__",
        kind: SigKind::Method,
        params: &[p("instance", CoreTy::Unknown), p("owner", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "MethodDescriptorType",
        name: "__get__",
        kind: SigKind::Method,
        params: &[p("instance", CoreTy::Unknown), p("owner", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "MethodType",
        name: "__get__",
        kind: SigKind::Method,
        params: &[p("instance", CoreTy::Unknown), p("owner", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "WrapperDescriptorType",
        name: "__get__",
        kind: SigKind::Method,
        params: &[p("instance", CoreTy::Unknown), p("owner", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "MethodType",
        name: "__new__",
        kind: SigKind::Method,
        params: &[p("func", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "GenericAlias",
        name: "__new__",
        kind: SigKind::Method,
        params: &[p("origin", CoreTy::Type)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "MappingProxyType",
        name: "__getitem__",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "MappingProxyType",
        name: "get",
        kind: SigKind::Method,
        params: &[p("key", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "MappingProxyType",
        name: "__or__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
    },
    StdlibSig {
        module: "types",
        qualifier: "MappingProxyType",
        name: "__ror__",
        kind: SigKind::Method,
        params: &[p("value", CoreTy::Typed)],
        enforceable: true,
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
    fn curated_compile_path_skip_curdir_uses_bool_wall() {
        let sig = get("compileall", "", "compile_path").expect("compile_path present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::ModuleFn);
        assert_eq!(sig.params[0].name, "skip_curdir");
        assert_eq!(sig.params[0].ty, CoreTy::Bool);
        assert_eq!(sig.params[1].ty, CoreTy::Int);
        assert_eq!(sig.params[3].ty, CoreTy::Int);
        assert_eq!(sig.params[5].ty, CoreTy::Int);
    }

    #[test]
    fn curated_configparser_get_section_uses_str_wall() {
        let sig = get("configparser", "ConfigParser", "get").expect("ConfigParser.get present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::Method);
        assert_eq!(sig.params[0].name, "section");
        assert_eq!(sig.params[0].ty, CoreTy::Str);
        assert_eq!(sig.params[1].name, "option");
        assert_eq!(sig.params[1].ty, CoreTy::Str);
    }

    #[test]
    fn curated_raw_configparser_walls_override_unknown_rows() {
        for name in ["get", "getboolean", "getfloat", "getint", "items"] {
            let sig = get("configparser", "RawConfigParser", name).expect("RawConfigParser method");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params[0].name, "section");
            assert_eq!(sig.params[0].ty, CoreTy::Str);
        }

        let init =
            get("configparser", "RawConfigParser", "__init__").expect("RawConfigParser init");
        assert!(init.enforceable);
        assert_eq!(init.params[0].name, "defaults");
        assert_eq!(init.params[0].ty, CoreTy::Typed);

        let read = get("configparser", "RawConfigParser", "read").expect("RawConfigParser read");
        assert!(read.enforceable);
        assert_eq!(read.params[0].name, "filenames");
        assert_eq!(read.params[0].ty, CoreTy::Typed);

        let read_dict =
            get("configparser", "RawConfigParser", "read_dict").expect("RawConfigParser read_dict");
        assert!(read_dict.enforceable);
        assert_eq!(read_dict.params[0].name, "dictionary");
        assert_eq!(read_dict.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_ctypes_factory_walls_override_unknown_rows() {
        for name in ["ARRAY", "CFUNCTYPE", "POINTER", "PYFUNCTYPE", "WINFUNCTYPE"] {
            let sig = get("ctypes", "", name).expect("ctypes factory present");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::ModuleFn);
            assert_eq!(sig.params[0].ty, CoreTy::Type);
        }

        let set_pointer =
            get("ctypes", "", "SetPointerType").expect("ctypes.SetPointerType present");
        assert!(set_pointer.enforceable);
        assert_eq!(set_pointer.kind, SigKind::ModuleFn);
        assert_eq!(set_pointer.params[0].name, "pointer");
        assert_eq!(set_pointer.params[0].ty, CoreTy::Type);

        let pointer = get("ctypes", "", "pointer").expect("ctypes.pointer present");
        assert!(pointer.enforceable);
        assert_eq!(pointer.params[0].name, "obj");
        assert_eq!(pointer.params[0].ty, CoreTy::Typed);

        let loader =
            get("ctypes", "LibraryLoader", "__init__").expect("ctypes.LibraryLoader.__init__");
        assert!(loader.enforceable);
        assert_eq!(loader.kind, SigKind::Method);
        assert_eq!(loader.params[0].name, "dlltype");
        assert_eq!(loader.params[0].ty, CoreTy::Type);
    }

    #[test]
    fn curated_curses_wrapper_overrides_variadic_callable_row() {
        let sig = get("curses", "", "wrapper").expect("curses.wrapper present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::ModuleFn);
        assert_eq!(sig.params[0].name, "func");
        assert_eq!(sig.params[0].ty, CoreTy::Typed);
        assert_eq!(sig.params[1].name, "args");
        assert_eq!(sig.params[1].ty, CoreTy::Unknown);
        assert!(sig.params[1].star);
    }

    #[test]
    fn curated_curses_ascii_and_textpad_walls_override_unknown_rows() {
        for name in ["alt", "ascii", "ctrl"] {
            let sig = get("curses.ascii", "", name).expect("curses.ascii helper present");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::ModuleFn);
            assert_eq!(sig.params[0].name, "c");
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }

        let edit = get("curses.textpad", "Textbox", "edit").expect("Textbox.edit present");
        assert!(edit.enforceable);
        assert_eq!(edit.kind, SigKind::Method);
        assert_eq!(edit.params[0].name, "validate");
        assert_eq!(edit.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_dataclasses_walls_override_unknown_rows() {
        for name in ["asdict", "astuple", "replace"] {
            let sig = get("dataclasses", "", name).expect("dataclasses helper present");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::ModuleFn);
            assert_eq!(sig.params[0].name, "obj");
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }

        let dataclass = get("dataclasses", "", "dataclass").expect("dataclass present");
        assert!(dataclass.enforceable);
        assert_eq!(dataclass.kind, SigKind::ModuleFn);
        assert_eq!(dataclass.params[0].name, "cls");
        assert_eq!(dataclass.params[0].ty, CoreTy::Type);

        let field_init = get("dataclasses", "Field", "__init__").expect("Field init present");
        assert!(field_init.enforceable);
        assert_eq!(field_init.kind, SigKind::Method);
        assert_eq!(field_init.params[0].name, "default");
        assert_eq!(field_init.params[0].ty, CoreTy::Typed);
        assert_eq!(field_init.params[2].name, "init");
        assert_eq!(field_init.params[2].ty, CoreTy::Typed);

        let set_name = get("dataclasses", "Field", "__set_name__").expect("Field set_name");
        assert!(set_name.enforceable);
        assert_eq!(set_name.params[0].name, "owner");
        assert_eq!(set_name.params[0].ty, CoreTy::Type);
        assert_eq!(set_name.params[1].name, "name");
        assert_eq!(set_name.params[1].ty, CoreTy::Str);

        for name in ["__class_getitem__", "__init__"] {
            let sig = get("dataclasses", "InitVar", name).expect("InitVar method");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params[0].name, "type");
            assert_eq!(sig.params[0].ty, CoreTy::Type);
        }
    }

    #[test]
    fn curated_datetime_sub_walls_override_unknown_rows() {
        for qualifier in ["date", "datetime"] {
            let sig = get("datetime", qualifier, "__sub__").expect("datetime sub present");
            assert!(sig.enforceable, "{qualifier}.__sub__");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params[0].name, "value");
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }
    }

    #[test]
    fn curated_decimal_round_ndigits_overrides_empty_generated_row() {
        let sig = get("decimal", "Decimal", "__round__").expect("Decimal.__round__ present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::Method);
        assert_eq!(sig.params[0].name, "ndigits");
        assert_eq!(sig.params[0].ty, CoreTy::Int);
    }

    #[test]
    fn curated_difflib_callback_walls_override_unknown_rows() {
        let differ = get("difflib", "Differ", "__init__").expect("Differ.__init__ present");
        assert!(differ.enforceable);
        assert_eq!(differ.kind, SigKind::Method);
        assert_eq!(differ.params[0].name, "linejunk");
        assert_eq!(differ.params[0].ty, CoreTy::Typed);

        let matcher =
            get("difflib", "SequenceMatcher", "__init__").expect("SequenceMatcher.__init__");
        assert!(matcher.enforceable);
        assert_eq!(matcher.params[0].name, "isjunk");
        assert_eq!(matcher.params[0].ty, CoreTy::Typed);

        let diff_bytes = get("difflib", "", "diff_bytes").expect("diff_bytes present");
        assert!(diff_bytes.enforceable);
        assert_eq!(diff_bytes.params[0].name, "dfunc");
        assert_eq!(diff_bytes.params[0].ty, CoreTy::Typed);

        let ndiff = get("difflib", "", "ndiff").expect("ndiff present");
        assert!(ndiff.enforceable);
        assert_eq!(ndiff.params[0].name, "a");
        assert_eq!(ndiff.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_dis_code_object_walls_override_unknown_rows() {
        for (name, param) in [
            ("code_info", "x"),
            ("dis", "x"),
            ("findlabels", "code"),
            ("findlinestarts", "code"),
            ("get_instructions", "x"),
            ("show_code", "co"),
        ] {
            let sig = get("dis", "", name).expect("dis module row present");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::ModuleFn);
            assert_eq!(sig.params[0].name, param);
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }

        let disassemble = get("dis", "", "disassemble").expect("disassemble present");
        assert!(disassemble.enforceable);
        assert_eq!(disassemble.params[0].name, "co");
        assert_eq!(disassemble.params[0].ty, CoreTy::Typed);
        assert_eq!(disassemble.params[1].name, "lasti");
        assert_eq!(disassemble.params[1].ty, CoreTy::Int);

        let bytecode = get("dis", "Bytecode", "__init__").expect("Bytecode.__init__ present");
        assert!(bytecode.enforceable);
        assert_eq!(bytecode.kind, SigKind::Method);
        assert_eq!(bytecode.params[0].name, "x");
        assert_eq!(bytecode.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_distutils_archive_make_archive_base_name_wall() {
        let sig = get("distutils.archive_util", "", "make_archive").expect("make_archive present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::ModuleFn);
        assert_eq!(sig.params[0].name, "base_name");
        assert_eq!(sig.params[0].ty, CoreTy::Str);
        assert_eq!(sig.params[1].name, "format");
        assert_eq!(sig.params[1].ty, CoreTy::Str);
    }

    #[test]
    fn curated_distutils_ccompiler_walls_override_unknown_rows() {
        let preprocess = get("distutils.ccompiler", "", "gen_preprocess_options")
            .expect("gen_preprocess_options present");
        assert!(preprocess.enforceable);
        assert_eq!(preprocess.kind, SigKind::ModuleFn);
        assert_eq!(preprocess.params[0].name, "macros");
        assert_eq!(preprocess.params[0].ty, CoreTy::List);

        let init = get("distutils.ccompiler", "CCompiler", "__init__")
            .expect("CCompiler.__init__ present");
        assert!(init.enforceable);
        assert_eq!(init.kind, SigKind::Method);
        assert_eq!(init.params[0].name, "verbose");
        assert_eq!(init.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_distutils_command_walls_override_unknown_rows() {
        let get_outputs = get(
            "distutils.command.build_py",
            "build_py",
            "get_outputs",
        )
        .expect("build_py.get_outputs present");
        assert!(get_outputs.enforceable);
        assert_eq!(get_outputs.kind, SigKind::Method);
        assert_eq!(get_outputs.params[0].name, "include_bytecode");
        assert_eq!(get_outputs.params[0].ty, CoreTy::Typed);

        let init = get("distutils.command.check", "SilentReporter", "__init__")
            .expect("SilentReporter.__init__ present");
        assert!(init.enforceable);
        assert_eq!(init.kind, SigKind::Method);
        assert_eq!(init.params[3].name, "stream");
        assert_eq!(init.params[3].ty, CoreTy::Typed);

        let search_cpp = get("distutils.command.config", "config", "search_cpp")
            .expect("config.search_cpp present");
        assert!(search_cpp.enforceable);
        assert_eq!(search_cpp.kind, SigKind::Method);
        assert_eq!(search_cpp.params[0].name, "pattern");
        assert_eq!(search_cpp.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_distutils_dist_distribution_attrs_wall() {
        let sig = get("distutils.dist", "Distribution", "__init__")
            .expect("Distribution.__init__ present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::Method);
        assert_eq!(sig.params[0].name, "attrs");
        assert_eq!(sig.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_distutils_file_util_src_path_walls() {
        for name in ["copy_file", "move_file"] {
            let sig = get("distutils.file_util", "", name).expect("file_util row present");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::ModuleFn);
            assert_eq!(sig.params[0].name, "src");
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
            assert_eq!(sig.params[1].name, "dst");
            assert_eq!(sig.params[1].ty, CoreTy::Typed);
        }
    }

    #[test]
    fn curated_distutils_filelist_walls_override_unknown_rows() {
        let translate = get("distutils.filelist", "", "translate_pattern")
            .expect("translate_pattern row present");
        assert!(translate.enforceable);
        assert_eq!(translate.kind, SigKind::ModuleFn);
        assert_eq!(translate.params[0].name, "pattern");
        assert_eq!(translate.params[0].ty, CoreTy::Str);

        let init = get("distutils.filelist", "FileList", "__init__")
            .expect("FileList.__init__ row present");
        assert!(init.enforceable);
        assert_eq!(init.kind, SigKind::Method);
        assert_eq!(init.params[0].name, "warn");
        assert_eq!(init.params[0].ty, CoreTy::Typed);

        for name in ["exclude_pattern", "include_pattern"] {
            let sig = get("distutils.filelist", "FileList", name)
                .expect("FileList pattern row present");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params[0].name, "pattern");
            assert_eq!(sig.params[0].ty, CoreTy::Str);
        }
    }

    #[test]
    fn curated_distutils_log_variadic_prefix_walls() {
        for qualifier in ["", "Log"] {
            let kind = if qualifier.is_empty() {
                SigKind::ModuleFn
            } else {
                SigKind::Method
            };

            for name in ["debug", "error", "fatal", "info", "warn"] {
                let sig = get("distutils.log", qualifier, name).expect("log row present");
                assert!(sig.enforceable, "{qualifier}.{name}");
                assert_eq!(sig.kind, kind);
                assert_eq!(sig.params[0].name, "msg");
                assert_eq!(sig.params[0].ty, CoreTy::Str);
                assert_eq!(sig.params[1].name, "args");
                assert_eq!(sig.params[1].ty, CoreTy::Unknown);
                assert!(sig.params[1].star);
            }

            let log = get("distutils.log", qualifier, "log").expect("log(level, msg) present");
            assert!(log.enforceable, "{qualifier}.log");
            assert_eq!(log.kind, kind);
            assert_eq!(log.params[0].name, "level");
            assert_eq!(log.params[0].ty, CoreTy::Int);
            assert_eq!(log.params[1].name, "msg");
            assert_eq!(log.params[1].ty, CoreTy::Str);
            assert_eq!(log.params[2].name, "args");
            assert_eq!(log.params[2].ty, CoreTy::Unknown);
            assert!(log.params[2].star);
        }
    }

    #[test]
    fn curated_distutils_sysconfig_walls_override_unknown_rows() {
        let config_var = get("distutils.sysconfig", "", "get_config_var")
            .expect("get_config_var row present");
        assert!(config_var.enforceable);
        assert_eq!(config_var.kind, SigKind::ModuleFn);
        assert_eq!(config_var.params[0].name, "name");
        assert_eq!(config_var.params[0].ty, CoreTy::Str);

        let config_vars = get("distutils.sysconfig", "", "get_config_vars")
            .expect("get_config_vars row present");
        assert!(config_vars.enforceable);
        assert_eq!(config_vars.kind, SigKind::ModuleFn);
        assert_eq!(config_vars.params[0].name, "arg");
        assert_eq!(config_vars.params[0].ty, CoreTy::Str);
        assert!(!config_vars.params[0].star);

        let python_inc = get("distutils.sysconfig", "", "get_python_inc")
            .expect("get_python_inc row present");
        assert!(python_inc.enforceable);
        assert_eq!(python_inc.kind, SigKind::ModuleFn);
        assert_eq!(python_inc.params[0].name, "plat_specific");
        assert_eq!(python_inc.params[0].ty, CoreTy::Typed);
        assert_eq!(python_inc.params[1].name, "prefix");
        assert_eq!(python_inc.params[1].ty, CoreTy::Typed);

        let python_lib = get("distutils.sysconfig", "", "get_python_lib")
            .expect("get_python_lib row present");
        assert!(python_lib.enforceable);
        assert_eq!(python_lib.kind, SigKind::ModuleFn);
        assert_eq!(python_lib.params[0].name, "plat_specific");
        assert_eq!(python_lib.params[0].ty, CoreTy::Typed);
        assert_eq!(python_lib.params[1].name, "standard_lib");
        assert_eq!(python_lib.params[1].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_distutils_util_walls_override_unknown_rows() {
        let byte_compile = get("distutils.util", "", "byte_compile")
            .expect("byte_compile row present");
        assert!(byte_compile.enforceable);
        assert_eq!(byte_compile.kind, SigKind::ModuleFn);
        assert_eq!(byte_compile.params[0].name, "py_files");
        assert_eq!(byte_compile.params[0].ty, CoreTy::List);

        let execute = get("distutils.util", "", "execute").expect("execute row present");
        assert!(execute.enforceable);
        assert_eq!(execute.kind, SigKind::ModuleFn);
        assert_eq!(execute.params[0].name, "func");
        assert_eq!(execute.params[0].ty, CoreTy::Typed);
        assert_eq!(execute.params[1].name, "args");
        assert_eq!(execute.params[1].ty, CoreTy::Unknown);
    }

    #[test]
    fn curated_doctest_walls_override_unknown_rows() {
        let run_docstring_examples = get("doctest", "", "run_docstring_examples")
            .expect("run_docstring_examples row present");
        assert!(run_docstring_examples.enforceable);
        assert_eq!(run_docstring_examples.kind, SigKind::ModuleFn);
        assert_eq!(run_docstring_examples.params[0].name, "f");
        assert_eq!(run_docstring_examples.params[0].ty, CoreTy::Unknown);
        assert_eq!(run_docstring_examples.params[1].name, "globs");
        assert_eq!(run_docstring_examples.params[1].ty, CoreTy::Dict);

        let doc_test = get("doctest", "DocTest", "__init__").expect("DocTest.__init__ present");
        assert!(doc_test.enforceable);
        assert_eq!(doc_test.kind, SigKind::Method);
        assert_eq!(doc_test.params[0].name, "examples");
        assert_eq!(doc_test.params[0].ty, CoreTy::List);

        let finder = get("doctest", "DocTestFinder", "__init__")
            .expect("DocTestFinder.__init__ present");
        assert!(finder.enforceable);
        assert_eq!(finder.kind, SigKind::Method);
        assert_eq!(finder.params[0].name, "verbose");
        assert_eq!(finder.params[0].ty, CoreTy::Bool);
    }

    #[test]
    fn curated_email_message_factory_walls_override_unknown_rows() {
        for (name, first_param) in [
            ("message_from_binary_file", "fp"),
            ("message_from_bytes", "s"),
            ("message_from_file", "fp"),
        ] {
            let sig = get("email", "", name).expect("email message factory row present");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::ModuleFn);
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }
    }

    #[test]
    fn curated_email_charset_body_encode_wall_overrides_unknown_row() {
        let sig = get("email.charset", "Charset", "body_encode")
            .expect("Charset.body_encode row present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::Method);
        assert_eq!(sig.params[0].name, "string");
        assert_eq!(sig.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_email_feedparser_factory_walls_override_unknown_rows() {
        for qualifier in ["BytesFeedParser", "FeedParser"] {
            let sig = get("email.feedparser", qualifier, "__init__")
                .expect("feedparser constructor row present");
            assert!(sig.enforceable, "{qualifier}");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params[0].name, "_factory");
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }
    }

    #[test]
    fn curated_email_generator_outfp_walls_override_unknown_rows() {
        for qualifier in ["BytesGenerator", "DecodedGenerator", "Generator"] {
            let sig = get("email.generator", qualifier, "__init__")
                .expect("email generator constructor row present");
            assert!(sig.enforceable, "{qualifier}");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params[0].name, "outfp");
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }
    }

    #[test]
    fn curated_email_headerregistry_base_class_wall_overrides_unknown_row() {
        let sig = get("email.headerregistry", "HeaderRegistry", "__init__")
            .expect("HeaderRegistry.__init__ row present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::Method);
        assert_eq!(sig.params[0].name, "base_class");
        assert_eq!(sig.params[0].ty, CoreTy::Type);
        assert_eq!(sig.params[1].name, "default_class");
        assert_eq!(sig.params[1].ty, CoreTy::Unknown);
        assert_eq!(sig.params[2].name, "use_default_map");
        assert_eq!(sig.params[2].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_email_iterators_typed_subpart_msg_wall_overrides_unknown_row() {
        let sig = get("email.iterators", "", "typed_subpart_iterator")
            .expect("typed_subpart_iterator row present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::ModuleFn);
        assert_eq!(sig.params[0].name, "msg");
        assert_eq!(sig.params[0].ty, CoreTy::Typed);
        assert_eq!(sig.params[1].name, "maintype");
        assert_eq!(sig.params[1].ty, CoreTy::Str);
        assert_eq!(sig.params[2].name, "subtype");
        assert_eq!(sig.params[2].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_email_message_serialization_unixfrom_walls_override_typed_rows() {
        for (qualifier, name, expected_len) in [
            ("MIMEPart", "as_string", 3),
            ("Message", "as_bytes", 2),
            ("Message", "as_string", 3),
        ] {
            let sig = get("email.message", qualifier, name)
                .expect("email.message serialization row present");
            assert!(sig.enforceable, "{qualifier}.{name}");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params.len(), expected_len, "{qualifier}.{name}");
            assert_eq!(sig.params[0].name, "unixfrom", "{qualifier}.{name}");
            assert_eq!(sig.params[0].ty, CoreTy::Bool, "{qualifier}.{name}");
        }
    }

    #[test]
    fn curated_email_message_mimepart_typed_walls_override_unknown_rows() {
        for (name, param_name) in [
            ("__init__", "policy"),
            ("attach", "payload"),
            ("get_body", "preferencelist"),
        ] {
            let sig = get("email.message", "MIMEPart", name)
                .expect("email.message MIMEPart row present");
            assert!(sig.enforceable, "MIMEPart.{name}");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params[0].name, param_name, "MIMEPart.{name}");
            assert_eq!(sig.params[0].ty, CoreTy::Typed, "MIMEPart.{name}");
        }
    }

    #[test]
    fn curated_email_message_message_typed_walls_override_unknown_rows() {
        for (name, param_name) in [
            ("__init__", "policy"),
            ("set_payload", "payload"),
            ("get_boundary", "failobj"),
            ("get_charsets", "failobj"),
            ("get_content_charset", "failobj"),
            ("get_filename", "failobj"),
            ("get_params", "failobj"),
        ] {
            let sig = get("email.message", "Message", name)
                .expect("email.message Message row present");
            assert!(sig.enforceable, "Message.{name}");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params[0].name, param_name, "Message.{name}");
            assert_eq!(sig.params[0].ty, CoreTy::Typed, "Message.{name}");
        }
    }

    #[test]
    fn curated_email_message_get_payload_index_wall_overrides_unknown_row() {
        let sig = get("email.message", "Message", "get_payload")
            .expect("email.message Message.get_payload row present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::Method);
        assert_eq!(sig.params[0].name, "i");
        assert_eq!(sig.params[0].ty, CoreTy::Int);
        assert_eq!(sig.params[1].name, "decode");
        assert_eq!(sig.params[1].ty, CoreTy::Unknown);
    }

    #[test]
    fn curated_ast_arguments_posonlyargs_wall_overrides_unknown_row() {
        let sig = get("ast", "arguments", "__init__").expect("ast.arguments.__init__ present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::Method);
        assert_eq!(sig.params[0].name, "posonlyargs");
        assert_eq!(sig.params[0].ty, CoreTy::List);
    }

    #[test]
    fn curated_email_mime_message_init_msg_wall_overrides_unknown_row() {
        let sig = get("email.mime.message", "MIMEMessage", "__init__")
            .expect("email.mime.message MIMEMessage.__init__ row present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::Method);
        assert_eq!(sig.params[0].name, "_msg");
        assert_eq!(sig.params[0].ty, CoreTy::Typed);
        assert_eq!(sig.params[1].name, "_subtype");
        assert_eq!(sig.params[1].ty, CoreTy::Str);
    }

    #[test]
    fn curated_email_parser_init_class_walls_override_unknown_rows() {
        for qualifier in ["BytesParser", "Parser"] {
            let sig = get("email.parser", qualifier, "__init__")
                .expect("email.parser constructor row present");
            assert!(sig.enforceable, "{qualifier}.__init__");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params[0].name, "_class", "{qualifier}.__init__");
            assert_eq!(sig.params[0].ty, CoreTy::Typed, "{qualifier}.__init__");
        }
    }

    #[test]
    fn curated_email_policy_header_source_parse_list_wall_overrides_unknown_row() {
        let sig = get("email.policy", "EmailPolicy", "header_source_parse")
            .expect("email.policy EmailPolicy.header_source_parse row present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::Method);
        assert_eq!(sig.params[0].name, "sourcelines");
        assert_eq!(sig.params[0].ty, CoreTy::List);
    }

    #[test]
    fn curated_email_utils_decode_params_list_wall_overrides_unknown_row() {
        let sig =
            get("email.utils", "", "decode_params").expect("email.utils.decode_params row present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::ModuleFn);
        assert_eq!(sig.params[0].name, "params");
        assert_eq!(sig.params[0].ty, CoreTy::List);
    }

    #[test]
    fn curated_email_utils_formataddr_pair_wall_overrides_unknown_row() {
        let sig = get("email.utils", "", "formataddr").expect("email.utils.formataddr row present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::ModuleFn);
        assert_eq!(sig.params[0].name, "pair");
        assert_eq!(sig.params[0].ty, CoreTy::Tuple);
        assert_eq!(sig.params[1].name, "charset");
        assert_eq!(sig.params[1].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_email_utils_localtime_dt_wall_overrides_unknown_row() {
        let sig = get("email.utils", "", "localtime").expect("email.utils.localtime row present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::ModuleFn);
        assert_eq!(sig.params[0].name, "dt");
        assert_eq!(sig.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_email_utils_parseaddr_string_wall_overrides_unknown_row() {
        let sig = get("email.utils", "", "parseaddr").expect("email.utils.parseaddr row present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::ModuleFn);
        assert_eq!(sig.params[0].name, "addr");
        assert_eq!(sig.params[0].ty, CoreTy::Str);
    }

    #[test]
    fn curated_email_utils_parsedate_string_walls_override_unknown_rows() {
        for name in ["parsedate", "parsedate_tz"] {
            let sig = get("email.utils", "", name).expect("email.utils date parser row present");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::ModuleFn, "{name}");
            assert_eq!(sig.params[0].name, "data", "{name}");
            assert_eq!(sig.params[0].ty, CoreTy::Str, "{name}");
        }
    }

    #[test]
    fn curated_enum_walls_override_unknown_generated_rows() {
        let call = get("enum", "EnumMeta", "__call__").expect("EnumMeta.__call__ row present");
        assert!(call.enforceable);
        assert_eq!(call.kind, SigKind::Method);
        assert_eq!(call.params[0].name, "value");
        assert_eq!(call.params[0].ty, CoreTy::Str);
        assert_eq!(call.params[1].name, "names");
        assert_eq!(call.params[1].ty, CoreTy::Typed);

        for qualifier in ["Enum", "StrEnum"] {
            let sig = get("enum", qualifier, "_generate_next_value_")
                .expect("enum _generate_next_value_ row present");
            assert!(sig.enforceable, "{qualifier}");
            assert_eq!(sig.kind, SigKind::Method, "{qualifier}");
            assert_eq!(sig.params[0].name, "name", "{qualifier}");
            assert_eq!(sig.params[0].ty, CoreTy::Str, "{qualifier}");
            assert_eq!(sig.params[1].name, "start", "{qualifier}");
            assert_eq!(sig.params[1].ty, CoreTy::Int, "{qualifier}");
        }

        for name in ["__and__", "__contains__", "__or__", "__xor__"] {
            let sig = get("enum", "Flag", name).expect("Flag method row present");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::Method, "{name}");
            assert_eq!(sig.params[0].name, "other", "{name}");
            assert_eq!(sig.params[0].ty, CoreTy::Typed, "{name}");
        }

        for (name, param) in [
            ("global_enum", "cls"),
            ("pickle_by_enum_name", "self"),
            ("unique", "enumeration"),
        ] {
            let sig = get("enum", "", name).expect("enum module function row present");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::ModuleFn, "{name}");
            assert_eq!(sig.params[0].name, param, "{name}");
            assert_eq!(sig.params[0].ty, CoreTy::Typed, "{name}");
        }

        for qualifier in ["member", "nonmember"] {
            let sig = get("enum", qualifier, "__init__").expect("enum wrapper init row present");
            assert!(sig.enforceable, "{qualifier}");
            assert_eq!(sig.kind, SigKind::Method, "{qualifier}");
            assert_eq!(sig.params[0].name, "value", "{qualifier}");
            assert_eq!(sig.params[0].ty, CoreTy::Typed, "{qualifier}");
        }
    }

    #[test]
    fn curated_distutils_fancy_getopt_walls() {
        let func = get("distutils.fancy_getopt", "", "fancy_getopt")
            .expect("fancy_getopt row present");
        assert!(func.enforceable);
        assert_eq!(func.kind, SigKind::ModuleFn);
        assert_eq!(func.params[0].name, "options");
        assert_eq!(func.params[0].ty, CoreTy::List);

        let init = get("distutils.fancy_getopt", "FancyGetopt", "__init__")
            .expect("FancyGetopt.__init__ present");
        assert!(init.enforceable);
        assert_eq!(init.kind, SigKind::Method);
        assert_eq!(init.params[0].name, "option_table");
        assert_eq!(init.params[0].ty, CoreTy::Typed);

        let getopt = get("distutils.fancy_getopt", "FancyGetopt", "getopt")
            .expect("FancyGetopt.getopt present");
        assert!(getopt.enforceable);
        assert_eq!(getopt.kind, SigKind::Method);
        assert_eq!(getopt.params[0].name, "args");
        assert_eq!(getopt.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_contextlib_walls_override_unknown_rows() {
        for name in ["asynccontextmanager", "contextmanager"] {
            let sig = get("contextlib", "", name).expect("contextlib decorator present");
            assert!(sig.enforceable, "{name}");
            assert_eq!(sig.kind, SigKind::ModuleFn);
            assert_eq!(sig.params[0].name, "func");
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }

        for (qualifier, name, param) in [
            ("AbstractAsyncContextManager", "__aexit__", "exc_type"),
            ("AbstractContextManager", "__exit__", "exc_type"),
            ("chdir", "__init__", "path"),
            ("nullcontext", "__init__", "enter_result"),
        ] {
            let sig = get("contextlib", qualifier, name).expect("contextlib method present");
            assert!(sig.enforceable, "{qualifier}.{name}");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params[0].name, param);
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }
    }

    #[test]
    fn curated_copy_copyreg_walls_override_unknown_rows() {
        for (module, name, param) in [
            ("copy", "copy", "x"),
            ("copy", "deepcopy", "x"),
            ("copyreg", "add_extension", "module"),
            ("copyreg", "constructor", "object"),
            ("copyreg", "pickle", "ob_type"),
            ("copyreg", "remove_extension", "module"),
        ] {
            let sig = get(module, "", name).expect("copy/copyreg function present");
            assert!(sig.enforceable, "{module}.{name}");
            assert_eq!(sig.kind, SigKind::ModuleFn);
            assert_eq!(sig.params[0].name, param);
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }

        let remove_extension =
            get("copyreg", "", "remove_extension").expect("copyreg.remove_extension present");
        assert_eq!(remove_extension.params[2].name, "code");
        assert_eq!(remove_extension.params[2].ty, CoreTy::Int);
    }

    #[test]
    fn curated_csv_dictreader_f_wall_overrides_unknown_row() {
        let sig = get("csv", "DictReader", "__init__").expect("csv.DictReader.__init__ present");
        assert!(sig.enforceable);
        assert_eq!(sig.kind, SigKind::Method);
        assert_eq!(sig.params[0].name, "f");
        assert_eq!(sig.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_concurrent_executor_max_workers_uses_int_wall() {
        for (module, qualifier) in [
            ("concurrent.futures.interpreter", "InterpreterPoolExecutor"),
            ("concurrent.futures.process", "ProcessPoolExecutor"),
            ("concurrent.futures.thread", "ThreadPoolExecutor"),
        ] {
            let sig = get(module, qualifier, "__init__").expect("executor init present");
            assert!(sig.enforceable, "{module}.{qualifier}.__init__");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params[0].name, "max_workers");
            assert_eq!(sig.params[0].ty, CoreTy::Int);
        }
    }

    #[test]
    fn curated_concurrent_initializer_uses_typed_wall() {
        for (module, qualifier, name) in [
            (
                "concurrent.futures.interpreter",
                "InterpreterPoolExecutor",
                "prepare_context",
            ),
            ("concurrent.futures.interpreter", "WorkerContext", "prepare"),
            (
                "concurrent.futures.thread",
                "ThreadPoolExecutor",
                "prepare_context",
            ),
            ("concurrent.futures.thread", "WorkerContext", "__init__"),
            ("concurrent.futures.thread", "WorkerContext", "prepare"),
        ] {
            let sig = get(module, qualifier, name).expect("initializer sig present");
            assert!(sig.enforceable, "{module}.{qualifier}.{name}");
            assert_eq!(sig.kind, SigKind::Method);
            assert_eq!(sig.params[0].name, "initializer");
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }
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
    fn curated_pathlib_walls_override_unknown_generated_rows() {
        for (qualifier, name, first_param, first_ty) in [
            ("Path", "__exit__", "t", CoreTy::Type),
            ("Path", "copy", "target", CoreTy::Typed),
            ("Path", "copy_into", "target_dir", CoreTy::Typed),
            ("Path", "move", "target", CoreTy::Typed),
            ("Path", "move_into", "target_dir", CoreTy::Typed),
            ("Path", "open", "mode", CoreTy::Str),
            ("Path", "resolve", "strict", CoreTy::Bool),
            ("Path", "unlink", "missing_ok", CoreTy::Bool),
            ("Path", "walk", "top_down", CoreTy::Bool),
            ("PurePath", "is_relative_to", "other", CoreTy::Typed),
        ] {
            let sig = get("pathlib", qualifier, name).expect("pathlib row present");
            assert!(sig.enforceable, "pathlib.{qualifier}.{name} must stay enforceable");
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, first_ty);
        }

        let walk = get("pathlib", "Path", "walk").expect("Path.walk row present");
        assert_eq!(walk.params[2].name, "follow_symlinks");
        assert_eq!(walk.params[2].ty, CoreTy::Bool);
    }

    #[test]
    fn curated_misc_stdlib_single_walls_override_generated_rows() {
        for (module, qualifier, name, param_idx, param_name, param_ty) in [
            ("genericpath", "", "commonprefix", 0, "m", CoreTy::Typed),
            ("glob", "", "glob0", 0, "dirname", CoreTy::Typed),
            ("glob", "", "glob1", 0, "dirname", CoreTy::Typed),
            (
                "graphlib",
                "TopologicalSorter",
                "__init__",
                0,
                "graph",
                CoreTy::Typed,
            ),
            ("gzip", "", "open", 0, "filename", CoreTy::Typed),
            ("gzip", "GzipFile", "__init__", 0, "filename", CoreTy::Typed),
            ("imghdr", "", "what", 0, "file", CoreTy::Typed),
            ("imghdr", "", "what", 1, "h", CoreTy::Bytes),
            ("importlib.util", "", "module_for_loader", 0, "fxn", CoreTy::Typed),
            ("importlib.util", "", "set_loader", 0, "fxn", CoreTy::Typed),
            ("importlib.util", "", "set_package", 0, "fxn", CoreTy::Typed),
            ("inspect", "", "classify_class_attrs", 0, "cls", CoreTy::Type),
            ("inspect", "Signature", "__init__", 0, "parameters", CoreTy::Typed),
            ("inspect", "", "formatargspec", 0, "args", CoreTy::List),
            ("inspect", "", "formatargvalues", 0, "args", CoreTy::List),
            ("inspect", "", "get_annotations", 0, "obj", CoreTy::Typed),
            ("inspect", "", "getasyncgenlocals", 0, "agen", CoreTy::Typed),
            ("inspect", "", "getasyncgenstate", 0, "agen", CoreTy::Typed),
            ("inspect", "", "getblock", 0, "lines", CoreTy::List),
            ("inspect", "", "getcallargs", 0, "func", CoreTy::Typed),
            ("inspect", "", "getcoroutinelocals", 0, "coroutine", CoreTy::Typed),
            ("inspect", "", "getcoroutinestate", 0, "coroutine", CoreTy::Typed),
            ("inspect", "", "getgeneratorstate", 0, "generator", CoreTy::Typed),
            ("inspect", "", "getmembers", 1, "predicate", CoreTy::Typed),
            ("inspect", "", "getmembers_static", 1, "predicate", CoreTy::Typed),
            ("inspect", "", "getmro", 0, "cls", CoreTy::Type),
            ("inspect", "", "isasyncgenfunction", 0, "obj", CoreTy::Typed),
            ("inspect", "", "iscoroutinefunction", 0, "obj", CoreTy::Typed),
            ("inspect", "", "isgeneratorfunction", 0, "obj", CoreTy::Typed),
            ("inspect", "", "markcoroutinefunction", 0, "func", CoreTy::Typed),
            ("inspect", "", "unwrap", 0, "func", CoreTy::Typed),
            ("inspect", "", "walktree", 0, "classes", CoreTy::List),
            ("ipaddress", "IPv4Network", "__init__", 1, "strict", CoreTy::Bool),
            ("ipaddress", "IPv6Network", "__init__", 1, "strict", CoreTy::Bool),
            ("ipaddress", "", "get_mixed_type_key", 0, "obj", CoreTy::Typed),
            ("ipaddress", "", "ip_interface", 0, "address", CoreTy::Typed),
            ("ipaddress", "", "ip_network", 0, "address", CoreTy::Typed),
            ("ipaddress", "", "summarize_address_range", 0, "first", CoreTy::Typed),
            (
                "importlib.metadata._meta",
                "SimplePath",
                "read_text",
                0,
                "encoding",
                CoreTy::Typed,
            ),
            (
                "importlib.readers",
                "MultiplexedPath",
                "joinpath",
                0,
                "child",
                CoreTy::Str,
            ),
            (
                "importlib.resources._common",
                "",
                "files",
                0,
                "anchor",
                CoreTy::Typed,
            ),
            (
                "importlib.resources._common",
                "",
                "package_to_anchor",
                0,
                "func",
                CoreTy::Typed,
            ),
            (
                "importlib.resources._functional",
                "",
                "contents",
                0,
                "anchor",
                CoreTy::Typed,
            ),
            (
                "importlib.resources._functional",
                "",
                "is_resource",
                0,
                "anchor",
                CoreTy::Typed,
            ),
            (
                "importlib.resources._functional",
                "",
                "open_binary",
                0,
                "anchor",
                CoreTy::Typed,
            ),
            (
                "importlib.resources._functional",
                "",
                "open_text",
                0,
                "anchor",
                CoreTy::Typed,
            ),
            (
                "importlib.resources._functional",
                "",
                "path",
                0,
                "anchor",
                CoreTy::Typed,
            ),
            (
                "importlib.resources._functional",
                "",
                "read_binary",
                0,
                "anchor",
                CoreTy::Typed,
            ),
            (
                "importlib.resources._functional",
                "",
                "read_text",
                0,
                "anchor",
                CoreTy::Typed,
            ),
            (
                "importlib.resources.abc",
                "Traversable",
                "open",
                0,
                "mode",
                CoreTy::Str,
            ),
            (
                "importlib.resources.simple",
                "ResourceHandle",
                "joinpath",
                0,
                "name",
                CoreTy::Typed,
            ),
            (
                "importlib.resources.simple",
                "ResourceHandle",
                "open",
                0,
                "mode",
                CoreTy::Str,
            ),
            ("io", "Writer", "write", 0, "data", CoreTy::Typed),
            (
                "lib2to3.fixer_base",
                "BaseFix",
                "__init__",
                0,
                "options",
                CoreTy::Typed,
            ),
            (
                "lib2to3.fixer_base",
                "BaseFix",
                "match",
                0,
                "node",
                CoreTy::Typed,
            ),
            (
                "lib2to3.main",
                "StdoutRefactoringTool",
                "log_error",
                0,
                "msg",
                CoreTy::Str,
            ),
            (
                "lib2to3.pgen2.literals",
                "",
                "escape",
                0,
                "m",
                CoreTy::Typed,
            ),
            (
                "lib2to3.pgen2.pgen",
                "DFAState",
                "__init__",
                0,
                "nfaset",
                CoreTy::Dict,
            ),
            (
                "lib2to3.pgen2.pgen",
                "ParserGenerator",
                "raise_error",
                0,
                "msg",
                CoreTy::Str,
            ),
            (
                "lib2to3.pgen2.pgen",
                "ParserGenerator",
                "simplify_dfa",
                0,
                "dfa",
                CoreTy::List,
            ),
            (
                "lib2to3.pgen2.tokenize",
                "",
                "generate_tokens",
                0,
                "readline",
                CoreTy::Typed,
            ),
            (
                "lib2to3.pgen2.tokenize",
                "",
                "tokenize",
                0,
                "readline",
                CoreTy::Typed,
            ),
            (
                "lib2to3.pgen2.tokenize",
                "Untokenizer",
                "compat",
                0,
                "token",
                CoreTy::Tuple,
            ),
            (
                "lib2to3.pytree",
                "Base",
                "replace",
                0,
                "new",
                CoreTy::Typed,
            ),
            (
                "lib2to3.refactor",
                "RefactoringTool",
                "log_debug",
                0,
                "msg",
                CoreTy::Str,
            ),
            (
                "lib2to3.refactor",
                "RefactoringTool",
                "log_error",
                0,
                "msg",
                CoreTy::Str,
            ),
            (
                "lib2to3.refactor",
                "RefactoringTool",
                "log_message",
                0,
                "msg",
                CoreTy::Str,
            ),
            (
                "lib2to3.refactor",
                "RefactoringTool",
                "refactor_doctest",
                0,
                "block",
                CoreTy::List,
            ),
            (
                "lib2to3.refactor",
                "RefactoringTool",
                "refactor_stdin",
                0,
                "doctests_only",
                CoreTy::Bool,
            ),
            ("locale", "", "format", 0, "percent", CoreTy::Typed),
            (
                "locale",
                "",
                "getdefaultlocale",
                0,
                "envvars",
                CoreTy::Tuple,
            ),
            (
                "locale",
                "",
                "getpreferredencoding",
                0,
                "do_setlocale",
                CoreTy::Bool,
            ),
            ("logging", "", "captureWarnings", 0, "capture", CoreTy::Bool),
            (
                "logging",
                "",
                "getLevelName",
                0,
                "level",
                CoreTy::IntOrStr,
            ),
            ("logging", "", "log", 0, "level", CoreTy::Int),
            ("logging", "", "makeLogRecord", 0, "dict", CoreTy::Typed),
            (
                "logging",
                "",
                "setLogRecordFactory",
                0,
                "factory",
                CoreTy::Typed,
            ),
            ("logging", "", "shutdown", 0, "handlerList", CoreTy::Typed),
            (
                "logging",
                "BufferingFormatter",
                "format",
                0,
                "records",
                CoreTy::Typed,
            ),
            (
                "logging",
                "BufferingFormatter",
                "formatFooter",
                0,
                "records",
                CoreTy::Typed,
            ),
            (
                "logging",
                "BufferingFormatter",
                "formatHeader",
                0,
                "records",
                CoreTy::Typed,
            ),
            (
                "logging",
                "Logger",
                "findCaller",
                0,
                "stack_info",
                CoreTy::Bool,
            ),
            ("logging", "Logger", "log", 0, "level", CoreTy::Int),
            (
                "logging",
                "LoggerAdapter",
                "__init__",
                0,
                "logger",
                CoreTy::Typed,
            ),
            (
                "logging",
                "LoggerAdapter",
                "log",
                0,
                "level",
                CoreTy::Int,
            ),
            (
                "logging",
                "LoggerAdapter",
                "process",
                1,
                "kwargs",
                CoreTy::Dict,
            ),
            (
                "logging",
                "Manager",
                "setLogRecordFactory",
                0,
                "factory",
                CoreTy::Typed,
            ),
            (
                "logging",
                "StreamHandler",
                "__init__",
                0,
                "stream",
                CoreTy::Typed,
            ),
            (
                "logging",
                "StreamHandler",
                "setStream",
                0,
                "stream",
                CoreTy::Typed,
            ),
            ("logging.config", "", "dictConfig", 0, "config", CoreTy::Typed),
            (
                "logging.config",
                "BaseConfigurator",
                "__init__",
                0,
                "config",
                CoreTy::Typed,
            ),
            (
                "logging.config",
                "BaseConfigurator",
                "as_tuple",
                0,
                "value",
                CoreTy::Typed,
            ),
            (
                "logging.config",
                "BaseConfigurator",
                "configure_custom",
                0,
                "config",
                CoreTy::Dict,
            ),
            (
                "logging.config",
                "ConvertingDict",
                "__getitem__",
                0,
                "key",
                CoreTy::Typed,
            ),
            (
                "logging.config",
                "ConvertingDict",
                "get",
                0,
                "key",
                CoreTy::Typed,
            ),
            (
                "logging.config",
                "ConvertingDict",
                "pop",
                0,
                "key",
                CoreTy::Typed,
            ),
            (
                "logging.config",
                "ConvertingList",
                "__getitem__",
                0,
                "key",
                CoreTy::Typed,
            ),
            (
                "logging.config",
                "ConvertingMixin",
                "convert_with_key",
                2,
                "replace",
                CoreTy::Bool,
            ),
            (
                "logging.config",
                "ConvertingTuple",
                "__getitem__",
                0,
                "key",
                CoreTy::Typed,
            ),
            ("http.client", "", "parse_headers", 0, "fp", CoreTy::Typed),
            (
                "http.client",
                "HTTPConnection",
                "putheader",
                0,
                "header",
                CoreTy::Typed,
            ),
            (
                "http.client",
                "IncompleteRead",
                "__init__",
                0,
                "partial",
                CoreTy::Bytes,
            ),
            (
                "http.cookiejar",
                "DefaultCookiePolicy",
                "__init__",
                0,
                "blocked_domains",
                CoreTy::Typed,
            ),
            (
                "http.server",
                "BaseHTTPRequestHandler",
                "log_error",
                0,
                "format",
                CoreTy::Str,
            ),
            (
                "http.server",
                "BaseHTTPRequestHandler",
                "log_message",
                0,
                "format",
                CoreTy::Str,
            ),
            ("html.parser", "HTMLParser", "goahead", 0, "end", CoreTy::Bool),
            ("pkgutil", "", "extend_path", 0, "path", CoreTy::Typed),
            ("platform", "", "platform", 0, "aliased", CoreTy::Bool),
            ("pprint", "PrettyPrinter", "format", 1, "context", CoreTy::Dict),
            (
                "py_compile",
                "PyCompileError",
                "__init__",
                0,
                "exc_type",
                CoreTy::Type,
            ),
            ("time", "", "get_clock_info", 0, "name", CoreTy::Str),
            (
                "asyncio.coroutines",
                "",
                "coroutine",
                0,
                "func",
                CoreTy::Typed,
            ),
            (
                "http.cookies",
                "BaseCookie",
                "value_encode",
                0,
                "val",
                CoreTy::Typed,
            ),
            (
                "multiprocessing.process",
                "BaseProcess",
                "__init__",
                0,
                "group",
                CoreTy::Typed,
            ),
            (
                "rlcompleter",
                "Completer",
                "__init__",
                0,
                "namespace",
                CoreTy::Typed,
            ),
            (
                "unittest.runner",
                "TextTestResult",
                "__init__",
                0,
                "stream",
                CoreTy::Typed,
            ),
            ("uuid", "UUID", "__setattr__", 0, "name", CoreTy::Typed),
            (
                "webbrowser",
                "GenericBrowser",
                "__init__",
                0,
                "name",
                CoreTy::Typed,
            ),
            (
                "xml.etree.ElementInclude",
                "",
                "default_loader",
                0,
                "href",
                CoreTy::Typed,
            ),
        ] {
            let sig = get(module, qualifier, name).expect("curated row present");
            assert!(sig.enforceable, "{module}.{qualifier}.{name}");
            assert_eq!(sig.params[param_idx].name, param_name);
            assert_eq!(sig.params[param_idx].ty, param_ty);
        }
    }

    #[test]
    fn curated_imaplib_idler_exit_wall_overrides_unknown_generated_row() {
        let sig = get("imaplib", "Idler", "__exit__").expect("Idler.__exit__ present");
        assert!(sig.enforceable);
        assert_eq!(sig.params[1].name, "exc_val");
        assert_eq!(sig.params[1].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_mailcap_findmatch_caps_wall_overrides_unknown_generated_row() {
        let sig = get("mailcap", "", "findmatch").expect("mailcap.findmatch present");
        assert!(sig.enforceable);
        assert_eq!(sig.params[0].name, "caps");
        assert_eq!(sig.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_mailbox_walls_override_unknown_generated_rows() {
        let mailbox = get("mailbox", "Mailbox", "__init__").expect("Mailbox.__init__ present");
        assert!(mailbox.enforceable);
        assert_eq!(mailbox.params[0].name, "path");
        assert_eq!(mailbox.params[0].ty, CoreTy::Typed);

        let subdir =
            get("mailbox", "MaildirMessage", "set_subdir").expect("set_subdir present");
        assert!(subdir.enforceable);
        assert_eq!(subdir.params[0].name, "subdir");
        assert_eq!(subdir.params[0].ty, CoreTy::Str);

        let sequences = get("mailbox", "MH", "set_sequences").expect("MH.set_sequences present");
        assert!(sequences.enforceable);
        assert_eq!(sequences.params[0].name, "sequences");
        assert_eq!(sequences.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_optparse_constructor_walls_override_unknown_generated_rows() {
        let container =
            get("optparse", "OptionContainer", "__init__").expect("OptionContainer.__init__ present");
        assert!(container.enforceable);
        assert_eq!(container.params[0].name, "option_class");
        assert_eq!(container.params[0].ty, CoreTy::Type);
        assert_eq!(container.params[2].name, "description");
        assert_eq!(container.params[2].ty, CoreTy::Typed);

        let values = get("optparse", "Values", "__init__").expect("Values.__init__ present");
        assert!(values.enforceable);
        assert_eq!(values.params[0].name, "defaults");
        assert_eq!(values.params[0].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_statistics_sequence_walls_override_unknown_generated_rows() {
        for (name, first_param) in [
            ("correlation", "x"),
            ("covariance", "x"),
            ("kde", "data"),
            ("kde_random", "data"),
            ("linear_regression", "regressor"),
        ] {
            let sig = get("statistics", "", name).expect("statistics row present");
            assert!(sig.enforceable, "statistics.{name} must stay enforceable");
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }

        let kde = get("statistics", "", "kde").expect("statistics.kde present");
        assert_eq!(kde.params[1].name, "h");
        assert_eq!(kde.params[1].ty, CoreTy::Float);
    }

    #[test]
    fn curated_tkinter_dialog_cnf_wall_overrides_unknown_generated_row() {
        let sig = get("tkinter.dialog", "Dialog", "__init__").expect("Dialog.__init__ present");
        assert!(sig.enforceable);
        assert_eq!(sig.params[0].name, "master");
        assert_eq!(sig.params[0].ty, CoreTy::Unknown);
        assert_eq!(sig.params[1].name, "cnf");
        assert_eq!(sig.params[1].ty, CoreTy::Typed);
    }

    #[test]
    fn curated_unittest_case_protocol_walls_override_unknown_generated_rows() {
        for (qualifier, name, first_param) in [
            ("", "addModuleCleanup", "function"),
            ("", "enterModuleContext", "cm"),
            ("", "expectedFailure", "test_item"),
            ("FunctionTestCase", "__init__", "testFunc"),
            ("TestCase", "addClassCleanup", "function"),
            ("TestCase", "enterClassContext", "cm"),
        ] {
            let sig = get("unittest.case", qualifier, name).expect("unittest.case row present");
            assert!(sig.enforceable, "unittest.case.{qualifier}.{name}");
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, CoreTy::Typed);
        }
    }

    #[test]
    fn curated_zipfile_walls_override_unknown_generated_rows() {
        for (qualifier, name, first_param, first_ty) in [
            ("CompleteDirs", "make", "source", CoreTy::Typed),
            ("Path", "open", "mode", CoreTy::Str),
            ("ZipExtFile", "__init__", "fileobj", CoreTy::Typed),
            ("ZipFile", "__exit__", "type", CoreTy::Type),
            ("ZipFile", "__init__", "file", CoreTy::Typed),
            ("ZipFile", "setpassword", "pwd", CoreTy::Bytes),
        ] {
            let sig = get("zipfile", qualifier, name).expect("zipfile row present");
            assert!(sig.enforceable, "zipfile.{qualifier}.{name}");
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, first_ty);
        }
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

    #[test]
    fn curated_defaultdict_walls_override_unknown_generated_rows() {
        for (name, first_param, first_ty) in [
            ("__init__", "default_factory", CoreTy::Typed),
            ("__missing__", "key", CoreTy::Typed),
            ("__or__", "value", CoreTy::Dict),
            ("__ror__", "value", CoreTy::Dict),
        ] {
            let sig = get("collections", "defaultdict", name).expect("defaultdict row present");
            assert!(sig.enforceable, "defaultdict.{name} must stay enforceable");
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, first_ty);
        }
    }

    #[test]
    fn curated_deque_walls_override_unknown_generated_rows() {
        for (name, first_param, first_ty) in [
            ("__add__", "value", CoreTy::Typed),
            ("__ge__", "value", CoreTy::Typed),
            ("__gt__", "value", CoreTy::Typed),
            ("__init__", "iterable", CoreTy::Typed),
            ("__le__", "value", CoreTy::Typed),
            ("__lt__", "value", CoreTy::Typed),
            ("append", "x", CoreTy::Typed),
            ("appendleft", "x", CoreTy::Typed),
            ("count", "x", CoreTy::Typed),
            ("index", "x", CoreTy::Typed),
            ("remove", "value", CoreTy::Typed),
        ] {
            let sig = get("collections", "deque", name).expect("deque row present");
            assert!(sig.enforceable, "deque.{name} must stay enforceable");
            assert_eq!(sig.params[0].name, first_param);
            assert_eq!(sig.params[0].ty, first_ty);
        }

        let index = get("collections", "deque", "index").expect("deque.index row present");
        assert_eq!(index.params[1].ty, CoreTy::Int);
        assert_eq!(index.params[2].ty, CoreTy::Int);
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
