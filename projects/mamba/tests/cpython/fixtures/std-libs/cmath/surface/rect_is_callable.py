# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "surface"
# case = "rect_is_callable"
# subject = "cmath.rect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cmath.rect: rect_is_callable (surface)."""
import cmath

assert callable(cmath.rect)
print("rect_is_callable OK")
