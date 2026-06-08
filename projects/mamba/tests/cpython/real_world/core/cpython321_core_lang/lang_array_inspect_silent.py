# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_array_inspect_silent"
# subject = "cpython321.lang_array_inspect_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_array_inspect_silent.py"
# status = "filled"
# ///
"""cpython321.lang_array_inspect_silent: execute CPython 3.12 seed lang_array_inspect_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# typed-buffer / introspection pair pinned by atomic 161: `array`
# (the documented `array(typecode, iterable)` constructor element
# population + `len` / `list` / `index` / `append` / `extend` /
# `reverse` instance methods) and `inspect` (the documented
# `getfullargspec` / `isbuiltin` attribute surface +
# `isfunction` / `isclass` predicate contracts +
# `signature(fn).parameters` parameter-map population).
#
# The matching subset (heapq full push / pop / heapify /
# nlargest / nsmallest / merge / heappushpop / heapreplace
# surface, bisect_left / bisect_right / insort + variants,
# array.typecode attribute on int + float typecodes, inspect
# hasattr signature / isfunction / isclass / ismethod) is
# covered by `test_heapq_bisect_array_inspect_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • array.array("i", [1,2,3,4]) populates the buffer —
#     len == 4 (mamba: returns 0, iterable is dropped on
#     construction);
#   • list(array.array("i", [1,2,3,4])) == [1,2,3,4] —
#     iteration contract (mamba: returns []);
#   • a.append(5) extends the buffer — len after == original+1
#     (mamba: len stays 0, append no-op);
#   • a.extend([6,7]) extends by the supplied iterable
#     (mamba: TypeError, 'object is not iterable');
#   • a.reverse() reverses the buffer (mamba: TypeError, leaves
#     buffer empty);
#   • a.index(3) == 2 — element-index lookup (mamba: returns
#     None);
#   • hasattr(inspect, "getfullargspec") is True — documented
#     full-arg-spec introspection helper (mamba: False);
#   • hasattr(inspect, "isbuiltin") is True — documented
#     builtin-function predicate (mamba: False);
#   • inspect.isfunction(def_fn) is True — predicate identity
#     on user-defined function (mamba: returns False,
#     predicate inverted);
#   • inspect.isfunction(1) is False — predicate identity on
#     integer (mamba: returns True, predicate inverted);
#   • inspect.isclass(int) is True — predicate identity on
#     builtin class (mamba: returns False);
#   • str(inspect.signature(fn)) renders the full annotated
#     parameter list — stringified signature contract (mamba:
#     returns "()" — parameters silently dropped);
#   • list(inspect.signature(fn).parameters.keys()) ==
#     ["a","b","args","kw"] — parameter-map population
#     contract (mamba: AttributeError, 'NoneType' object has
#     no attribute 'keys').
import array as _array_mod
import inspect as _inspect_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
array: Any = _array_mod
inspect: Any = _inspect_mod


# Probe target — must live at module level so the inspect
# signature contract can be exercised; mamba elides class /
# function identifiers declared inside try/except blocks.
def _free_fn(a: int, b: int = 2, *args: Any, **kw: Any) -> int:
    return a + b + len(args) + len(kw)


_ledger: list[int] = []

# 1) array.array — constructor populates buffer from iterable
_a = array.array("i", [1, 2, 3, 4])
assert len(_a) == 4; _ledger.append(1)
assert list(_a) == [1, 2, 3, 4]; _ledger.append(1)

# 2) array.array — append / extend / reverse instance methods
_a.append(5)
assert len(_a) == 5; _ledger.append(1)
assert list(_a)[-1] == 5; _ledger.append(1)

_a.extend([6, 7])
assert len(_a) == 7; _ledger.append(1)
assert list(_a)[-2:] == [6, 7]; _ledger.append(1)

# 3) array.array — index lookup
assert _a.index(3) == 2; _ledger.append(1)

# 4) array.array — reverse mutation
_a.reverse()
assert list(_a) == [7, 6, 5, 4, 3, 2, 1]; _ledger.append(1)

# 5) inspect — documented helper attribute surface
assert hasattr(inspect, "getfullargspec") == True; _ledger.append(1)
assert hasattr(inspect, "isbuiltin") == True; _ledger.append(1)

# 6) inspect — isfunction / isclass predicate identity
assert inspect.isfunction(_free_fn) == True; _ledger.append(1)
assert inspect.isfunction(1) == False; _ledger.append(1)
assert inspect.isclass(int) == True; _ledger.append(1)
assert inspect.isclass(1) == False; _ledger.append(1)

# 7) inspect.signature — stringified shape + parameter map
_sig = inspect.signature(_free_fn)
assert str(_sig) == "(a: int, b: int = 2, *args: Any, **kw: Any) -> int"; _ledger.append(1)
assert list(_sig.parameters.keys()) == ["a", "b", "args", "kw"]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_array_inspect_silent {sum(_ledger)} asserts")
