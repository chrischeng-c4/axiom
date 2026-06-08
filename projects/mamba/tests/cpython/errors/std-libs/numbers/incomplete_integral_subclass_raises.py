# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "errors"
# case = "incomplete_integral_subclass_raises"
# subject = "numbers.Integral"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Integral: incomplete_integral_subclass_raises (errors)."""
import numbers

_raised = False
try:
    type('WrongInt', (numbers.Integral,), {})()
except TypeError:
    _raised = True
assert _raised, "incomplete_integral_subclass_raises: expected TypeError"
print("incomplete_integral_subclass_raises OK")
