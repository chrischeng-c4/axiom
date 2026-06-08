# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "stray_byteorder_marker_raises"
# subject = "struct.calcsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.calcsize: stray_byteorder_marker_raises (errors)."""
import struct

_raised = False
try:
    struct.calcsize("i@i")
except struct.error:
    _raised = True
assert _raised, "stray_byteorder_marker_raises: expected struct.error"
print("stray_byteorder_marker_raises OK")
