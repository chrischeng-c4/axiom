# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "surface"
# case = "real_is_callable"
# subject = "numbers.Real"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""numbers.Real: real_is_callable (surface)."""
import numbers

assert callable(numbers.Real)
print("real_is_callable OK")
