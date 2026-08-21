# Spec seed for CPython TypeError contract on the hash / iter / len /
# int / float type-coercion corners that mamba silently accepts.
# Surface: CPython rejects (1) `hash(list)` / `hash(dict)` /
# `hash(set)` because mutable containers are not hashable —
# TypeError("unhashable type: 'list/dict/set'"), not silent integer
# fingerprint; (2) `{1, list}` / `{list: v}` because the membership
# probe needs to hash the list element / key — TypeError, not silent
# build that pretends the list compares-equal to itself; (3)
# `iter(int)` / `iter(None)` because neither type implements
# `__iter__` — TypeError("'int' object is not iterable"), not silent
# None that downstream `for x in iter(42)` would treat as zero-shot;
# (4) `len(int)` because int has no `__len__` — TypeError("object of
# type 'int' has no len()"), not silent 0; (5) `int(complex)` /
# `float(complex)` because converting complex to a real scalar loses
# the imaginary part — TypeError("can't convert complex to int/float"),
# not silent 0 / 0.0 that pretends the imaginary part doesn't exist.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • hash([1, 2, 3])             → mamba: int          (TypeError)
#   • hash({1: 2})                → mamba: int          (TypeError)
#   • hash({1, 2, 3})             → mamba: int          (TypeError)
#   • {1, [1, 2]}                 → mamba: set          (TypeError)
#   • {[1, 2]: "v"}               → mamba: dict         (TypeError)
#   • iter(42)                    → mamba: None         (TypeError)
#   • iter(None)                  → mamba: None         (TypeError)
#   • len(42)                     → mamba: 0            (TypeError)
#   • int(1+2j)                   → mamba: 0            (TypeError)
#   • float(1+2j)                 → mamba: 0.0          (TypeError)
#
# CPython contract:
#   hash(list)            → TypeError("unhashable type: 'list'");
#   hash(dict)            → TypeError("unhashable type: 'dict'");
#   hash(set)             → TypeError("unhashable type: 'set'");
#   {1, [1, 2]}           → TypeError("unhashable type: 'list'");
#   {[1, 2]: "v"}         → TypeError("unhashable type: 'list'");
#   iter(int)             → TypeError("'int' object is not iterable");
#   iter(None)            → TypeError("'NoneType' object is not iterable");
#   len(int)              → TypeError("object of type 'int' has no len()");
#   int(complex)          → TypeError("can't convert complex to int");
#   float(complex)        → TypeError("can't convert complex to float").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_list_val: Any = [1, 2, 3]
_dict_val: Any = {1: 2}
_set_val: Any = {1, 2, 3}
_int_val: Any = 42
_none_val: Any = None
_complex_val: Any = 1 + 2j

# hash(list) — list is unhashable
try:
    _ = hash(_list_val)
    raise AssertionError("hash(list) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash(dict) — dict is unhashable
try:
    _ = hash(_dict_val)
    raise AssertionError("hash(dict) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hash(set) — set is unhashable
try:
    _ = hash(_set_val)
    raise AssertionError("hash(set) must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1, [1, 2]} — set with list element needs to hash the list
try:
    _ = {1, _list_val}
    raise AssertionError("{1, list} must raise TypeError")
except TypeError:
    _ledger.append(1)

# {[1, 2]: "v"} — dict with list key needs to hash the list
try:
    _ = {_list_val: "v"}
    raise AssertionError("{list: v} must raise TypeError")
except TypeError:
    _ledger.append(1)

# iter(int) — int is not iterable
try:
    _ = iter(_int_val)
    raise AssertionError("iter(int) must raise TypeError")
except TypeError:
    _ledger.append(1)

# iter(None) — NoneType is not iterable
try:
    _ = iter(_none_val)
    raise AssertionError("iter(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# len(int) — int has no __len__
try:
    _ = len(_int_val)
    raise AssertionError("len(int) must raise TypeError")
except TypeError:
    _ledger.append(1)

# int(complex) — can't convert complex to int
try:
    _ = int(_complex_val)
    raise AssertionError("int(complex) must raise TypeError")
except TypeError:
    _ledger.append(1)

# float(complex) — can't convert complex to float
try:
    _ = float(_complex_val)
    raise AssertionError("float(complex) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_hash_iter_coerce_silent {sum(_ledger)} asserts")
