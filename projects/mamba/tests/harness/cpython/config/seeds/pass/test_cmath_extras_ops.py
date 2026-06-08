# Operational AssertionPass seed for `cmath` surface not covered by
# `test_cmath_ops`. That seed exercises sqrt(-1), pi/e constants, and
# polar/rect on 3+4j. This seed asserts the IEEE-classification
# trio (isnan/isinf/isfinite on complex), elementary functions
# (exp(0)==1, log(1)==0, sin/cos at 0), the extra module constants
# (tau, inf, nan, infj), and the polar/rect round-trip on the
# imaginary axis (phase(j) == pi/2, rect(1, pi).real == -1).
import cmath
import math
_ledger: list[int] = []

# Classification on complex
assert cmath.isnan(complex(float("nan"), 0)); _ledger.append(1)
assert cmath.isinf(complex(float("inf"), 0)); _ledger.append(1)
assert cmath.isfinite(complex(1, 2)); _ledger.append(1)
assert not cmath.isnan(complex(1, 2)); _ledger.append(1)
assert not cmath.isinf(complex(1, 2)); _ledger.append(1)

# Elementary functions at 0
assert cmath.exp(0) == complex(1, 0); _ledger.append(1)
assert cmath.log(1) == complex(0, 0); _ledger.append(1)
assert cmath.sin(0) == complex(0, 0); _ledger.append(1)

# Module constants
assert cmath.tau == 2 * math.pi; _ledger.append(1)
assert cmath.inf == float("inf"); _ledger.append(1)
assert math.isnan(cmath.nan); _ledger.append(1)
assert cmath.infj == complex(0, float("inf")); _ledger.append(1)

# Phase on the imaginary unit
assert cmath.phase(complex(0, 1)) == math.pi / 2; _ledger.append(1)
assert cmath.phase(complex(1, 0)) == 0.0; _ledger.append(1)
assert cmath.phase(complex(-1, 0)) == math.pi; _ledger.append(1)

# rect inverse — rect(1, pi) is approximately -1 + 0j
z = cmath.rect(1, math.pi)
assert z.real == -1.0; _ledger.append(1)
assert abs(z.imag) < 1e-9; _ledger.append(1)

# rect(r, 0) is purely real
assert cmath.rect(5, 0) == complex(5, 0); _ledger.append(1)
assert cmath.rect(2.5, 0).imag == 0.0; _ledger.append(1)

# polar of (5, 0) returns (5, 0)
r, theta = cmath.polar(complex(5, 0))
assert r == 5.0; _ledger.append(1)
assert theta == 0.0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_cmath_extras_ops {sum(_ledger)} asserts")
