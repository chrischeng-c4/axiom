# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "no_args_typeerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: no_args_typeerror (errors)."""
import array

_raised = False
try:
    array.array()
except TypeError:
    _raised = True
assert _raised, "no_args_typeerror: expected TypeError"
print("no_args_typeerror OK")
