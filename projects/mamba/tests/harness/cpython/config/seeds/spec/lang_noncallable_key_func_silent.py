# Spec seed for CPython TypeError contract on the higher-order-builtin
# `non_callable_key_or_func` corners where mamba silently coerces
# the operand (treats it as "no key" / "no predicate" / "identity
# function" / "start-of-zero") instead of raising the canonical
# "'<type>' object is not callable" TypeError.
#
# Surface: CPython rejects (1) `min(it, key=non_callable)` /
# `max(it, key=non_callable)` / `sorted(it, key=non_callable)`
# because the `key` parameter must be a callable —
# TypeError("'<type>' object is not callable"); (2)
# `list(filter(non_callable, it))` / `list(map(non_callable, it))`
# because the function parameter must be a callable —
# TypeError("'<type>' object is not callable"); (3)
# `sum(it, non_numeric_start)` because the `start` parameter must
# support `+` against the iterable element type —
# TypeError("unsupported operand type(s) for +: '<a>' and '<b>'");
# (4) `functools.reduce(non_callable, it)` because the function
# parameter must be a callable that takes two arguments —
# TypeError("'<type>' object is not callable").
#
# Mamba accepts every form and silently:
#   - returns the unsorted / unfiltered / un-mapped result for
#     `min` / `max` / `sorted` (key ignored, defaults to identity);
#   - returns the truthy-filter result (or empty list) for
#     `filter(non_callable, it)` — masking the operator's intent to
#     filter by predicate;
#   - returns `[None, None, ...]` for `map(non_callable, it)` —
#     producing a list of `None` instead of the canonical "'<type>'
#     object is not callable" TypeError;
#   - returns the sum-of-elements (ignoring start) for
#     `sum(it, non_num_start)` — masking the call-site bug where
#     `start` accidentally arrived as `None` or a dict;
#   - returns `None` for `functools.reduce(non_callable, it)` —
#     propagating downstream as a "NoneType has no attr X" error
#     instead of the canonical "'<type>' object is not callable".
#
# Existing lang_aggregate_fn_wrong_type_silent.py covers `sum([str])`
# (the ELEMENT-side wrong type) and `min([])` / `max([])` (the
# EMPTY-iterable ValueError). Existing
# lang_typeerror_calls_subscripts.py covers `non_callable()` (the
# direct call-site). This seed covers the FRESH divergence family —
# the HIGHER-ORDER-BUILTIN argument-type-routing (where the
# `key=` / `function=` / `start=` parameter itself is the violation,
# not the iterable or element).
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • min([1,2,3], key=5)                → mamba: 1          (TypeError)
#   • min([1,2,3], key='abc')            → mamba: 1          (TypeError)
#   • min([1,2,3], key={1:2})            → mamba: 1          (TypeError)
#   • min([1,2,3], key=(1,2))            → mamba: 1          (TypeError)
#   • min([1,2,3], key=b'abc')           → mamba: 1          (TypeError)
#   • max([1,2,3], key=5)                → mamba: 3          (TypeError)
#   • max([1,2,3], key=b'abc')           → mamba: 3          (TypeError)
#   • sorted([1,2,3], key=5)             → mamba: [1,2,3]    (TypeError)
#   • sorted([1,2,3], key='abc')         → mamba: [1,2,3]    (TypeError)
#   • sorted([1,2,3], key={1:2})         → mamba: [1,2,3]    (TypeError)
#   • list(filter(5, [1,2]))             → mamba: [1,2]      (TypeError)
#   • list(filter('a', [1,2]))           → mamba: []         (TypeError)
#   • list(filter([1,2], [3,4]))         → mamba: []         (TypeError)
#   • list(filter({1:2}, [1,2]))         → mamba: []         (TypeError)
#   • list(map(5, [1,2]))                → mamba: [3, 3]     (TypeError)
#   • list(map('a', [1,2]))              → mamba: []         (TypeError)
#   • list(map({1:2}, [1,2]))            → mamba: [None,None](TypeError)
#   • sum([1,2,3], None)                 → mamba: 6          (TypeError)
#   • sum([1,2,3], {1:2})                → mamba: 6          (TypeError)
#   • functools.reduce(5, [1,2])         → mamba: 3          (TypeError)
#   • functools.reduce(None, [1,2])      → mamba: None       (TypeError)
#   • functools.reduce('a', [1,2])       → mamba: None       (TypeError)
#   • functools.reduce({1:2}, [1,2])     → mamba: None       (TypeError)
#
# CPython contract (uniform across every form):
#   min/max/sorted(it, key=non_callable)
#       → TypeError("'<type>' object is not callable");
#   filter(non_callable, it) / map(non_callable, it)
#       → TypeError("'<type>' object is not callable");
#   sum(it, non_num_start)
#       → TypeError("unsupported operand type(s) for +:
#                    '<start>' and '<elem>'");
#   functools.reduce(non_callable, it)
#       → TypeError("'<type>' object is not callable").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
import functools
_ledger: list[int] = []

_k_int: Any = 5
_k_str: Any = 'abc'
_k_dict: Any = {1: 2}
_k_tuple: Any = (1, 2)
_k_bytes: Any = b'abc'
_k_list: Any = [1, 2]
_k_none: Any = None

# min([1,2,3], key=5) — key must be callable
try:
    _ = min([1, 2, 3], key=_k_int)
    raise AssertionError("min(.., key=5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# min([1,2,3], key='abc')
try:
    _ = min([1, 2, 3], key=_k_str)
    raise AssertionError("min(.., key='abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# min([1,2,3], key={1:2})
try:
    _ = min([1, 2, 3], key=_k_dict)
    raise AssertionError("min(.., key={1:2}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# min([1,2,3], key=(1,2))
try:
    _ = min([1, 2, 3], key=_k_tuple)
    raise AssertionError("min(.., key=(1,2)) must raise TypeError")
except TypeError:
    _ledger.append(1)

# min([1,2,3], key=b'abc')
try:
    _ = min([1, 2, 3], key=_k_bytes)
    raise AssertionError("min(.., key=b'abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# max([1,2,3], key=5)
try:
    _ = max([1, 2, 3], key=_k_int)
    raise AssertionError("max(.., key=5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# max([1,2,3], key=b'abc')
try:
    _ = max([1, 2, 3], key=_k_bytes)
    raise AssertionError("max(.., key=b'abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# sorted([1,2,3], key=5)
try:
    _ = sorted([1, 2, 3], key=_k_int)
    raise AssertionError("sorted(.., key=5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# sorted([1,2,3], key='abc')
try:
    _ = sorted([1, 2, 3], key=_k_str)
    raise AssertionError("sorted(.., key='abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# sorted([1,2,3], key={1:2})
try:
    _ = sorted([1, 2, 3], key=_k_dict)
    raise AssertionError("sorted(.., key={1:2}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# list(filter(5, [1,2])) — function must be callable
try:
    _ = list(filter(_k_int, [1, 2]))
    raise AssertionError("filter(5, ..) must raise TypeError")
except TypeError:
    _ledger.append(1)

# list(filter('a', [1,2]))
try:
    _ = list(filter(_k_str, [1, 2]))
    raise AssertionError("filter('a', ..) must raise TypeError")
except TypeError:
    _ledger.append(1)

# list(filter([1,2], [3,4]))
try:
    _ = list(filter(_k_list, [3, 4]))
    raise AssertionError("filter([1,2], ..) must raise TypeError")
except TypeError:
    _ledger.append(1)

# list(filter({1:2}, [1,2]))
try:
    _ = list(filter(_k_dict, [1, 2]))
    raise AssertionError("filter({1:2}, ..) must raise TypeError")
except TypeError:
    _ledger.append(1)

# list(map(5, [1,2])) — function must be callable
try:
    _ = list(map(_k_int, [1, 2]))
    raise AssertionError("map(5, ..) must raise TypeError")
except TypeError:
    _ledger.append(1)

# list(map('a', [1,2]))
try:
    _ = list(map(_k_str, [1, 2]))
    raise AssertionError("map('a', ..) must raise TypeError")
except TypeError:
    _ledger.append(1)

# list(map({1:2}, [1,2]))
try:
    _ = list(map(_k_dict, [1, 2]))
    raise AssertionError("map({1:2}, ..) must raise TypeError")
except TypeError:
    _ledger.append(1)

# sum([1,2,3], None) — start must support __add__ with int
try:
    _ = sum([1, 2, 3], _k_none)
    raise AssertionError("sum(.., None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# sum([1,2,3], {1:2})
try:
    _ = sum([1, 2, 3], _k_dict)
    raise AssertionError("sum(.., {1:2}) must raise TypeError")
except TypeError:
    _ledger.append(1)

# functools.reduce(5, [1,2]) — function must be callable
try:
    _ = functools.reduce(_k_int, [1, 2])
    raise AssertionError("reduce(5, ..) must raise TypeError")
except TypeError:
    _ledger.append(1)

# functools.reduce(None, [1,2])
try:
    _ = functools.reduce(_k_none, [1, 2])
    raise AssertionError("reduce(None, ..) must raise TypeError")
except TypeError:
    _ledger.append(1)

# functools.reduce('a', [1,2])
try:
    _ = functools.reduce(_k_str, [1, 2])
    raise AssertionError("reduce('a', ..) must raise TypeError")
except TypeError:
    _ledger.append(1)

# functools.reduce({1:2}, [1,2])
try:
    _ = functools.reduce(_k_dict, [1, 2])
    raise AssertionError("reduce({1:2}, ..) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_noncallable_key_func_silent {sum(_ledger)} asserts")
