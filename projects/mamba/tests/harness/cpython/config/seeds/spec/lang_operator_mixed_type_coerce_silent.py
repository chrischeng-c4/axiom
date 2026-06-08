# Spec seed for CPython TypeError contract on the binary-operator
# corners that mamba silently returns `None` from when both operands
# are funneled through `Any`-typed holders. Surface: CPython's binary
# operators (`+`, `-`, `*`, `/`) reject (1) `int + str` because the
# `+` protocol is type-checked against both operands' `__add__` /
# `__radd__` — `unsupported operand type(s) for +: 'int' and 'str'`,
# not silent `None`; (2) `str - str` because strings have no `-`
# protocol at all — TypeError, not silent `None`; (3) `list - list`
# because list has no `__sub__` (set difference is set-only) —
# TypeError, not silent `None`; (4) `dict + dict` / `dict + list`
# because dict has no concatenation protocol (the merge operator is
# `|` on 3.9+, not `+`) — TypeError, not silent `None`; (5) `str *
# str` / `list * list` because sequence multiplication requires an
# int second operand — TypeError, not silent `None`. Existing
# `lang_*_silent` seeds touch arithmetic / coercion corners, but
# none specifically cover the binary-operator type-mismatch family
# routed through `Any`.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • 1 + "a"                     → mamba: None     (TypeError)
#   • "abc" - "a"                 → mamba: None     (TypeError)
#   • [1, 2, 3] - [1]             → mamba: None     (TypeError)
#   • {1: 2} + {3: 4}             → mamba: None     (TypeError)
#   • {1: 2} + [3, 4]             → mamba: None     (TypeError)
#   • "abc" * "abc"               → mamba: None     (TypeError)
#   • [1, 2] * [3, 4]             → mamba: None     (TypeError)
#
# CPython contract:
#   int + str         → TypeError("unsupported operand type(s) for +:
#                              'int' and 'str'");
#   str - str         → TypeError("unsupported operand type(s) for -:
#                              'str' and 'str'");
#   list - list       → TypeError("unsupported operand type(s) for -:
#                              'list' and 'list'");
#   dict + dict       → TypeError("unsupported operand type(s) for +:
#                              'dict' and 'dict'");
#   dict + list       → TypeError("unsupported operand type(s) for +:
#                              'dict' and 'list'");
#   str * str         → TypeError("can't multiply sequence by non-
#                              int of type 'str'");
#   list * list       → TypeError("can't multiply sequence by non-
#                              int of type 'list'").
#
# `Any`-typed holders push the operands past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised. Without the `Any` wrapping
# mamba refuses to compile the expression (`operand type mismatch`),
# which would mean the fixture never reaches a runtime to compare.
from typing import Any
_ledger: list[int] = []

_int: Any = 1
_str_a: Any = "a"
_str_abc: Any = "abc"
_list_123: Any = [1, 2, 3]
_list_1: Any = [1]
_list_34: Any = [3, 4]
_list_12: Any = [1, 2]
_dict_12: Any = {1: 2}
_dict_34: Any = {3: 4}

# 1 + "a" — int + str has no defined protocol
try:
    _ = _int + _str_a
    raise AssertionError("1 + 'a' must raise TypeError")
except TypeError:
    _ledger.append(1)

# "abc" - "a" — str has no __sub__
try:
    _ = _str_abc - _str_a
    raise AssertionError("'abc' - 'a' must raise TypeError")
except TypeError:
    _ledger.append(1)

# [1, 2, 3] - [1] — list has no __sub__ (set difference is set-only)
try:
    _ = _list_123 - _list_1
    raise AssertionError("[1,2,3] - [1] must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1: 2} + {3: 4} — dict has no __add__ (use | on 3.9+)
try:
    _ = _dict_12 + _dict_34
    raise AssertionError("{1:2} + {3:4} must raise TypeError")
except TypeError:
    _ledger.append(1)

# {1: 2} + [3, 4] — dict + list cross-type concat
try:
    _ = _dict_12 + _list_34
    raise AssertionError("{1:2} + [3,4] must raise TypeError")
except TypeError:
    _ledger.append(1)

# "abc" * "abc" — str * str: seq mul requires int
try:
    _ = _str_abc * _str_abc
    raise AssertionError("'abc' * 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# [1, 2] * [3, 4] — list * list: seq mul requires int
try:
    _ = _list_12 * _list_34
    raise AssertionError("[1,2] * [3,4] must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_operator_mixed_type_coerce_silent {sum(_ledger)} asserts")
