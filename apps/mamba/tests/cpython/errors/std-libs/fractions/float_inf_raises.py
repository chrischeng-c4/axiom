# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "errors"
# case = "float_inf_raises"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: float_inf_raises (errors)."""
import fractions

_raised = False
try:
    fractions.Fraction(float('inf'))
except OverflowError:
    _raised = True
assert _raised, "float_inf_raises: expected OverflowError"
print("float_inf_raises OK")
