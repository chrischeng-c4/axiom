# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_attributeerror_unknown_attr"
# subject = "cpython321.lang_attributeerror_unknown_attr"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_attributeerror_unknown_attr.py"
# status = "filled"
# ///
"""cpython321.lang_attributeerror_unknown_attr: execute CPython 3.12 seed lang_attributeerror_unknown_attr"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython AttributeError contract on unknown-attribute
# READ and on setattr of an immutable built-in instance.
# Surface: CPython raises
#   AttributeError("'<type>' object has no attribute '<name>'")
# on any bare attribute read where the attribute isn't defined on the
# instance or its class, and
#   AttributeError("'<type>' object has no attribute '<name>'")
# on any setattr against an immutable built-in (int / str / float /
# None / bytes / tuple / frozenset).
#
# Probes:
#   • bare READ of an unknown attribute on each built-in scalar /
#     container: int / str / list / dict / None / tuple / set / bytes;
#   • setattr against immutable instances: int / None.
#
# Mamba 0.3.60 currently silently returns from a bare unknown-attribute
# read on every built-in type tested, and silently completes setattr
# against int / None. Note that mamba DOES raise AttributeError on the
# adjacent surface "method-call typo" (e.g. `_lst.appen(3)`), so the
# divergence is specifically on the bare-read / bare-setattr paths and
# not on the call-attribute path.
#
# `Any`-typed holders push the receiver past static type-checkers
# (Pyright) and past mamba's compile-time enforcement so the runtime
# divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_i: Any = 1
_s: Any = "abc"
_l: Any = [1, 2]
_d: Any = {1: 2}
_n: Any = None
_t: Any = (1, 2)
_st: Any = {1, 2}
_b: Any = b"abc"

# int.unknownattr — bare read on int instance
try:
    _ = _i.unknownattr
    raise AssertionError("int.unknownattr must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# str.unknownattr
try:
    _ = _s.unknownattr
    raise AssertionError("str.unknownattr must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# list.unknownattr
try:
    _ = _l.unknownattr
    raise AssertionError("list.unknownattr must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# dict.unknownattr
try:
    _ = _d.unknownattr
    raise AssertionError("dict.unknownattr must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# None.unknownattr
try:
    _ = _n.unknownattr
    raise AssertionError("None.unknownattr must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# tuple.unknownattr
try:
    _ = _t.unknownattr
    raise AssertionError("tuple.unknownattr must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# set.unknownattr
try:
    _ = _st.unknownattr
    raise AssertionError("set.unknownattr must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# bytes.unknownattr
try:
    _ = _b.unknownattr
    raise AssertionError("bytes.unknownattr must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# setattr on int — int instances are immutable, no slot for new attrs;
# CPython raises AttributeError "'int' object has no attribute 'foo'"
try:
    _i2: Any = 1
    _i2.foo = "bar"
    raise AssertionError("int.foo = 'bar' must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# setattr on None — NoneType is also fully immutable
try:
    _n2: Any = None
    _n2.foo = "bar"
    raise AssertionError("None.foo = 'bar' must raise AttributeError")
except AttributeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_attributeerror_unknown_attr {sum(_ledger)} asserts")
