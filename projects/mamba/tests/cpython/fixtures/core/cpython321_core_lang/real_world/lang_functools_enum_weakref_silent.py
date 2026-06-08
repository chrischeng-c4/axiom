# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_functools_enum_weakref_silent"
# subject = "cpython321.lang_functools_enum_weakref_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_functools_enum_weakref_silent.py"
# status = "filled"
# ///
"""cpython321.lang_functools_enum_weakref_silent: execute CPython 3.12 seed lang_functools_enum_weakref_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# higher-order / enum / weakref / GC-introspection / interpreter-
# index / filesystem-path sextet pinned by atomic 158: `functools`
# (the documented `partial` keyword-arg binding +
# `lru_cache` decorator + `cmp_to_key` sort-key adapter + class
# `__name__` identity), `enum` (the documented `Enum` member
# name / value attribute + `Enum(value)` lookup + `Enum["name"]`
# subscript surface), `weakref` (the documented `ref(obj)()`
# referent callable + `WeakValueDictionary` / `WeakKeyDictionary`
# / `WeakSet` / `ref` bare-class identity), `gc` (the documented
# `get_referrers` / `get_referents` introspection helpers),
# `sys` (the documented `version_info` integer-index access +
# `maxsize` 64-bit value), and `pathlib` (the documented `Path`
# instance accessors `.name` / `.stem` / `.suffix` / `.parent` /
# `.parts` + `.is_absolute` method).
#
# The matching subset (functools.reduce no-init + with-init,
# functools.partial positional binding, gc.isenabled +
# gc.collect + gc.get_threshold + gc.get_count tuple shape,
# sys.byteorder + sys.platform + bare attribute hasattr surface,
# pathlib module attribute + Path constructor non-raise) is
# covered by `test_functools_gc_sys_pathlib_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • functools.partial(int, base=16)("ff") == 255 — keyword-arg
#     binding contract (mamba: returns None);
#   • @functools.lru_cache(maxsize=None) decorator preserves
#     callable identity — _fib(10) == 55 (mamba: returns None,
#     decorator wraps callable into broken stub);
#   • @functools.cache decorated function returns wrapped value
#     on second call (mamba: returns None on memoized hit);
#   • functools.cmp_to_key(cmp)-keyed sort produces the
#     documented ascending order (mamba: returns input order,
#     cmp-key dispatch is a no-op);
#   • functools.reduce.__name__ == "reduce" — class identity on
#     the documented public symbol (mamba: returns None);
#   • functools.partial.__name__ == "partial" (mamba: None);
#   • Color.RED.name == "RED" — Enum member-name attribute
#     (mamba: returns None);
#   • Color.RED.value == 1 — Enum member-value attribute
#     (mamba: None);
#   • Color(1) is Color.RED — Enum value-lookup constructor
#     (mamba: returns an empty Color() instance, not the
#     canonical member);
#   • Color["RED"] is Color.RED — Enum subscript lookup (mamba:
#     TypeError, 'type' object is not subscriptable);
#   • weakref.ref(obj)() is obj — referent-callable round-trip
#     (mamba: returns None, weak references are not retaining
#     the referent at all);
#   • weakref.ref.__name__ == "ReferenceType" — bare class
#     identity (mamba: returns None);
#   • weakref.WeakValueDictionary.__name__ == "WeakValueDictionary"
#     (mamba: None);
#   • weakref.WeakKeyDictionary.__name__ == "WeakKeyDictionary"
#     (mamba: None);
#   • weakref.WeakSet.__name__ == "WeakSet" (mamba: None);
#   • hasattr(gc, "get_referrers") is True — GC-introspection
#     surface (mamba: False);
#   • hasattr(gc, "get_referents") is True (mamba: False);
#   • sys.version_info[0] == 3 — integer index access on
#     version-info named-tuple (mamba: KeyError, '0');
#   • sys.maxsize == 9223372036854775807 — 64-bit max-int
#     contract (mamba: returns 140737488355327, 48-bit value);
#   • pathlib.Path("/tmp/foo/bar.txt").name == "bar.txt" —
#     filename component accessor (mamba: returns None);
#   • pathlib.Path("/tmp/foo/bar.txt").stem == "bar" —
#     stem component accessor (mamba: returns None);
#   • pathlib.Path("/tmp/foo/bar.txt").suffix == ".txt" —
#     extension component accessor (mamba: returns None);
#   • pathlib.Path("/tmp/foo/bar.txt").is_absolute() == True
#     — absolute-path predicate (mamba: AttributeError,
#     'PosixPath' object has no attribute 'is_absolute').
import functools as _functools_mod
import enum as _enum_mod
import weakref as _weakref_mod
import gc as _gc_mod
import sys as _sys_mod
import pathlib as _pathlib_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
functools: Any = _functools_mod
enum: Any = _enum_mod
weakref: Any = _weakref_mod
gc: Any = _gc_mod
sys: Any = _sys_mod
pathlib: Any = _pathlib_mod


# Enum + dummy referent must live at module level — mamba
# elides class identifiers declared inside try/except blocks.
class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


class _RefTarget:
    pass


_ledger: list[int] = []

# 1) functools.partial — keyword-arg binding
_partial_kw = functools.partial(int, base=16)
assert _partial_kw("ff") == 255; _ledger.append(1)


# 2) functools.lru_cache + cache — decorator preserves callable
@functools.lru_cache(maxsize=None)
def _fib(n: int) -> int:
    if n < 2:
        return n
    return _fib(n - 1) + _fib(n - 2)


assert _fib(10) == 55; _ledger.append(1)


@functools.cache
def _double_spec(x: int) -> int:
    return x * 2


assert _double_spec(5) == 10; _ledger.append(1)
assert _double_spec(5) == 10; _ledger.append(1)

# 3) functools.cmp_to_key — sort-key dispatch
_key = functools.cmp_to_key(lambda x, y: len(x) - len(y))
assert sorted(["bb", "a", "ccc"], key=_key) == ["a", "bb", "ccc"]; _ledger.append(1)

# 4) functools — class __name__ identity
assert functools.reduce.__name__ == "reduce"; _ledger.append(1)
assert functools.partial.__name__ == "partial"; _ledger.append(1)

# 5) Enum — member name / value attribute
_color_cls: Any = Color
_color_red: Any = Color.RED
assert _color_red.name == "RED"; _ledger.append(1)
assert _color_red.value == 1; _ledger.append(1)

# 6) Enum — value-lookup constructor + subscript lookup
assert _color_cls(1) is Color.RED; _ledger.append(1)
assert _color_cls["RED"] is Color.RED; _ledger.append(1)

# 7) weakref — referent callable + class identity
_target = _RefTarget()
_wref = weakref.ref(_target)
assert _wref() is _target; _ledger.append(1)
assert weakref.ref.__name__ == "ReferenceType"; _ledger.append(1)
assert weakref.WeakValueDictionary.__name__ == "WeakValueDictionary"; _ledger.append(1)
assert weakref.WeakKeyDictionary.__name__ == "WeakKeyDictionary"; _ledger.append(1)
assert weakref.WeakSet.__name__ == "WeakSet"; _ledger.append(1)

# 8) gc — introspection helper surface
assert hasattr(gc, "get_referrers") == True; _ledger.append(1)
assert hasattr(gc, "get_referents") == True; _ledger.append(1)

# 9) sys — version_info indexing + maxsize 64-bit value
assert sys.version_info[0] == 3; _ledger.append(1)
assert sys.maxsize == 9223372036854775807; _ledger.append(1)

# 10) pathlib — Path instance accessors
_p = pathlib.Path("/tmp/foo/bar.txt")
assert _p.name == "bar.txt"; _ledger.append(1)
assert _p.stem == "bar"; _ledger.append(1)
assert _p.suffix == ".txt"; _ledger.append(1)
assert _p.is_absolute() == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_functools_enum_weakref_silent {sum(_ledger)} asserts")
