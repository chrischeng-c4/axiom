# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "surface"
# case = "numerator_attr"
# subject = "fractions.Fraction(3, 4)"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fractions.Fraction(3, 4): numerator_attr (surface)."""
import fractions

assert hasattr(fractions.Fraction(3, 4), "numerator")
print("numerator_attr OK")
