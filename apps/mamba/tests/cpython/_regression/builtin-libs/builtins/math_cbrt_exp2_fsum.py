# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# math.cbrt / math.exp2 / math.fsum — three Py3.11+/3.8+ float
# helpers that were previously AttributeError on mamba (#1326).

import math

# math.cbrt(x) — Py3.11+. Real cube root with sign.
print(math.cbrt(27))         # 3.0
print(math.cbrt(-8))         # -2.0
print(math.cbrt(0))           # 0.0
print(math.cbrt(2.0))         # 1.2599210498948732

# math.exp2(x) — Py3.11+. 2**x; faster + more accurate than pow(2, x).
print(math.exp2(0))           # 1.0
print(math.exp2(8))           # 256.0
print(math.exp2(-1))          # 0.5
print(math.exp2(0.5))         # 1.4142135623730951

# math.fsum(iterable) — accurate floating-point sum (Shewchuk).
# Naive sum of [0.1, 0.2, 0.3] is 0.6000000000000001 — fsum returns 0.6.
print(math.fsum([0.1, 0.2, 0.3]))               # 0.6
print(math.fsum([1.0, 1e100, 1.0, -1e100]))     # 2.0  (cancellation-safe)
print(math.fsum([]))                              # 0.0
print(math.fsum([1, 2, 3]))                       # 6.0  (int input promoted)
