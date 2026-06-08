# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fractions"
# dimension = "errors"
# case = "float_nan_raises"
# subject = "fractions.Fraction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fractions.py"
# status = "filled"
# ///
"""fractions.Fraction: float_nan_raises (errors)."""
import fractions

_raised = False
try:
    fractions.Fraction(float('nan'))
except ValueError:
    _raised = True
assert _raised, "float_nan_raises: expected ValueError"
print("float_nan_raises OK")
