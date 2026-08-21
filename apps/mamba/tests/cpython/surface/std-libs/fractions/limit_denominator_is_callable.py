# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "surface"
# case = "limit_denominator_is_callable"
# subject = "fractions.Fraction.limit_denominator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fractions.Fraction.limit_denominator: limit_denominator_is_callable (surface)."""
import fractions

assert callable(fractions.Fraction.limit_denominator)
print("limit_denominator_is_callable OK")
