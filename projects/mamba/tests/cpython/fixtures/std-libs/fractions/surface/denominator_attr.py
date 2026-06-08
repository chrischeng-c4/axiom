# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "surface"
# case = "denominator_attr"
# subject = "fractions.Fraction(3, 4)"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fractions.Fraction(3, 4): denominator_attr (surface)."""
import fractions

assert hasattr(fractions.Fraction(3, 4), "denominator")
print("denominator_attr OK")
