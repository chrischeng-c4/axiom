# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "invalid_format_char_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: invalid_format_char_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("z", 1)
except struct.error:
    _raised = True
assert _raised, "invalid_format_char_raises: expected struct.error"
print("invalid_format_char_raises OK")
