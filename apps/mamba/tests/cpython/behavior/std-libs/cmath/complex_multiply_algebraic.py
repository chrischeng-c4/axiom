# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "behavior"
# case = "complex_multiply_algebraic"
# subject = "cmath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath: complex multiplication follows the algebraic rule: (1+2j)*(3+4j) == -5+10j"""
import cmath  # noqa: F401

_a = 1 + 2j
_b = 3 + 4j
# (1+2j)(3+4j) = 3+4j+6j+8j^2 = 3+10j-8 = -5+10j
assert _a * _b == -5 + 10j, f"complex multiply = {_a * _b!r}"
print("complex_multiply_algebraic OK")
