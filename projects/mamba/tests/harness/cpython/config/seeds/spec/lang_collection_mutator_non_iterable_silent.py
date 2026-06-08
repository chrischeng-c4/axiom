# Spec seed for CPython TypeError contract on the collection-
# mutator / set-method corners that mamba silently no-ops or
# returns an empty container. Surface: CPython rejects (1)
# `dict.update(non-mapping)` because `update` requires a
# mapping or iterable-of-2-tuples — TypeError("'<type>' object
# is not iterable") for `dict.update(5)` / `dict.update(None)`;
# (2) `list.extend(non-iterable)` because `extend` walks the
# iterator protocol — TypeError("'<type>' object is not
# iterable") for `list.extend(5)` / `list.extend(None)`;
# (3) `set.union(non-iterable)` / `.intersection(...)` /
# `.difference(...)` because every set-builder method walks
# the iterator protocol — TypeError; (4) `list[float]` and
# `list[None]` because list indices must be int (or slice) —
# TypeError. Mamba accepts every form and silently no-ops the
# mutator (leaving the collection untouched) or returns an
# empty result, so code like `cache.update(maybe_dict)` where
# `maybe_dict` is accidentally `None` silently leaves the cache
# unchanged, and `seen.update(other_seen_or_None)` silently
# misses every record from `other_seen` whenever it's `None`.
#
# Existing lang_typeerror_* seeds cover str/bytes method-arg
# type-checks (`''.join([1])`, `.startswith(int)`, etc.) but
# none specifically cover the mutator/set-method family routed
# through `Any`.
#
# Probes (every form CPython rejects, mamba silently no-ops):
#   • d.update(5)                  → mamba: dict unchanged    (TypeError)
#   • d.update(None)               → mamba: dict unchanged    (TypeError)
#   • d.update(3.14)               → mamba: dict unchanged    (TypeError)
#   • l.extend(5)                  → mamba: list unchanged    (TypeError)
#   • l.extend(None)               → mamba: list unchanged    (TypeError)
#   • l.extend(3.14)               → mamba: list unchanged    (TypeError)
#   • s.union(5)                   → mamba: set unchanged     (TypeError)
#   • s.union(None)                → mamba: set unchanged     (TypeError)
#   • s.intersection(5)            → mamba: set()             (TypeError)
#   • s.intersection(None)         → mamba: set()             (TypeError)
#   • s.difference(5)              → mamba: set unchanged     (TypeError)
#   • s.difference(None)           → mamba: set unchanged     (TypeError)
#   • lst[5.5]                     → mamba: None              (TypeError)
#   • lst[None]                    → mamba: None              (TypeError)
#
# CPython contract (uniform across every mutator):
#   dict.update(non-mapping)
#       → TypeError("'<type>' object is not iterable" or
#                    "cannot convert dictionary update sequence
#                     element #0 to a sequence");
#   list.extend(non-iterable)
#       → TypeError("'<type>' object is not iterable");
#   set.union / intersection / difference (non-iterable)
#       → TypeError("'<type>' object is not iterable");
#   list[float] / list[None]
#       → TypeError("list indices must be integers or slices,
#                    not <type>").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_int: Any = 5
_none: Any = None
_float: Any = 3.14

# dict.update(int) — non-mapping
try:
    _d: Any = {"a": 1}
    _d.update(_int)
    raise AssertionError("d.update(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.update(None)
try:
    _d: Any = {"a": 1}
    _d.update(_none)
    raise AssertionError("d.update(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict.update(float)
try:
    _d: Any = {"a": 1}
    _d.update(_float)
    raise AssertionError("d.update(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# list.extend(int) — non-iterable
try:
    _l: Any = [1, 2]
    _l.extend(_int)
    raise AssertionError("l.extend(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# list.extend(None)
try:
    _l: Any = [1, 2]
    _l.extend(_none)
    raise AssertionError("l.extend(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# list.extend(float)
try:
    _l: Any = [1, 2]
    _l.extend(_float)
    raise AssertionError("l.extend(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# set.union(int) — non-iterable
try:
    _s: Any = {1, 2}
    _ = _s.union(_int)
    raise AssertionError("s.union(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# set.union(None)
try:
    _s: Any = {1, 2}
    _ = _s.union(_none)
    raise AssertionError("s.union(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# set.intersection(int)
try:
    _s: Any = {1, 2}
    _ = _s.intersection(_int)
    raise AssertionError("s.intersection(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# set.intersection(None)
try:
    _s: Any = {1, 2}
    _ = _s.intersection(_none)
    raise AssertionError("s.intersection(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# set.difference(int)
try:
    _s: Any = {1, 2}
    _ = _s.difference(_int)
    raise AssertionError("s.difference(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# set.difference(None)
try:
    _s: Any = {1, 2}
    _ = _s.difference(_none)
    raise AssertionError("s.difference(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# list[float] — list indices must be int
try:
    _l: Any = [1, 2, 3]
    _ = _l[5.5]
    raise AssertionError("list[5.5] must raise TypeError")
except TypeError:
    _ledger.append(1)

# list[None]
try:
    _l: Any = [1, 2, 3]
    _ = _l[_none]
    raise AssertionError("list[None] must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_collection_mutator_non_iterable_silent {sum(_ledger)} asserts")
