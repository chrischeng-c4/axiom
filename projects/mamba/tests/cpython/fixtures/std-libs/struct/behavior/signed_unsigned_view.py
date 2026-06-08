# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "signed_unsigned_view"
# subject = "struct.unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.unpack: the bytes of signed '>i' -1 re-interpreted as unsigned '>I' read back as 0xFFFFFFFF (two's-complement view)"""
import struct

# Pack -1 as a signed int, then read the same bytes as unsigned.
_neg = struct.pack(">i", -1)
_unsigned = struct.unpack(">I", _neg)
assert _unsigned == (0xFFFFFFFF,), f"unsigned view of -1 = {_unsigned!r}"

print("signed_unsigned_view OK")
