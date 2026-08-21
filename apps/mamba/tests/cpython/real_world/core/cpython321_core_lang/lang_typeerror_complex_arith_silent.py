# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_typeerror_complex_arith_silent"
# subject = "cpython321.lang_typeerror_complex_arith_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_typeerror_complex_arith_silent.py"
# status = "filled"
# ///
"""cpython321.lang_typeerror_complex_arith_silent: execute CPython 3.12 seed lang_typeerror_complex_arith_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython TypeError contract on complex-number
# arithmetic / ordering. Surface: CPython 3 makes `complex` strict
# about (1) floor-division, modulo, and divmod (the codomain of these
# operations is undefined for complex numbers, so they're TypeError on
# both same-type and mixed-type forms), and (2) ordering comparisons
# (`<`, `<=`, `>`, `>=`) for which complex has no defined ordering
# even against another complex. Mamba 0.3.60 silently returns `None`
# / `False` instead of dispatching the protocol fallback to TypeError.
# Existing lang_typeerror_arithmetic / lang_typeerror_ordering_mixed
# seeds cover str<int / list<int / dict<dict / sorted-mixed and
# generic non-numeric arithmetic angles; this seed adds the
# complex-floordiv / complex-modulo / complex-divmod / complex-ordering
# family.
#
# Probes (every form CPython raises TypeError on, mamba returns a
# wrong-shape value):
#   • complex % complex            → mamba: None  (TypeError)
#   • complex // complex           → mamba: None  (TypeError)
#   • divmod(complex, complex)     → mamba: None  (TypeError)
#   • complex < complex            → mamba: False (TypeError)
#   • complex <= complex           → mamba: False (TypeError)
#   • complex > complex            → mamba: False (TypeError)
#   • complex >= complex           → mamba: False (TypeError)
#   • complex < int                → mamba: False (TypeError)
#   • complex < float              → mamba: False (TypeError)
#   • int >= complex               → mamba: False (TypeError)
#   • int % complex / int // complex / float % complex
#                                  → mamba: None  (TypeError)
#   • complex % int / complex // int / divmod(complex, int) / divmod(int, complex)
#                                  → mamba: None  (TypeError)
#
# CPython contract:
#   complex // complex     → TypeError("unsupported operand type(s)
#                                  for //: 'complex' and 'complex'");
#   complex % complex      → TypeError("unsupported operand type(s)
#                                  for %: 'complex' and 'complex'");
#   divmod(complex, _)     → TypeError("unsupported operand type(s)
#                                  for divmod(): 'complex' and ...");
#   complex < complex      → TypeError("'<' not supported between
#                                  instances of 'complex' and
#                                  'complex'");
#   complex < int / float  → TypeError("'<' not supported between
#                                  instances of 'complex' and
#                                  '<typename>'").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_c: Any = complex(1, 2)
_d: Any = complex(3, 4)
_i: Any = 5
_f: Any = 1.5

# complex % complex — modulo undefined for complex
try:
    _ = _c % _d
    raise AssertionError("complex % complex must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex // complex — floor-div undefined for complex
try:
    _ = _c // _d
    raise AssertionError("complex // complex must raise TypeError")
except TypeError:
    _ledger.append(1)

# divmod(complex, complex)
try:
    _ = divmod(_c, _d)
    raise AssertionError("divmod(complex, complex) must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex < complex — ordering undefined for complex
try:
    _ = _c < _d
    raise AssertionError("complex < complex must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex <= complex
try:
    _ = _c <= _d
    raise AssertionError("complex <= complex must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex > complex
try:
    _ = _c > _d
    raise AssertionError("complex > complex must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex >= complex
try:
    _ = _c >= _d
    raise AssertionError("complex >= complex must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex < int
try:
    _ = _c < _i
    raise AssertionError("complex < int must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex < float
try:
    _ = _c < _f
    raise AssertionError("complex < float must raise TypeError")
except TypeError:
    _ledger.append(1)

# int >= complex
try:
    _ = _i >= _c
    raise AssertionError("int >= complex must raise TypeError")
except TypeError:
    _ledger.append(1)

# int % complex
try:
    _ = _i % _c
    raise AssertionError("int % complex must raise TypeError")
except TypeError:
    _ledger.append(1)

# int // complex
try:
    _ = _i // _c
    raise AssertionError("int // complex must raise TypeError")
except TypeError:
    _ledger.append(1)

# float % complex
try:
    _ = _f % _c
    raise AssertionError("float % complex must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex % int
try:
    _ = _c % _i
    raise AssertionError("complex % int must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex // int
try:
    _ = _c // _i
    raise AssertionError("complex // int must raise TypeError")
except TypeError:
    _ledger.append(1)

# divmod(int, complex)
try:
    _ = divmod(_i, _c)
    raise AssertionError("divmod(int, complex) must raise TypeError")
except TypeError:
    _ledger.append(1)

# divmod(complex, int)
try:
    _ = divmod(_c, _i)
    raise AssertionError("divmod(complex, int) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_complex_arith_silent {sum(_ledger)} asserts")
