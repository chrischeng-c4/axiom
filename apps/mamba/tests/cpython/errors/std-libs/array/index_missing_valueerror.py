# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "index_missing_valueerror"
# subject = "array.array.index"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.index: index_missing_valueerror (errors)."""
import array

_raised = False
try:
    array.array("i", [1, 2, 3]).index(99)
except ValueError:
    _raised = True
assert _raised, "index_missing_valueerror: expected ValueError"
print("index_missing_valueerror OK")
