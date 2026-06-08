# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "truediv_by_zero_zerodivision"
# subject = "operator.truediv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.truediv: truediv_by_zero_zerodivision (errors)."""
import operator

_raised = False
try:
    operator.truediv(1, 0)
except ZeroDivisionError:
    _raised = True
assert _raised, "truediv_by_zero_zerodivision: expected ZeroDivisionError"
print("truediv_by_zero_zerodivision OK")
