# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "errors"
# case = "instantiate_real_raises"
# subject = "numbers.Real"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Real: instantiate_real_raises (errors)."""
import numbers

_raised = False
try:
    numbers.Real()
except TypeError:
    _raised = True
assert _raised, "instantiate_real_raises: expected TypeError"
print("instantiate_real_raises OK")
