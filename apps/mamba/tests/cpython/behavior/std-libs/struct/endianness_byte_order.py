# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "endianness_byte_order"
# subject = "struct.pack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack: big-endian '>I' and little-endian '<I' lay an asymmetric u32 down in opposite byte order; >i 1 == b'\\x00\\x00\\x00\\x01', <i 1 == b'\\x01\\x00\\x00\\x00'"""
import struct

# An asymmetric u32 packs in opposite byte order under the two endian prefixes.
assert struct.pack(">I", 0x01020304) == b"\x01\x02\x03\x04", "big-endian order"
assert struct.pack("<I", 0x01020304) == b"\x04\x03\x02\x01", "little-endian order"

# Big-endian writes the MSB first; little-endian the LSB first.
assert struct.pack(">i", 1) == b"\x00\x00\x00\x01", "big-endian 1"
assert struct.pack("<i", 1) == b"\x01\x00\x00\x00", "little-endian 1"

print("endianness_byte_order OK")
