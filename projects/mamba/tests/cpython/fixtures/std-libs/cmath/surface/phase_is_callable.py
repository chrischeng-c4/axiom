# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "phase_is_callable"
# subject = "cmath.phase"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.phase: phase_is_callable (surface)."""
import cmath

assert callable(cmath.phase)
print("phase_is_callable OK")
