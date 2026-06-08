# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "inf_is_float"
# subject = "math.inf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.inf: inf_is_float (surface)."""
import math

assert type(math.inf).__name__ == "float"
print("inf_is_float OK")
