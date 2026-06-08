# Spec seed for CPython TypeError / ValueError contract on the
# aggregate / arithmetic builtins that mamba silently coerces
# to 0 / None. Surface: CPython rejects (1) `sum([str, ...])` /
# `sum([], "x")` because `sum` requires numeric addends and a
# numeric (non-str) start — TypeError("unsupported operand type(s)
# for +" / "sum() can't sum strings"), not silent 0; (2) `min([])`
# / `max([])` because reducing an empty iterable has no result —
# ValueError("min()/max() arg is an empty sequence"), not silent
# None; (3) `divmod(non-num, non-num)` because divmod requires a
# numeric pair — TypeError, not silent None; (4) `round(non-num)`
# / `abs(non-num)` because both delegate to `__round__` / `__abs__`
# which non-numeric types don't define — TypeError, not silent 0.
# This family pins the aggregate / arithmetic builtin contract on
# non-numeric operands — every form below silently returns a coerced
# value on mamba where CPython would raise.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • sum(["a", "b"])                → mamba: 0      (TypeError)
#   • sum([], "x")                   → mamba: 0      (TypeError)
#   • sum([1, None, 3])              → mamba: 4      (TypeError)
#   • min([])                        → mamba: None   (ValueError)
#   • max([])                        → mamba: None   (ValueError)
#   • divmod("a", "b")               → mamba: None   (TypeError)
#   • divmod([], 2)                  → mamba: None   (TypeError)
#   • round("x")                     → mamba: 0      (TypeError)
#   • round(None)                    → mamba: 0      (TypeError)
#   • abs("x")                       → mamba: 0      (TypeError)
#   • abs([])                        → mamba: 0      (TypeError)
#
# CPython contract:
#   sum(iter[, start]) where addends are non-numeric or start is str
#       → TypeError("unsupported operand type(s) for +: ..."
#                   / "sum() can't sum strings");
#   min(iter) / max(iter) where iter is empty
#       → ValueError("min()/max() arg is an empty sequence");
#   divmod(a, b) where a or b is non-numeric
#       → TypeError("unsupported operand type(s) for divmod()");
#   round(x) / abs(x) where x is non-numeric and lacks __round__
#       / __abs__
#       → TypeError("type ... doesn't define __round__/__abs__").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_str_list: Any = ["a", "b"]
_empty: Any = []
_mixed: Any = [1, None, 3]
_str_start: Any = "x"
_str_a: Any = "a"
_str_b: Any = "b"
_str_x: Any = "x"
_none: Any = None
_two: Any = 2

# sum(["a", "b"]) — strings can't be summed by sum()
try:
    _ = sum(_str_list)
    raise AssertionError("sum(['a','b']) must raise TypeError")
except TypeError:
    _ledger.append(1)

# sum([], "x") — start must be numeric, not str
try:
    _ = sum(_empty, _str_start)
    raise AssertionError("sum([], 'x') must raise TypeError")
except TypeError:
    _ledger.append(1)

# sum([1, None, 3]) — None can't be added to int
try:
    _ = sum(_mixed)
    raise AssertionError("sum([1, None, 3]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# min([]) — empty iterable has no minimum
try:
    _ = min(_empty)
    raise AssertionError("min([]) must raise ValueError")
except ValueError:
    _ledger.append(1)

# max([]) — empty iterable has no maximum
try:
    _ = max(_empty)
    raise AssertionError("max([]) must raise ValueError")
except ValueError:
    _ledger.append(1)

# divmod("a", "b") — strings don't support divmod
try:
    _ = divmod(_str_a, _str_b)
    raise AssertionError("divmod('a', 'b') must raise TypeError")
except TypeError:
    _ledger.append(1)

# divmod([], 2) — list doesn't support divmod
try:
    _ = divmod(_empty, _two)
    raise AssertionError("divmod([], 2) must raise TypeError")
except TypeError:
    _ledger.append(1)

# round("x") — str doesn't define __round__
try:
    _ = round(_str_x)
    raise AssertionError("round('x') must raise TypeError")
except TypeError:
    _ledger.append(1)

# round(None) — NoneType doesn't define __round__
try:
    _ = round(_none)
    raise AssertionError("round(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# abs("x") — str doesn't define __abs__
try:
    _ = abs(_str_x)
    raise AssertionError("abs('x') must raise TypeError")
except TypeError:
    _ledger.append(1)

# abs([]) — list doesn't define __abs__
try:
    _ = abs(_empty)
    raise AssertionError("abs([]) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_aggregate_fn_wrong_type_silent {sum(_ledger)} asserts")
