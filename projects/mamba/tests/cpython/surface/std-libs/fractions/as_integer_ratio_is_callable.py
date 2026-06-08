# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "surface"
# case = "as_integer_ratio_is_callable"
# subject = "fractions.Fraction.as_integer_ratio"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fractions.Fraction.as_integer_ratio: as_integer_ratio_is_callable (surface)."""
import fractions

assert callable(fractions.Fraction.as_integer_ratio)
print("as_integer_ratio_is_callable OK")
