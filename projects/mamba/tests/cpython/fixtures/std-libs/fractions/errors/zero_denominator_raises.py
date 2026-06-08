# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "errors"
# case = "zero_denominator_raises"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: zero_denominator_raises (errors)."""
import fractions

_raised = False
try:
    fractions.Fraction(1, 0)
except ZeroDivisionError:
    _raised = True
assert _raised, "zero_denominator_raises: expected ZeroDivisionError"
print("zero_denominator_raises OK")
