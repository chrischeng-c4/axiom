# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "floor_is_callable"
# subject = "math.floor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.floor: floor_is_callable (surface)."""
import math

assert callable(math.floor)
print("floor_is_callable OK")
