# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "float_roundtrip"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: '>f' is 32-bit IEEE 754: packing then unpacking 3.14 round-trips to within float32 precision (abs error < 1e-3)"""
import struct

# '>f' is 4 bytes; the round-trip is only approximate at float32 precision.
_fv = struct.pack(">f", 3.14)
assert len(_fv) == 4, f"float32 width = {len(_fv)!r}"
_fu = struct.unpack(">f", _fv)
assert abs(_fu[0] - 3.14) < 0.001, f"float round-trip = {_fu[0]!r}"

print("float_roundtrip OK")
