# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_attr_missing_silent_none"
# subject = "cpython321.lang_attr_missing_silent_none"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_attr_missing_silent_none.py"
# status = "filled"
# ///
"""cpython321.lang_attr_missing_silent_none: execute CPython 3.12 seed lang_attr_missing_silent_none"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython AttributeError contract on the missing-
# attribute corners that mamba silently returns `None` from. Surface:
# CPython rejects reading any unknown attribute on any builtin
# scalar / container / NoneType / function / module — every
# `obj.<missing>` raises `AttributeError("'<type>' object has no
# attribute '<name>'")`. Mamba silently returns `None` for the same
# read, meaning downstream code that does e.g. `if obj.handler:
# obj.handler(arg)` silently no-ops on a typo'd attribute name
# instead of failing loud. This is one of the highest-impact silent
# divergences in mamba because attribute lookup is a daily operation
# across every Python codebase. Existing `lang_*_silent` seeds cover
# call-arity / type-coerce / immutable-mutation corners but the
# missing-attribute family hasn't been pinned yet.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • (42).nope                 → mamba: None        (AttributeError)
#   • (1.5).nope                → mamba: None        (AttributeError)
#   • (1+1j).nope               → mamba: None        (AttributeError)
#   • (True).nope               → mamba: None        (AttributeError)
#   • "abc".nope                → mamba: None        (AttributeError)
#   • b"abc".nope               → mamba: None        (AttributeError)
#   • (1, 2).nope               → mamba: None        (AttributeError)
#   • [1, 2].nope               → mamba: None        (AttributeError)
#   • {"a": 1}.nope             → mamba: None        (AttributeError)
#   • {1, 2}.nope               → mamba: None        (AttributeError)
#   • frozenset([1]).nope       → mamba: None        (AttributeError)
#   • None.nope                 → mamba: None        (AttributeError)
#   • len.nope                  → mamba: None        (AttributeError)
#
# CPython contract (uniform across every type):
#   any_obj.<missing>
#       → AttributeError("'<type>' object has no attribute '<name>'").
#
# `Any`-typed holders push the receiver past static type-checkers
# (Pyright) and past mamba's compile-time attribute resolution so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_int: Any = 42
_float: Any = 1.5
_complex: Any = 1 + 1j
_bool: Any = True
_str: Any = "abc"
_bytes: Any = b"abc"
_tup: Any = (1, 2)
_list: Any = [1, 2]
_dict: Any = {"a": 1}
_set: Any = {1, 2}
_fset: Any = frozenset([1, 2])
_none: Any = None
_fn: Any = len

# int has no .nope
try:
    _ = _int.nope
    raise AssertionError("(42).nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# float has no .nope
try:
    _ = _float.nope
    raise AssertionError("(1.5).nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# complex has no .nope
try:
    _ = _complex.nope
    raise AssertionError("(1+1j).nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# bool has no .nope
try:
    _ = _bool.nope
    raise AssertionError("True.nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# str has no .nope
try:
    _ = _str.nope
    raise AssertionError("'abc'.nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# bytes has no .nope
try:
    _ = _bytes.nope
    raise AssertionError("b'abc'.nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# tuple has no .nope
try:
    _ = _tup.nope
    raise AssertionError("(1,2).nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# list has no .nope
try:
    _ = _list.nope
    raise AssertionError("[1,2].nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# dict has no .nope
try:
    _ = _dict.nope
    raise AssertionError("{'a':1}.nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# set has no .nope
try:
    _ = _set.nope
    raise AssertionError("{1,2}.nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# frozenset has no .nope
try:
    _ = _fset.nope
    raise AssertionError("frozenset([1,2]).nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# NoneType has no .nope
try:
    _ = _none.nope
    raise AssertionError("None.nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# builtin function has no .nope
try:
    _ = _fn.nope
    raise AssertionError("len.nope must raise AttributeError")
except AttributeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_attr_missing_silent_none {sum(_ledger)} asserts")
