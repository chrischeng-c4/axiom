# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "sin_cos_pythagorean_identity"
# subject = "cmath.sin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.sin: the Pythagorean identity sin(z)**2 + cos(z)**2 == 1 holds for a complex argument"""
import cmath

_z = 1 + 2j
_sq_sum = cmath.sin(_z) ** 2 + cmath.cos(_z) ** 2
assert abs(_sq_sum - 1) < 1e-12, f"sin^2+cos^2 = 1 for complex z: {_sq_sum!r}"
print("sin_cos_pythagorean_identity OK")
