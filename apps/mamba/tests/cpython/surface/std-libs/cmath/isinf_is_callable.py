# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "isinf_is_callable"
# subject = "cmath.isinf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.isinf: isinf_is_callable (surface)."""
import cmath

assert callable(cmath.isinf)
print("isinf_is_callable OK")
