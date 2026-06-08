# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "float_into_int_code_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = "struct shim does not reject a float for an int code; coerces/truncates (WI #3929)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: float_into_int_code_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("i", 1.5)
except struct.error:
    _raised = True
assert _raised, "float_into_int_code_raises: expected struct.error"
print("float_into_int_code_raises OK")
