# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "errors"
# case = "instantiate_rational_raises"
# subject = "numbers.Rational"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Rational: instantiate_rational_raises (errors)."""
import numbers

_raised = False
try:
    numbers.Rational()
except TypeError:
    _raised = True
assert _raised, "instantiate_rational_raises: expected TypeError"
print("instantiate_rational_raises OK")
