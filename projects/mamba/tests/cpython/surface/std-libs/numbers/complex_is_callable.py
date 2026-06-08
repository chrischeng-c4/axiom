# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "surface"
# case = "complex_is_callable"
# subject = "numbers.Complex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""numbers.Complex: complex_is_callable (surface)."""
import numbers

assert callable(numbers.Complex)
print("complex_is_callable OK")
