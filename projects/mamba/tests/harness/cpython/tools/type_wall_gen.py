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
import json
import os
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from wall_gen_core import PEP723Header  # noqa: E402

MAMBA_DIR = Path(__file__).resolve().parents[4]
DEFAULT_TYPESHED_STDLIB = MAMBA_DIR / "vendor" / "typeshed" / "stdlib"
TYPESHED_STDLIB = Path(
    os.environ.get("MAMBA_TYPESHED_STDLIB", DEFAULT_TYPESHED_STDLIB)
).resolve()
OUT_DIR = MAMBA_DIR / "tests" / "cpython" / "type"
# --emit-rust output: the typeshed-derived stdlib signature table consumed by
# src/types/stdlib_sigs.rs (the ① Type-wall call-site hook).
RUST_SIGS_OUT = MAMBA_DIR / "src" / "types" / "stdlib_sigs_generated.rs"
RUST_SPECS_OUT = MAMBA_DIR / "src" / "types" / "stdlib_specs_generated.rs"
RUST_SPECS_DATA_OUT = MAMBA_DIR / "src" / "types" / "stdlib_specs_generated.json"

# Closed scalar map: the ONLY typeshed BARE (non-subscripted) annotations we
# encode as a concrete, *enforceable* CoreTy. EVERYTHING else (Any, object,
# unions, Optional, typevars, generics, forward-refs, protocols, path-likes,
# Sequence/Iterable/Callable, …) collapses to CoreTy::Unknown so the hook
# skips-when-unsure (zero false positives). `list`/`tuple`/`dict`/`type` are
# almost always SUBSCRIPTED in typeshed (`list[str]`, `type[C]`); those forms
# are mapped separately by `_CONTAINER_SUBSCRIPT_CORE_TY` below, to the same
# CoreTy their bare identifier maps to here (the type ARGUMENT is irrelevant
# to a negative scalar wall).
#
# bytes/bool/complex/list/tuple/dict/type each already have a dedicated
# `CoreTy` variant that `check_expr.rs`'s ① hook enforces: Bytes/Complex/
# List/Tuple/Dict/Type as a *negative scalar wall* (reject a concrete
# int/float/str/bool/None actual that can provably never be that type; skip
# every richer/dynamic value), and Bool via the ordinary scalar-compatibility
# rule (mamba is force-typed, so a genuinely wrong scalar — a str/int literal
# fed to a `bool` param — MUST raise even where CPython's duck typing would
# accept it; see `types_compatible`'s Bool<->Int promotion, which is
# intentionally one-directional).
#
# `memoryview` stays mapped to Unknown (no positive mapping wired here — out
# of this generator pass's scope) and so does a bare `None` annotation (a
# vanishingly rare real positional contract, never worth a row).
SCALAR_CORE_TY = {
    "int": "Int",
    "float": "Float",
    "str": "Str",
    "bytes": "Bytes",
    "bool": "Bool",
    "complex": "Complex",
    "list": "List",
    "tuple": "Tuple",
    "dict": "Dict",
    "type": "Type",
}

# Subscripted container/type-object annotations (`list[str]`, `dict[str,
# int]`, `type[C]`) map to the SAME CoreTy as their bare form above — the
# type argument is irrelevant to a negative scalar wall (a concrete
# int/float/str/bool/None actual can never BE a list, no matter what it would
# hold). Consulted from the `ast.Subscript` arm of `core_ty_of`, mirroring how
# `_SUBSCRIPT_TYPED_BASES` (IO/TextIO/BinaryIO) already folds through
# `core_ty_of` regardless of its own subscript argument.
_CONTAINER_SUBSCRIPT_CORE_TY = {
    "list": "List",
    "tuple": "Tuple",
    "dict": "Dict",
    "type": "Type",
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
# maps EACH positional parameter to a closed CoreTy, and emits a deterministic
# `const STDLIB_SIGS_GENERATED` consumed by src/types/stdlib_sigs.rs. The
# guardrails (skip-when-unsure, Bool->Int / Int->Float allow, stop-at-star) all
# live in the Rust hook and are unchanged; this table only declares the
# contract, and is deliberately conservative:
#   * ANY *args / *  param         -> enforceable = false (alignment uncertain)
#   * SINGLE signature row         -> enforceable if any param is not Unknown
#   * OVERLOADED 3.12 row set      -> keep identical always-typed CoreTy,
#                                     else fold always-typed positions to Typed
#   * NO checkable param           -> enforceable = false (nothing to check)
# The row is still EMITTED in every case (documented negative / skip), exactly
# like the PoC's b64encode/factorial guards — the hook reads `enforceable`.
#
# #887: each row ALSO carries `ret` — the callable's typeshed RETURN
# annotation, closed to the same positive concrete scalars `check.rs`'s
# `core_ty_to_type_id` maps to a real `Ty` (`Int`/`Float`/`Str`/`Bool`/
# `None`; see `return_core_ty_of`). `ret` is INDEPENDENT of `enforceable`
# (returns are fed into inference regardless of whether the call's ARGUMENTS
# are enforceable — a zero-arg call like `os.getcwd()` is never enforceable
# but its `str` return still matters) and deliberately narrower than the
# param-position `CoreTy` map (no Bytes/List/Tuple/Dict/Type/Typed — those
# are negative-scalar-wall concepts with no positive `Ty`, and class-typed
# returns are out of scope; see the #887 issue thread).
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


# #882: `CoreTy::TypedNamed("<contract>")` carries a POSITIVE structural
# predicate (see `check_expr.rs`) instead of `Typed`'s bare-class-only
# rejection. `core_ty_of` deliberately does NOT recognize PathLike/
# SupportsIndex-family names as `TypedNamed` structurally/typeshed-wide —
# an earlier version of this generator did, and regenerating flipped ~340
# rows (every typeshed `SupportsIndex`/`StrPath`/`PathLike`-annotated param
# in the stdlib) from `Typed` to `TypedNamed`, which broke currently-green
# `errors/`-dimension fixtures that rely on a RUNTIME-catchable `TypeError`
# for the exact same call shape (confirmed concretely via
# `errors/std-libs/math/factorial_float_typeerror.py` — `math.factorial`'s
# `SupportsIndex` param — and `errors/std-libs/os/fspath_int_raises_typeerror.py`
# — `os.fspath`'s `PathLike`-family param — both otherwise-passing today via
# mamba's runtime validation, both would hard-fail to compile with the
# blanket rollout). `TypedNamed` is instead applied ONLY via the explicit,
# individually-reviewed `_PARAM_CORE_TY_OVERRIDE` allowlist below — the same
# compile-time-vs-runtime-catchable-TypeError precedent as `_L3_FOLD_EXCLUDE`,
# applied opt-in instead of opt-out since the new predicate is strictly
# sharper than the pre-#882 wall it can replace.


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


# Per-module conservative name->CoreTy map consumed by `core_ty_of`: typevars,
# PEP 695 type params, and aliases that resolve to a wildcard type. Most names
# stay `"Unknown"`; constrained/bound `TypeVar(...)` definitions can narrow to a
# non-Unknown wall. Set by `rust_rows` before walking each `.pyi`.
_CTX_NAME_CORE: dict[str, str] = {}
# GLOBAL conservative name->CoreTy map across ALL stubs so imported/global uses
# of typevars or wildcard aliases resolve too. Duplicate names only stay
# non-Unknown when every observed definition agrees on the same wall.
_GLOBAL_NAME_CORE: dict[str, str] = {}


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


def _merge_name_core(dst: dict[str, str], name: str, core: str) -> None:
    """Conservative merge: duplicate spellings only keep a non-Unknown wall
    when every observed definition agrees on it exactly."""
    prev = dst.get(name)
    if prev is None:
        dst[name] = core
    elif prev != core:
        dst[name] = "Unknown"


def _name_core_override(node: ast.expr, name_core: dict[str, str]) -> str | None:
    """Resolve a simple name/attr through a conservative local map."""
    if isinstance(node, ast.Name):
        return name_core.get(node.id)
    if isinstance(node, ast.Attribute):
        return name_core.get(node.attr)
    return None


def _typevar_call_core(node: ast.Call, *, name_core: dict[str, str] | None = None) -> str:
    """Conservative wall for `TypeVar(...)` definitions.

    Rules for #840:
    - unconstrained TypeVar / ParamSpec / TypeVarTuple stay Unknown
    - `TypeVar("T", A, B, ...)` with >=2 constraints becomes Typed only when
      every constraint itself maps non-Unknown
    - `TypeVar("T", bound=X)` inherits `core_ty_of(X)` only when that bound is
      non-wildcard and non-Unknown (e.g. `bound=Hashable` stays Unknown)
    """
    func_name = (
        node.func.id if isinstance(node.func, ast.Name)
        else node.func.attr if isinstance(node.func, ast.Attribute)
        else ""
    )
    if func_name != "TypeVar":
        return "Unknown"
    constraints = node.args[1:]
    if len(constraints) >= 2:
        def _constraint_core(arg: ast.expr) -> str:
            if name_core is not None:
                override = _name_core_override(arg, name_core)
                if override is not None:
                    return override
            return core_ty_of(arg)
        return (
            "Typed"
            if all(_constraint_core(arg) != "Unknown" for arg in constraints)
            else "Unknown"
        )
    for kw in node.keywords:
        if kw.arg == "bound" and not _expr_is_wildcard(kw.value):
            override = None if name_core is None else _name_core_override(kw.value, name_core)
            ct = override if override is not None else core_ty_of(kw.value)
            if ct != "Unknown":
                return ct
    return "Unknown"


def _collect_module_name_core(tree: ast.Module) -> dict[str, str]:
    """Names defined in THIS module that `core_ty_of` should resolve
    conservatively: TypeVar-family targets, PEP 695 `type_params`, and aliases
    that resolve to a wildcard (`Unused = object`, `_X: TypeAlias = Any | int`).
    Missing one only risks a false positive, so collection is structural and
    backed by the `_typevar_convention` fallback for imported typevars."""
    name_core: dict[str, str] = {}
    typevar_assigns: list[tuple[str, ast.Call]] = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign):
            val = node.value
            is_tv = (
                isinstance(val, ast.Call)
                and isinstance(val.func, ast.Name)
                and val.func.id in ("TypeVar", "ParamSpec", "TypeVarTuple")
            )
            if is_tv:
                for t in node.targets:
                    if isinstance(t, ast.Name):
                        typevar_assigns.append((t.id, val))
            elif _expr_is_wildcard(val):
                for t in node.targets:
                    if isinstance(t, ast.Name):
                        _merge_name_core(name_core, t.id, "Unknown")
        elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            if node.value is not None and _expr_is_wildcard(node.value):
                _merge_name_core(name_core, node.target.id, "Unknown")
        elif isinstance(node, ast.TypeAlias) and isinstance(node.name, ast.Name):
            if _expr_is_wildcard(node.value):
                _merge_name_core(name_core, node.name.id, "Unknown")
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            for tp in getattr(node, "type_params", []):  # PEP 695 def f[T] / class C[T]
                _merge_name_core(name_core, tp.name, "Unknown")
    for name, val in typevar_assigns:
        _merge_name_core(name_core, name, _typevar_call_core(val, name_core=name_core))
    return name_core


def _collect_global_name_core() -> dict[str, str]:
    """Conservative union of `_collect_module_name_core` across EVERY typeshed stdlib stub.

    A typevar or `= Any` alias defined in module A is frequently imported and
    used in module B's signatures; B's local scan cannot see A's definition, so
    without a global pass those names wrongly map to `Typed` and would reject a
    valid bare class (a real false positive an adversarial typeshed audit caught:
    `_StrPathT`, `_TC`, `_RetAddress`, `AnnotationForm`, …). A global name only
    keeps a non-Unknown wall when every observed definition agrees; otherwise it
    degrades to Unknown, which only costs a conservative MISS."""
    g: dict[str, str] = {}
    for pyi in TYPESHED_STDLIB.rglob("*.pyi"):
        try:
            tree = ast.parse(pyi.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        for name, ct in _collect_module_name_core(tree).items():
            _merge_name_core(g, name, ct)
    return g


# Subscripted bases (besides protocols/ABCs) a BARE no-base no-method class can
# never inhabit, regardless of the type argument: `IO`/`TextIO`/`BinaryIO` (a
# class instance is not an open stream).
#
# `type[...]` is DELIBERATELY EXCLUDED from THIS allowlist (mapped to
# `CoreTy::Type` by `_CONTAINER_SUBSCRIPT_CORE_TY` instead — see `core_ty_of`
# L3a, above L3b): a CLASS OBJECT is a valid `type`, and mamba's checker types
# a bare class VALUE (`C`) identically to an instance (`C()`) — both
# `Ty::Class{name}` — so `webbrowser.register('x', ExampleBrowser)` /
# `copyreg.pickle(C, ...)` / `inspect.unwrap(C)` / `dataclasses.fields(@dataclass
# class)` all pass the class itself, which CPython accepts, and which the hook
# must not reject. `CoreTy::Type`'s bare-instance rejection only fires on a
# CALL-shaped actual (`_W()`), never a bare class name reference (`C`), so it
# stays false-positive-clean for exactly these calls (verified by the ②
# behavior gate — this comment used to warn of 9 real FPs from an earlier,
# less precise bare-class detector). `Callable[...]` is still excluded — a
# class object is callable too, and there is no `CoreTy::Callable`.
# `tuple`/`list`/`dict`/`set` protocol/ABC bases are also excluded from THIS
# allowlist: `list`/`tuple`/`dict` map to their own CoreTy via
# `_CONTAINER_SUBSCRIPT_CORE_TY`; `set` has no CoreTy — a bare class is not
# one anyway, so it stays Unknown.
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


# #884: closed scalar-union folds for a PURE-scalar union (every non-`None`
# member itself maps to one of the concrete `SCALAR_CORE_TY` variants, never a
# nominal `Typed` contract). Each entry reuses a CoreTy variant `check_expr.rs`
# already enforces for its OWN sake — no new CoreTy/enforcement code needed —
# keyed by the set of concrete-scalar ACTUAL kinds
# (`Int`/`Float`/`Str`/`Bool`) it accepts, per `check_stdlib_scalar_arg` /
# `types_compatible`'s one-directional Bool->Int->Float promotion:
#   CoreTy::Bool     accepts {Bool}
#   CoreTy::Str      accepts {Str}
#   CoreTy::Int      accepts {Int, Bool}
#   CoreTy::Float    accepts {Int, Float, Bool}
#   CoreTy::IntOrStr accepts {Int, Bool, Str}
# Ordered smallest accept-set first so the fold picks the MOST PRECISE variant
# whose accept-set safely covers the union's concrete members (never rejects a
# value the union actually declares valid).
_SCALAR_UNION_FOLD: list[tuple[str, frozenset[str]]] = [
    ("Bool", frozenset({"Bool"})),
    ("Str", frozenset({"Str"})),
    ("Int", frozenset({"Int", "Bool"})),
    ("Float", frozenset({"Int", "Float", "Bool"})),
    ("IntOrStr", frozenset({"Int", "Bool", "Str"})),
]

# CoreTy variants whose `check_expr.rs` enforcement is a *negative scalar
# wall*: reject EVERY concrete-scalar actual (Int/Float/Str/Bool) unconditionally,
# with no promotion exception. They are therefore mutually interchangeable as a
# fold target whenever a pure-scalar union has NO Int/Float/Str/Bool member (a
# union of e.g. `list[...] | tuple[...]` or `tuple[...] | type[...]`) — any
# member present is an exact-safe representative. Ordered by observed typeshed
# prevalence for a stable, deterministic pick.
_NEGWALL_CORE_TY = ("Tuple", "List", "Dict", "Type", "Bytes")

# #884: known-runtime-catchable exceptions to the L3 scalar-union fold. Each
# entry is a (module, qualifier, function-name, param-name) whose typeshed
# annotation IS a pure scalar union (folds to a concrete CoreTy under
# `_union_core_ty`, e.g. `str | None` -> `CoreTy::Str`) but whose REAL
# implementation performs its OWN runtime type check and raises a CATCHABLE
# `TypeError` that fixtures rely on catching at RUNTIME via `try/except`.
# Folding these turns the ① hook into a compile-time-uncatchable wall, which
# aborts the whole compile before the `try/except` ever runs -- flipping a
# PASS (oracle raises+catches TypeError) into a hard compile FAIL. Confirmed
# via `errors/std-libs/logging/getlogger_int_name_raises.py`
# (`logging.getLogger(123)`: typeshed `name: str | None` folds to
# `CoreTy::Str`, but CPython's real `Logger.manager.getLogger` runs its own
# `isinstance(name, str)` check and raises `TypeError`, which the fixture
# expects to observe and catch). Pinned back to the pre-#884 `Typed` wall
# (bare-instance-only rejection) for exactly this param, so the runtime path
# stays reachable -- mirrors this codebase's `strict_keyword_wall` precedent
# in `check_expr.rs` for the identical compile-time-vs-runtime tension.
#
# Scoped to EXACTLY the confirmed case: `logging.Manager.getLogger` (the
# method) has NO evidence of the same runtime-catchable path -- its own
# `type/std-libs/logging/Manager__getLogger__name_as_str_wrong.py` fixture
# (a `# mamba-strict-type:` conformance case) actively WANTS the strict
# fold, and excluding it there breaks that fixture (MAMBA_TYPE_LEAKED) for
# no offsetting benefit -- so only the module-level function is excluded.
_L3_FOLD_EXCLUDE: frozenset[tuple[str, str, str, str]] = frozenset({
    ("logging", "", "getLogger", "name"),
})

# Hand-pinned param CoreTy overrides that predate #887 and are NOT expressible
# via `_L3_FOLD_EXCLUDE` (which only ever forces `Typed`). Same
# compile-time-vs-runtime-catchable-TypeError rationale as `_L3_FOLD_EXCLUDE`,
# but the target CoreTy differs per entry, so this is a direct
# (module, qualifier, name, param) -> CoreTy override applied unconditionally
# (regardless of the naturally-computed `ct`, including the `_L3_FOLD_EXCLUDE`
# pass above) right before the enforceable/emit decision:
#
# - `sys.setswitchinterval.interval`: typeshed's bare `float` annotation would
#   compute `CoreTy::Float`, but CPython's real implementation performs its own
#   runtime check and raises a CATCHABLE `TypeError`/`ValueError` for a
#   non-float `interval` (see `errors/std-libs/sys/setswitchinterval_str_raises.py`
#   and `setswitchinterval_zero_raises.py`) that a full-`Float` compile-time wall
#   would make unreachable. Pinned to `Typed` (bare-instance-only rejection).
# - `linecache.getline.lineno`: typeshed's bare `int` would compute
#   `CoreTy::Int`, but real callers pass runtime-validated values whose
#   mistyped case must stay reachable at runtime. Pinned to `Unknown` (skip).
# - `msilib.change_sequence.seq`: typeshed's `Sequence[...]`-ish annotation
#   would compute `CoreTy::Typed`, but `type/std-libs/msilib/
#   change_sequence__seq_as_Sequence_wrong.py` is an active `xfail` — mamba
#   does not yet enforce this contract. Pinned to `Unknown` (skip).
# - #882 `CoreTy::TypedNamed("<contract>")` seeds: `chr`/`hex`/`oct`/`bin`'s
#   `SupportsIndex` param is pinned to the tagged contract rather than
#   typeshed's plain `Typed` fold. All four are individually verified safe:
#   `check_expr.rs`'s pre-existing hardcoded `index_protocol_ok` check
#   already compile-time-rejects a non-`SupportsIndex` actual for exactly
#   these four builtins today (this override just gives Path A the same
#   positive predicate Path B already enforces — additive, not new
#   rejections), and no `errors/`/`behavior`-dimension fixture calls any of
#   them with a literal wrong-typed scalar expecting a runtime catch (only
#   bare-class shapes, e.g. `type/builtin-libs/builtins/
#   chr__i_as_SupportsIndex_wrong.py`, which already passes today via the
#   pre-existing `Typed` bare-class rule). `os.fspath.path` is pinned on ALL
#   THREE of its `@overload` branches (`str`/`bytes`/`PathLike[AnyStr]`) to
#   the SAME tag so `merge_overload_params` trivially agrees and folds them
#   into one `TypedNamed("PathLike")` row — the literal #882 AC1 case
#   (`os.fspath(42)` rejected; `os.fspath('x')`/`os.fspath(b'x')` accepted).
#   Note this DOES retire the runtime-only path
#   `errors/std-libs/os/fspath_int_raises_typeerror.py` relied on (that
#   fixture's `os.fspath(42)` now hard-fails to compile instead of raising at
#   runtime) — an explicit, reviewed, single-fixture consequence of AC1
#   itself ("os.fspath(42) rejected at type-check"), not a false positive:
#   42 is genuinely never path-like. Deliberately NOT rolled out beyond these
#   5 rows — see the `TypedNamed` note just below `_is_protocol_name` for why
#   a typeshed-wide rollout regresses other currently-green `errors/` fixtures.
_PARAM_CORE_TY_OVERRIDE: dict[tuple[str, str, str, str], str] = {
    ("sys", "", "setswitchinterval", "interval"): "Typed",
    ("linecache", "", "getline", "lineno"): "Unknown",
    ("msilib", "", "change_sequence", "seq"): "Unknown",
    ("builtins", "", "chr", "i"): "TypedNamed:SupportsIndex",
    ("builtins", "", "hex", "number"): "TypedNamed:SupportsIndex",
    ("builtins", "", "oct", "number"): "TypedNamed:SupportsIndex",
    ("builtins", "", "bin", "number"): "TypedNamed:SupportsIndex",
    ("os", "", "fspath", "path"): "TypedNamed:PathLike",
}

# #887 regen fallout: typeshed renamed the public `typing_extensions.Sentinel`
# class to a lowercase `sentinel` implementation class, re-exported via a
# module-level `Sentinel = sentinel` alias assignment (PEP 661, guarded by a
# `sys.version_info >= (3, 15)` branch that swaps in `from builtins import
# sentinel as sentinel` on 3.15+). This walker only understands `ClassDef` /
# `FunctionDef` nodes, not alias `Assign` statements, so the class rows land
# under qualifier `"sentinel"` and the public constructor call site
# (`from typing_extensions import Sentinel; Sentinel(...)`, matched by the
# call-site's literal imported name) no longer resolves. Rename the qualifier
# back to the real public name for this one class so the existing
# `type/std-libs/typing_extensions/Sentinel__init__name_as_str_wrong.py` wall
# stays reachable — this is a qualifier rename, not a scope change (#887 is
# only ret-modeling); a general alias-assignment walk is out of scope here.
_QUALIFIER_ALIAS: dict[tuple[str, str], str] = {
    ("typing_extensions", "sentinel"): "Sentinel",
}


def _union_core_ty(members: list[ast.expr]) -> str:
    """L2/L3 fold. Unknown the moment any non-`None` member is Unknown (a bare
    class inhabits the union iff it inhabits some member, so one wildcard /
    typevar / un-analyzable member opens the door and must abort). A lone
    `None` slice yields no checkable member -> Unknown.

    Otherwise: if any member is a nominal `Typed` contract, the union stays
    `Typed` (unchanged L2 fold — bare-instance-only rejection; folding a
    Typed-mixed union needs per-combination analysis out of this pass's
    scope). Else EVERY member is a concrete scalar (#884 L3): fold into the
    tightest existing closed scalar-union `CoreTy` whose accept-set safely
    covers every member (`_SCALAR_UNION_FOLD` / `_NEGWALL_CORE_TY`) instead of
    the weaker `Typed` bare-instance wall. A combination no existing variant
    can safely cover (e.g. `float | str` — no `FloatOrStr` variant exists)
    stays `Typed` rather than risk a false positive."""
    tys: set[str] = set()
    for m in members:
        if _is_none_member(m):
            continue
        ct = core_ty_of(m)
        if ct == "Unknown":
            return "Unknown"
        tys.add(ct)
    if not tys:
        return "Unknown"
    if "Typed" in tys:
        return "Typed"
    concrete = tys & {"Int", "Float", "Str", "Bool"}
    if not concrete:
        for name in _NEGWALL_CORE_TY:
            if name in tys:
                return name
        return "Typed"  # unreachable: SCALAR_CORE_TY has no other member
    for name, accept in _SCALAR_UNION_FOLD:
        if concrete <= accept:
            return name
    return "Typed"


def core_ty_of(node: ast.expr | None) -> str:
    """Map a typeshed annotation to a closed CoreTy variant name.

    Bare scalar builtins (int/float/str/bytes/bool/complex) and bare/subscripted
    `list`/`tuple`/`dict`/`type` map to a concrete CoreTy (see `SCALAR_CORE_TY`
    / `_CONTAINER_SUBSCRIPT_CORE_TY`). Every NOMINAL contract a bare `_W()`
    cannot inhabit — `Supports*`/`*Like` protocols, collection ABCs, and any
    other nominal class name or concrete alias — maps to `Typed` (the hook
    rejects a bare class instance against it). Wildcards (object/Any/Unused/
    Never/…), typevars (a bare class is a valid binding), and richer shapes
    whose membership needs analysis (other subscripted generics, unions) stay
    Unknown so the hook never enforces against them — skip-when-unsure."""
    if node is None:
        return "Unknown"
    if isinstance(node, ast.Name):
        ident = node.id
        if ident in SCALAR_CORE_TY:
            return SCALAR_CORE_TY[ident]
        if _is_protocol_name(ident):
            return "Typed"
        if ident in _CTX_NAME_CORE:
            return _CTX_NAME_CORE[ident]
        if ident in _GLOBAL_NAME_CORE:
            return _GLOBAL_NAME_CORE[ident]
        if ident in _WILDCARD_TYPES or "Any" in ident:
            return "Unknown"
        if _typevar_convention(ident):
            return "Unknown"
        # Nominal class name or alias to a concrete (non-wildcard) type: a bare
        # no-base no-method class instance can satisfy none of them.
        return "Typed"
    if isinstance(node, ast.Attribute):
        # e.g. `os.PathLike`, `ast.AST`, `types.FrameType`, `_typeshed.SupportsRead`.
        attr = node.attr
        if _is_protocol_name(attr):
            return "Typed"
        if attr in _CTX_NAME_CORE:
            return _CTX_NAME_CORE[attr]
        if attr in _GLOBAL_NAME_CORE:
            return _GLOBAL_NAME_CORE[attr]
        if attr in _WILDCARD_TYPES or "Any" in attr:
            return "Unknown"
        if _typevar_convention(attr):
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
        # A typevar / cross-module wildcard-alias base is excluded from BOTH
        # branches below (a bare class IS a valid typevar binding, so neither
        # the container scalar map nor the nominal-protocol allowlist may fire).
        if (
            not base_name
            or base_name in _CTX_NAME_CORE
            or base_name in _GLOBAL_NAME_CORE
            or _typevar_convention(base_name)
        ):
            return _CTX_NAME_CORE.get(base_name, _GLOBAL_NAME_CORE.get(base_name, "Unknown"))
        # ---- L3a: subscripted container/type-object -> its own negative
        # scalar-wall CoreTy. `list[str]`, `dict[str, int]`, `type[C]` — the
        # type ARGUMENT is irrelevant to the hook (a concrete int/float/str/
        # bool/None actual can never BE a list no matter what it would hold),
        # so every subscript of these bases carries the SAME CoreTy as its
        # bare form (`_CONTAINER_SUBSCRIPT_CORE_TY`, mirrors `SCALAR_CORE_TY`).
        if base_name in _CONTAINER_SUBSCRIPT_CORE_TY:
            return _CONTAINER_SUBSCRIPT_CORE_TY[base_name]
        # ---- L3b: non-inhabitable subscripted protocol -> Typed. -------------
        # `Iterable[str]`, `Sequence[int]`, `Callable[..., T]`, `IO[str]`,
        # `SupportsRead[bytes]`, `os.PathLike[str]` — the type ARG is
        # irrelevant: a bare no-base no-method class instance has no `__iter__`/
        # `__call__`/`write`, and is not an open stream, so it inhabits none of
        # these. The positive predicate is a CLOSED allowlist (the same
        # bare-class-can't-inhabit names the bare-`Name` path trusts —
        # `_ABC_PROTOCOLS` / `_is_protocol_name` = Supports*/`*Like`/`*Buffer` —
        # plus the closed IO set). Union/Optional bases handled above;
        # Sequence/Mapping bases stay Unknown.
        if (
            base_name in _ABC_PROTOCOLS
            or _is_protocol_name(base_name)
            or base_name in _SUBSCRIPT_TYPED_BASES
        ):
            return "Typed"
        return "Unknown"
    # Tuple, Constant, etc. — membership needs analysis, so skip.
    return "Unknown"


# #887: closed RETURN-position scalar map. Deliberately much narrower than
# `SCALAR_CORE_TY` (params): a return type is fed FORWARD into inference at
# every call site, so only the CoreTy variants `check.rs::core_ty_to_type_id`
# maps to a real concrete `Ty` (`Int`/`Float`/`Str`/`Bool`) are safe to emit —
# anything else has no positive `Ty` representation (`Bytes`/`Complex`/
# `List`/`Tuple`/`Dict`/`Type`/`Typed` are NEGATIVE scalar-wall concepts, used
# only to reject an impossible concrete arg, never to positively assert a
# result's type). `None` is handled separately below (a literal `-> None`
# return, not a bare annotation lookup).
_RETURN_SCALAR_CORE_TY = {
    "int": "Int",
    "float": "Float",
    "str": "Str",
    "bool": "Bool",
}


def return_core_ty_of(node: ast.expr | None) -> str:
    """Map a typeshed RETURN annotation to a closed, positively-assertable
    CoreTy: bare `int`/`float`/`str`/`bool` and a literal `-> None` return map
    to their concrete CoreTy (`core_ty_to_type_id` gives each of these a real
    `Ty`); every richer annotation (Optional, Union, generics, nominal
    classes/protocols, containers, or no annotation at all) stays `Unknown` so
    the call-site hook never feeds a speculative type into inference — only
    ADDS a concrete return type on a closed, false-positive-clean set. Class-
    typed returns (unlocking the Method-receiver factory pattern) are out of
    scope for this mapper; see #887's follow-up."""
    if node is None:
        return "Unknown"
    if isinstance(node, ast.Constant) and node.value is None:
        return "None"
    if isinstance(node, ast.Name) and node.id in _RETURN_SCALAR_CORE_TY:
        return _RETURN_SCALAR_CORE_TY[node.id]
    return "Unknown"


def _collect_params(fn: ast.FunctionDef | ast.AsyncFunctionDef, drop_first: bool):
    """Return (params, has_star) for a callable.

    `params` is a list of (name, core_ty) for the positional parameters
    (posonly + args), in order, with `self`/`cls` dropped for methods.
    `has_star` is True iff the callable has a `*args` (vararg): positional
    alignment past it is uncertain, so render marks one trailing star/Unknown
    row to document the unaligned tail."""
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


def _sig_row(mod, qualifier, name, kind, params, has_star, overloaded, v312=True, ret="Unknown"):
    """Build a serializable signature row dict for the Rust table.

    `qualifier` is passed through `_QUALIFIER_ALIAS` first (module-level
    `Public = _impl` re-export aliases this walker doesn't track structurally
    — see its docstring).

    An *all-scalar* single-signature row is enforced in full. A row whose fixed
    positional prefix has checkable params remains enforceable even when a
    trailing `*args` exists: the fixed prefix is still alignable, and render
    appends one trailing star/Unknown row so the variadic tail is documented but
    skipped. Overloaded names remain non-enforceable, including any merged row
    that carries a star. A row with no checkable fixed param is non-enforceable
    and keeps its full param list for documentation.

    `ret` (#887) is INDEPENDENT of `enforceable`: it is fed into the CALL
    EXPRESSION's inferred type at the call site regardless of whether the
    call's arguments are enforceable (a zero-arg call like `os.getcwd()` is
    never `enforceable`, but its `str` return still must flow into
    inference)."""
    # A single-signature sig is enforceable if its fixed positional prefix has
    # ANY checkable param; a trailing `*args` does not block enforcement of that
    # prefix because render appends an explicit star/Unknown tail row and the
    # hook stops there. Overloaded names stay non-enforceable — which overload's
    # param types apply? (the `logging.getLevelName` int|str overload is exactly
    # the false-positive hazard a wholesale enforce would hit). Emit the FULL
    # fixed param list so the hook aligns positions past skipped Unknowns.
    # A scalar (Int/Float/Str/Bytes/Bool/Complex/List/Tuple/Dict/Type) param is
    # checked by value (a negative scalar wall, or — Int/Float/Str/Bool — the
    # ordinary compatibility rule); a `Typed` protocol param is checked by the
    # bare-class rule. `core_ty_of` never emits anything else besides these and
    # `Unknown`, so "checkable" is exactly "not Unknown".
    qualifier = _QUALIFIER_ALIAS.get((mod, qualifier), qualifier)
    if params:
        params = [
            (pn, "Typed") if (mod, qualifier, name, pn) in _L3_FOLD_EXCLUDE and ct != "Unknown"
            else (pn, ct)
            for pn, ct in params
        ]
        params = [
            (pn, _PARAM_CORE_TY_OVERRIDE.get((mod, qualifier, name, pn), ct))
            for pn, ct in params
        ]
    has_checkable = any(ct != "Unknown" for _name, ct in params)
    enforceable = (not overloaded) and has_checkable
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
        ret=ret,
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
            # #887: `__init__` is ALWAYS typeshed-annotated `-> None` (Python's
            # own constructor contract), but a `Cls(...)` CALL actually
            # produces an instance of `Cls` — feeding the literal `None`
            # annotation into the call-site return type would be wrong (and
            # class-typed construction returns are out of this issue's
            # scope). Force `Unknown` for constructors specifically.
            ret = "Unknown" if m.name == "__init__" else return_core_ty_of(m.returns)
            yield _sig_row(mod, cls.split(".")[0], m.name, "Method",
                           params, has_star, overloaded, v312, ret=ret)
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
                               params, has_star, overloaded, v312,
                               ret=return_core_ty_of(node.returns))
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
    global _CTX_NAME_CORE, _GLOBAL_NAME_CORE
    # One global pass first: cross-module typevars / `= Any` aliases that a
    # per-module scan cannot resolve at the use site.
    _GLOBAL_NAME_CORE = _collect_global_name_core()
    for pyi in sorted(TYPESHED_STDLIB.rglob("*.pyi")):
        mod = module_name(pyi)
        try:
            tree = ast.parse(pyi.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        # Per-module typevar / wildcard-alias map consumed by `core_ty_of`.
        _CTX_NAME_CORE = _collect_module_name_core(tree)
        counts: dict[tuple[str, str], int] = {}
        _count_defs(tree.body, mod, mod, counts)
        yield from _walk_module_rust(tree.body, mod, counts)


# --- Lossless structured signature manifest ---------------------------------

_SPEC_BUILTINS = {
    "bool", "bytearray", "bytes", "complex", "dict", "float", "frozenset",
    "int", "list", "memoryview", "object", "range", "set", "slice", "str",
    "tuple", "type",
}
_SPEC_SPECIALS = {
    "Any", "AnyStr", "Callable", "ClassVar", "Concatenate", "Final",
    "Generic", "Literal", "LiteralString", "Never", "NoReturn", "NotRequired",
    "ParamSpec", "Protocol", "Required", "Self", "TypeAlias", "TypeGuard",
    "TypeIs", "TypeVar", "TypeVarTuple", "Unpack",
}
_SPEC_TYPE_PARAM_CTORS = {"TypeVar", "TypeVarTuple", "ParamSpec"}


def _spec_dotted_name(node):
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        base = _spec_dotted_name(node.value)
        return f"{base}.{node.attr}" if base else None
    return None


def _spec_span(node, source):
    if node is None:
        return dict(path=source, line=0, column=0, end_line=0, end_column=0)
    return dict(
        path=source,
        line=getattr(node, "lineno", 0),
        column=getattr(node, "col_offset", 0),
        end_line=getattr(node, "end_lineno", 0) or 0,
        end_column=getattr(node, "end_col_offset", 0) or 0,
    )


def _spec_resolve_relative_module(mod, is_package, level, imported):
    if level == 0:
        return imported or ""
    base = mod.split(".") if is_package else mod.split(".")[:-1]
    remove = max(0, level - 1)
    if remove:
        base = base[:-remove]
    if imported:
        base.extend(imported.split("."))
    return ".".join(base)


def _spec_type_param_call(node, info):
    if not isinstance(node, ast.Call):
        return None
    name = _spec_dotted_name(node.func)
    if not name:
        return None
    root, *tail = name.split(".")
    resolved_module = ""
    resolved_name = name
    if root in info["imports"]:
        resolved_module, imported = info["imports"][root]
        resolved_name = ".".join(
            piece for piece in ([imported] if imported else []) + tail if piece
        )
    elif len(tail) == 1 and root in {"typing", "typing_extensions"}:
        resolved_module = root
        resolved_name = tail[0]
    elif not tail:
        resolved_name = root
    if (
        resolved_name in _SPEC_TYPE_PARAM_CTORS
        and resolved_module in {"", "typing", "typing_extensions"}
    ):
        return resolved_name
    return None


def _spec_type_alias_annotation(node):
    name = _spec_dotted_name(node)
    return bool(name and name.split(".")[-1] == "TypeAlias")


def _spec_string_items(node):
    if not isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        return None
    values = []
    for item in node.elts:
        if not isinstance(item, ast.Constant) or not isinstance(item.value, str):
            return None
        values.append(item.value)
    return values


def _spec_method_only_body(body):
    """Whether every class-body member is losslessly represented as a method."""
    for node in body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            decorators = decorator_names(node)
            if decorators & {"classmethod", "staticmethod", "property", "setter"}:
                return False
            if node.name.startswith("_") and not (
                node.name.startswith("__") and node.name.endswith("__")
            ):
                return False
            continue
        if isinstance(node, ast.Pass):
            continue
        if isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant):
            if node.value.value is Ellipsis or isinstance(node.value.value, str):
                continue
        if isinstance(node, ast.Assign):
            if (
                len(node.targets) == 1
                and isinstance(node.targets[0], ast.Name)
                and node.targets[0].id == "__slots__"
            ):
                continue
        if isinstance(node, ast.If):
            result = _eval_version_test(node.test)
            if result is None:
                return False
            active = node.body if result else node.orelse
            if not _spec_method_only_body(active):
                return False
            continue
        # Attributed and otherwise unmodelled members make the structural
        # inventory open. They may still be inspected, but never used to prove
        # protocol acceptance.
        return False
    return True


def _spec_scan_scope(info, body, qualifier="", v312=True):
    """Collect the class/type namespace visible under Python 3.12.

    Callable rows retain all guarded branches with an explicit ``py312`` bit,
    but classes and exports have no per-row availability field. Their namespace
    must therefore exclude statically inactive declarations at collection time.
    """
    if not v312:
        return
    for node in body:
        if isinstance(node, ast.Import) and not qualifier:
            for item in node.names:
                local = item.asname or item.name.split(".")[0]
                info["imports"][local] = (item.name, "")
        elif isinstance(node, ast.ImportFrom) and not qualifier:
            origin = _spec_resolve_relative_module(
                info["module"], info["is_package"], node.level, node.module
            )
            for item in node.names:
                if item.name == "*":
                    info["star_imports"].append(origin)
                    continue
                local = item.asname or item.name
                info["imports"][local] = (origin, item.name)
                if item.asname == item.name:
                    info["explicit_reexports"].add(local)
        elif isinstance(node, ast.ClassDef):
            class_qualifier = f"{qualifier}.{node.name}" if qualifier else node.name
            bases = {
                _spec_dotted_name(base.value if isinstance(base, ast.Subscript) else base)
                for base in node.bases
            }
            kind = "Protocol" if any(base and base.split(".")[-1] == "Protocol" for base in bases) else "Nominal"
            info["classes"][class_qualifier] = kind
            info["classes"].setdefault(node.name, kind)
            info["class_decls"].append(
                dict(
                    module=info["module"],
                    qualifier=class_qualifier,
                    name=node.name,
                    kind=kind,
                    bases=list(node.bases),
                    method_only_complete=_spec_method_only_body(node.body),
                    node=node,
                )
            )
            _spec_scan_scope(info, node.body, class_qualifier, v312)
        elif isinstance(node, (ast.Assign, ast.AnnAssign)):
            target = None
            value = node.value
            if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
                target = node.targets[0].id
            elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
                target = node.target.id
            if target is None:
                continue
            if not qualifier and target == "__all__":
                values = _spec_string_items(value)
                if values is not None:
                    info["all_names"] = set(values)
                continue
            ctor = _spec_type_param_call(value, info)
            if ctor:
                key = f'{info["module"]}::{qualifier}::{target}'
                info["type_param_decls"].append(
                    dict(key=key, name=target, kind=ctor, value=value, qualifier=qualifier)
                )
                info["type_params"][(qualifier, target)] = key
            elif isinstance(node, ast.AnnAssign) and _spec_type_alias_annotation(node.annotation):
                info["aliases"].add((qualifier, target))
                info["aliases"].add(("", target))
                info["alias_decls"].append(
                    dict(
                        key=f'{info["module"]}::{qualifier}::{target}',
                        module=info["module"],
                        qualifier=qualifier,
                        name=target,
                        value=value,
                    )
                )
        elif isinstance(node, ast.AugAssign) and not qualifier:
            if isinstance(node.target, ast.Name) and node.target.id == "__all__":
                values = _spec_string_items(node.value)
                if values is not None:
                    if info["all_names"] is None:
                        info["all_names"] = set()
                    info["all_names"].update(values)
        elif hasattr(ast, "TypeAlias") and isinstance(node, ast.TypeAlias):
            if not isinstance(node.name, ast.Name) or node.type_params:
                continue
            target = node.name.id
            info["aliases"].add((qualifier, target))
            info["aliases"].add(("", target))
            info["alias_decls"].append(
                dict(
                    key=f'{info["module"]}::{qualifier}::{target}',
                    module=info["module"],
                    qualifier=qualifier,
                    name=target,
                    value=node.value,
                )
            )
        elif isinstance(node, ast.If):
            for branch, branch_v312 in _branch_v312(node, v312):
                _spec_scan_scope(info, branch, qualifier, branch_v312)


def _spec_resolve_exports(infos, local):
    """Resolve public symbols through explicit and star re-exports."""
    info_by_module = {info["module"]: info for info in infos}
    resolved = dict(local)
    for _ in range(len(infos) + 1):
        candidates = {key: {target} for key, target in local.items()}
        for info in infos:
            allowed = info["all_names"]
            for name, (origin, imported) in info["imports"].items():
                if not imported or (
                    name not in info["explicit_reexports"]
                    and (allowed is None or name not in allowed)
                ):
                    continue
                target = resolved.get((origin, imported))
                if target is not None:
                    candidates.setdefault((info["module"], name), set()).add(target)
            for origin in info["star_imports"]:
                source = info_by_module.get(origin)
                source_all = source["all_names"] if source is not None else None
                for (module, name), target in resolved.items():
                    if module != origin:
                        continue
                    if source_all is not None:
                        if name not in source_all:
                            continue
                    elif name.startswith("_"):
                        continue
                    if allowed is not None and name not in allowed:
                        continue
                    candidates.setdefault((info["module"], name), set()).add(target)
        updated = {
            key: next(iter(targets))
            for key, targets in candidates.items()
            if len(targets) == 1
        }
        if updated == resolved:
            return resolved
        resolved = updated
    return resolved


def _spec_resolve_class_exports(infos):
    local = {}
    for info in infos:
        for decl in info["class_decls"]:
            if "." not in decl["qualifier"]:
                local[(info["module"], decl["name"])] = (
                    info["module"], decl["qualifier"]
                )
    return _spec_resolve_exports(infos, local)


def _spec_resolve_callable_exports(infos, callables):
    local = {
        (row["module"], row["name"]): (row["module"], row["name"])
        for row in callables
        if not row["qualifier"] and row["kind"] == "ModuleFn"
    }
    return _spec_resolve_exports(infos, local)


def _spec_parse_modules():
    infos = []
    failures = []
    for pyi in sorted(TYPESHED_STDLIB.rglob("*.pyi")):
        mod = module_name(pyi)
        source = f'vendor/typeshed/stdlib/{pyi.relative_to(TYPESHED_STDLIB).as_posix()}'
        try:
            tree = ast.parse(pyi.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError as exc:
            failures.append(f"{source}:{exc.lineno}:{exc.offset}: {exc.msg}")
            continue
        info = dict(
            module=mod,
            source=source,
            tree=tree,
            is_package=pyi.name == "__init__.pyi",
            imports={},
            star_imports=[],
            explicit_reexports=set(),
            all_names=None,
            classes={},
            class_decls=[],
            aliases=set(),
            alias_decls=[],
            type_param_decls=[],
            type_params={},
        )
        _spec_scan_scope(info, tree.body)
        infos.append(info)
    if failures:
        raise RuntimeError("typeshed parse failures:\n" + "\n".join(failures))
    return infos


class _SpecCorpus:
    def __init__(self, infos):
        self.infos = infos
        self.info_by_module = {info["module"]: info for info in infos}
        self.nodes = []
        self.node_ids = {}
        self.edges = []
        self.params = []
        self.type_params = []
        self.type_param_ids = {}
        self.type_param_edges = []
        self.aliases = []
        self.alias_targets = {}
        self.classes = []
        self.class_ids = {}
        self.class_method_edges = []
        self.class_export_targets = _spec_resolve_class_exports(infos)
        self.class_exports = []
        self.callable_exports = []
        self.class_only_callables = []
        self.class_callables = []
        self.decorators = []
        self.guards = []
        self.callables = []
        self.source_files = []
        self.source_file_ids = {}
        self.source_spans = []
        self.source_span_ids = {}
        self.type_uses = []
        self.type_use_ids = {}
        self.global_symbols = {}
        for info in infos:
            for name, kind in info["classes"].items():
                if "." not in name:
                    self.global_symbols[(info["module"], name)] = kind
            for qualifier, name in info["aliases"]:
                if not qualifier:
                    self.global_symbols[(info["module"], name)] = "Alias"
        class_decls = {}
        for info in infos:
            for decl in info["class_decls"]:
                key = (decl["module"], decl["qualifier"])
                previous = class_decls.get(key)
                if previous is None:
                    class_decls[key] = (dict(decl), info)
                else:
                    previous[0]["method_only_complete"] = False
                    if decl["kind"] == "Protocol":
                        previous[0]["kind"] = "Protocol"
        for key in sorted(class_decls):
            self.class_ids[key] = len(self.classes)
            self.classes.append(None)
        self.class_kinds = {
            key: decl["kind"] for key, (decl, _info) in class_decls.items()
        }
        decls = sorted(
            (
                (decl, info)
                for info in infos
                for decl in info["type_param_decls"]
            ),
            key=lambda item: item[0]["key"],
        )
        for decl, _info in decls:
            idx = len(self.type_params)
            self.type_param_ids[decl["key"]] = idx
            self.type_params.append(None)
            if not decl["qualifier"]:
                self.global_symbols[(_info["module"], decl["name"])] = "TypeParam"
        self.missing = self._add_node(("Missing",), dict(kind="Missing"))
        for decl, info in decls:
            self._fill_type_param(decl, info)
        alias_decls = sorted(
            (
                (decl, info)
                for info in infos
                for decl in info["alias_decls"]
            ),
            key=lambda item: item[0]["key"],
        )
        pending_aliases = []
        for decl, info in alias_decls:
            target = self.intern(decl["value"], info, decl["qualifier"])
            pending_aliases.append((decl, target))
            if not decl["qualifier"]:
                self.alias_targets[(decl["module"], decl["name"])] = target
        for decl, target in pending_aliases:
            referenced = self.referenced_type_params([target])
            start = len(self.type_param_edges)
            self.type_param_edges.extend(referenced)
            row = dict(
                module=decl["module"],
                qualifier=decl["qualifier"],
                name=decl["name"],
                target=target,
                type_params=(start, len(referenced)),
            )
            self.aliases.append(row)
        for key in sorted(class_decls):
            decl, info = class_decls[key]
            self._fill_class(self.class_ids[key], decl, info)
        for (module, name), target in sorted(self.class_export_targets.items()):
            class_id = self.class_ids.get(target)
            if class_id is not None:
                self.class_exports.append(
                    dict(module=module, name=name, class_id=class_id)
                )

    def _add_node(self, key, row):
        found = self.node_ids.get(key)
        if found is not None:
            return found
        idx = len(self.nodes)
        self.node_ids[key] = idx
        self.nodes.append(row)
        return idx

    def _add_edges(self, values):
        start = len(self.edges)
        self.edges.extend(values)
        return start, len(values)

    def intern_span(self, span):
        path = span["path"]
        file_id = self.source_file_ids.get(path)
        if file_id is None:
            file_id = len(self.source_files)
            self.source_file_ids[path] = file_id
            self.source_files.append(path)
        key = (
            file_id,
            span["line"],
            span["column"],
            span["end_line"],
            span["end_column"],
        )
        found = self.source_span_ids.get(key)
        if found is not None:
            return found
        idx = len(self.source_spans)
        self.source_span_ids[key] = idx
        self.source_spans.append(key)
        return idx

    def intern_type_use(self, ty, span):
        source = self.intern_span(span)
        key = (ty, source)
        found = self.type_use_ids.get(key)
        if found is not None:
            return found
        idx = len(self.type_uses)
        self.type_use_ids[key] = idx
        self.type_uses.append(key)
        return idx

    def _scope_candidates(self, qualifier):
        current = qualifier
        while True:
            yield current
            if not current or "." not in current:
                break
            current = current.rsplit(".", 1)[0]
        if qualifier:
            yield ""

    def _resolve_name(self, info, qualifier, dotted):
        root, *tail = dotted.split(".")
        for scope in self._scope_candidates(qualifier):
            key = info["type_params"].get((scope, root))
            if key is not None and not tail:
                return "TypeParam", self.type_param_ids[key]
        if root in info["imports"]:
            module, imported = info["imports"][root]
            if imported:
                pieces = [imported] + tail
                name = ".".join(piece for piece in pieces if piece)
            elif root == module.split(".")[0]:
                absolute = dotted
                candidates = [
                    candidate
                    for candidate in self.info_by_module
                    if absolute.startswith(candidate + ".")
                ]
                if candidates:
                    module = max(candidates, key=len)
                    name = absolute[len(module) + 1:]
                else:
                    name = ".".join(tail)
            else:
                name = ".".join(tail)
            if not name:
                name = root
            symbol_kind = self.global_symbols.get((module, name), "Imported")
            target = self.class_export_targets.get((module, name))
            if target is not None:
                symbol_kind = self.class_kinds[target]
            if symbol_kind == "TypeParam":
                imported_info = self.info_by_module.get(module)
                if imported_info:
                    key = imported_info["type_params"].get(("", name))
                    if key is not None:
                        return "TypeParam", self.type_param_ids[key]
                symbol_kind = "Unresolved"
            if module in {"typing", "typing_extensions"} and name in _SPEC_SPECIALS:
                symbol_kind = "Special"
            return "Name", (module, name, symbol_kind)
        if not tail and root in _SPEC_BUILTINS:
            return "Name", ("builtins", root, "Builtin")
        if not tail and root in {"None", "Any", "Never", "NoReturn", "Self"}:
            return "Special", root
        if not tail and (
            (qualifier, root) in info["aliases"] or ("", root) in info["aliases"]
        ):
            return "Name", (info["module"], root, "Alias")
        if dotted in info["classes"]:
            return "Name", (info["module"], dotted, info["classes"][dotted])
        if not tail and root in info["classes"]:
            return "Name", (info["module"], root, info["classes"][root])
        if root in _SPEC_SPECIALS:
            return "Name", ("typing", dotted, "Special")
        return "Name", (info["module"], dotted, "Unresolved")

    def intern(self, node, info, qualifier, literal_context=False):
        if node is None:
            return self.missing
        if isinstance(node, ast.Constant):
            if node.value is None:
                kind = "LiteralNone" if literal_context else "None"
                return self._add_node((kind,), dict(kind=kind))
            if node.value is Ellipsis:
                return self._add_node(("Ellipsis",), dict(kind="Ellipsis"))
            if isinstance(node.value, bool):
                return self._add_node(("LiteralBool", node.value), dict(kind="LiteralBool", value=node.value))
            if isinstance(node.value, int):
                return self._add_node(("LiteralInt", node.value), dict(kind="LiteralInt", value=node.value))
            if isinstance(node.value, bytes):
                value = node.value.hex()
                return self._add_node(("LiteralBytes", value), dict(kind="LiteralBytes", value=value))
            if isinstance(node.value, str):
                expression = node.value
                if literal_context:
                    return self._add_node(
                        ("LiteralStr", expression),
                        dict(kind="LiteralStr", value=expression),
                    )
                try:
                    target_ast = ast.parse(expression, mode="eval").body
                    target = self.intern(target_ast, info, qualifier)
                except SyntaxError:
                    target = self._add_node(("Unsupported", expression), dict(kind="Unsupported", source=expression))
                return self._add_node(
                    ("ForwardRef", expression, target),
                    dict(kind="ForwardRef", expression=expression, target=target),
                )
        if (
            literal_context
            and isinstance(node, ast.UnaryOp)
            and isinstance(node.op, (ast.USub, ast.UAdd))
            and isinstance(node.operand, ast.Constant)
            and isinstance(node.operand.value, int)
            and not isinstance(node.operand.value, bool)
        ):
            value = node.operand.value
            if isinstance(node.op, ast.USub):
                value = -value
            return self._add_node(
                ("LiteralInt", value), dict(kind="LiteralInt", value=value)
            )
        dotted = _spec_dotted_name(node)
        if dotted:
            resolved, value = self._resolve_name(info, qualifier, dotted)
            if resolved == "TypeParam":
                return self._add_node(("TypeParam", value), dict(kind="TypeParam", id=value))
            if resolved == "Special":
                special = {"None": "None", "Any": "Any", "Never": "Never", "NoReturn": "Never", "Self": "SelfType"}[value]
                return self._add_node((special,), dict(kind=special))
            module, name, kind = value
            return self._add_node(
                ("Name", module, name, kind),
                dict(kind="Name", module=module, name=name, name_kind=kind),
            )
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.BitOr):
            items = []
            def collect(item):
                if isinstance(item, ast.BinOp) and isinstance(item.op, ast.BitOr):
                    collect(item.left); collect(item.right)
                else:
                    items.append(self.intern(item, info, qualifier))
            collect(node)
            key = ("Union", tuple(items))
            found = self.node_ids.get(key)
            if found is not None:
                return found
            start, length = self._add_edges(items)
            return self._add_node(key, dict(kind="Union", range=(start, length)))
        if isinstance(node, ast.Subscript):
            base = self.intern(node.value, info, qualifier)
            raw_args = list(node.slice.elts) if isinstance(node.slice, ast.Tuple) else [node.slice]
            base_node = self.nodes[base]
            literal_args = (
                base_node["kind"] == "Name"
                and base_node["module"] in {"typing", "typing_extensions"}
                and base_node["name"] == "Literal"
            )
            args = [
                self.intern(arg, info, qualifier, literal_context=literal_args)
                for arg in raw_args
            ]
            key = ("Apply", base, tuple(args))
            found = self.node_ids.get(key)
            if found is not None:
                return found
            start, length = self._add_edges(args)
            return self._add_node(key, dict(kind="Apply", base=base, range=(start, length)))
        if isinstance(node, ast.Tuple):
            items = [self.intern(item, info, qualifier) for item in node.elts]
            key = ("Tuple", tuple(items))
            found = self.node_ids.get(key)
            if found is not None:
                return found
            start, length = self._add_edges(items)
            return self._add_node(key, dict(kind="Tuple", range=(start, length)))
        if isinstance(node, ast.List):
            items = [self.intern(item, info, qualifier) for item in node.elts]
            key = ("ParamList", tuple(items))
            found = self.node_ids.get(key)
            if found is not None:
                return found
            start, length = self._add_edges(items)
            return self._add_node(key, dict(kind="ParamList", range=(start, length)))
        if isinstance(node, ast.Starred):
            inner = self.intern(node.value, info, qualifier)
            return self._add_node(("Unpack", inner), dict(kind="Unpack", inner=inner))
        source = ast.unparse(node) if hasattr(ast, "unparse") else ast.dump(node, include_attributes=False)
        return self._add_node(("Unsupported", source), dict(kind="Unsupported", source=source))

    def _fill_type_param(self, decl, info):
        call = decl["value"]
        args = list(call.args[1:])
        keywords = {kw.arg: kw.value for kw in call.keywords if kw.arg}
        constraints = [self.intern(arg, info, decl["qualifier"]) for arg in args]
        start, length = self._add_edges(constraints)
        bound_node = keywords.get("bound")
        default_node = keywords.get("default")
        variance = "Invariant"
        if isinstance(keywords.get("covariant"), ast.Constant) and keywords["covariant"].value is True:
            variance = "Covariant"
        elif isinstance(keywords.get("contravariant"), ast.Constant) and keywords["contravariant"].value is True:
            variance = "Contravariant"
        elif isinstance(keywords.get("infer_variance"), ast.Constant) and keywords["infer_variance"].value is True:
            variance = "Infer"
        idx = self.type_param_ids[decl["key"]]
        self.type_params[idx] = dict(
            key=decl["key"],
            name=decl["name"],
            kind=decl["kind"],
            variance=variance,
            bound=self.intern(bound_node, info, decl["qualifier"]) if bound_node else None,
            constraints=(start, length),
            default=self.intern(default_node, info, decl["qualifier"]) if default_node else None,
        )

    def _fill_class(self, class_id, decl, info):
        bases = [self.intern(base, info, decl["qualifier"]) for base in decl["bases"]]
        base_start, base_len = self._add_edges(bases)
        referenced = self.ordered_referenced_type_params(bases)
        type_param_start = len(self.type_param_edges)
        self.type_param_edges.extend(referenced)
        self.classes[class_id] = dict(
            module=decl["module"],
            qualifier=decl["qualifier"],
            name=decl["name"],
            kind=decl["kind"],
            type_params=(type_param_start, len(referenced)),
            bases=(base_start, base_len),
            methods=(0, 0),
            source=self.intern_span(_spec_span(decl["node"], info["source"])),
            method_only_complete=decl["method_only_complete"],
        )

    def ordered_referenced_type_params(self, roots):
        found = []
        found_set = set()
        seen = set()

        def visit(node_id):
            if node_id in seen:
                return
            seen.add(node_id)
            node = self.nodes[node_id]
            kind = node["kind"]
            if kind == "TypeParam":
                if node["id"] not in found_set:
                    found_set.add(node["id"])
                    found.append(node["id"])
            elif kind in {"Union", "Tuple", "ParamList"}:
                start, length = node["range"]
                for child in self.edges[start:start + length]:
                    visit(child)
            elif kind == "Apply":
                visit(node["base"])
                start, length = node["range"]
                for child in self.edges[start:start + length]:
                    visit(child)
            elif kind == "Unpack":
                visit(node["inner"])
            elif kind == "ForwardRef":
                visit(node["target"])
            elif kind == "Name" and node["name_kind"] == "Alias":
                target = self.alias_targets.get((node["module"], node["name"]))
                if target is not None:
                    visit(target)

        for root in roots:
            visit(root)
        return found

    def referenced_type_params(self, roots):
        return sorted(self.ordered_referenced_type_params(roots))


def _spec_full_params(fn):
    positional = list(fn.args.posonlyargs) + list(fn.args.args)
    default_start = len(positional) - len(fn.args.defaults)
    out = []
    for index, arg in enumerate(positional):
        kind = "PosOnly" if index < len(fn.args.posonlyargs) else "PosOrKw"
        out.append((arg, kind, index >= default_start))
    if fn.args.vararg is not None:
        out.append((fn.args.vararg, "VarPos", False))
    for arg, default in zip(fn.args.kwonlyargs, fn.args.kw_defaults):
        out.append((arg, "KwOnly", default is not None))
    if fn.args.kwarg is not None:
        out.append((fn.args.kwarg, "VarKw", False))
    return out


def _spec_walk_body(
    corpus, info, body, qualifier="", guards=(), class_inventory=False
):
    for node in body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if qualifier:
                if node.name.startswith("_") and not (node.name.startswith("__") and node.name.endswith("__")):
                    continue
            elif node.name.startswith("_"):
                continue
            decorators = [ast.unparse(item) for item in node.decorator_list]
            names = decorator_names(node)
            if not qualifier:
                binding = "ModuleFn"
            elif any(item.endswith(".setter") for item in decorators):
                binding = "PropertySet"
            elif "property" in names:
                binding = "PropertyGet"
            elif "classmethod" in names:
                binding = "ClassMethod"
            elif "staticmethod" in names:
                binding = "StaticMethod"
            else:
                binding = "InstanceMethod"
            implicit_first = bool(qualifier) and (binding != "StaticMethod" or node.name == "__new__")
            param_start = len(corpus.params)
            roots = []
            for index, (arg, kind, has_default) in enumerate(_spec_full_params(node)):
                ty = corpus.intern(arg.annotation, info, qualifier)
                roots.append(ty)
                type_use = corpus.intern_type_use(
                    ty, _spec_span(arg.annotation or arg, info["source"])
                )
                corpus.params.append(dict(
                    name=arg.arg,
                    kind=kind,
                    type_use=type_use,
                    has_default=has_default,
                    implicit_receiver=implicit_first and index == 0,
                ))
            ret = corpus.intern(node.returns, info, qualifier)
            roots.append(ret)
            referenced = corpus.referenced_type_params(roots)
            type_param_start = len(corpus.type_param_edges)
            corpus.type_param_edges.extend(referenced)
            decorator_start = len(corpus.decorators)
            corpus.decorators.extend(decorators)
            guard_start = len(corpus.guards)
            corpus.guards.extend(guards)
            ret_use = corpus.intern_type_use(
                ret, _spec_span(node.returns or node, info["source"])
            )
            source_span = corpus.intern_span(_spec_span(node, info["source"]))
            target = corpus.class_only_callables if class_inventory else corpus.callables
            target.append(dict(
                module=info["module"], qualifier=qualifier, name=node.name,
                kind=binding, params=(param_start, len(corpus.params) - param_start),
                type_params=(type_param_start, len(referenced)),
                ret=ret_use,
                decorators=(decorator_start, len(decorators)),
                guards=(guard_start, len(guards)), source=source_span,
                is_async=isinstance(node, ast.AsyncFunctionDef),
                py312=all(item["py312"] for item in guards), order=len(target),
            ))
        elif isinstance(node, ast.ClassDef):
            nested = f"{qualifier}.{node.name}" if qualifier else node.name
            _spec_walk_body(
                corpus,
                info,
                node.body,
                nested,
                guards,
                class_inventory or node.name.startswith("_"),
            )
        elif isinstance(node, ast.If):
            expression = ast.unparse(node.test)
            result = _eval_version_test(node.test)
            body_guard = dict(expression=expression, polarity=True, py312=result is not False)
            else_guard = dict(expression=expression, polarity=False, py312=result is not True)
            _spec_walk_body(
                corpus,
                info,
                node.body,
                qualifier,
                guards + (body_guard,),
                class_inventory,
            )
            _spec_walk_body(
                corpus,
                info,
                node.orelse,
                qualifier,
                guards + (else_guard,),
                class_inventory,
            )


def build_spec_corpus():
    infos = _spec_parse_modules()
    corpus = _SpecCorpus(infos)
    for info in infos:
        _spec_walk_body(corpus, info, info["tree"].body)
    def order_callables(rows):
        ordered = sorted(
            rows,
            key=lambda row: (
                row["module"], row["qualifier"], row["name"],
                row["kind"], row["order"],
            ),
        )
        current = None
        branch = 0
        for row in ordered:
            key = (row["module"], row["qualifier"], row["name"], row["kind"])
            if key != current:
                current = key
                branch = 0
            row["branch"] = branch
            branch += 1
        return ordered

    corpus.callables = order_callables(corpus.callables)
    corpus.class_callables = order_callables(
        [dict(row) for row in corpus.callables if row["qualifier"]]
        + corpus.class_only_callables
    )
    corpus.callable_exports = [
        dict(module=module, name=name, target_module=target[0], target_name=target[1])
        for (module, name), target in sorted(
            _spec_resolve_callable_exports(corpus.infos, corpus.callables).items()
        )
    ]
    for class_row in corpus.classes:
        method_ids = [
            index
            for index, callable_row in enumerate(corpus.class_callables)
            if callable_row["module"] == class_row["module"]
            and callable_row["qualifier"] == class_row["qualifier"]
            and callable_row["kind"] != "ModuleFn"
        ]
        start = len(corpus.class_method_edges)
        corpus.class_method_edges.extend(method_ids)
        class_row["methods"] = (start, len(method_ids))
    return corpus


def _rust_str(s: str) -> str:
    """Rust string literal escaping (module/class/param names are identifiers,
    but escape defensively)."""
    return s.replace("\\", "\\\\").replace('"', '\\"')


def _core_ty_rust(ct: str) -> str:
    """Render a `core_ty_of` string result as the `CoreTy` Rust expression
    that follows `CoreTy::` at a call site. Every other CoreTy name is a bare
    unit variant (`CoreTy::Typed`, `CoreTy::Int`, …); #882's `"TypedNamed:X"`
    tag is the one payload-carrying variant, rendered `TypedNamed("X")`."""
    if ct.startswith("TypedNamed:"):
        return f'TypedNamed("{_rust_str(ct[len("TypedNamed:"):])}")'
    return ct


def merge_overload_params(rows):
    """Merge a real @overload chain (>=2 signatures applicable to 3.12 at once)
    into ONE folded row for the 3.12-applicable set: a position is checkable
    only when EVERY overload has a non-Unknown contract there. If every branch
    agrees on the same CoreTy, keep that wall; otherwise degrade to `Typed` so
    overloaded groups still reject a bare sentinel without guessing which
    concrete scalar wall applies. Positions missing from some overload, or
    carrying Unknown in any branch, collapse to Unknown. Conservative on arity:
    only positions present in EVERY overload are considered."""
    base = dict(rows[0])
    param_lists = [r["params"] for r in rows]
    n = min(len(pl) for pl in param_lists)
    merged = []
    for i in range(n):
        ctys = {pl[i][1] for pl in param_lists}
        name = param_lists[0][i][0]
        if all(ct != "Unknown" for ct in ctys):
            merged.append((name, next(iter(ctys)) if len(ctys) == 1 else "Typed"))
        else:
            merged.append((name, "Unknown"))
    has_star = any(r.get("has_star") for r in rows)
    base["params"] = merged
    base["has_star"] = has_star
    base["enforceable"] = any(ct != "Unknown" for _, ct in merged)
    # #887: same AGREEMENT rule for the return type — a real `@overload` chain
    # only has ONE knowable return if every branch declares the SAME scalar;
    # otherwise (a `logging.getLevelName`-style int|str-return split) the
    # actual return depends on which overload the call site matched, which
    # this table cannot know, so collapse to Unknown.
    ret_values = {r.get("ret", "Unknown") for r in rows}
    base["ret"] = next(iter(ret_values)) if len(ret_values) == 1 else "Unknown"
    return base


def render_specs_json() -> str:
    corpus = build_spec_corpus()
    strings = []
    string_ids = {}

    def sid(value):
        found = string_ids.get(value)
        if found is not None:
            return found
        idx = len(strings)
        string_ids[value] = idx
        strings.append(value)
        return idx

    name_kind = {
        "Builtin": "b", "Special": "s", "Nominal": "n", "Protocol": "p",
        "Alias": "a", "Imported": "i", "Unresolved": "u",
    }
    param_kind = {"PosOnly": "p", "PosOrKw": "r", "VarPos": "v", "KwOnly": "k", "VarKw": "w"}
    callable_kind = {
        "ModuleFn": "m", "InstanceMethod": "i", "ClassMethod": "c",
        "StaticMethod": "s", "PropertyGet": "g", "PropertySet": "t",
    }
    type_param_kind = {"TypeVar": "t", "TypeVarTuple": "v", "ParamSpec": "p"}
    variance = {"Invariant": "i", "Covariant": "c", "Contravariant": "d", "Infer": "f"}
    class_kind = {"Nominal": "n", "Protocol": "p"}

    nodes = []
    for node in corpus.nodes:
        kind = node["kind"]
        if kind in {"Missing", "Any", "Never", "None", "SelfType", "Ellipsis", "LiteralNone"}:
            value = kind
        elif kind == "Unsupported":
            value = {"Unsupported": sid(node["source"])}
        elif kind == "Name":
            value = {"Name": {
                "module": sid(node["module"]), "name": sid(node["name"]),
                "kind": name_kind[node["name_kind"]],
            }}
        elif kind == "TypeParam":
            value = {"TypeParam": node["id"]}
        elif kind in {"Union", "Tuple", "ParamList"}:
            value = {kind: list(node["range"])}
        elif kind == "Apply":
            value = {"Apply": {"base": node["base"], "args": list(node["range"])}}
        elif kind == "Unpack":
            value = {"Unpack": node["inner"]}
        elif kind in {"LiteralInt", "LiteralBool"}:
            value = {kind: node["value"]}
        elif kind in {"LiteralStr", "LiteralBytes"}:
            value = {kind: sid(node["value"])}
        elif kind == "ForwardRef":
            value = {"ForwardRef": {"expression": sid(node["expression"]), "target": node["target"]}}
        else:
            raise AssertionError(f"unserialized TypeSpec node: {node}")
        nodes.append(value)

    type_params = [
        {
            "key": sid(row["key"]), "name": sid(row["name"]),
            "kind": type_param_kind[row["kind"]], "variance": variance[row["variance"]],
            "bound": row["bound"], "constraints": list(row["constraints"]),
            "default": row["default"],
        }
        for row in corpus.type_params
    ]
    aliases = [
        {
            "module": sid(row["module"]),
            "qualifier": sid(row["qualifier"]),
            "name": sid(row["name"]),
            "target": row["target"],
            "type_params": list(row["type_params"]),
        }
        for row in corpus.aliases
    ]
    classes = [
        [
            sid(row["module"]), sid(row["qualifier"]), sid(row["name"]),
            class_kind[row["kind"]], list(row["type_params"]),
            list(row["bases"]), list(row["methods"]), row["source"],
            row["method_only_complete"],
        ]
        for row in corpus.classes
    ]
    class_exports = [
        [sid(row["module"]), sid(row["name"]), row["class_id"]]
        for row in corpus.class_exports
    ]
    callable_exports = [
        [
            sid(row["module"]), sid(row["name"]),
            sid(row["target_module"]), sid(row["target_name"]),
        ]
        for row in corpus.callable_exports
    ]
    source_file_ids = [sid(path) for path in corpus.source_files]
    source_spans = [
        [source_file_ids[file_id], line, column, end_line, end_column]
        for file_id, line, column, end_line, end_column in corpus.source_spans
    ]
    params = [
        [sid(row["name"]), param_kind[row["kind"]], row["type_use"], row["has_default"], row["implicit_receiver"]]
        for row in corpus.params
    ]
    guards = [
        [sid(row["expression"]), row["polarity"], row["py312"]]
        for row in corpus.guards
    ]
    callables = [
        [
            sid(row["module"]), sid(row["qualifier"]), sid(row["name"]),
            callable_kind[row["kind"]], list(row["params"]), list(row["type_params"]),
            row["ret"], list(row["decorators"]), list(row["guards"]), row["source"],
            row["is_async"], row["branch"], row["py312"],
        ]
        for row in corpus.callables
    ]
    class_callables = [
        [
            sid(row["module"]), sid(row["qualifier"]), sid(row["name"]),
            callable_kind[row["kind"]], list(row["params"]), list(row["type_params"]),
            row["ret"], list(row["decorators"]), list(row["guards"]), row["source"],
            row["is_async"], row["branch"], row["py312"],
        ]
        for row in corpus.class_callables
    ]
    manifest = {
        "schema": 2,
        "strings": strings,
        "nodes": nodes,
        "edges": corpus.edges,
        "type_params": type_params,
        "type_param_edges": corpus.type_param_edges,
        "aliases": aliases,
        "classes": classes,
        "class_method_edges": corpus.class_method_edges,
        "class_exports": class_exports,
        "callable_exports": callable_exports,
        "source_spans": source_spans,
        "type_uses": [list(row) for row in corpus.type_uses],
        "params": params,
        "decorators": [sid(value) for value in corpus.decorators],
        "guards": guards,
        "callables": callables,
        "class_callables": class_callables,
    }
    return json.dumps(manifest, ensure_ascii=True, separators=(",", ":"), sort_keys=True) + "\n"


def render_specs_rust(specs_json: str) -> str:
    manifest = json.loads(specs_json)
    py312 = sum(bool(row[12]) for row in manifest["callables"])
    return (
        "//! GENERATED by tests/harness/cpython/tools/type_wall_gen.py --emit-rust.\n"
        "//! DO NOT EDIT BY HAND.\n"
        "//!\n"
        f"//! schema: {manifest['schema']}  branches: {len(manifest['callables'])}  py312: {py312}\n"
        f"//! params: {len(manifest['params'])}  type-params: {len(manifest['type_params'])}\n"
        f"//! aliases: {len(manifest['aliases'])}\n"
        f"//! classes: {len(manifest['classes'])}  class-exports: {len(manifest['class_exports'])}\n"
        f"//! class-method-edges: {len(manifest['class_method_edges'])}\n"
        f"//! callable-exports: {len(manifest['callable_exports'])}\n"
        f"//! class-callables: {len(manifest['class_callables'])}\n"
        f"//! type-nodes: {len(manifest['nodes'])}  type-edges: {len(manifest['edges'])}\n\n"
        "pub const MANIFEST_JSON: &str = include_str!(\"stdlib_specs_generated.json\");\n"
    )


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
            # No branch is definitively 3.12-applicable (unresolvable version
            # guard) AND more than one candidate exists — genuinely ambiguous,
            # so neither the args NOR the return type are trustworthy.
            row = dict(rs[0]); row["enforceable"] = False; row["ret"] = "Unknown"
            seen[key] = row
    ordered = [seen[k] for k in sorted(seen.keys())]

    n_total = len(ordered)
    n_enf = sum(1 for r in ordered if r["enforceable"])
    n_ret = sum(1 for r in ordered if r.get("ret", "Unknown") != "Unknown")

    lines: list[str] = []
    lines.append(
        "//! GENERATED by tests/harness/cpython/tools/type_wall_gen.py --emit-rust.\n"
        "//! DO NOT EDIT BY HAND. Regenerate with:\n"
        "//!   python3.12 tests/harness/cpython/tools/type_wall_gen.py --emit-rust\n"
        "//!\n"
        "//! Typeshed-derived ① Type-wall stdlib signature table. Each row maps a\n"
        "//! stdlib callable's positional params to the closed [`CoreTy`] scalar set\n"
        "//! (int/float/str/bytes/bool/complex/list/tuple/dict/type/None); everything\n"
        "//! richer is `CoreTy::Unknown`.\n"
        "//! A single-signature row is `enforceable` when its fixed positional\n"
        "//! prefix has at least one checkable param; a trailing `*args` is\n"
        "//! rendered as one star/Unknown tail row and skipped. A 3.12 overload\n"
        "//! set keeps identical non-Unknown `CoreTy` positions and folds\n"
        "//! divergent non-Unknown positions to `CoreTy::Typed`; missing positions become\n"
        "//! `CoreTy::Unknown`.\n"
        "//!\n"
        "//! #887: each row also carries `ret`, the callable's typeshed RETURN\n"
        "//! annotation closed to the concrete positive scalars (`Int`/`Float`/\n"
        "//! `Str`/`Bool`/`None`) that `check.rs::core_ty_to_type_id` maps to a real\n"
        "//! `Ty` — fed into inference at the call site (independent of\n"
        "//! `enforceable`, which governs ARGUMENT checking only). Everything richer\n"
        "//! (Optional/Union/generics/nominal classes/no annotation) is\n"
        "//! `CoreTy::Unknown`, which the call-site hook skips — never guessed.\n"
        f"//!\n"
        f"//! rows: {n_total}  ·  enforceable (scalar): {n_enf}  ·  "
        f"unknown-skipped: {n_total - n_enf}  ·  concrete return: {n_ret}\n"
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
                f'p("{_rust_str(n)}", CoreTy::{_core_ty_rust(ct)}, false)'
                for (n, ct) in r["params"]
            ]
            if r["has_star"]:
                parts.append('p("args", CoreTy::Unknown, true)')
            params_src = "&[" + ", ".join(parts) + "]"
        else:
            params_src = "&[]"
        enf = "true" if r["enforceable"] else "false"
        ret = r.get("ret", "Unknown")
        lines.append("    StdlibSig {")
        lines.append(f'        module: "{_rust_str(r["module"])}",')
        lines.append(f'        qualifier: "{_rust_str(r["qualifier"])}",')
        lines.append(f'        name: "{_rust_str(r["name"])}",')
        lines.append(f"        kind: {kind},")
        lines.append(f"        params: {params_src},")
        lines.append(f"        enforceable: {enf},")
        lines.append(f"        ret: CoreTy::{_core_ty_rust(ret)},")
        lines.append("    },")
    lines.append("];")
    lines.append("")
    return "\n".join(lines)


def emit_rust(check: bool) -> int:
    text = render_rust()
    specs_data = render_specs_json()
    specs_text = render_specs_rust(specs_data)
    if check:
        stale = False
        for path, expected in (
            (RUST_SIGS_OUT, text),
            (RUST_SPECS_OUT, specs_text),
            (RUST_SPECS_DATA_OUT, specs_data),
        ):
            if not path.exists():
                print(f"MISSING: {path} (run --emit-rust)")
                stale = True
                continue
            current = path.read_text(encoding="utf-8")
            if current != expected:
                print(f"STALE: {path} differs from --emit-rust output")
                stale = True
            else:
                print(f"OK: {path} is byte-for-byte up to date")
        return 1 if stale else 0
    RUST_SIGS_OUT.write_text(text, encoding="utf-8")
    RUST_SPECS_OUT.write_text(specs_text, encoding="utf-8")
    RUST_SPECS_DATA_OUT.write_text(specs_data, encoding="utf-8")
    n_total = text.count("    StdlibSig {")
    n_enf = text.count("        enforceable: true,")
    n_ret = n_total - text.count("        ret: CoreTy::Unknown,")
    print(f"wrote {RUST_SIGS_OUT}  ({n_total} sigs, {n_enf} enforceable, {n_ret} concrete return)")
    manifest = json.loads(specs_data)
    print(
        f"wrote {RUST_SPECS_OUT} + {RUST_SPECS_DATA_OUT}  "
        f"({len(manifest['callables'])} branches, {len(manifest['nodes'])} type nodes)"
    )
    return 0


def main() -> int:
    global TYPESHED_STDLIB

    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--module", help="only this dotted module")
    ap.add_argument("--kind", action="append",
                    choices=["module", "init", "smethod", "method"])
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--emit-rust", action="store_true",
                    help="(re)write both generated Rust signature artifacts")
    ap.add_argument("--check-rust", action="store_true",
                    help="assert both generated Rust signature artifacts are current")
    ap.add_argument(
        "--typeshed-stdlib",
        type=Path,
        help="typeshed stdlib directory (or set MAMBA_TYPESHED_STDLIB)",
    )
    args = ap.parse_args()

    if args.typeshed_stdlib is not None:
        TYPESHED_STDLIB = args.typeshed_stdlib.resolve()
    if not TYPESHED_STDLIB.is_dir():
        print(
            f"missing typeshed stdlib directory: {TYPESHED_STDLIB}",
            file=sys.stderr,
        )
        return 2

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
