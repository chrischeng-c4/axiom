# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "errors"
# case = "instantiate_integral_raises"
# subject = "numbers.Integral"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Integral: instantiate_integral_raises (errors)."""
import numbers

_raised = False
try:
    numbers.Integral()
except TypeError:
    _raised = True
assert _raised, "instantiate_integral_raises: expected TypeError"
print("instantiate_integral_raises OK")
