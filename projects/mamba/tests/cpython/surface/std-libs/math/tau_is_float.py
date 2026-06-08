# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "tau_is_float"
# subject = "math.tau"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.tau: tau_is_float (surface)."""
import math

assert type(math.tau).__name__ == "float"
print("tau_is_float OK")
