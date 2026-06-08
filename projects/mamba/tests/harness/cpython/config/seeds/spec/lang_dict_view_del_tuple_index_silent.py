# Spec seed for CPython TypeError / KeyError / ValueError contract
# on the dict-view subscript / missing-key del / tuple.index missing
# corners that mamba silently returns a default value or accepts the
# operation as a no-op. Surface: CPython rejects integer-subscripting
# of `dict.keys()` / `.values()` / `.items()` because the views are
# set-like (not Sequence) — TypeError("'dict_keys' object is not
# subscriptable"); raises KeyError on `del d[absent]`; and raises
# ValueError on `tuple.index(absent)`. Mamba accepts every form and
# silently returns an element/value/no-op, meaning code that does
# `first = list(d)[0]` accidentally written as `first = d.keys()[0]`
# silently passes, and `del cache[k]` for a stale key silently
# leaves the cache unchanged. tuple.index returning -1 mimics
# str.find rather than tuple.index, breaking the standard
# "value not present" branch.
#
# Probes (every form CPython rejects, mamba silently accepts):
#   • d.keys()[0]                  → mamba: 'a'                (TypeError)
#   • d.values()[0]                → mamba: 1                  (TypeError)
#   • d.items()[0]                 → mamba: ('a', 1)           (TypeError)
#   • d.keys()[1]                  → mamba: 'b'                (TypeError)
#   • d.values()[1]                → mamba: 2                  (TypeError)
#   • d.items()[1]                 → mamba: ('b', 2)           (TypeError)
#   • del d['missing']             → mamba: silent no-op       (KeyError)
#   • del d['nonexistent']         → mamba: silent no-op       (KeyError)
#   • (1,2,3).index(99)            → mamba: -1                 (ValueError)
#   • (1,2,3).index(0)             → mamba: -1                 (ValueError)
#   • ('a','b','c').index('z')     → mamba: -1                 (ValueError)
#
# CPython contract (uniform across every form):
#   d.keys()/d.values()/d.items() [int]
#       → TypeError("'dict_keys'/'dict_values'/'dict_items'
#                    object is not subscriptable");
#   del d[absent_key]
#       → KeyError(absent_key);
#   tuple.index(absent_value)
#       → ValueError("tuple.index(x): x not in tuple").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_d: Any = {"a": 1, "b": 2}
_d2: Any = {"x": 10, "y": 20, "z": 30}
_t_int: Any = (1, 2, 3)
_t_str: Any = ("a", "b", "c")

# dict_keys subscript — TypeError on CPython
try:
    _ = _d.keys()[0]
    raise AssertionError("d.keys()[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict_values subscript — TypeError on CPython
try:
    _ = _d.values()[0]
    raise AssertionError("d.values()[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict_items subscript — TypeError on CPython
try:
    _ = _d.items()[0]
    raise AssertionError("d.items()[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict_keys subscript with index 1 — TypeError on CPython
try:
    _ = _d.keys()[1]
    raise AssertionError("d.keys()[1] must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict_values subscript with index 1 — TypeError on CPython
try:
    _ = _d.values()[1]
    raise AssertionError("d.values()[1] must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict_items subscript with index 1 — TypeError on CPython
try:
    _ = _d.items()[1]
    raise AssertionError("d.items()[1] must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict_keys subscript on different dict — TypeError on CPython
try:
    _ = _d2.keys()[0]
    raise AssertionError("d2.keys()[0] must raise TypeError")
except TypeError:
    _ledger.append(1)

# dict_values subscript on different dict — TypeError on CPython
try:
    _ = _d2.values()[2]
    raise AssertionError("d2.values()[2] must raise TypeError")
except TypeError:
    _ledger.append(1)

# del d[missing] — KeyError on CPython
try:
    del _d["missing"]
    raise AssertionError("del d['missing'] must raise KeyError")
except KeyError:
    _ledger.append(1)

# del d[nonexistent] — KeyError on CPython
try:
    del _d["nonexistent"]
    raise AssertionError("del d['nonexistent'] must raise KeyError")
except KeyError:
    _ledger.append(1)

# del d[absent on different dict] — KeyError on CPython
try:
    del _d2["q"]
    raise AssertionError("del d2['q'] must raise KeyError")
except KeyError:
    _ledger.append(1)

# tuple.index — ValueError on CPython for absent value
try:
    _ = _t_int.index(99)
    raise AssertionError("(1,2,3).index(99) must raise ValueError")
except ValueError:
    _ledger.append(1)

# tuple.index — ValueError on CPython for absent zero
try:
    _ = _t_int.index(0)
    raise AssertionError("(1,2,3).index(0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# tuple.index — ValueError on CPython for absent str
try:
    _ = _t_str.index("z")
    raise AssertionError("('a','b','c').index('z') must raise ValueError")
except ValueError:
    _ledger.append(1)

# tuple.index — ValueError on CPython for absent str (lowercase miss)
try:
    _ = _t_str.index("A")
    raise AssertionError("('a','b','c').index('A') must raise ValueError")
except ValueError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_dict_view_del_tuple_index_silent {sum(_ledger)} asserts")
