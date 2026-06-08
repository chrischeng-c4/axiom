# Spec seed for CPython TypeError contract on item-assignment /
# del corners over immutable sequences that mamba silently
# no-ops. Surface: CPython rejects every form of in-place
# mutation through `obj[i] = v` / `del obj[i]` / `obj[i:j] = v`
# when `obj` is an immutable sequence (`tuple` / `str` / `bytes`)
# because those types don't define `__setitem__` / `__delitem__` —
# TypeError("'<type>' object does not support item assignment" /
# "'<type>' object doesn't support item deletion"). Mamba accepts
# the subscript-assignment / del syntax and silently leaves the
# original immutable object unchanged, meaning code that tries to
# mutate an immutable through subscript syntax silently goes
# nowhere — a recipe for off-by-one / state-loss bugs that vanish
# without a stack trace. (Note: `.append` / `.add` on immutable
# types DO raise AttributeError on mamba — only subscript / del
# silently no-op, so this seed pins the silent class.)
#
# Probes (every form CPython rejects, mamba silently no-ops):
#   • tup[0] = 99           → mamba: (1,2,3) unchanged   (TypeError)
#   • str[0] = "X"          → mamba: 'abc' unchanged     (TypeError)
#   • bytes[0] = 99         → mamba: b'abc' unchanged    (TypeError)
#   • del tup[0]            → mamba: (1,2,3) unchanged   (TypeError)
#   • del str[0]            → mamba: 'abc' unchanged     (TypeError)
#   • del bytes[0]          → mamba: b'abc' unchanged    (TypeError)
#   • tup[0:2] = [99, 99]   → mamba: (1,2,3) unchanged   (TypeError)
#   • str[0:1] = "X"        → mamba: 'abc' unchanged     (TypeError)
#   • del tup[0:1]          → mamba: (1,2,3) unchanged   (TypeError)
#
# CPython contract:
#   immutable_seq[i] = v
#       → TypeError("'<type>' object does not support item
#                   assignment");
#   del immutable_seq[i]
#       → TypeError("'<type>' object doesn't support item
#                   deletion");
#   immutable_seq[i:j] = iterable
#       → TypeError (same family).
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_tup: Any = (1, 2, 3)
_str: Any = "abc"
_bytes: Any = b"abc"

# tup[0] = 99
try:
    _tup[0] = 99
    raise AssertionError("tup[0] = 99 must raise TypeError")
except TypeError:
    _ledger.append(1)

# str[0] = "X"
try:
    _str[0] = "X"
    raise AssertionError("str[0] = 'X' must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes[0] = 99
try:
    _bytes[0] = 99
    raise AssertionError("bytes[0] = 99 must raise TypeError")
except TypeError:
    _ledger.append(1)

# del tup[0]
try:
    del _tup[0]
    raise AssertionError("del tup[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# del str[0]
try:
    del _str[0]
    raise AssertionError("del str[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# del bytes[0]
try:
    del _bytes[0]
    raise AssertionError("del bytes[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# tup[0:2] = [99, 99]
try:
    _tup[0:2] = [99, 99]
    raise AssertionError("tup[0:2] = [99, 99] must raise TypeError")
except TypeError:
    _ledger.append(1)

# str[0:1] = "X"
try:
    _str[0:1] = "X"
    raise AssertionError("str[0:1] = 'X' must raise TypeError")
except TypeError:
    _ledger.append(1)

# del tup[0:1]
try:
    del _tup[0:1]
    raise AssertionError("del tup[0:1] must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_immutable_mutate_silent {sum(_ledger)} asserts")
