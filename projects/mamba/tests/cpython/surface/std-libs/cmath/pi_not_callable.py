# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "pi_not_callable"
# subject = "cmath.pi"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.pi: pi_not_callable (surface)."""
import cmath

assert not callable(cmath.pi)
print("pi_not_callable OK")
