# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "trailing_repeat_count_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: trailing_repeat_count_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("12345")
except struct.error:
    _raised = True
assert _raised, "trailing_repeat_count_raises: expected struct.error"
print("trailing_repeat_count_raises OK")
