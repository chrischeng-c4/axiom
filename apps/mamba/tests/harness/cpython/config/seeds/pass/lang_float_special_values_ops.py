# Operational AssertionPass seed for float special values, rounding,
# and cross-type arithmetic. Surface: `float("inf")` / `float("-inf")`
# parse to positive/negative infinity that compares larger/smaller
# than any finite value; `float("nan")` parses to a NaN sentinel that
# is *never equal to itself* under `==`. The math module exposes
# canonical `math.inf` and `math.nan` constants; `math.isnan()` is
# the supported predicate for NaN testing. `round()` follows banker's
# rounding (round-half-to-even): `round(2.5) == 2`, `round(3.5) == 4`,
# `round(-2.5) == -2`. `round(x, ndigits)` rounds to the requested
# decimal precision. `float()` accepts ints, bools, and decimal /
# scientific-notation strings; `abs()` strips the sign on both ints
# and floats. Cross-type arithmetic between int and float promotes
# to float for `==`, `<`, `>` comparisons, and `min`/`max` returns
# the smaller/larger value regardless of type. `repr(1.5) == "1.5"`
# and `str(1.5) == "1.5"` are stable canonical text forms.
import math
_ledger: list[int] = []

# is_integer predicate
assert (1.5).is_integer() == False; _ledger.append(1)
assert (2.0).is_integer() == True; _ledger.append(1)
assert (0.0).is_integer() == True; _ledger.append(1)
assert (-3.0).is_integer() == True; _ledger.append(1)
assert (3.14).is_integer() == False; _ledger.append(1)

# float() string parsing (decimal, scientific, signed)
assert float("3.14") == 3.14; _ledger.append(1)
assert float("1e2") == 100.0; _ledger.append(1)
assert float("1.5e-3") == 0.0015; _ledger.append(1)
assert float("-2.5") == -2.5; _ledger.append(1)

# Special values
assert float("inf") == float("inf"); _ledger.append(1)
assert float("nan") != float("nan"); _ledger.append(1)  # NaN-not-equal axiom
assert float("inf") > 1e308; _ledger.append(1)
assert float("-inf") < -1e308; _ledger.append(1)

# round() with banker's rounding (round-half-to-even)
assert round(3.5) == 4; _ledger.append(1)
assert round(2.5) == 2; _ledger.append(1)
assert round(-2.5) == -2; _ledger.append(1)
assert round(3.14159, 2) == 3.14; _ledger.append(1)

# abs on positive and negative floats
assert abs(-3.14) == 3.14; _ledger.append(1)
assert abs(3.14) == 3.14; _ledger.append(1)

# min / max with floats
assert max(1.5, 2.5, 0.5) == 2.5; _ledger.append(1)
assert min(1.5, 2.5, 0.5) == 0.5; _ledger.append(1)

# Cross-type construction
assert float(42) == 42.0; _ledger.append(1)
assert float(True) == 1.0; _ledger.append(1)
assert float(False) == 0.0; _ledger.append(1)

# Stable text forms
assert repr(1.5) == "1.5"; _ledger.append(1)
assert str(1.5) == "1.5"; _ledger.append(1)

# Cross-type int-float comparison promotes to float
assert 1.0 == 1; _ledger.append(1)
assert 1.0 < 2; _ledger.append(1)
assert 2.0 > 1; _ledger.append(1)

# math.inf and math.isnan canonical constants/predicates
assert math.inf == float("inf"); _ledger.append(1)
assert math.isnan(math.nan); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_float_special_values_ops {sum(_ledger)} asserts")
