# Operational AssertionPass seed for `cmath` complex-number math.
# Surface: cmath.sqrt of a negative real returns a pure imaginary;
# the module constants pi/e match the math module; complex magnitude,
# phase, polar, and rect form a coherent polar-cartesian system.
# Companion to stub/test_cmath.py — vendored unittest seed.
import cmath
import math
_ledger: list[int] = []
# sqrt of -1 is the imaginary unit
assert cmath.sqrt(-1) == 1j; _ledger.append(1)
# Module constants match math
assert cmath.pi == math.pi; _ledger.append(1)
assert cmath.e == math.e; _ledger.append(1)
# 3 + 4i has magnitude 5 (classic Pythagorean triple)
z = complex(3, 4)
assert abs(z) == 5.0; _ledger.append(1)
# polar form (r, theta) — magnitude matches abs
r, theta = cmath.polar(z)
assert r == 5.0; _ledger.append(1)
# rect of (5, 0) is just (5 + 0j) — purely real
assert cmath.rect(5, 0) == complex(5, 0); _ledger.append(1)
# phase of 1+0j is 0
assert cmath.phase(complex(1, 0)) == 0.0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_cmath_ops {sum(_ledger)} asserts")
