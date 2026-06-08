# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# math.tanh / asinh / acosh / atanh + math.nextafter / math.ulp.
# Six previously AttributeError on mamba (#1328). All implemented via
# stable f64 methods; nextafter/ulp ride on `next_up`/`next_down`.

import math

# Hyperbolic tangent + inverse hyperbolics.
print(math.tanh(1))                # 0.7615941559557649
print(math.tanh(0))                # 0.0
print(math.asinh(1))               # 0.881373587019543
print(math.acosh(1))               # 0.0
print(math.acosh(2))               # 1.3169578969248166
print(math.atanh(0.5))             # 0.5493061443340549
print(math.atanh(0))               # 0.0

# Domain errors raise ValueError.
try:
    math.acosh(0.5)
except ValueError:
    print("acosh<1 -> ValueError")
try:
    math.atanh(1.0)
except ValueError:
    print("atanh|x|>=1 -> ValueError")

# nextafter(x, y) — single ULP step toward y.
print(math.nextafter(1.0, 2.0))    # 1.0000000000000002
print(math.nextafter(1.0, 0.0))    # 0.9999999999999999
print(math.nextafter(1.0, 1.0))    # 1.0  (x == y returns y)

# ulp(x) — value of least-significant bit.
print(math.ulp(1.0))               # 2.220446049250313e-16
print(math.ulp(0.0))               # 5e-324
print(math.ulp(float("inf")))      # inf
print(math.ulp(float("nan")))      # nan
