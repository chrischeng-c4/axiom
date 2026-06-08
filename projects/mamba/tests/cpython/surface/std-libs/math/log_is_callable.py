# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "surface"
# case = "log_is_callable"
# subject = "math.log"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.log: log_is_callable (surface)."""
import math

assert callable(math.log)
print("log_is_callable OK")
