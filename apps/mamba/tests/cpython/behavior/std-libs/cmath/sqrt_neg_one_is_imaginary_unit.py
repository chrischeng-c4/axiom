# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "sqrt_neg_one_is_imaginary_unit"
# subject = "cmath.sqrt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.sqrt: sqrt(-1) is the imaginary unit 1j: real part ~ 0, imag part ~ 1"""
import cmath

_sq = cmath.sqrt(-1)
assert abs(_sq.real) < 1e-15, f"sqrt(-1).real ~ 0 = {_sq.real!r}"
assert abs(_sq.imag - 1) < 1e-15, "sqrt(-1).imag = 1"
print("sqrt_neg_one_is_imaginary_unit OK")
