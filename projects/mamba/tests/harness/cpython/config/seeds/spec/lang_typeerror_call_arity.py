# Spec seed for CPython TypeError contract on call-shape mismatches.
# Surface: CPython raises TypeError when a callable receives:
#   • fewer positional args than required (missing required arg);
#   • more positional args than accepted;
#   • an unexpected keyword argument;
#   • the same parameter twice (positional + kwarg duplicate);
#   • a positional-only parameter (PEP 570 `/`) passed as kwarg;
#   • a keyword-only parameter (`*`) passed positionally;
#   • too few or too many args to a fixed-arity built-in (len, abs,
#     ord, chr);
#   • a wrong-shape argument to a built-in (ord("ab") — multi-char
#     str instead of single char);
#   • instantiation of an abstract class with unimplemented
#     `@abstractmethod` members.
#
# Mamba 0.3.60 currently DOES NOT raise TypeError at runtime on any
# of these forms; each silently no-ops (returns None) or executes
# with whatever args it receives. Mamba's static checker DOES catch
# extra-args / missing-args at compile time, so `Any`-typed call
# targets are required to push the probe past static dispatch and
# into the runtime — that's where the divergence lives.
#
# This seed pins Fail today so the runner surfaces drift when mamba
# grows runtime call-arity / unexpected-kwarg / abstract-instantiation
# rejection (mass-promote candidate via
# `git mv spec/lang_typeerror_call_arity.py pass/`).
from typing import Any
_ledger: list[int] = []

def _f1(a, b):
    return a + b

_f1_any: Any = _f1
_len: Any = len
_abs: Any = abs
_ord: Any = ord
_chr: Any = chr

# Missing required arg
try:
    _ = _f1_any(1)
    raise AssertionError("missing required arg must raise TypeError")
except TypeError:
    _ledger.append(1)

# Too many positional args
try:
    _ = _f1_any(1, 2, 3)
    raise AssertionError("too many positional args must raise TypeError")
except TypeError:
    _ledger.append(1)

# Unexpected keyword argument
try:
    _ = _f1_any(1, 2, c=3)
    raise AssertionError("unexpected kwarg must raise TypeError")
except TypeError:
    _ledger.append(1)

# Duplicate (positional + kwarg for same parameter)
try:
    _ = _f1_any(1, a=10)
    raise AssertionError("duplicate arg must raise TypeError")
except TypeError:
    _ledger.append(1)

# positional-only parameter (PEP 570) passed as kwarg
def _f_pos(a, b, /, c):
    return a + b + c

_f_pos_any: Any = _f_pos
try:
    _ = _f_pos_any(1, b=2, c=3)
    raise AssertionError("pos-only as kwarg must raise TypeError")
except TypeError:
    _ledger.append(1)

# keyword-only parameter passed positionally
def _f_kw(a, *, b):
    return a + b

_f_kw_any: Any = _f_kw
try:
    _ = _f_kw_any(1, 2)
    raise AssertionError("kw-only as positional must raise TypeError")
except TypeError:
    _ledger.append(1)

# Built-in len() with zero args
try:
    _ = _len()
    raise AssertionError("len() with no args must raise TypeError")
except TypeError:
    _ledger.append(1)

# Built-in len() with too many args
try:
    _ = _len([1], [2])
    raise AssertionError("len(x, y) must raise TypeError")
except TypeError:
    _ledger.append(1)

# Built-in abs() with zero args
try:
    _ = _abs()
    raise AssertionError("abs() with no args must raise TypeError")
except TypeError:
    _ledger.append(1)

# Built-in ord() with zero args
try:
    _ = _ord()
    raise AssertionError("ord() with no args must raise TypeError")
except TypeError:
    _ledger.append(1)

# Built-in ord() with multi-char str (wrong shape)
try:
    _ = _ord("ab")
    raise AssertionError("ord(multi-char) must raise TypeError")
except TypeError:
    _ledger.append(1)

# Built-in chr() with zero args
try:
    _ = _chr()
    raise AssertionError("chr() with no args must raise TypeError")
except TypeError:
    _ledger.append(1)

# Instantiating an abstract class (CPython requires all
# @abstractmethod members to be overridden)
from abc import ABC, abstractmethod

class _Shape(ABC):
    @abstractmethod
    def area(self): ...

_Shape_any: Any = _Shape
try:
    _ = _Shape_any()
    raise AssertionError("instantiating abstract class must raise TypeError")
except TypeError:
    _ledger.append(1)

# Mixed extras: kwargs-before-positionals via *args/**kwargs splat
def _f_two(a, b):
    return a - b

_f_two_any: Any = _f_two

# Too many keyword args via **kwargs splat
try:
    _ = _f_two_any(**{"a": 1, "b": 2, "c": 3})
    raise AssertionError("**kwargs with extra key must raise TypeError")
except TypeError:
    _ledger.append(1)

# Missing arg via *args splat
try:
    _ = _f_two_any(*[1])
    raise AssertionError("*args with too few must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_call_arity {sum(_ledger)} asserts")
