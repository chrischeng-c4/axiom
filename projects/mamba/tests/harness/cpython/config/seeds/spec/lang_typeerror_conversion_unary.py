# Spec seed for CPython TypeError contract on numeric-conversion
# constructors, bitwise operators against non-int operands, and
# unary `-`/`+`/`~` against non-numeric operands. Surface: CPython
# rejects every form below with TypeError; mamba 0.3.60 silently
# coerces or returns `None` instead of dispatching the
# `__int__`/`__float__`/`__and__`/`__neg__`/... protocol → TypeError
# fallback. Existing lang_typeerror_* seeds cover binary `+`/`-`/`*`
# cross-type arithmetic, calls/subscripts, iter-required positions,
# and unhashable-type usage; conversion-constructor / bitwise /
# unary-operator divergence is the surface this seed adds.
#
# Probes (every form CPython raises TypeError on, mamba silently
# returns 0 / 0.0 / 0j / None / a numeric result):
#   • int(None)         → mamba: 0
#   • int([])           → mamba: 0
#   • int({})           → mamba: 0
#   • int((1,))         → mamba: 0
#   • int({1, 2})       → mamba: 0
#   • float(None)       → mamba: 0.0
#   • float([])         → mamba: 0.0
#   • float({})         → mamba: 0.0
#   • complex([])       → mamba: 0j
#   • complex(None)     → mamba: 0j
#   • 1 & 'a'           → mamba: None
#   • 1 | 'a'           → mamba: None
#   • 1 ^ 'a'           → mamba: None
#   • ~'a'              → mamba: None
#   • ~[]               → mamba: None
#   • ~None             → mamba: None
#   • ~1.5              → mamba: -2.9999… (float invert — wrong type!)
#   • 1.5 & 1           → mamba: None
#   • -'a'              → mamba: None
#   • -[]               → mamba: None
#   • -None             → mamba: None
#   • +'a'              → mamba: None
#   • +[]               → mamba: None
#   • divmod(1, 'a')    → mamba: None
#   • pow(1, 'a')       → mamba: None
#
# CPython contract:
#   int(non-numeric)    → TypeError("int() argument must be a string,
#                            a bytes-like object or a real number,
#                            not '<typename>'");
#   float(non-numeric)  → TypeError("float() argument must be a
#                            string or a real number, not '<typename>'");
#   complex(non-numeric)→ TypeError("complex() first argument must
#                            be a string or a number, not '<typename>'");
#   int & str           → TypeError("unsupported operand type(s) for
#                            &: 'int' and 'str'");
#   ~<non-int>          → TypeError("bad operand type for unary ~:
#                            '<typename>'");
#   -<non-numeric>      → TypeError("bad operand type for unary -:
#                            '<typename>'");
#   divmod(int, str)    → TypeError("unsupported operand type(s) for
#                            divmod(): 'int' and 'str'");
#   pow(int, str)       → TypeError("unsupported operand type(s) for
#                            ** or pow(): 'int' and 'str'").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_None_any: Any = None
_empty_list: Any = []
_empty_dict: Any = {}
_tup_any: Any = (1,)
_set_any: Any = {1, 2}
_one: Any = 1
_str_a: Any = "a"
_lst: Any = [1]
_flt: Any = 1.5

# int(None) — None is not a numeric / string / bytes
try:
    _ = int(_None_any)
    raise AssertionError("int(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# int([]) — list is not a numeric source
try:
    _ = int(_empty_list)
    raise AssertionError("int([]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# int({}) — dict is not a numeric source
try:
    _ = int(_empty_dict)
    raise AssertionError("int({}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# int((1,)) — tuple is not a numeric source (CPython doesn't iterate)
try:
    _ = int(_tup_any)
    raise AssertionError("int((1,)) must raise TypeError")
except TypeError:
    _ledger.append(1)

# int({1, 2}) — set is not a numeric source
try:
    _ = int(_set_any)
    raise AssertionError("int({1, 2}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# float(None)
try:
    _ = float(_None_any)
    raise AssertionError("float(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# float([])
try:
    _ = float(_empty_list)
    raise AssertionError("float([]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# float({})
try:
    _ = float(_empty_dict)
    raise AssertionError("float({}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex([])
try:
    _ = complex(_empty_list)
    raise AssertionError("complex([]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex(None)
try:
    _ = complex(_None_any)
    raise AssertionError("complex(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 1 & 'a' — bitwise AND across int/str
try:
    _ = _one & _str_a
    raise AssertionError("1 & 'a' must raise TypeError")
except TypeError:
    _ledger.append(1)

# 1 | 'a' — bitwise OR across int/str
try:
    _ = _one | _str_a
    raise AssertionError("1 | 'a' must raise TypeError")
except TypeError:
    _ledger.append(1)

# 1 ^ 'a' — bitwise XOR across int/str
try:
    _ = _one ^ _str_a
    raise AssertionError("1 ^ 'a' must raise TypeError")
except TypeError:
    _ledger.append(1)

# ~'a' — unary invert on str
try:
    _ = ~_str_a
    raise AssertionError("~'a' must raise TypeError")
except TypeError:
    _ledger.append(1)

# ~[] — unary invert on list
try:
    _ = ~_lst
    raise AssertionError("~[] must raise TypeError")
except TypeError:
    _ledger.append(1)

# ~None — unary invert on NoneType
try:
    _ = ~_None_any
    raise AssertionError("~None must raise TypeError")
except TypeError:
    _ledger.append(1)

# ~1.5 — unary invert on float (mamba returns -2.9999… ≈ -(x+1))
try:
    _ = ~_flt
    raise AssertionError("~1.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# 1.5 & 1 — bitwise AND across float/int
try:
    _ = _flt & _one
    raise AssertionError("1.5 & 1 must raise TypeError")
except TypeError:
    _ledger.append(1)

# -'a' — unary negate on str
try:
    _ = -_str_a
    raise AssertionError("-'a' must raise TypeError")
except TypeError:
    _ledger.append(1)

# -[] — unary negate on list
try:
    _ = -_lst
    raise AssertionError("-[] must raise TypeError")
except TypeError:
    _ledger.append(1)

# -None — unary negate on NoneType
try:
    _ = -_None_any
    raise AssertionError("-None must raise TypeError")
except TypeError:
    _ledger.append(1)

# +'a' — unary positive on str
try:
    _ = +_str_a
    raise AssertionError("+'a' must raise TypeError")
except TypeError:
    _ledger.append(1)

# +[] — unary positive on list
try:
    _ = +_lst
    raise AssertionError("+[] must raise TypeError")
except TypeError:
    _ledger.append(1)

# divmod(1, 'a') — heterogeneous divmod
try:
    _ = divmod(_one, _str_a)
    raise AssertionError("divmod(1, 'a') must raise TypeError")
except TypeError:
    _ledger.append(1)

# pow(1, 'a') — heterogeneous pow
try:
    _ = pow(_one, _str_a)
    raise AssertionError("pow(1, 'a') must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_conversion_unary {sum(_ledger)} asserts")
