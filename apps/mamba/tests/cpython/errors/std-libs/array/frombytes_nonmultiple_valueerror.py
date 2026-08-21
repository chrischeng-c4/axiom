# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "frombytes_nonmultiple_valueerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: frombytes_nonmultiple_valueerror (errors)."""
import array

_raised = False
try:
    array.array("i", b"12345")
except ValueError:
    _raised = True
assert _raised, "frombytes_nonmultiple_valueerror: expected ValueError"
print("frombytes_nonmultiple_valueerror OK")
