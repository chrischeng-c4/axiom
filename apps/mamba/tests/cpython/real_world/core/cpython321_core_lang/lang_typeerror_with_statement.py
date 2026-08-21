# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typeerror_with_statement"
# subject = "cpython321.lang_typeerror_with_statement"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typeerror_with_statement.py"
# status = "filled"
# ///
"""cpython321.lang_typeerror_with_statement: execute CPython 3.12 seed lang_typeerror_with_statement"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on `with`-statement
# context-manager protocol.
# Surface: CPython raises
#   TypeError("'<type>' object does not support the context manager
#            protocol")
# when `with <obj>` is entered with a value whose type implements
# neither __enter__/__exit__ nor an asynchronous context manager. The
# message also adds "(missed __exit__ method)" / "(missed __enter__
# method)" when only one of the two dunders is defined.
#
# We probe both built-in scalar/container types (int, None, str, list,
# float, dict, tuple, set, bytes) AND user-defined classes that define
# only one of __enter__/__exit__. CPython rejects all of them at the
# `with`-entry hook.
#
# Mamba 0.3.60 currently DOES NOT raise TypeError on ANY of these
# forms; the body of the `with` block executes silently (the context
# manager protocol is not enforced at runtime). This seed pins Fail
# today so the runner surfaces drift when mamba starts honoring the
# context-manager protocol.
#
# `Any`-typed holders push the receiver past static checkers
# (Pyright) and mamba's compile-time enforcement so the runtime
# divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_i: Any = 1
try:
    with _i:
        pass
    raise AssertionError("with int must raise TypeError")
except TypeError:
    _ledger.append(1)

_n: Any = None
try:
    with _n:
        pass
    raise AssertionError("with None must raise TypeError")
except TypeError:
    _ledger.append(1)

_s: Any = "abc"
try:
    with _s:
        pass
    raise AssertionError("with str must raise TypeError")
except TypeError:
    _ledger.append(1)

_lst: Any = [1, 2, 3]
try:
    with _lst:
        pass
    raise AssertionError("with list must raise TypeError")
except TypeError:
    _ledger.append(1)

_f: Any = 1.5
try:
    with _f:
        pass
    raise AssertionError("with float must raise TypeError")
except TypeError:
    _ledger.append(1)

_d: Any = {"a": 1}
try:
    with _d:
        pass
    raise AssertionError("with dict must raise TypeError")
except TypeError:
    _ledger.append(1)

_t: Any = (1, 2)
try:
    with _t:
        pass
    raise AssertionError("with tuple must raise TypeError")
except TypeError:
    _ledger.append(1)

_st: Any = {1, 2}
try:
    with _st:
        pass
    raise AssertionError("with set must raise TypeError")
except TypeError:
    _ledger.append(1)

_b: Any = b"abc"
try:
    with _b:
        pass
    raise AssertionError("with bytes must raise TypeError")
except TypeError:
    _ledger.append(1)

# User-defined class with only __enter__ — missing __exit__ should
# trip the TypeError at the `with`-entry hook
class _OnlyEnter:
    def __enter__(self):
        return self

_oe: Any = _OnlyEnter()
try:
    with _oe:
        pass
    raise AssertionError("with OnlyEnter must raise TypeError")
except TypeError:
    _ledger.append(1)

# User-defined class with only __exit__ — missing __enter__ should
# trip the TypeError at the `with`-entry hook
class _OnlyExit:
    def __exit__(self, *_args):
        return False

_ox: Any = _OnlyExit()
try:
    with _ox:
        pass
    raise AssertionError("with OnlyExit must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_with_statement {sum(_ledger)} asserts")
