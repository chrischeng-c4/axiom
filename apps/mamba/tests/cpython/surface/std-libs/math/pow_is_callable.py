# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "pow_is_callable"
# subject = "math.pow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.pow: pow_is_callable (surface)."""
import math

assert callable(math.pow)
print("pow_is_callable OK")
