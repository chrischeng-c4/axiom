# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "trunc_is_callable"
# subject = "math.trunc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.trunc: trunc_is_callable (surface)."""
import math

assert callable(math.trunc)
print("trunc_is_callable OK")
