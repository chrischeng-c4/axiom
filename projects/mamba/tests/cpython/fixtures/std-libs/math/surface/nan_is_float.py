# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "nan_is_float"
# subject = "math.nan"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.nan: nan_is_float (surface)."""
import math

assert type(math.nan).__name__ == "float"
print("nan_is_float OK")
