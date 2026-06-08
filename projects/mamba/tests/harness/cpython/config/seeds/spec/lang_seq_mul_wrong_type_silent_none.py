# Spec seed for CPython TypeError contract on the sequence-
# multiplication corners that mamba silently returns `None` from.
# Surface: CPython rejects `seq * non_int` for every sequence type
# (`str` / `bytes` / `list` / `tuple`) because the repeat operator
# is integer-only — TypeError("can't multiply sequence by non-int
# of type '<type>'"). `seq * 2.5`, `seq * "2"`, `seq * None`,
# `seq * [3]`, `seq * complex` all raise on CPython. Mamba accepts
# every form and silently returns `None`, meaning code that does
# `padding = " " * width` silently turns into `padding = None`
# when `width` is a float, str, or other non-int. This is the
# sequence-repeat protocol's strong-typing contract — every Python
# sequence delegates `__mul__` to `int.__index__` on the
# right operand, so non-integer types must raise.
#
# Probes (every form CPython rejects, mamba silently returns None):
#   • 'ab' * 2.5              → mamba: None         (TypeError)
#   • [1,2] * 2.5             → mamba: None         (TypeError)
#   • (1,2) * 2.5             → mamba: None         (TypeError)
#   • b'ab' * 2.5             → mamba: None         (TypeError)
#   • 'ab' * '2'              → mamba: None         (TypeError)
#   • [1,2] * '2'             → mamba: None         (TypeError)
#   • 'ab' * None             → mamba: None         (TypeError)
#   • [1,2] * None            → mamba: None         (TypeError)
#   • [1,2] * [3]             → mamba: None         (TypeError)
#   • 'ab' * [3]              → mamba: None         (TypeError)
#   • 'ab' * (2+0j)           → mamba: None         (TypeError)
#
# CPython contract (uniform across every sequence type):
#   <sequence> * <non-int-non-bool>
#       → TypeError("can't multiply sequence by non-int of type
#                   '<bad-type>'").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_str: Any = "ab"
_list: Any = [1, 2]
_tup: Any = (1, 2)
_bytes: Any = b"ab"

_float_mul: Any = 2.5
_str_mul: Any = "2"
_none_mul: Any = None
_list_mul: Any = [3]
_complex_mul: Any = 2+0j

# str * float
try:
    _ = _str * _float_mul
    raise AssertionError("'ab' * 2.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# list * float
try:
    _ = _list * _float_mul
    raise AssertionError("[1,2] * 2.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# tuple * float
try:
    _ = _tup * _float_mul
    raise AssertionError("(1,2) * 2.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes * float
try:
    _ = _bytes * _float_mul
    raise AssertionError("b'ab' * 2.5 must raise TypeError")
except TypeError:
    _ledger.append(1)

# str * str
try:
    _ = _str * _str_mul
    raise AssertionError("'ab' * '2' must raise TypeError")
except TypeError:
    _ledger.append(1)

# list * str
try:
    _ = _list * _str_mul
    raise AssertionError("[1,2] * '2' must raise TypeError")
except TypeError:
    _ledger.append(1)

# str * None
try:
    _ = _str * _none_mul
    raise AssertionError("'ab' * None must raise TypeError")
except TypeError:
    _ledger.append(1)

# list * None
try:
    _ = _list * _none_mul
    raise AssertionError("[1,2] * None must raise TypeError")
except TypeError:
    _ledger.append(1)

# list * list
try:
    _ = _list * _list_mul
    raise AssertionError("[1,2] * [3] must raise TypeError")
except TypeError:
    _ledger.append(1)

# str * list
try:
    _ = _str * _list_mul
    raise AssertionError("'ab' * [3] must raise TypeError")
except TypeError:
    _ledger.append(1)

# str * complex
try:
    _ = _str * _complex_mul
    raise AssertionError("'ab' * (2+0j) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_seq_mul_wrong_type_silent_none {sum(_ledger)} asserts")
