# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "unpack_non_buffer_type_error"
# subject = "struct.unpack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_struct.py"
# status = "filled"
# ///
"""struct.unpack: unpack_non_buffer_type_error (errors)."""
import struct

_raised = False
try:
    struct.unpack("b", 0)
except TypeError:
    _raised = True
assert _raised, "unpack_non_buffer_type_error: expected TypeError"
print("unpack_non_buffer_type_error OK")
