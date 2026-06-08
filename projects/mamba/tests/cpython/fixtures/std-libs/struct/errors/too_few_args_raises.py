# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "too_few_args_raises"
# subject = "struct.pack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: too_few_args_raises (errors)."""
import struct

_raised = False
try:
    struct.pack("ii", 1)
except struct.error:
    _raised = True
assert _raised, "too_few_args_raises: expected struct.error"
print("too_few_args_raises OK")
