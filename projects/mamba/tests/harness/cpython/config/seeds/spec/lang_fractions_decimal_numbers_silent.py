# Operational AssertionPass seed for SILENT divergences across the
# fractions.Fraction class-identity / arithmetic contract +
# decimal extended module-helper surface + decimal.Decimal class-
# identity / str-value contract + numbers ABC isinstance contract
# pinned by atomic 189: `fractions` (the documented
# `type(fractions.Fraction(1, 4)).__name__ == "Fraction"` class-
# identity contract + the documented `Fraction(1, 4) +
# Fraction(1, 4) == Fraction(1, 2)` arithmetic value contract),
# `decimal` (the documented `getcontext` / `setcontext` /
# `localcontext` / `ROUND_HALF_UP` / `ROUND_HALF_EVEN` /
# `ROUND_DOWN` / `ROUND_UP` / `ROUND_CEILING` / `ROUND_FLOOR` /
# `Context` / `InvalidOperation` extended function / class /
# sentinel / exception identifier surface + the documented
# `type(decimal.Decimal("0.1")).__name__ == "Decimal"` class-
# identity contract + the documented `str(decimal.Decimal
# ("0.1")) == "0.1"` string-value contract + the documented
# `decimal.Decimal("0.1") + decimal.Decimal("0.2") ==
# decimal.Decimal("0.3")` arithmetic value contract), and
# `numbers` (the documented `isinstance(1, numbers.Integral)
# == True` ABC isinstance contract + the documented
# `isinstance(1.0, numbers.Real) == True` ABC isinstance
# contract).
#
# The matching subset (full math hasattr + values, partial
# cmath hasattr + values, partial fractions hasattr +
# numerator/denominator, partial decimal hasattr (Decimal),
# full numbers hasattr) is covered by
# `test_math_cmath_fractions_decimal_numbers_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(fractions.Fraction(1, 4)).__name__ == "Fraction"
#     — documented class identity (mamba: returns "int" —
#     the Fraction constructor short-circuits to an integer
#     handle that still surfaces .numerator / .denominator);
#   • fractions.Fraction(1, 4) + fractions.Fraction(1, 4) ==
#     fractions.Fraction(1, 2) — documented arithmetic value
#     contract (mamba: the sum collapses to a large integer
#     placeholder and fails equality with Fraction(1, 2));
#   • hasattr(decimal, "getcontext") is True — documented
#     function identifier (mamba: False);
#   • hasattr(decimal, "setcontext") is True — documented
#     function identifier (mamba: False);
#   • hasattr(decimal, "localcontext") is True — documented
#     function identifier (mamba: False);
#   • hasattr(decimal, "ROUND_HALF_UP") is True — documented
#     rounding-mode sentinel (mamba: False);
#   • hasattr(decimal, "ROUND_HALF_EVEN") is True —
#     documented rounding-mode sentinel (mamba: False);
#   • hasattr(decimal, "ROUND_DOWN") is True — documented
#     rounding-mode sentinel (mamba: False);
#   • hasattr(decimal, "ROUND_UP") is True — documented
#     rounding-mode sentinel (mamba: False);
#   • hasattr(decimal, "ROUND_CEILING") is True — documented
#     rounding-mode sentinel (mamba: False);
#   • hasattr(decimal, "ROUND_FLOOR") is True — documented
#     rounding-mode sentinel (mamba: False);
#   • hasattr(decimal, "Context") is True — documented class
#     identifier (mamba: False);
#   • hasattr(decimal, "InvalidOperation") is True —
#     documented exception identifier (mamba: False);
#   • type(decimal.Decimal("0.1")).__name__ == "Decimal" —
#     documented class identity (mamba: returns "int" — the
#     Decimal constructor short-circuits to an integer
#     handle);
#   • str(decimal.Decimal("0.1")) == "0.1" — documented
#     string-value contract (mamba: returns a large integer
#     placeholder string, not "0.1");
#   • decimal.Decimal("0.1") + decimal.Decimal("0.2") ==
#     decimal.Decimal("0.3") — documented arithmetic value
#     contract (mamba: the sum returns a negative-integer
#     placeholder unrelated to Decimal("0.3"));
#   • isinstance(1, numbers.Integral) == True — documented
#     ABC isinstance contract (mamba: False — int is not
#     registered against the mamba numbers.Integral ABC);
#   • isinstance(1.0, numbers.Real) == True — documented
#     ABC isinstance contract (mamba: False — float is not
#     registered against the mamba numbers.Real ABC).
import fractions as _fractions_mod
import decimal as _decimal_mod
import numbers as _numbers_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance-method / value-contract behavior
# that mamba's bundled type stubs do not surface accurately.
fractions: Any = _fractions_mod
decimal: Any = _decimal_mod
numbers: Any = _numbers_mod


_ledger: list[int] = []

# 1) fractions.Fraction — class-identity contract
_f = fractions.Fraction(1, 4)
assert type(_f).__name__ == "Fraction"; _ledger.append(1)

# 2) fractions.Fraction — arithmetic value contract
assert fractions.Fraction(1, 4) + fractions.Fraction(1, 4) == fractions.Fraction(1, 2); _ledger.append(1)

# 3) decimal — extended module-helper surface
assert hasattr(decimal, "getcontext") == True; _ledger.append(1)
assert hasattr(decimal, "setcontext") == True; _ledger.append(1)
assert hasattr(decimal, "localcontext") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_HALF_UP") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_HALF_EVEN") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_DOWN") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_UP") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_CEILING") == True; _ledger.append(1)
assert hasattr(decimal, "ROUND_FLOOR") == True; _ledger.append(1)
assert hasattr(decimal, "Context") == True; _ledger.append(1)
assert hasattr(decimal, "InvalidOperation") == True; _ledger.append(1)

# 4) decimal.Decimal — class-identity contract
_d = decimal.Decimal("0.1")
assert type(_d).__name__ == "Decimal"; _ledger.append(1)

# 5) decimal.Decimal — string-value contract
assert str(decimal.Decimal("0.1")) == "0.1"; _ledger.append(1)

# 6) decimal.Decimal — arithmetic value contract
assert decimal.Decimal("0.1") + decimal.Decimal("0.2") == decimal.Decimal("0.3"); _ledger.append(1)

# 7) numbers — ABC isinstance contract
assert isinstance(1, numbers.Integral) == True; _ledger.append(1)
assert isinstance(1.0, numbers.Real) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_fractions_decimal_numbers_silent {sum(_ledger)} asserts")
