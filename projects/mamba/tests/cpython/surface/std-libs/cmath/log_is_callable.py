# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "log_is_callable"
# subject = "cmath.log"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.log: log_is_callable (surface)."""
import cmath

assert callable(cmath.log)
print("log_is_callable OK")
