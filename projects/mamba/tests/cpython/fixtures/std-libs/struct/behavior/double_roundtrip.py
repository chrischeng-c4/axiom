# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "double_roundtrip"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: '>d' is 64-bit IEEE 754: packing then unpacking 3.141592653589793 round-trips exactly"""
import struct

# '>d' is 8 bytes and round-trips a Python float (itself a C double) exactly.
_dv = struct.pack(">d", 3.141592653589793)
assert len(_dv) == 8, f"float64 width = {len(_dv)!r}"
_du = struct.unpack(">d", _dv)
assert _du[0] == 3.141592653589793, f"double round-trip = {_du[0]!r}"

print("double_roundtrip OK")
