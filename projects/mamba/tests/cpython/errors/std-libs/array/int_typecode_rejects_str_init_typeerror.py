# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "errors"
# case = "int_typecode_rejects_str_init_typeerror"
# subject = "array.array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: int_typecode_rejects_str_init_typeerror (errors)."""
import array

_raised = False
try:
    array.array("b", "foo")
except TypeError:
    _raised = True
assert _raised, "int_typecode_rejects_str_init_typeerror: expected TypeError"
print("int_typecode_rejects_str_init_typeerror OK")
