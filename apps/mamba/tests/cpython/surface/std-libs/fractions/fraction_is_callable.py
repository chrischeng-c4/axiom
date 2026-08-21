# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "surface"
# case = "fraction_is_callable"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fractions.Fraction: fraction_is_callable (surface)."""
import fractions

assert callable(fractions.Fraction)
print("fraction_is_callable OK")
