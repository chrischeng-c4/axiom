# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "sqrt_neg_four_is_two_j"
# subject = "cmath.sqrt"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.sqrt: sqrt(-4) == 2j on the principal branch"""
import cmath

assert abs(cmath.sqrt(-4) - 2j) < 1e-14, f"sqrt(-4) = {cmath.sqrt(-4)!r}"
print("sqrt_neg_four_is_two_j OK")
