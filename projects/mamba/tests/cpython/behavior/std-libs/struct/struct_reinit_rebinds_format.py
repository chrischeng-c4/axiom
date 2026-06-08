# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_reinit_rebinds_format"
# subject = "struct.Struct"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.Struct: calling __init__ again on a Struct rebinds it to a new format: Struct('i') then __init__('ii') changes .size 4 -> 8 and round-trips two ints"""
import struct

# Re-running __init__ rebinds the Struct to a new format.
r = struct.Struct("i")
assert r.size == 4, "single int size"
r.__init__("ii")
assert r.size == 8, "reinit to two ints"
assert r.unpack(struct.pack("ii", 7, 9)) == (7, 9), "reinit round-trip"

print("struct_reinit_rebinds_format OK")
