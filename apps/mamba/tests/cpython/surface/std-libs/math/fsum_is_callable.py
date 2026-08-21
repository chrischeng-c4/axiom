# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "fsum_is_callable"
# subject = "math.fsum"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.fsum: fsum_is_callable (surface)."""
import math

assert callable(math.fsum)
print("fsum_is_callable OK")
