# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "errors"
# case = "instantiate_complex_raises"
# subject = "numbers.Complex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Complex: instantiate_complex_raises (errors)."""
import numbers

_raised = False
try:
    numbers.Complex()
except TypeError:
    _raised = True
assert _raised, "instantiate_complex_raises: expected TypeError"
print("instantiate_complex_raises OK")
