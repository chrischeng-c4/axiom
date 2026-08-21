# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "dangling_repeat_count_raises"
# subject = "struct.calcsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.calcsize: dangling_repeat_count_raises (errors)."""
import struct

_raised = False
try:
    struct.calcsize("4")
except struct.error:
    _raised = True
assert _raised, "dangling_repeat_count_raises: expected struct.error"
print("dangling_repeat_count_raises OK")
