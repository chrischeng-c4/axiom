# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "int_array_rejects_float_typeerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: int_array_rejects_float_typeerror (errors)."""
import array

_raised = False
try:
    array.array("i", [1, 2.5])
except TypeError:
    _raised = True
assert _raised, "int_array_rejects_float_typeerror: expected TypeError"
print("int_array_rejects_float_typeerror OK")
