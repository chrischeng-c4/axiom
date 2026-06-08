# Spec seed for CPython TypeError contract on writes to immutable
# sequences. Surface: CPython's tuple, str, bytes, and range types
# do not support `__setitem__` or `__delitem__`; attempting either
# raises TypeError ("'tuple' object does not support item assignment"
# / "...does not support item deletion"). Slice assignment to the
# same immutables also raises TypeError. Augmented assignment that
# rebinds a name to a new value (e.g., `t += (4,)`) is fine — only
# in-place item/slice mutation must fault.
#
# Mamba 0.3.60 currently DOES NOT raise TypeError on any of these
# forms; each `_t[0] = v`, `_s[0] = "x"`, `_b[0] = 1`, and the slice
# / del variants silently no-op. This seed pins Fail today so the
# runner surfaces drift when mamba grows item-mutation rejection
# on immutable sequence types.
#
# `Any`-typed holders keep static type-checkers (Pyright) from
# flagging the intentional item-assignment-to-immutable before
# runtime.
from typing import Any
_ledger: list[int] = []

_t: Any = (1, 2, 3)
_s: Any = "abc"
_b: Any = b"abc"
_r: Any = range(3)
_fs: Any = frozenset({1, 2, 3})

# tuple item assignment
try:
    _t[0] = 100
    raise AssertionError("tuple[0] = v must raise TypeError")
except TypeError:
    _ledger.append(1)

# str item assignment
try:
    _s[0] = "X"
    raise AssertionError("str[0] = c must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes item assignment
try:
    _b[0] = 1
    raise AssertionError("bytes[0] = i must raise TypeError")
except TypeError:
    _ledger.append(1)

# range item assignment
try:
    _r[0] = 99
    raise AssertionError("range[0] = v must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple item deletion
try:
    del _t[0]
    raise AssertionError("del tuple[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# str item deletion
try:
    del _s[0]
    raise AssertionError("del str[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes item deletion
try:
    del _b[0]
    raise AssertionError("del bytes[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# range item deletion
try:
    del _r[0]
    raise AssertionError("del range[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple slice assignment
try:
    _t[0:1] = (100,)
    raise AssertionError("tuple[0:1] = (v,) must raise TypeError")
except TypeError:
    _ledger.append(1)

# str slice assignment
try:
    _s[0:1] = "X"
    raise AssertionError("str[0:1] = c must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes slice assignment
try:
    _b[0:1] = b"X"
    raise AssertionError("bytes[0:1] = b must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple slice deletion
try:
    del _t[0:1]
    raise AssertionError("del tuple[0:1] must raise TypeError")
except TypeError:
    _ledger.append(1)

# str slice deletion
try:
    del _s[0:1]
    raise AssertionError("del str[0:1] must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes slice deletion
try:
    del _b[0:1]
    raise AssertionError("del bytes[0:1] must raise TypeError")
except TypeError:
    _ledger.append(1)

# frozenset item-style subscript with subscript assignment makes no
# sense (frozensets aren't indexable) but the "item assignment"
# form on a frozenset should still be a TypeError — the type-check
# fault dominates the unsubscriptable fault under CPython.
try:
    _fs[0] = 99
    raise AssertionError("frozenset[0] = v must raise TypeError")
except TypeError:
    _ledger.append(1)

# Two more: tuple/bytes with extended slice
try:
    _t[::2] = (99,)
    raise AssertionError("tuple[::2] = (v,) must raise TypeError")
except TypeError:
    _ledger.append(1)

try:
    _b[::2] = b"X"
    raise AssertionError("bytes[::2] = b must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_immutable_mutation {sum(_ledger)} asserts")
