# Operational AssertionPass seed for SILENT divergences across the
# arbitrary-precision / exact-rational / abstract-numeric-tower
# trio pinned by atomic 153: `decimal` (the documented
# `Decimal(...)` arbitrary-precision class + the
# `getcontext().prec` 28-digit default + the ROUND_HALF_EVEN /
# ROUND_UP / ROUND_DOWN rounding-mode sentinels + the
# `Decimal(...).quantize` method), `fractions` (the documented
# `Fraction(...)` exact-rational class + the
# Fraction.__name__ identifier), and `numbers` (the documented
# abstract-numeric-tower ABCs — Number / Complex / Real /
# Rational / Integral + the `isinstance(1, Integral)` /
# `isinstance(1.5, Real)` numeric-tower predicates that
# CPython uses to classify built-in `int` / `float` /
# `complex` instances).
#
# The matching subset (math.pi / e / tau / inf / nan
# reflexive-inequality, math.sqrt / pow / log / log10 / log2,
# math.sin / cos at 0, math.floor / ceil / trunc / abs,
# math.gcd / lcm / factorial / comb / perm, math.isnan /
# isinf / isfinite / isclose, math.copysign / fabs / fmod /
# hypot / dist / exp / expm1, cmath.pi / e / inf + nan
# reflexive-inequality, cmath.sqrt of negative + polar / rect
# / phase + isnan / isinf) is covered by
# `test_math_cmath_numeric_value_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • Decimal("0.1") + Decimal("0.2") == Decimal("0.3") —
#     arbitrary-precision arithmetic (mamba: returns
#     -140737488355325, an integer handle that overflows
#     instead of an exact Decimal — Decimal is an opaque int
#     wrapper on mamba);
#   • Decimal("0.1") * Decimal("3") == Decimal("0.3") (mamba:
#     returns -70368744177652, integer-handle arithmetic);
#   • getcontext().prec == 28 — the documented 28-digit
#     default precision (mamba: returns None);
#   • ROUND_HALF_EVEN == "ROUND_HALF_EVEN" — rounding-mode
#     sentinel (mamba: returns None);
#   • ROUND_UP == "ROUND_UP" (mamba: None);
#   • ROUND_DOWN == "ROUND_DOWN" (mamba: None);
#   • Decimal("3.14159").quantize(Decimal("0.01")) ==
#     Decimal("3.14") — quantize round-to-significant
#     (mamba: AttributeError, 'int' object has no attribute
#     'quantize');
#   • Decimal.__name__ == "Decimal" — bare class identity
#     (mamba: hasattr returns False on the class identifier);
#   • Fraction(1, 3) + Fraction(1, 6) == Fraction(1, 2) —
#     exact-rational arithmetic (mamba: returns
#     2199023255557, integer-handle arithmetic);
#   • str(Fraction(2, 4)) == "1/2" — auto-reduction
#     (mamba: returns "1099511627780", integer-handle repr);
#   • Fraction.__name__ == "Fraction" — bare class identity
#     (mamba: None);
#   • numbers.Number.__name__ == "Number" — abstract-numeric
#     tower class identity (mamba: None);
#   • numbers.Complex.__name__ == "Complex" (mamba: None);
#   • numbers.Real.__name__ == "Real" (mamba: None);
#   • numbers.Rational.__name__ == "Rational" (mamba: None);
#   • numbers.Integral.__name__ == "Integral" (mamba: None);
#   • isinstance(1, numbers.Integral) is True — int is in
#     the Integral tower (mamba: returns False — built-in int
#     does NOT subscribe to numbers.Integral);
#   • isinstance(1.5, numbers.Real) is True — float is in
#     the Real tower (mamba: returns False — built-in float
#     does NOT subscribe to numbers.Real).
import decimal as _decimal_mod
import fractions as _fractions_mod
import numbers as _numbers_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / abstract-tower
# predicates that mamba's bundled type stubs do not surface
# accurately.
decimal: Any = _decimal_mod
fractions: Any = _fractions_mod
numbers: Any = _numbers_mod


_ledger: list[int] = []

# 1) decimal — arbitrary-precision arithmetic
assert decimal.Decimal("0.1") + decimal.Decimal("0.2") == decimal.Decimal("0.3"); _ledger.append(1)
assert decimal.Decimal("0.1") * decimal.Decimal("3") == decimal.Decimal("0.3"); _ledger.append(1)

# 2) decimal — getcontext().prec default
assert decimal.getcontext().prec == 28; _ledger.append(1)

# 3) decimal — rounding-mode sentinels
assert decimal.ROUND_HALF_EVEN == "ROUND_HALF_EVEN"; _ledger.append(1)
assert decimal.ROUND_UP == "ROUND_UP"; _ledger.append(1)
assert decimal.ROUND_DOWN == "ROUND_DOWN"; _ledger.append(1)

# 4) decimal — quantize round-to-significant
assert decimal.Decimal("3.14159").quantize(decimal.Decimal("0.01")) == decimal.Decimal("3.14"); _ledger.append(1)

# 5) decimal — bare class identity
assert decimal.Decimal.__name__ == "Decimal"; _ledger.append(1)

# 6) fractions — exact-rational arithmetic
assert fractions.Fraction(1, 3) + fractions.Fraction(1, 6) == fractions.Fraction(1, 2); _ledger.append(1)

# 7) fractions — auto-reduction in str form
assert str(fractions.Fraction(2, 4)) == "1/2"; _ledger.append(1)

# 8) fractions — bare class identity
assert fractions.Fraction.__name__ == "Fraction"; _ledger.append(1)

# 9) numbers — abstract-numeric tower class identity
assert numbers.Number.__name__ == "Number"; _ledger.append(1)
assert numbers.Complex.__name__ == "Complex"; _ledger.append(1)
assert numbers.Real.__name__ == "Real"; _ledger.append(1)
assert numbers.Rational.__name__ == "Rational"; _ledger.append(1)
assert numbers.Integral.__name__ == "Integral"; _ledger.append(1)

# 10) numbers — built-in numeric tower subscription
assert isinstance(1, numbers.Integral) == True; _ledger.append(1)
assert isinstance(1.5, numbers.Real) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_decimal_fractions_numbers_silent {sum(_ledger)} asserts")
