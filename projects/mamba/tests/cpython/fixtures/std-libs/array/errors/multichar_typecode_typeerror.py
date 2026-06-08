# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "multichar_typecode_typeerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: multichar_typecode_typeerror (errors)."""
import array

_raised = False
try:
    array.array("xx")
except TypeError:
    _raised = True
assert _raised, "multichar_typecode_typeerror: expected TypeError"
print("multichar_typecode_typeerror OK")
