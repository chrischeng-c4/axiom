# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "e_is_float"
# subject = "math.e"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.e: e_is_float (surface)."""
import math

assert type(math.e).__name__ == "float"
print("e_is_float OK")
