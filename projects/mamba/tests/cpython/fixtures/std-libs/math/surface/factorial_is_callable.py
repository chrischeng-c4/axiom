# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "factorial_is_callable"
# subject = "math.factorial"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.factorial: factorial_is_callable (surface)."""
import math

assert callable(math.factorial)
print("factorial_is_callable OK")
