# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "isqrt_is_callable"
# subject = "math.isqrt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.isqrt: isqrt_is_callable (surface)."""
import math

assert callable(math.isqrt)
print("isqrt_is_callable OK")
