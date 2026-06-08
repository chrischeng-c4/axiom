# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "surface"
# case = "rational_is_callable"
# subject = "numbers.Rational"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""numbers.Rational: rational_is_callable (surface)."""
import numbers

assert callable(numbers.Rational)
print("rational_is_callable OK")
