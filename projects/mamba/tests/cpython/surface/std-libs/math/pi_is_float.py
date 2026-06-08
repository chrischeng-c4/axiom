# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "pi_is_float"
# subject = "math.pi"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.pi: pi_is_float (surface)."""
import math

assert type(math.pi).__name__ == "float"
print("pi_is_float OK")
