# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "errors"
# case = "divide_by_zero_fraction_raises"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: divide_by_zero_fraction_raises (errors)."""
import fractions

_raised = False
try:
    fractions.Fraction(1, 2) / fractions.Fraction(0)
except ZeroDivisionError:
    _raised = True
assert _raised, "divide_by_zero_fraction_raises: expected ZeroDivisionError"
print("divide_by_zero_fraction_raises OK")
