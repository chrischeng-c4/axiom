# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "too_many_args_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: too_many_args_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("i", 1, 2, 3)
except struct.error:
    _raised = True
assert _raised, "too_many_args_raises: expected struct.error"
print("too_many_args_raises OK")
