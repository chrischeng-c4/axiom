# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `round(x, ndigits)` return-type bug:
#
# CPython's contract is "ndigits given (even `0`/negative) → result type
# matches x; ndigits omitted → int". So:
#
#   round(3.7)          → 4    (int)
#   round(3.7, 0)       → 4.0  (float — ndigits given, float in)
#   round(3.7, 2)       → 3.7  (float)
#   round(1234.5, -2)   → 1200.0 (float — ndigits given)
#   round(1234, -2)     → 1200 (int — int in, int out)
#
# Mamba's `mb_round` (runtime/builtins.rs) decided int-vs-float based on
# `n > 0` only. So `round(3.7, 0)` and `round(3.7, -2)` cast through
# `as i64` and dropped the float type.
#
# Fix: the dispatcher passes `MbValue::none()` when ndigits is omitted,
# so we can distinguish "no ndigits" from "ndigits == 0". When ndigits
# is given AND value is float, always emit float — even at n == 0 or
# negative n.

# Most important: ndigits == 0 with float input must stay float.
print(round(3.7, 0))           # 4.0
print(round(3.14, 0))          # 3.0
print(round(0.5, 0))           # 0.0  (banker's: 0.5 → 0)
print(round(1.5, 0))           # 2.0  (banker's: 1.5 → 2)

# ndigits omitted ⇒ int.
print(round(3.7))              # 4
print(round(3.14))             # 3
print(round(2.5))              # 2    (banker's)
print(round(0.5))              # 0    (banker's)

# Positive ndigits — already worked, but keep as regression coverage.
print(round(3.14159, 2))       # 3.14
print(round(3.14159, 4))       # 3.1416
print(round(2.675, 2))         # 2.67  (FP repr; matches CPython)

# Negative ndigits with float ⇒ float (this is the second part of the fix).
print(round(1234.5, -2))       # 1200.0
print(round(987.6, -1))        # 990.0

# Int input is unchanged: ndigits given or not, result stays int.
print(round(1234, -2))         # 1200
print(round(1234, 0))          # 1234
print(round(42))               # 42

# Type assertions.
print(type(round(3.7, 0)).__name__)         # float
print(type(round(3.7)).__name__)            # int
print(type(round(1234.5, -2)).__name__)     # float
print(type(round(1234, -2)).__name__)       # int
