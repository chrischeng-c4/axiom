# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "bad_type_for_int_code_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = "struct shim does not type-check pack args; accepts non-int silently (WI #3929)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: bad_type_for_int_code_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("i", "not_int")
except struct.error:
    _raised = True
assert _raised, "bad_type_for_int_code_raises: expected struct.error"
print("bad_type_for_int_code_raises OK")
