# Spec seed for CPython TypeError contract on the iteration corners
# that mamba silently coerces to empty container / None. Surface:
# CPython rejects every form of "iterate this non-iterable" —
# `iter(non_iterable)`, `list(non_iterable)`, `tuple(non_iterable)`,
# `set(non_iterable)`, `dict(non_iterable)`, `sorted(non_iterable)`,
# `reversed(non_iterable)`, and `for x in non_iterable: ...`. All
# raise TypeError("<type> object is not iterable") because none of
# `int` / `float` / `bool` / `None` define `__iter__`. Mamba prints
# a "TypeError: object is not iterable" diagnostic to stderr but
# does NOT raise the exception — the call returns silently with an
# empty container or None, meaning downstream `for x in maybe_iter:`
# silently no-ops on accidentally-passed-non-iterable input. This
# is the protocol-class divergence at the heart of the iterator
# protocol — `__iter__` is one of the most fundamental Python
# operations.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • iter(42)              → mamba: None         (TypeError)
#   • iter(3.14)            → mamba: None         (TypeError)
#   • iter(None)            → mamba: None         (TypeError)
#   • iter(True)            → mamba: None         (TypeError)
#   • list(42)              → mamba: []           (TypeError)
#   • tuple(None)           → mamba: ()           (TypeError)
#   • set(42)               → mamba: set()        (TypeError)
#   • dict(42)              → mamba: {}           (TypeError)
#   • sorted(42)            → mamba: []           (TypeError)
#   • reversed(42)          → mamba: None         (TypeError)
#   • for x in 42: ...      → mamba: no-op        (TypeError)
#   • for x in None: ...    → mamba: no-op        (TypeError)
#
# CPython contract (uniform across the iterator protocol):
#   <iter-consumer>(non-iterable)
#       → TypeError("'<type>' object is not iterable").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_int: Any = 42
_float: Any = 3.14
_none: Any = None
_bool: Any = True

# iter(42)
try:
    _ = iter(_int)
    raise AssertionError("iter(42) must raise TypeError")
except TypeError:
    _ledger.append(1)

# iter(3.14)
try:
    _ = iter(_float)
    raise AssertionError("iter(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# iter(None)
try:
    _ = iter(_none)
    raise AssertionError("iter(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# iter(True)
try:
    _ = iter(_bool)
    raise AssertionError("iter(True) must raise TypeError")
except TypeError:
    _ledger.append(1)

# list(42)
try:
    _ = list(_int)
    raise AssertionError("list(42) must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple(None)
try:
    _ = tuple(_none)
    raise AssertionError("tuple(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# set(42)
try:
    _ = set(_int)
    raise AssertionError("set(42) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict(42)
try:
    _ = dict(_int)
    raise AssertionError("dict(42) must raise TypeError")
except TypeError:
    _ledger.append(1)

# sorted(42)
try:
    _ = sorted(_int)
    raise AssertionError("sorted(42) must raise TypeError")
except TypeError:
    _ledger.append(1)

# reversed(42)
try:
    _ = reversed(_int)
    raise AssertionError("reversed(42) must raise TypeError")
except TypeError:
    _ledger.append(1)

# for x in 42 — implicit iter(42) inside for-loop
try:
    for _x in _int:
        pass
    raise AssertionError("for x in 42 must raise TypeError")
except TypeError:
    _ledger.append(1)

# for x in None — implicit iter(None) inside for-loop
try:
    for _x in _none:
        pass
    raise AssertionError("for x in None must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_iter_non_iterable_silent {sum(_ledger)} asserts")
