# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "surface"
# case = "is_integer_is_callable"
# subject = "fractions.Fraction.is_integer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fractions.Fraction.is_integer: is_integer_is_callable (surface)."""
import fractions

assert callable(fractions.Fraction.is_integer)
print("is_integer_is_callable OK")
