# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "append_type_mismatch_typeerror"
# subject = "array.array.append"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array.append: append_type_mismatch_typeerror (errors)."""
import array

_raised = False
try:
    array.array("i", [1, 2, 3]).append("x")
except TypeError:
    _raised = True
assert _raised, "append_type_mismatch_typeerror: expected TypeError"
print("append_type_mismatch_typeerror OK")
