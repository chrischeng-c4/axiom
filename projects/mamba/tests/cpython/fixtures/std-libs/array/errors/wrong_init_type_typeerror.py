# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "wrong_init_type_typeerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: wrong_init_type_typeerror (errors)."""
import array

_raised = False
try:
    array.array("i", "abc")
except TypeError:
    _raised = True
assert _raised, "wrong_init_type_typeerror: expected TypeError"
print("wrong_init_type_typeerror OK")
