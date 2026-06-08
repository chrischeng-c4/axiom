# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "pop_empty_indexerror"
# subject = "array.array.pop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array.pop: pop_empty_indexerror (errors)."""
import array

_raised = False
try:
    array.array("i").pop()
except IndexError:
    _raised = True
assert _raised, "pop_empty_indexerror: expected IndexError"
print("pop_empty_indexerror OK")
