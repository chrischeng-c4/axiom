# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "sumprod_is_callable"
# subject = "math.sumprod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sumprod: sumprod_is_callable (surface)."""
import math

assert callable(math.sumprod)
print("sumprod_is_callable OK")
