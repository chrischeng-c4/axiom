# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "surface"
# case = "integral_is_callable"
# subject = "numbers.Integral"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""numbers.Integral: integral_is_callable (surface)."""
import numbers

assert callable(numbers.Integral)
print("integral_is_callable OK")
