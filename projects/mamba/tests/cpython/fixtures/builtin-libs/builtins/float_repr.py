# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Three CPython-divergent gaps in `str(float)` / `repr(float)` / `print(float)`:
#
# 1. Non-finite values rendered as Rust's `NaN` instead of Python's `nan`.
#    `str(float('nan'))` → `'NaN'` (was) vs `'nan'` (CPython).
#
# 2. Small floats printed with leading zeros instead of scientific notation.
#    `repr(1.5e-7)` → `'0.00000015'` (was) vs `'1.5e-07'` (CPython).
#
# 3. Large floats printed in long-form decimal instead of scientific.
#    `repr(1e16)` → `'10000000000000000.0'` (was) vs `'1e+16'` (CPython).
#
# Root cause: four parallel float-stringifiers (`mb_print`, `mb_str`,
# `mb_repr`, plus the `print_value_str` / `print_repr` helpers in
# `runtime/builtins.rs`) and `value_to_string` in `runtime/string_ops.rs`
# each rolled their own `format!("{f:.1}")` / `format!("{f}")`. Fix unifies
# them behind `string_ops::python_float_repr`, which: (a) lowercases nan/inf,
# (b) switches to `{:e}` (CPython exponent) for `|x| < 1e-4` or `|x| >= 1e16`,
# (c) appends `.0` to whole-number plain decimals.

# Non-finite — lowercase, no sign on nan.
print(float("nan"))                  # nan
print(float("inf"))                  # inf
print(float("-inf"))                 # -inf
print(float("-nan"))                 # nan
print(repr(float("nan")))            # nan
print(str(float("nan")))             # nan

# Small floats — switch to scientific below 1e-4.
print(1.5e-7)                        # 1.5e-07
print(0.00001)                       # 1e-05
print(0.0001)                        # 0.0001  (boundary — stays plain)
print(repr(1.5e-7))                  # 1.5e-07
print(str(0.000001))                 # 1e-06

# Large floats — switch to scientific at or above 1e16.
print(1e16)                          # 1e+16
print(1e15)                          # 1000000000000000.0  (boundary — plain)
print(repr(1e20))                    # 1e+20
print(str(2.5e18))                   # 2.5e+18

# Whole-number floats keep `.0`.
print(1.0)                           # 1.0
print(-0.0)                          # -0.0
print(repr(10.0))                    # 10.0

# f-strings flow through the same path.
print(f"{float('nan')}")             # nan
print(f"{1.5e-7}")                   # 1.5e-07
print(f"{1e16}")                     # 1e+16
