# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "setitem_out_of_range_indexerror"
# subject = "operator.setitem"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.setitem: setitem_out_of_range_indexerror (errors)."""
import operator

_raised = False
try:
    operator.setitem([0, 1, 2], 4, 99)
except IndexError:
    _raised = True
assert _raised, "setitem_out_of_range_indexerror: expected IndexError"
print("setitem_out_of_range_indexerror OK")
