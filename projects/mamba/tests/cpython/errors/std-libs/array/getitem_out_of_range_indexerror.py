# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "getitem_out_of_range_indexerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: getitem_out_of_range_indexerror (errors)."""
import array

_raised = False
try:
    array.array("i", [1, 2, 3])[99]
except IndexError:
    _raised = True
assert _raised, "getitem_out_of_range_indexerror: expected IndexError"
print("getitem_out_of_range_indexerror OK")
