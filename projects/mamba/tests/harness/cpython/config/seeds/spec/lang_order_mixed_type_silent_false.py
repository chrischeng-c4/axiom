# Spec seed for CPython TypeError contract on the mixed-type
# ordering corners that mamba silently returns `False` from.
# Surface: CPython rejects (1) any ordering comparison
# (`<` / `>` / `<=` / `>=`) between operands of incomparable
# types — TypeError("'<' not supported between instances of
# '<typeA>' and '<typeB>'"), not silent `False`; (2) ordering on
# complex numbers because `complex` has no natural ordering on the
# complex plane — TypeError("'<' not supported between instances
# of 'complex' and 'complex'"), not silent `False`. Existing
# `lang_*_silent` seeds touch coercion / coerce-to-None / immutable-
# mutation corners but the ordering-on-mixed-types family hasn't
# been pinned yet. This is a high-impact silent-coercion class —
# downstream code that does e.g. `min(items)` over a heterogeneous
# list silently picks "an item" instead of failing loud.
#
# Probes (every form CPython rejects, mamba silently returns False):
#   • 5 < "abc"               → mamba: False        (TypeError)
#   • 5 > "abc"               → mamba: False        (TypeError)
#   • 5 <= "abc"              → mamba: False        (TypeError)
#   • 5 >= "abc"              → mamba: False        (TypeError)
#   • "abc" < 5               → mamba: False        (TypeError)
#   • [1, 2] < {"a": 1}       → mamba: False        (TypeError)
#   • [1, 2] < {1, 2}         → mamba: False        (TypeError)
#   • None < 5                → mamba: False        (TypeError)
#   • None > 5                → mamba: False        (TypeError)
#   • b"abc" < "abc"          → mamba: False        (TypeError)
#   • (1+1j) < (2+1j)         → mamba: False        (TypeError)
#   • (1+1j) > (2+1j)         → mamba: False        (TypeError)
#   • (1+1j) <= (2+1j)        → mamba: False        (TypeError)
#   • (1+1j) < 5              → mamba: False        (TypeError)
#
# CPython contract:
#   ord(<typeA>, <typeB>) where types incomparable
#       → TypeError("'<' not supported between instances of '<typeA>'
#                   and '<typeB>'");
#   complex < complex
#       → TypeError("'<' not supported between instances of 'complex'
#                   and 'complex'").
#
# `Any`-typed holders push the operands past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_int: Any = 5
_str: Any = "abc"
_list: Any = [1, 2]
_dict: Any = {"a": 1}
_set: Any = {1, 2}
_complex: Any = 1 + 1j
_complex2: Any = 2 + 1j
_none: Any = None
_bytes: Any = b"abc"

# int < str
try:
    _ = _int < _str
    raise AssertionError("5 < 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# int > str
try:
    _ = _int > _str
    raise AssertionError("5 > 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# int <= str
try:
    _ = _int <= _str
    raise AssertionError("5 <= 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# int >= str
try:
    _ = _int >= _str
    raise AssertionError("5 >= 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# str < int
try:
    _ = _str < _int
    raise AssertionError("'abc' < 5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# list < dict
try:
    _ = _list < _dict
    raise AssertionError("[1,2] < {'a':1} must raise TypeError")
except TypeError:
    _ledger.append(1)

# list < set
try:
    _ = _list < _set
    raise AssertionError("[1,2] < {1,2} must raise TypeError")
except TypeError:
    _ledger.append(1)

# None < int
try:
    _ = _none < _int
    raise AssertionError("None < 5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# None > int
try:
    _ = _none > _int
    raise AssertionError("None > 5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes < str
try:
    _ = _bytes < _str
    raise AssertionError("b'abc' < 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex < complex — complex has no natural ordering
try:
    _ = _complex < _complex2
    raise AssertionError("(1+1j) < (2+1j) must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex > complex
try:
    _ = _complex > _complex2
    raise AssertionError("(1+1j) > (2+1j) must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex <= complex
try:
    _ = _complex <= _complex2
    raise AssertionError("(1+1j) <= (2+1j) must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex < int
try:
    _ = _complex < _int
    raise AssertionError("(1+1j) < 5 must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_order_mixed_type_silent_false {sum(_ledger)} asserts")
