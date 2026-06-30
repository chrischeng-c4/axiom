#!/usr/bin/env python3.12
"""Generate ① Type wall cases from typeshed stdlib signatures.

The Type wall = one wrong-typed-arg case per typeshed signature. mamba is
force-typed, so feeding a signature a wrong-typed argument MUST raise — even
where CPython silently accepts. Red is correct until the runtime enforces; the
wall only has to exist to mark what the keep must fix.

Covers four call shapes synthesized from typeshed:
  - module function    from M import f;            f(wrong, *samples)
  - __init__           from M import C;            C(wrong, *samples)
  - static/classmethod from M import C;            C.m(wrong, *samples)
  - instance method    obj = object.__new__(C);    obj.m(wrong, *samples)

Wrong value for the first (non-self) positional: a wrong literal for a simple
builtin annotation, else a `_W()` sentinel — an instance of a private class that
is of NO stdlib type, so it violates any typed parameter (object/Any/untyped
first params are skipped — nothing is wrong for them). Other required positionals
get a valid sample when simple, else `None` (best-effort arity).

    python3.12 type_wall_gen.py --dry-run
    python3.12 type_wall_gen.py --write
"""

from __future__ import annotations

import argparse
import ast
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from wall_gen_core import PEP723Header  # noqa: E402

MAMBA_DIR = Path(__file__).resolve().parents[4]
TYPESHED_STDLIB = MAMBA_DIR / "vendor" / "typeshed" / "stdlib"
OUT_DIR = MAMBA_DIR / "tests" / "cpython" / "type"
# --emit-rust output: the typeshed-derived stdlib signature table consumed by
# src/types/stdlib_sigs.rs (the ① Type-wall call-site hook).
RUST_SIGS_OUT = MAMBA_DIR / "src" / "types" / "stdlib_sigs_generated.rs"

# Closed scalar map: the ONLY typeshed annotations we encode as a concrete,
# *enforceable* CoreTy. EVERYTHING else (Any, object, unions, Optional,
# typevars, generics, forward-refs, protocols, path-likes,
# Sequence/Iterable/Type/Callable, …) collapses to CoreTy::Unknown so the hook
# skips-when-unsure (zero false positives).
#
# DELIBERATELY EXCLUDED from the enforceable scalar set (mapped to Unknown):
#   * bool  — Python's `bool` is a subclass of `int`, so a `bool`-annotated
#             parameter accepts `0`/`1`/any int at the call site (`html.escape
#             ("x", 1)` is valid). The hook's compatibility rule allows
#             Bool->Int but NOT Int->Bool, so enforcing a Bool param would
#             reject those correct int calls — a false positive. Honesty over
#             coverage: bool params are NON-enforceable.
#   * bytes — there is no concrete `Ty::Bytes` scalar (bytes literals infer to
#             `Any`), so `core_ty_to_type_id` already treats it as
#             non-enforceable. We map it to Unknown so the whole row is honestly
#             marked non-enforceable rather than silently skipped per-param.
#   * None  — a bare `None` annotation is vanishingly rare as a real positional
#             contract and never worth a row; left to Unknown.
SCALAR_CORE_TY = {
    "int": "Int",
    "float": "Float",
    "str": "Str",
}

WRONG_VALUE = {
    "int": '"not_an_int"', "float": '"not_a_float"', "complex": '"not_a_complex"',
    "bool": '"not_a_bool"', "str": "12345", "bytes": "12345", "bytearray": "12345",
    "memoryview": "12345", "list": "12345", "tuple": "12345", "set": "12345",
    "frozenset": "12345", "dict": "12345",
}
SAMPLE_VALUE = {
    "int": "0", "float": "0.0", "complex": "0j", "bool": "True", "str": '""',
    "bytes": 'b""', "bytearray": "bytearray()", "memoryview": 'memoryview(b"")',
    "list": "[]", "tuple": "()", "set": "set()", "frozenset": "frozenset()", "dict": "{}",
}
SENTINEL = "_W()"
NOT_WRONGABLE = {
    "object",
    "Any",
    "_typeshed.Incomplete",
    "Incomplete",
    "type",
    "Callable",
}
NOT_WRONGABLE_SIGNATURE_PARAMS = {
    ("aifc", "", "open", "f"),
    ("argparse", "Action", "__init__", "option_strings"),
    ("argparse", "ArgumentParser", "format_help", "formatter"),
    ("argparse", "ArgumentParser", "format_usage", "formatter"),
    ("argparse", "ArgumentParser", "parse_args", "args"),
    ("argparse", "ArgumentParser", "parse_intermixed_args", "args"),
    ("argparse", "ArgumentParser", "parse_known_args", "args"),
    ("argparse", "ArgumentParser", "parse_known_intermixed_args", "args"),
    ("argparse", "BooleanOptionalAction", "__init__", "option_strings"),
    ("array", "array", "__add__", "value"),
    ("array", "array", "__delitem__", "key"),
    ("array", "array", "__ge__", "value"),
    ("array", "array", "__getitem__", "key"),
    ("array", "array", "__gt__", "value"),
    ("array", "array", "__iadd__", "value"),
    ("array", "array", "__le__", "value"),
    ("array", "array", "__lt__", "value"),
    ("array", "array", "__new__", "typecode"),
    ("array", "array", "__setitem__", "key"),
    ("array", "array", "append", "v"),
    ("array", "array", "count", "v"),
    ("array", "array", "fromlist", "list"),
    ("array", "array", "index", "v"),
    ("array", "array", "remove", "v"),
    # CPython 3.12 exposes no callable ParsingError.filename(value) API. A
    # stale generated fixture here fails the CPython oracle before mamba runs.
    ("configparser", "ParsingError", "filename", "value"),
    # dataclasses.is_dataclass(obj) is a query helper: CPython 3.12 accepts
    # arbitrary objects and returns False for non-dataclasses, and the current
    # generated Rust signature is Unknown/non-enforceable.
    ("dataclasses", "", "is_dataclass", "obj"),
}
BUILTINS = "builtins"
NON_RUNTIME_STUB_MODULE_PREFIXES = ("_typeshed",)


def annotation_label(node: ast.expr | None) -> str:
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Subscript) and isinstance(node.value, ast.Name):
        return node.value.id
    if isinstance(node, ast.Attribute):
        return node.attr
    return "typed"


def is_not_wrongable(node: ast.expr | None) -> bool:
    """Broad/abstract annotations cannot anchor a wrong-typed runtime case."""
    if node is None:
        return True
    label = annotation_label(node)
    return label in NOT_WRONGABLE or _typevar_convention(label)


def is_signature_param_not_wrongable(mod: str, cls: str | None, func: str, param: str) -> bool:
    """Specific typeshed rows whose emitted CoreTy row is Unknown/non-enforceable."""
    return (mod, cls or "", func, param) in NOT_WRONGABLE_SIGNATURE_PARAMS


def sample_annotation(node: ast.expr | None) -> str | None:
    if isinstance(node, ast.Name) and node.id in SAMPLE_VALUE:
        return node.id
    return None


def module_name(pyi: Path) -> str:
    rel = pyi.relative_to(TYPESHED_STDLIB).with_suffix("")
    parts = [p for p in rel.parts if p != "__init__"]
    return ".".join(parts) if parts else rel.stem


def decorator_names(fn: ast.FunctionDef | ast.AsyncFunctionDef) -> set[str]:
    out: set[str] = set()
    for d in fn.decorator_list:
        if isinstance(d, ast.Name):
            out.add(d.id)
        elif isinstance(d, ast.Attribute):
            out.add(d.attr)
    return out


def synth_call(fn: ast.FunctionDef | ast.AsyncFunctionDef, drop_first: bool):
    """Return (param, label, args_list) targeting the first wrongable positional.

    Scans all positionals (not just the first) for one with a type contract that
    can be violated; earlier positionals get valid samples. None only if NO
    positional is wrongable (genuinely no positional type contract)."""
    pos = fn.args.posonlyargs + fn.args.args
    if drop_first:
        pos = pos[1:]
    if not pos:
        return None
    target = next((i for i, p in enumerate(pos) if not is_not_wrongable(p.annotation)), None)
    if target is None:
        return None
    n_required = len(pos) - len(fn.args.defaults)
    upto = max(target + 1, n_required)
    args: list[str] = []
    tparam = tlabel = ""
    for i, p in enumerate(pos[:upto]):
        if i == target:
            tlabel = annotation_label(p.annotation)
            args.append(WRONG_VALUE[tlabel] if tlabel in WRONG_VALUE else SENTINEL)
            tparam = p.arg
        else:
            s = sample_annotation(p.annotation)
            args.append(SAMPLE_VALUE[s] if s else "None")
    return tparam, tlabel, args


def _mk(mod, kind, cls, func, got):
    return dict(mod=mod, kind=kind, cls=cls, func=func,
                param=got[0], label=got[1], args=got[2])


def _walk_class(body, mod, cls, kinds, v312=True):
    """Methods directly in a class body, recursing into If-version blocks. Dunder
    methods (except __init__) are emitted as instance-method calls obj.__x__(wrong)."""
    if not v312:
        return
    for m in body:
        if isinstance(m, (ast.FunctionDef, ast.AsyncFunctionDef)):
            decos = decorator_names(m)
            is_static = "staticmethod" in decos
            is_class = "classmethod" in decos
            if m.name == "__init__":
                kind = "init"
            elif is_static or is_class:
                kind = "smethod"
            elif m.name.startswith("__") and m.name.endswith("__"):
                kind = "method"          # dunder -> obj.__x__(wrong)
            elif m.name.startswith("_"):
                continue                  # single-underscore private
            else:
                kind = "method"
            if kind in kinds:
                got = synth_call(m, drop_first=not is_static)
                if got and not is_signature_param_not_wrongable(mod, cls, m.name, got[0]):
                    yield _mk(mod, kind, cls, m.name, got)
        elif isinstance(m, ast.ClassDef):
            if not m.name.startswith("_"):
                yield from _walk_class(m.body, mod, f"{cls}.{m.name}", kinds, v312)
        elif isinstance(m, ast.If):
            for branch, bv in _branch_v312(m, v312):
                yield from _walk_class(branch, mod, cls, kinds, bv)


def _walk_module(body, mod, kinds, v312=True):
    """Module-level defs and classes, recursing into If-version blocks."""
    if not v312:
        return
    for node in body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if "module" in kinds and not node.name.startswith("_"):
                got = synth_call(node, drop_first=False)
                if got and not is_signature_param_not_wrongable(mod, None, node.name, got[0]):
                    yield _mk(mod, "module", None, node.name, got)
        elif isinstance(node, ast.ClassDef):
            if not node.name.startswith("_"):
                yield from _walk_class(node.body, mod, node.name, kinds, v312)
        elif isinstance(node, ast.If):
            for branch, bv in _branch_v312(node, v312):
                yield from _walk_module(branch, mod, kinds, bv)


def candidates(kinds: set[str]):
    for pyi in sorted(TYPESHED_STDLIB.rglob("*.pyi")):
        mod = module_name(pyi)
        if any(
            mod == prefix or mod.startswith(f"{prefix}.")
            for prefix in NON_RUNTIME_STUB_MODULE_PREFIXES
        ):
            continue
        try:
            tree = ast.parse(pyi.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        yield from _walk_module(tree.body, mod, kinds)


def render(c: dict) -> tuple[str, str]:
    lib = c["mod"].replace(".", "_")
    bucket = "builtin-libs" if c["mod"] == BUILTINS else "std-libs"
    label = c["label"]
    arglist = c["args"]
    args = ", ".join(arglist)
    mod, cls, func, param, kind = c["mod"], c["cls"], c["func"], c["param"], c["kind"]
    sentinel = "class _W:\n    pass\n\n\n" if SENTINEL in arglist else ""
    cls_top = cls.split(".")[0] if cls else ""    # importable top-level class
    cls_id = cls.replace(".", "_") if cls else ""  # filesystem-safe class id

    if kind == "module":
        case = f"{func}__{param}_as_{label}_wrong"
        subject = f"{mod}.{func}({param}: {label})"
        imp = "" if mod == BUILTINS else f"from {mod} import {func}\n"
        call = f"{func}({args})"
        prelude = imp
    elif kind == "init":
        case = f"{cls_id}__init__{param}_as_{label}_wrong"
        subject = f"{mod}.{cls}.__init__({param}: {label})"
        prelude = f"from {mod} import {cls_top}\n"
        call = f"{cls}({args})"
    elif kind == "smethod":
        case = f"{cls_id}__{func}__{param}_as_{label}_wrong"
        subject = f"{mod}.{cls}.{func}({param}: {label})"
        prelude = f"from {mod} import {cls_top}\n"
        call = f"{cls}.{func}({args})"
    else:
        case = f"{cls_id}__{func}__{param}_as_{label}_wrong"
        subject = f"{mod}.{cls}.{func}({param}: {label})"
        prelude = f"from {mod} import {cls_top}\nobj = object.__new__({cls})\n"
        call = f"obj.{func}({args})"

    src = c["mod"].replace(".", "/")
    header = PEP723Header(
        bucket=bucket, lib=lib, dimension="type", case=case, subject=subject,
        kind="semantic",
        xfail=f"force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed {param}",
        mem_carveout="", source=f"vendor/typeshed/stdlib/{src}.pyi", status="filled",
        strict_type="TypeError",
    ).render()
    text = header + f'''"""Type wall: {subject}; call it with the wrong type.

typeshed contract: {param} is {label}. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

{sentinel}{prelude}try:
    {call}  # {param}: {label} <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
'''
    return f"{bucket}/{lib}/{case}.py", text


# --------------------------------------------------------------------------- #
# --emit-rust: typeshed-derived StdlibSig/ParamSig/CoreTy table.
#
# This is an ADDITIVE mode. It does NOT touch the type-wall fixture generation
# above. It walks the same typeshed `.pyi` files with the same `ast.parse`,
# maps EACH positional parameter to a closed CoreTy (scalar set only, else
# Unknown), and emits a deterministic `const STDLIB_SIGS_GENERATED` consumed by
# src/types/stdlib_sigs.rs. The guardrails (skip-when-unsure, Bool->Int /
# Int->Float allow, stop-at-star) all live in the Rust hook and are unchanged;
# this table only declares the contract, and is deliberately conservative:
#   * ANY Unknown param            -> enforceable = false
#   * ANY *args / *  param         -> enforceable = false (alignment uncertain)
#   * OVERLOADED name (>=2 defs)   -> enforceable = false (which overload?)
#   * NO concrete-scalar param     -> enforceable = false (nothing to check)
# The row is still EMITTED in every case (documented negative / skip), exactly
# like the PoC's b64encode/factorial guards — the hook reads `enforceable`.
# --------------------------------------------------------------------------- #


def _is_protocol_name(ident: str) -> bool:
    """A coercion/path PROTOCOL name a *bare* class instance can never satisfy.

    Restricted to the `Supports*` / `_Supports*` coercion protocols
    (`SupportsIndex`, `_SupportsFloatOrIndex`, `SupportsAbs`, …) and `*Like`
    path protocols (`PathLike`, `FileDescriptorLike`). Deliberately EXCLUDES
    TypeVars (`_T`, `AnyStr` — a bare class IS a valid TypeVar binding) and bare
    nominal class names (which would need MRO to judge a non-bare subclass), so
    mapping these to `CoreTy::Typed` only ever rejects a no-base no-method class,
    which satisfies none of them."""
    return (
        ident.startswith("Supports")
        or ident.startswith("_Supports")
        or ident.endswith("Like")
        or ident.endswith("Buffer")        # Buffer/ReadableBuffer/WriteableBuffer (audioop, struct)
        or ident in _ABC_PROTOCOLS         # bare collection ABCs (traceback, etc.)
    )


# Bare collection/iteration ABCs a no-method no-base class can never satisfy.
# Only matched as a BARE Name (`Iterable`, not `Iterable[int]` — that is a
# Subscript → Unknown), so this never touches a parameterized container.
# NOTE: `Hashable` is deliberately EXCLUDED — `object` provides a default
# `__hash__`, so a bare class IS Hashable and rejecting it would false-positive.
# Every ABC kept here needs a dunder `object` does NOT define (__iter__,
# __contains__, __len__, __next__, __reversed__, __await__).
_ABC_PROTOCOLS = frozenset({
    "Iterable", "Iterator", "Reversible", "Collection", "Container",
    "Sized", "Awaitable", "AsyncIterable", "AsyncIterator",
})


# Names a BARE (`object`-only base, no methods) class instance CAN legitimately
# inhabit, so rejecting a `_W()` against them would be a FALSE POSITIVE. Mapped
# to Unknown (skip-when-unsure). `Unused`/`Incomplete` are `_typeshed` aliases
# for `object`/`Any`; `Never`/`NoReturn` are bottom types (a type-checker
# fiction with no runtime contract); `Hashable` is satisfied by every object via
# the default `object.__hash__`.
_WILDCARD_TYPES = frozenset({
    "object", "Any", "Self", "type", "None", "NoneType",
    "Incomplete", "Unused", "Never", "NoReturn", "Hashable",
})


def _typevar_convention(ident: str) -> bool:
    """Unambiguous TypeVar / ParamSpec spellings — a fallback for typevars
    IMPORTED from another module (module-local `X = TypeVar(...)` and PEP 695
    `type_params` are collected structurally in `_collect_module_exclusions`).
    A bare class is a valid binding for any typevar, so rejecting it would be a
    false positive — these stay Unknown."""
    return (
        re.fullmatch(r"[A-Z]", ident) is not None          # PEP 484 single letter: T S K V
        or re.fullmatch(r"_[A-Z]\d?", ident) is not None   # _T _S _T1
        or ident.endswith("_co")                            # covariant
        or ident.endswith("_contra")                        # contravariant
        or ident == "AnyStr"
    )


# Per-module set of names `core_ty_of` must treat as Unknown: module-local
# typevars + PEP 695 type params + aliases that resolve to a wildcard type.
# Set by `rust_rows` before walking each `.pyi`.
_CTX_EXCLUDE: set[str] = set()
# GLOBAL union of those names across ALL stubs — a typevar / `= Any` alias is
# routinely defined in one module and imported+used in another, where the local
# scan cannot see its definition. Set once by `rust_rows`.
_GLOBAL_EXCLUDE: set[str] = set()


def _expr_is_wildcard(e: ast.expr | None) -> bool:
    """True iff `e` denotes a type a BARE `_W()` can legitimately inhabit — i.e.
    `object`/`Any`/`Unused`/… directly, or a `|`-union / `Union[...]` /
    `Optional[...]` with such a member. A CONTAINER subscript (`tuple[Any, ...]`,
    `dict[str, Any]`, `list[Any]`, `type[object]`, `Callable[..., Any]`) is NOT a
    wildcard: a bare class is not a tuple/list/dict/type/callable, so an `Any`
    sitting in its type arguments is irrelevant — only Union/Optional members
    matter. (Treating containers as wildcards over-excludes ~40 enforceable
    nominal aliases like `socketserver._Address = str | tuple[Any, ...]`.)"""
    if isinstance(e, ast.Name):
        return e.id in _WILDCARD_TYPES
    if isinstance(e, ast.Attribute):
        return e.attr in _WILDCARD_TYPES
    if isinstance(e, ast.BinOp) and isinstance(e.op, ast.BitOr):
        return _expr_is_wildcard(e.left) or _expr_is_wildcard(e.right)
    if isinstance(e, ast.Subscript):
        base = e.value
        base_name = (
            base.id if isinstance(base, ast.Name)
            else base.attr if isinstance(base, ast.Attribute)
            else ""
        )
        if base_name in ("Union", "Optional"):  # only these recurse into members
            sub = e.slice
            elts = sub.elts if isinstance(sub, ast.Tuple) else [sub]
            return any(_expr_is_wildcard(x) for x in elts)
        return False  # tuple[...]/dict[...]/type[...]/Callable[...] — concrete
    return False


def _collect_module_exclusions(tree: ast.Module) -> set[str]:
    """Names defined in THIS module that `core_ty_of` must NOT map to `Typed`:
    TypeVar/ParamSpec/TypeVarTuple targets, PEP 695 `type_params`, and aliases
    that resolve to a wildcard (`Unused = object`, `_X: TypeAlias = Any | int`).
    Missing one only risks a false positive, so collection is structural and
    backed by the `_typevar_convention` fallback for imported typevars."""
    excl: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign):
            val = node.value
            is_tv = (
                isinstance(val, ast.Call)
                and isinstance(val.func, ast.Name)
                and val.func.id in ("TypeVar", "ParamSpec", "TypeVarTuple")
            )
            if is_tv or _expr_is_wildcard(val):
                for t in node.targets:
                    if isinstance(t, ast.Name):
                        excl.add(t.id)
        elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            if node.value is not None and _expr_is_wildcard(node.value):
                excl.add(node.target.id)
        elif isinstance(node, ast.TypeAlias) and isinstance(node.name, ast.Name):
            if _expr_is_wildcard(node.value):
                excl.add(node.name.id)
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            for tp in getattr(node, "type_params", []):  # PEP 695 def f[T] / class C[T]
                excl.add(tp.name)
    return excl


def _collect_global_exclusions() -> set[str]:
    """Union of `_collect_module_exclusions` across EVERY typeshed stdlib stub.

    A typevar or `= Any` alias defined in module A is frequently imported and
    used in module B's signatures; B's local scan cannot see A's definition, so
    without a global pass those names wrongly map to `Typed` and would reject a
    valid bare class (a real false positive an adversarial typeshed audit caught:
    `_StrPathT`, `_TC`, `_RetAddress`, `AnnotationForm`, …). Excluding a name
    globally only ever costs a conservative MISS — never a false positive — so
    the union is safe even where a name is also a real class elsewhere."""
    g: set[str] = set()
    for pyi in TYPESHED_STDLIB.rglob("*.pyi"):
        try:
            tree = ast.parse(pyi.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        g |= _collect_module_exclusions(tree)
    return g


# Subscripted bases (besides protocols/ABCs) a BARE no-base no-method class can
# never inhabit, regardless of the type argument: `IO`/`TextIO`/`BinaryIO` (a
# class instance is not an open stream).
#
# `type[...]` and `Callable[...]` are DELIBERATELY EXCLUDED: a CLASS OBJECT is a
# valid `type` and is callable, and mamba's checker types a bare class VALUE
# (`C`) identically to an instance (`C()`) — both `Ty::Class{name}` — so the
# bare-class hook would reject `webbrowser.register('x', ExampleBrowser)` /
# `copyreg.pickle(C, ...)` / `inspect.unwrap(C)` / `dataclasses.fields(@dataclass
# class)`, all of which pass the class itself and which CPython accepts. (Caught
# as 9 real FPs by the full behavior gate.) `tuple`/`list`/`dict`/`set` are also
# excluded — the scalar map declines them and a bare class is not one anyway.
_SUBSCRIPT_TYPED_BASES = frozenset({
    "IO", "TextIO", "BinaryIO",
})


def _subscript_base_name(node: ast.Subscript) -> str:
    """Bare identifier of a Subscript's base (`Iterable` from `Iterable[str]`,
    `os.PathLike` -> `PathLike`), else "" for an un-named base."""
    base = node.value
    if isinstance(base, ast.Name):
        return base.id
    if isinstance(base, ast.Attribute):
        return base.attr
    return ""


def _flatten_union_members(node: ast.expr) -> list[ast.expr]:
    """Flatten a `|`-union BinOp tree into its leaf member expressions."""
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.BitOr):
        return _flatten_union_members(node.left) + _flatten_union_members(node.right)
    return [node]


def _is_none_member(e: ast.expr) -> bool:
    """A `None`/`NoneType` union arm — benign for bare-class inhabitation (a bare
    class instance is never `None`), so the L2 fold SKIPS it rather than aborting
    (this is why `str | None` / `Optional[Nominal]` still flip)."""
    if isinstance(e, ast.Constant) and e.value is None:
        return True
    if isinstance(e, ast.Name):
        return e.id in ("None", "NoneType")
    if isinstance(e, ast.Attribute):
        return e.attr in ("None", "NoneType")
    return False


def _union_core_ty(members: list[ast.expr]) -> str:
    """L2 fold: `Typed` iff EVERY non-`None` member is itself non-Unknown (a scalar
    or a nominal `Typed`); Unknown the moment any member is Unknown. A bare class
    inhabits the union iff it inhabits some member, so one Unknown (wildcard /
    typevar / un-analyzable) member opens the door and must abort. A lone `None`
    slice yields no checkable member -> Unknown."""
    saw_real = False
    for m in members:
        if _is_none_member(m):
            continue
        saw_real = True
        if core_ty_of(m) == "Unknown":
            return "Unknown"
    return "Typed" if saw_real else "Unknown"


def core_ty_of(node: ast.expr | None) -> str:
    """Map a typeshed annotation to a closed CoreTy variant name.

    Bare scalar builtins (int/float/str) map to a concrete scalar CoreTy. Every
    NOMINAL contract a bare `_W()` cannot inhabit — `Supports*`/`*Like` protocols,
    collection ABCs, and any nominal class name or concrete alias — maps to
    `Typed` (the hook rejects a bare class instance against it). Wildcards
    (object/Any/Unused/Never/…), typevars (a bare class is a valid binding), and
    richer shapes whose membership needs analysis (subscripted generics, unions)
    stay Unknown so the hook never enforces against them — skip-when-unsure."""
    if node is None:
        return "Unknown"
    if isinstance(node, ast.Name):
        ident = node.id
        if ident in SCALAR_CORE_TY:
            return SCALAR_CORE_TY[ident]
        if _is_protocol_name(ident):
            return "Typed"
        if ident in _WILDCARD_TYPES or "Any" in ident:
            return "Unknown"
        if ident in _CTX_EXCLUDE or ident in _GLOBAL_EXCLUDE or _typevar_convention(ident):
            return "Unknown"
        # Nominal class name or alias to a concrete (non-wildcard) type: a bare
        # no-base no-method class instance can satisfy none of them.
        return "Typed"
    if isinstance(node, ast.Attribute):
        # e.g. `os.PathLike`, `ast.AST`, `types.FrameType`, `_typeshed.SupportsRead`.
        attr = node.attr
        if _is_protocol_name(attr):
            return "Typed"
        if attr in _WILDCARD_TYPES or "Any" in attr:
            return "Unknown"
        if attr in _GLOBAL_EXCLUDE or _typevar_convention(attr):
            return "Unknown"
        return "Typed"
    # ---- L2: clean Union / Optional (no wildcard member) -> Typed. -----------
    # A bare `_W()` inhabits a union iff it inhabits SOME member. If EVERY member
    # is a concrete contract a bare class cannot inhabit (scalar or nominal
    # `Typed`), it inhabits NONE -> safely `Typed`; any Unknown member (wildcard
    # object/Any, a typevar like `AnyStr`, an un-analyzable subscript) opens the
    # door -> abort to Unknown. A bare `None` member is benign (skipped, not
    # aborting): a bare instance is never `None`, and the hook never rejects a
    # `None` actual, so `str | None` flips and `StringIO(None)` stays valid.
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.BitOr):
        return _union_core_ty(_flatten_union_members(node))
    if isinstance(node, ast.Subscript):
        base_name = _subscript_base_name(node)
        if base_name in ("Union", "Optional"):
            sub = node.slice
            members = sub.elts if isinstance(sub, ast.Tuple) else [sub]
            return _union_core_ty(members)
        # ---- L3: non-inhabitable subscripted protocol / container -> Typed. --
        # `Iterable[str]`, `Sequence[int]`, `Callable[..., T]`, `type[C]`,
        # `IO[str]`, `SupportsRead[bytes]`, `os.PathLike[str]` — the type ARG is
        # irrelevant: a bare no-base no-method class instance has no `__iter__`/
        # `__call__`/`write`, and is not a `type` object / open stream, so it
        # inhabits none of these. The positive predicate is a CLOSED allowlist
        # (the same bare-class-can't-inhabit names the bare-`Name` path trusts —
        # `_ABC_PROTOCOLS` / `_is_protocol_name` = Supports*/`*Like`/`*Buffer` —
        # plus the closed Callable/type/IO set). The base is still excluded if it
        # is a typevar / cross-module wildcard alias (a bare class IS a valid
        # typevar binding). Union/Optional bases handled above; tuple/list/dict/
        # Sequence/Mapping bases stay Unknown.
        if (
            base_name
            and base_name not in _CTX_EXCLUDE
            and base_name not in _GLOBAL_EXCLUDE
            and not _typevar_convention(base_name)
            and (
                base_name in _ABC_PROTOCOLS
                or _is_protocol_name(base_name)
                or base_name in _SUBSCRIPT_TYPED_BASES
            )
        ):
            return "Typed"
        return "Unknown"
    # Tuple, Constant, etc. — membership needs analysis, so skip.
    return "Unknown"


def _collect_params(fn: ast.FunctionDef | ast.AsyncFunctionDef, drop_first: bool):
    """Return (params, has_star) for a callable.

    `params` is a list of (name, core_ty) for the positional parameters
    (posonly + args), in order, with `self`/`cls` dropped for methods.
    `has_star` is True iff the callable has a `*args` (vararg): positional
    alignment past it is uncertain, so such signatures are non-enforceable."""
    pos = fn.args.posonlyargs + fn.args.args
    if drop_first:
        pos = pos[1:]
    params = [(p.arg, core_ty_of(p.annotation)) for p in pos]
    has_star = fn.args.vararg is not None
    return params, has_star


def _scalar_prefix_len(params) -> int:
    """Length of the leading run of concrete-scalar (Int/Float/Str) params.

    The ① hook walks positional args against `params` IN ORDER and stops the
    moment it runs out of rows (`params.get(idx)` -> None -> break). So a row may
    safely enforce its LEADING scalar run and emit *nothing* past the first
    non-scalar param: positions at/after the first non-scalar are simply not
    checked. bool/bytes/None already map to no concrete scalar in
    `core_ty_to_type_id`, but — per the make-or-break invariant test in
    `stdlib_sigs.rs` — an *enforceable* row may carry ONLY Int/Float/Str params,
    so we truncate at the first param that is not one of those three."""
    n = 0
    for _name, ct in params:
        if ct in ("Int", "Float", "Str"):
            n += 1
        else:
            break
    return n


# --- Python-3.12 version-guard resolution (Rust-sig emission only) -----------
# typeshed wraps version-specific signatures in `if sys.version_info <op> (3,N):`
# blocks. For OVERLOAD detection we count only the defs APPLICABLE to our target
# (3.12) — two variants of one method guarded into different Python versions are
# NOT a simultaneous `@overload`, so counting both as an overload spuriously
# disables enforcement. Rows from non-3.12 branches are still EMITTED (so a
# method that exists only in a newer version keeps its sig and its fixture stays
# green), but tagged `v312=False`; the render dedup prefers the 3.12 variant.
_PYVER = (3, 12)


def _is_version_info(node) -> bool:
    if isinstance(node, ast.Attribute) and node.attr == "version_info":
        return True
    if isinstance(node, ast.Name) and node.id == "version_info":
        return True
    if isinstance(node, ast.Subscript):
        return _is_version_info(node.value)
    return False


def _const_version(node):
    if isinstance(node, ast.Tuple):
        out = []
        for e in node.elts:
            if isinstance(e, ast.Constant) and isinstance(e.value, int):
                out.append(e.value)
            else:
                return None
        return tuple(out)
    if isinstance(node, ast.Constant) and isinstance(node.value, int):
        return (node.value,)
    return None


_VSWAP = {ast.Lt: ast.Gt, ast.Gt: ast.Lt, ast.LtE: ast.GtE, ast.GtE: ast.LtE,
          ast.Eq: ast.Eq, ast.NotEq: ast.NotEq}


def _eval_version_test(test):
    """True/False if `test` is a resolvable `sys.version_info` comparison under
    _PYVER, else None (unsure → both branches apply)."""
    if not isinstance(test, ast.Compare) or len(test.ops) != 1:
        return None
    left, op, right = test.left, type(test.ops[0]), test.comparators[0]
    if _is_version_info(left):
        ver = _const_version(right)
    elif _is_version_info(right):
        ver = _const_version(left)
        op = _VSWAP.get(op)
    else:
        return None
    if ver is None or op is None:
        return None
    a = _PYVER[:len(ver)]
    return {ast.GtE: a >= ver, ast.Gt: a > ver, ast.LtE: a <= ver,
            ast.Lt: a < ver, ast.Eq: a == ver, ast.NotEq: a != ver}.get(op)


def _branch_v312(node, v312):
    """For an `ast.If`, yield (branch_body, v312_applicable) for body & orelse.
    A body is 3.12-applicable iff the guard is not statically False under 3.12
    (and the enclosing context is applicable); orelse iff not statically True."""
    res = _eval_version_test(node.test)
    yield node.body, (v312 and res is not False)
    yield node.orelse, (v312 and res is not True)


def _sig_row(mod, qualifier, name, kind, params, has_star, overloaded, v312=True):
    """Build a serializable signature row dict for the Rust table.

    An *all-scalar* star-free single-signature row is enforced in full. A row
    whose LEADING params are scalar but which is then interrupted by a non-scalar
    param (Unknown/bool/bytes/None) is still enforceable on that leading scalar
    PREFIX only: we emit the prefix as the row's params (so the emitted
    enforceable row stays all-scalar, satisfying the `stdlib_sigs.rs` invariant)
    and the hook checks exactly those leading positions. Overloaded names and
    `*args` rows are never enforceable (which overload / where does positional
    alignment end?). A row with no leading scalar param is non-enforceable and
    keeps its full param list for documentation."""
    # A sig is enforceable if it has ANY concrete-scalar param (not just a
    # LEADING run) and is neither `*args` nor overloaded. The ① hook walks params
    # in positional order and SKIPS Unknown params (`core_ty_to_type_id` -> None)
    # while still advancing the index, so a scalar param sitting BEHIND an Unknown
    # param (`f(a: Unknown, b: str)`) is enforced correctly at its real position.
    # Overloaded names stay non-enforceable — which overload's param types apply?
    # (the `logging.getLevelName` int|str overload is exactly the false-positive
    # hazard a wholesale enforce would hit). Emit the FULL param list so the hook
    # aligns positions past the skipped Unknowns.
    # A scalar (Int/Float/Str) param is checked by value; a `Typed` protocol
    # param is checked by the bare-class rule. Either makes the sig enforceable.
    has_checkable = any(ct in ("Int", "Float", "Str", "Typed") for _name, ct in params)
    enforceable = (not has_star) and (not overloaded) and has_checkable
    emitted_params = params
    return dict(
        module=mod,
        qualifier=qualifier,
        name=name,
        kind=kind,          # "ModuleFn" | "Method"
        params=emitted_params,  # list[(name, core_ty)] — prefix if truncated
        has_star=has_star,
        enforceable=enforceable,
        v312=v312,
    )


def _walk_class_rust(body, mod, cls, counts, v312=True):
    """Yield method signature rows from a class body (recurse into If-version /
    nested classes). Dunder + public methods become `Method` rows keyed on the
    top-level class name. `counts[(scope, name)]` is consulted for overloads."""
    for m in body:
        if isinstance(m, (ast.FunctionDef, ast.AsyncFunctionDef)):
            decos = decorator_names(m)
            is_static = "staticmethod" in decos
            if m.name == "__init__":
                pass  # constructors are emitted as Method rows on the class
            elif m.name.startswith("_") and not (
                m.name.startswith("__") and m.name.endswith("__")
            ):
                continue  # single-underscore private
            params, has_star = _collect_params(m, drop_first=not is_static)
            overloaded = counts.get((f"{mod}::{cls}", m.name), 0) >= 2
            yield _sig_row(mod, cls.split(".")[0], m.name, "Method",
                           params, has_star, overloaded, v312)
        elif isinstance(m, ast.ClassDef):
            if not m.name.startswith("_"):
                yield from _walk_class_rust(m.body, mod, f"{cls}.{m.name}", counts, v312)
        elif isinstance(m, ast.If):
            for branch, bv in _branch_v312(m, v312):
                yield from _walk_class_rust(branch, mod, cls, counts, bv)


def _walk_module_rust(body, mod, counts, v312=True):
    """Yield module-level function rows + class method rows."""
    for node in body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if not node.name.startswith("_"):
                params, has_star = _collect_params(node, drop_first=False)
                overloaded = counts.get((mod, node.name), 0) >= 2
                yield _sig_row(mod, "", node.name, "ModuleFn",
                               params, has_star, overloaded, v312)
        elif isinstance(node, ast.ClassDef):
            if not node.name.startswith("_"):
                yield from _walk_class_rust(node.body, mod, node.name, counts, v312)
        elif isinstance(node, ast.If):
            for branch, bv in _branch_v312(node, v312):
                yield from _walk_module_rust(branch, mod, counts, bv)


def _count_defs(body, mod, scope, counts):
    """Pre-pass: count def occurrences per (scope, name) to detect overloads.

    scope = `mod` for module-level fns, `mod::Cls` for methods (same key the
    walkers use). A name with >=2 defs in a scope is an `@overload` chain (or a
    version-guarded redefinition) and is marked non-enforceable."""
    for node in body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            counts[(scope, node.name)] = counts.get((scope, node.name), 0) + 1
        elif isinstance(node, ast.ClassDef):
            _count_defs(node.body, mod, f"{mod}::{node.name}", counts)
        elif isinstance(node, ast.If):
            # Count only the branches APPLICABLE to 3.12: version-guarded variants
            # of one method live in mutually-exclusive branches and are NOT a
            # simultaneous overload, so counting both falsely disables enforcement.
            for branch, bv in _branch_v312(node, True):
                if bv:
                    _count_defs(branch, mod, scope, counts)


def rust_rows():
    """Yield every signature row across typeshed stdlib, deterministically."""
    global _CTX_EXCLUDE, _GLOBAL_EXCLUDE
    # One global pass first: cross-module typevars / `= Any` aliases that a
    # per-module scan cannot resolve at the use site.
    _GLOBAL_EXCLUDE = _collect_global_exclusions()
    for pyi in sorted(TYPESHED_STDLIB.rglob("*.pyi")):
        mod = module_name(pyi)
        try:
            tree = ast.parse(pyi.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        # Per-module typevar / wildcard-alias exclusions consumed by `core_ty_of`.
        _CTX_EXCLUDE = _collect_module_exclusions(tree)
        counts: dict[tuple[str, str], int] = {}
        _count_defs(tree.body, mod, mod, counts)
        yield from _walk_module_rust(tree.body, mod, counts)


def _rust_str(s: str) -> str:
    """Rust string literal escaping (module/class/param names are identifiers,
    but escape defensively)."""
    return s.replace("\\", "\\\\").replace('"', '\\"')


_ENF_SCALARS = {"Int", "Float", "Str"}


def merge_overload_params(rows):
    """Merge a real @overload chain (>=2 signatures applicable to 3.12 at once)
    into ONE enforceable row by AGREEMENT: a position is enforced only when
    EVERY overload has the SAME enforceable scalar CoreTy there. A valid call
    must satisfy some overload, but if all overloads demand the same scalar at a
    position, any concrete-scalar arg disjoint from it is wrong for all of them
    (provably false-positive-clean). Positions where the overloads disagree, or
    where any is non-scalar/Unknown, collapse to Unknown (which also ends the
    enforceable prefix the hook reads). Conservative on arity: only positions
    present in EVERY overload are considered."""
    base = dict(rows[0])
    param_lists = [r["params"] for r in rows]
    n = min(len(pl) for pl in param_lists)
    merged = []
    for i in range(n):
        ctys = {pl[i][1] for pl in param_lists}
        name = param_lists[0][i][0]
        if len(ctys) == 1 and next(iter(ctys)) in _ENF_SCALARS:
            merged.append((name, next(iter(ctys))))
        else:
            merged.append((name, "Unknown"))
    has_star = any(r.get("has_star") for r in rows)
    base["params"] = merged
    base["has_star"] = has_star
    base["enforceable"] = (not has_star) and any(ct in _ENF_SCALARS for _, ct in merged)
    return base


def render_rust() -> str:
    """Render the deterministic `STDLIB_SIGS_GENERATED` Rust source."""
    rows = list(rust_rows())
    # Deduplicate on (module, qualifier, name, kind), VERSION-AWARE: prefer the
    # signature applicable to Python 3.12 (`v312`). A method guarded into
    # different Python versions yields one row per branch; exactly one is the
    # 3.12 contract and stays authoritative (keeps its enforceable flag). Only a
    # GENUINE ambiguity — two or more signatures applicable to 3.12 at once (a
    # real `@overload` chain or platform variants) — is forced non-enforceable.
    # A method that exists only in a non-3.12 branch keeps its single sig (so its
    # fixture stays green) unless it too is ambiguous.
    by_key: dict[tuple, list] = {}
    for r in rows:
        key = (r["module"], r["qualifier"], r["name"], r["kind"])
        by_key.setdefault(key, []).append(r)
    seen: dict[tuple, dict] = {}
    for key, rs in by_key.items():
        v312_rows = [r for r in rs if r.get("v312", True)]
        if len(v312_rows) == 1:
            seen[key] = v312_rows[0]
        elif v312_rows:
            # Real @overload chain: enforce only positions ALL overloads agree on.
            seen[key] = merge_overload_params(v312_rows)
        elif len(rs) == 1:
            seen[key] = rs[0]
        else:
            row = dict(rs[0]); row["enforceable"] = False
            seen[key] = row
    ordered = [seen[k] for k in sorted(seen.keys())]

    n_total = len(ordered)
    n_enf = sum(1 for r in ordered if r["enforceable"])

    lines: list[str] = []
    lines.append(
        "//! GENERATED by tests/harness/cpython/tools/type_wall_gen.py --emit-rust.\n"
        "//! DO NOT EDIT BY HAND. Regenerate with:\n"
        "//!   python3.12 tests/harness/cpython/tools/type_wall_gen.py --emit-rust\n"
        "//!\n"
        "//! Typeshed-derived ① Type-wall stdlib signature table. Each row maps a\n"
        "//! stdlib callable's positional params to the closed [`CoreTy`] scalar set\n"
        "//! (int/float/str/bytes/bool/None); everything richer is `CoreTy::Unknown`.\n"
        "//! A row is `enforceable` only when it is non-overloaded, star-free, and\n"
        "//! every positional param is a concrete scalar — otherwise the hook skips\n"
        "//! it (skip-when-unsure, zero false positives on correct calls).\n"
        f"//!\n"
        f"//! rows: {n_total}  ·  enforceable (scalar): {n_enf}  ·  "
        f"unknown-skipped: {n_total - n_enf}\n"
    )
    lines.append("")
    lines.append("use super::stdlib_sigs::{CoreTy, ParamSig, SigKind, StdlibSig};")
    lines.append("")
    lines.append("const fn p(name: &'static str, ty: CoreTy, star: bool) -> ParamSig {")
    lines.append("    ParamSig { name, ty, star }")
    lines.append("}")
    lines.append("")
    lines.append("/// Typeshed-derived stdlib signatures. See module docs.")
    lines.append("pub const STDLIB_SIGS_GENERATED: &[StdlibSig] = &[")
    for r in ordered:
        kind = "SigKind::ModuleFn" if r["kind"] == "ModuleFn" else "SigKind::Method"
        if r["params"] or r["has_star"]:
            parts = [
                f'p("{_rust_str(n)}", CoreTy::{ct}, false)'
                for (n, ct) in r["params"]
            ]
            if r["has_star"]:
                parts.append('p("args", CoreTy::Unknown, true)')
            params_src = "&[" + ", ".join(parts) + "]"
        else:
            params_src = "&[]"
        enf = "true" if r["enforceable"] else "false"
        lines.append("    StdlibSig {")
        lines.append(f'        module: "{_rust_str(r["module"])}",')
        lines.append(f'        qualifier: "{_rust_str(r["qualifier"])}",')
        lines.append(f'        name: "{_rust_str(r["name"])}",')
        lines.append(f"        kind: {kind},")
        lines.append(f"        params: {params_src},")
        lines.append(f"        enforceable: {enf},")
        lines.append("    },")
    lines.append("];")
    lines.append("")
    return "\n".join(lines)


def emit_rust(check: bool) -> int:
    text = render_rust()
    if check:
        if not RUST_SIGS_OUT.exists():
            print(f"MISSING: {RUST_SIGS_OUT} (run --emit-rust)")
            return 1
        current = RUST_SIGS_OUT.read_text(encoding="utf-8")
        if current != text:
            print(f"STALE: {RUST_SIGS_OUT} differs from --emit-rust output")
            return 1
        print(f"OK: {RUST_SIGS_OUT} is byte-for-byte up to date")
        return 0
    RUST_SIGS_OUT.write_text(text, encoding="utf-8")
    n_total = text.count("    StdlibSig {")
    n_enf = text.count("        enforceable: true,")
    print(f"wrote {RUST_SIGS_OUT}  ({n_total} sigs, {n_enf} enforceable)")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--module", help="only this dotted module")
    ap.add_argument("--kind", action="append",
                    choices=["module", "init", "smethod", "method"])
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--emit-rust", action="store_true",
                    help="(re)write src/types/stdlib_sigs_generated.rs")
    ap.add_argument("--check-rust", action="store_true",
                    help="assert stdlib_sigs_generated.rs is byte-for-byte current")
    args = ap.parse_args()

    # Additive Rust-table modes. Disjoint from fixture generation above.
    if args.check_rust:
        return emit_rust(check=True)
    if getattr(args, "emit_rust", False):
        return emit_rust(check=False)

    kinds = set(args.kind) if args.kind else {"module", "init", "smethod", "method"}
    rows = list(candidates(kinds))
    if args.module:
        rows = [r for r in rows if r["mod"] == args.module]

    if args.dry_run or not (args.write or args.module):
        by_kind: dict[str, int] = {}
        n_sentinel = 0
        for r in rows:
            k = str(r["kind"])
            by_kind[k] = by_kind.get(k, 0) + 1
            arglist = r["args"]
            if isinstance(arglist, list) and SENTINEL in arglist:
                n_sentinel += 1
        print(f"generable type-wall cases: {len(rows)}  (sentinel-typed: {n_sentinel})")
        for k, n in sorted(by_kind.items(), key=lambda kv: -kv[1]):
            print(f"  {n:5d}  {k}")
        return 0

    written = 0
    seen: set[str] = set()
    for c in rows:
        rel, text = render(c)
        if rel in seen:
            continue
        seen.add(rel)
        path = OUT_DIR / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        written += 1
    print(f"wrote {written} type-wall cases under {OUT_DIR}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
