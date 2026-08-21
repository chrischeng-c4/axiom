# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "ceil_is_callable"
# subject = "math.ceil"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.ceil: ceil_is_callable (surface)."""
import math

assert callable(math.ceil)
print("ceil_is_callable OK")
