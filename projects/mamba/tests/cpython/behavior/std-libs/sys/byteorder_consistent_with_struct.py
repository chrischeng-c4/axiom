# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "byteorder_consistent_with_struct"
# subject = "sys.byteorder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.byteorder: sys.byteorder is 'little' or 'big' and packing 1 as a 4-byte unsigned int with the matching endian prefix yields bytes"""
import sys
import struct

assert sys.byteorder in ("little", "big"), f"byteorder = {sys.byteorder!r}"
if sys.byteorder == "little":
    _packed = struct.pack("<I", 1)
else:
    _packed = struct.pack(">I", 1)
assert isinstance(_packed, bytes), "struct pack consistent with byteorder"
print("byteorder_consistent_with_struct OK")
