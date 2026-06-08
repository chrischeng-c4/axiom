# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_inspect_decimal_difflib_random_silent"
# subject = "cpython321.lang_inspect_decimal_difflib_random_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_inspect_decimal_difflib_random_silent.py"
# status = "filled"
# ///
"""cpython321.lang_inspect_decimal_difflib_random_silent: execute CPython 3.12 seed lang_inspect_decimal_difflib_random_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `types` / `inspect` / `gc` / `random` / `string` / `decimal` /
# `difflib` seven-pack pinned to atomic 228:
# `types` (the documented `SimpleNamespace(a=1, b=2).a == 1`
# instance-attribute value contract — mamba's
# `types.SimpleNamespace(...)` raises `AttributeError: 'dict'
# object has no attribute 'SimpleNamespace'` at call site
# even though `hasattr(types, "SimpleNamespace")` returns
# True), `inspect` (the documented extended
# `hasattr(inspect, "Parameter") / "Signature" / "ismodule" /
# "getsource" / "getsourcefile" / "getfile" / "currentframe" /
# "stack" == True` extended hasattr surface), `gc` (the
# documented `hasattr(gc, "get_referrers") / "get_referents"
# == True` extended hasattr surface), `random` (the documented
# seeded-PRNG value contract — mamba's `random.seed(42);
# random.random()` returns 0.3745401188473625 instead of the
# documented CPython value 0.6394267984578837, and
# `hasattr(random, "SystemRandom") == True` returns False),
# `string` (the documented `hasattr(string, "printable") ==
# True` ASCII-printable constant), `decimal` (the documented
# extended `hasattr(decimal, "Context") / "getcontext" /
# "setcontext" / "localcontext" / "ROUND_HALF_EVEN" /
# "ROUND_HALF_UP" / "ROUND_DOWN" / "InvalidOperation" /
# "DivisionByZero" == True` extended hasattr surface), and
# `difflib` (the documented extended `hasattr(difflib,
# "Differ") / "ndiff" / "context_diff" / "restore" /
# "HtmlDiff" == True` extended hasattr surface plus the
# documented `get_close_matches("appel", ["apple", "ape",
# "peach"]) == ["apple", "ape"]` cutoff-filter value contract
# — mamba silently returns the unfiltered three-element
# `["apple", "ape", "peach"]` list).
#
# Behavioral edges that CONFORM on mamba (json round-trip,
# copy/deepcopy value-equivalence, pickle round-trip, weakref
# top-level hasattr, types top-level type-name hasattr, abc
# full hasattr, gc common surface, atexit full hasattr, signal
# common surface, string letter/digit constants, re common
# surface) are covered in the matching pass fixture
# `test_json_copy_pickle_weakref_abc_gc_value_ops`.
from typing import Any
import types as _types_mod
import inspect as _inspect_mod
import gc as _gc_mod
import random as _random_mod
import string as _string_mod
import decimal as _decimal_mod
import difflib as _difflib_mod

types: Any = _types_mod
inspect: Any = _inspect_mod
gc: Any = _gc_mod
random: Any = _random_mod
string: Any = _string_mod
decimal: Any = _decimal_mod
difflib: Any = _difflib_mod


_ledger: list[int] = []

# 1) types.SimpleNamespace — instance-attribute value contract
#    (mamba: hasattr returns True but the call site raises
#    AttributeError because `types` is typed as a dict)
try:
    _ns = types.SimpleNamespace(a=1, b=2)
    _ok = _ns.a == 1 and _ns.b == 2
except AttributeError:
    _ok = False
assert _ok == True; _ledger.append(1)

# 2) inspect — extended module hasattr surface
#    (mamba: Parameter / Signature / ismodule / getsource /
#    getsourcefile / getfile / currentframe / stack all False)
assert hasattr(inspect, "Parameter") == True; _ledger.append(1)
assert hasattr(inspect, "Signature") == True; _ledger.append(1)
assert hasattr(inspect, "ismodule") == True; _ledger.append(1)
assert hasattr(inspect, "getsource") == True; _ledger.append(1)
assert hasattr(inspect, "getsourcefile") == True; _ledger.append(1)
assert hasattr(inspect, "getfile") == True; _ledger.append(1)
assert hasattr(inspect, "currentframe") == True; _ledger.append(1)
assert hasattr(inspect, "stack") == True; _ledger.append(1)

# 3) gc — extended module hasattr surface
#    (mamba: get_referrers / get_referents both False)
assert hasattr(gc, "get_referrers") == True; _ledger.append(1)
assert hasattr(gc, "get_referents") == True; _ledger.append(1)

# 4) random — seeded-PRNG value contract + extended hasattr
#    (mamba: seed-42 random() returns 0.3745... not 0.6394...;
#    SystemRandom hasattr False)
random.seed(42)
assert random.random() == 0.6394267984578837; _ledger.append(1)
assert hasattr(random, "SystemRandom") == True; _ledger.append(1)

# 5) string — extended hasattr surface
#    (mamba: printable hasattr False)
assert hasattr(string, "printable") == True; _ledger.append(1)

# 6) decimal — extended module hasattr surface
#    (mamba: Context / getcontext / setcontext / localcontext /
#    ROUND_HALF_EVEN / ROUND_HALF_UP / ROUND_DOWN /
#    InvalidOperation / DivisionByZero all False)
assert hasattr(decimal, "Context") == True; _ledger.append(1)
assert hasattr(decimal, "getcontext") == True; _ledger.append(1)
assert hasattr(decimal, "setcontext") == True; _ledger.append(1)
assert hasattr(decimal, "localcontext") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_HALF_EVEN") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_HALF_UP") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_DOWN") == True; _ledger.append(1)
assert hasattr(decimal, "InvalidOperation") == True; _ledger.append(1)
assert hasattr(decimal, "DivisionByZero") == True; _ledger.append(1)

# 7) difflib — extended module hasattr surface + cutoff-filter
#    value contract (mamba: Differ / ndiff / context_diff /
#    restore / HtmlDiff all False, plus get_close_matches
#    returns the unfiltered list ["apple", "ape", "peach"])
assert hasattr(difflib, "Differ") == True; _ledger.append(1)
assert hasattr(difflib, "ndiff") == True; _ledger.append(1)
assert hasattr(difflib, "context_diff") == True; _ledger.append(1)
assert hasattr(difflib, "restore") == True; _ledger.append(1)
assert hasattr(difflib, "HtmlDiff") == True; _ledger.append(1)
assert difflib.get_close_matches("appel", ["apple", "ape", "peach"]) == ["apple", "ape"]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_inspect_decimal_difflib_random_silent {sum(_ledger)} asserts")
