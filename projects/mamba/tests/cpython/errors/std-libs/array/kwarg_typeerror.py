# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "kwarg_typeerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: kwarg_typeerror (errors)."""
import array

_raised = False
try:
    array.array(spam=42)
except TypeError:
    _raised = True
assert _raised, "kwarg_typeerror: expected TypeError"
print("kwarg_typeerror OK")
