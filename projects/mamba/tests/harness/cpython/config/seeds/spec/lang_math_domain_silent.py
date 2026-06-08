# Spec seed for CPython ZeroDivisionError / ValueError contract on
# the math-domain corners that mamba silently returns IEEE-754 inf /
# -inf / nan from. Surface: CPython rejects (1) `pow(0, -1)` /
# `0.0 ** -1` because raising zero to a negative power is a
# division-by-zero on the underlying `1 / 0**|n|` rewrite —
# ZeroDivisionError, not silent `inf`; (2) `math.log(0)` /
# `math.log10(0)` / `math.log2(0)` because log is undefined at
# zero — ValueError("math domain error"), not silent `-inf`; (3)
# `math.log10(-x)` / `math.log2(-x)` because log of a negative
# real is complex (and `math` is the real-only module — `cmath`
# handles complex) — ValueError, not silent `nan`; (4)
# `math.asin(x)` / `math.acos(x)` for `|x| > 1` because arcsin /
# arccos are only defined on [-1, 1] in the reals —
# ValueError("math domain error"), not silent `nan` that
# downstream `if math.acos(x) < pi/2` would silently mis-branch
# on. Existing math fixtures touch the matching trig / log
# domain but the domain-error family hasn't been pinned for the
# silent-coercion contract yet.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • pow(0, -1)        → mamba: inf            (ZeroDivisionError)
#   • 0.0 ** -1         → mamba: inf            (ZeroDivisionError)
#   • math.log10(0)     → mamba: -inf           (ValueError)
#   • math.log2(0)      → mamba: -inf           (ValueError)
#   • math.log10(-5)    → mamba: nan            (ValueError)
#   • math.asin(2)      → mamba: nan            (ValueError)
#   • math.acos(2)      → mamba: nan            (ValueError)
#   • math.asin(-2)     → mamba: nan            (ValueError)
#   • math.acos(-2)     → mamba: nan            (ValueError)
#
# CPython contract:
#   pow(0, -1)             → ZeroDivisionError("0.0 cannot be raised
#                                  to a negative power");
#   0.0 ** -1              → ZeroDivisionError("0.0 cannot be raised
#                                  to a negative power");
#   math.log10(0) / math.log2(0)
#                          → ValueError("math domain error");
#   math.log10(-5)         → ValueError("math domain error");
#   math.asin(2) / math.acos(2) / math.asin(-2) / math.acos(-2)
#                          → ValueError("math domain error").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised. The two ZDE probes also
# defeat constant-fold optimizations that might evaluate `0 ** -1`
# at compile time.
from typing import Any
import math
_ledger: list[int] = []

_zero_int: Any = 0
_neg_one: Any = -1
_zero_float: Any = 0.0
_neg_five: Any = -5
_two: Any = 2
_neg_two: Any = -2

# pow(0, -1) — raising zero to a negative power
try:
    _ = pow(_zero_int, _neg_one)
    raise AssertionError("pow(0, -1) must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# 0.0 ** -1 — same identity, float receiver
try:
    _ = _zero_float ** _neg_one
    raise AssertionError("0.0 ** -1 must raise ZeroDivisionError")
except ZeroDivisionError:
    _ledger.append(1)

# math.log10(0) — log of zero
try:
    _ = math.log10(_zero_int)
    raise AssertionError("math.log10(0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# math.log2(0) — log2 of zero
try:
    _ = math.log2(_zero_int)
    raise AssertionError("math.log2(0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# math.log10(-5) — log of negative real
try:
    _ = math.log10(_neg_five)
    raise AssertionError("math.log10(-5) must raise ValueError")
except ValueError:
    _ledger.append(1)

# math.asin(2) — arcsin out of [-1, 1] domain
try:
    _ = math.asin(_two)
    raise AssertionError("math.asin(2) must raise ValueError")
except ValueError:
    _ledger.append(1)

# math.acos(2) — arccos out of [-1, 1] domain
try:
    _ = math.acos(_two)
    raise AssertionError("math.acos(2) must raise ValueError")
except ValueError:
    _ledger.append(1)

# math.asin(-2) — arcsin out of domain (negative side)
try:
    _ = math.asin(_neg_two)
    raise AssertionError("math.asin(-2) must raise ValueError")
except ValueError:
    _ledger.append(1)

# math.acos(-2) — arccos out of domain (negative side)
try:
    _ = math.acos(_neg_two)
    raise AssertionError("math.acos(-2) must raise ValueError")
except ValueError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_math_domain_silent {sum(_ledger)} asserts")
