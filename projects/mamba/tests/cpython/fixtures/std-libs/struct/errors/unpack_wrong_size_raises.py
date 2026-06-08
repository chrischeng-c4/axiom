# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "errors"
# case = "unpack_wrong_size_raises"
# subject = "struct.unpack"
# kind = "mechanical"
# xfail = "struct shim does not validate buffer size; zero-pads/accepts instead of raising (WI #3929)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.unpack: unpack_wrong_size_raises (errors)."""
import struct

_raised = False
try:
    struct.unpack("ii", b"only4bytes")
except struct.error:
    _raised = True
assert _raised, "unpack_wrong_size_raises: expected struct.error"
print("unpack_wrong_size_raises OK")
