# Spec seed for CPython TypeError contract on the set / frozenset
# method family where the ARGUMENT IS NOT AN ITERABLE (`int`, `None`)
# or NOT HASHABLE (`list`, `dict`) for the add / remove / discard
# elementwise paths. Mamba silently accepts every form and returns
# either the unchanged set, an empty set, or a Boolean answer derived
# from "the right side is empty" — masking the call-site bug where
# the wrong type accidentally reached `.union(...)` / `.update(...)` /
# `.issubset(...)` / `.add(...)`.
#
# Surface (CPython rejects, mamba silently coerces):
#   (1) `set.union(non_iter)` / `set.intersection(non_iter)` /
#       `set.difference(non_iter)` / `set.symmetric_difference(non_iter)`
#       — TypeError("'<type>' object is not iterable");
#   (2) `set.update(non_iter)` / `set.difference_update(non_iter)` /
#       `set.intersection_update(non_iter)` /
#       `set.symmetric_difference_update(non_iter)`
#       — TypeError("'<type>' object is not iterable");
#   (3) `set.issubset(non_iter)` / `set.issuperset(non_iter)` /
#       `set.isdisjoint(non_iter)`
#       — TypeError("'<type>' object is not iterable");
#   (4) `set(non_iter)` / `frozenset(non_iter)` constructor
#       — TypeError("'<type>' object is not iterable");
#   (5) `set.add(unhashable)` / `set.discard(unhashable)` /
#       `set.remove(unhashable)`
#       — TypeError("unhashable type: '<type>'") on CPython;
#       mamba either silently accepts the list/dict into the set
#       (`add`/`discard`) or raises KeyError instead of TypeError
#       (`remove`) — wrong-exception-class divergence.
#
# Mamba behavior:
#   • `{1,2,3}.union(5)` → `{1, 2, 3}` (silent)
#   • `{1,2,3}.intersection(5)` → `set()` (silent — treats 5 as empty)
#   • `{1,2,3}.difference(5)` → `{1, 2, 3}` (silent)
#   • `{1,2,3}.symmetric_difference(5)` → `{1, 2, 3}` (silent)
#   • `{1,2,3}.issubset(5)` → `False` (silent — non-iter treated as empty)
#   • `{1,2,3}.issuperset(5)` → `True` (silent — non-iter treated as empty)
#   • `{1,2,3}.isdisjoint(5)` → `True` (silent)
#   • `{1,2,3}.update(5)` → `{1, 2, 3}` (silent no-op)
#   • `set(5)` → `set()` (silent)
#   • `frozenset(5)` → `frozenset()` (silent)
#   • `{1,2,3}.add({'a':1})` → `{1, 2, 3, {'a': 1}}` (silently
#     inserts an unhashable dict into the set — internal-consistency
#     break);
#   • `{1,2,3}.remove({1:2})` → `KeyError` (wrong exception class,
#     CPython is TypeError; the unhashable element should fail the
#     type check BEFORE the membership check);
#   • `{1,2,3}.discard([1])` → silent (no membership match because
#     mamba can't hash the list, but it doesn't raise either).
#
# CPython contract (uniform across every form):
#   set/frozenset.union/.intersection/.difference/.symmetric_difference
#       / .update / .difference_update / .intersection_update /
#       .symmetric_difference_update / .issubset / .issuperset /
#       .isdisjoint / set() / frozenset()
#       on a non-iterable → TypeError("'<type>' object is not iterable")
#   set.add / .discard / .remove on an unhashable element
#       → TypeError("unhashable type: '<type>'")
#
# `Any`-typed module-level holders push the operand past static
# type-checkers (Pyright) and past mamba's compile-time argtype
# enforcement so the runtime divergence is exercised.
from typing import Any

_ledger: list[int] = []

_int: Any = 5
_none: Any = None
_lst: Any = [1, 2]
_dct: Any = {"a": 1}

# (1) Non-mutating set methods with non-iterable

# {1,2,3}.union(5) — TypeError: 'int' object is not iterable
try:
    _ = {1, 2, 3}.union(_int)
    raise AssertionError("{1,2,3}.union(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.union(None)
try:
    _ = {1, 2, 3}.union(_none)
    raise AssertionError("{1,2,3}.union(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.intersection(5)
try:
    _ = {1, 2, 3}.intersection(_int)
    raise AssertionError("{1,2,3}.intersection(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.intersection(None)
try:
    _ = {1, 2, 3}.intersection(_none)
    raise AssertionError("{1,2,3}.intersection(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.difference(5)
try:
    _ = {1, 2, 3}.difference(_int)
    raise AssertionError("{1,2,3}.difference(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.difference(None)
try:
    _ = {1, 2, 3}.difference(_none)
    raise AssertionError("{1,2,3}.difference(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.symmetric_difference(5)
try:
    _ = {1, 2, 3}.symmetric_difference(_int)
    raise AssertionError("{1,2,3}.symmetric_difference(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.symmetric_difference(None)
try:
    _ = {1, 2, 3}.symmetric_difference(_none)
    raise AssertionError("{1,2,3}.symmetric_difference(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# (2) Mutating set methods with non-iterable

# {1,2,3}.update(5)
try:
    _s = {1, 2, 3}
    _s.update(_int)
    raise AssertionError("{1,2,3}.update(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.update(None)
try:
    _s = {1, 2, 3}
    _s.update(_none)
    raise AssertionError("{1,2,3}.update(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.difference_update(5)
try:
    _s = {1, 2, 3}
    _s.difference_update(_int)
    raise AssertionError("{1,2,3}.difference_update(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.intersection_update(5)
try:
    _s = {1, 2, 3}
    _s.intersection_update(_int)
    raise AssertionError("{1,2,3}.intersection_update(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.symmetric_difference_update(5)
try:
    _s = {1, 2, 3}
    _s.symmetric_difference_update(_int)
    raise AssertionError("{1,2,3}.symmetric_difference_update(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# (3) Predicate set methods with non-iterable

# {1,2,3}.issubset(5)
try:
    _ = {1, 2, 3}.issubset(_int)
    raise AssertionError("{1,2,3}.issubset(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.issuperset(5)
try:
    _ = {1, 2, 3}.issuperset(_int)
    raise AssertionError("{1,2,3}.issuperset(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.isdisjoint(5)
try:
    _ = {1, 2, 3}.isdisjoint(_int)
    raise AssertionError("{1,2,3}.isdisjoint(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.isdisjoint(None)
try:
    _ = {1, 2, 3}.isdisjoint(_none)
    raise AssertionError("{1,2,3}.isdisjoint(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# (4) Constructors

# set(5) — TypeError
try:
    _ = set(_int)
    raise AssertionError("set(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# set(None)
try:
    _ = set(_none)
    raise AssertionError("set(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# frozenset(5)
try:
    _ = frozenset(_int)
    raise AssertionError("frozenset(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# frozenset(None)
try:
    _ = frozenset(_none)
    raise AssertionError("frozenset(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# (5) Unhashable-element operations

# {1,2,3}.add([1,2]) — unhashable list
try:
    _s = {1, 2, 3}
    _s.add(_lst)
    raise AssertionError("{1,2,3}.add([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.add({'a':1}) — unhashable dict
try:
    _s = {1, 2, 3}
    _s.add(_dct)
    raise AssertionError("{1,2,3}.add({'a':1}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.discard([1,2]) — unhashable list
try:
    _s = {1, 2, 3}
    _s.discard(_lst)
    raise AssertionError("{1,2,3}.discard([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1,2,3}.remove([1,2]) — unhashable list; CPython requires TypeError,
# not KeyError, because the type-check happens before the membership
# check (mamba inverts this — raises KeyError on the missing hash).
try:
    _s = {1, 2, 3}
    _s.remove(_lst)
    raise AssertionError("{1,2,3}.remove([1,2]) must raise TypeError (not KeyError)")
except TypeError:
    _ledger.append(1)

# {1,2,3}.remove({'a':1})
try:
    _s = {1, 2, 3}
    _s.remove(_dct)
    raise AssertionError("{1,2,3}.remove({'a':1}) must raise TypeError (not KeyError)")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_set_method_non_iterable_silent {sum(_ledger)} asserts")
