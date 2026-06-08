# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "bad_typecode_valueerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: bad_typecode_valueerror (errors)."""
import array

_raised = False
try:
    array.array("Z")
except ValueError:
    _raised = True
assert _raised, "bad_typecode_valueerror: expected ValueError"
print("bad_typecode_valueerror OK")
