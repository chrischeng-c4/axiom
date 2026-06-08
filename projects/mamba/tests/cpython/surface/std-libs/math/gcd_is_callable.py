# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "gcd_is_callable"
# subject = "math.gcd"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.gcd: gcd_is_callable (surface)."""
import math

assert callable(math.gcd)
print("gcd_is_callable OK")
