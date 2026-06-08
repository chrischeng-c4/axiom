# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "surface"
# case = "number_is_callable"
# subject = "numbers.Number"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""numbers.Number: number_is_callable (surface)."""
import numbers

assert callable(numbers.Number)
print("number_is_callable OK")
