# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "remove_missing_valueerror"
# subject = "array.array.remove"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.remove: remove_missing_valueerror (errors)."""
import array

_raised = False
try:
    array.array("i", [1, 2, 3]).remove(99)
except ValueError:
    _raised = True
assert _raised, "remove_missing_valueerror: expected ValueError"
print("remove_missing_valueerror OK")
