# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "mixed_typecode_concat_typeerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: mixed_typecode_concat_typeerror (errors)."""
import array

_raised = False
try:
    array.array("i", [1]) + array.array("d", [1.0])
except TypeError:
    _raised = True
assert _raised, "mixed_typecode_concat_typeerror: expected TypeError"
print("mixed_typecode_concat_typeerror OK")
