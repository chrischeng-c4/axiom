# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "pack_into_writes_at_offset"
# subject = "struct.pack_into"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack_into: pack_into writes the packed field at the given byte offset inside a mutable buffer; two '>I' writes at offsets 0 and 4 leave the rest untouched and match the standalone pack output"""
import struct

# pack_into writes at the given offset; the two writes match standalone pack().
_buf = bytearray(8)
struct.pack_into(">I", _buf, 0, 0xDEAD)
struct.pack_into(">I", _buf, 4, 0xBEEF)
assert _buf[:4] == struct.pack(">I", 0xDEAD), "pack_into at offset 0"
assert _buf[4:] == struct.pack(">I", 0xBEEF), "pack_into at offset 4"

# Struct.pack_into writes the field at the requested offset, too.
text = b"Reykjavik rocks, eow!"
s = struct.Struct("21s")
big = bytearray(100)
s.pack_into(big, 10, text)
assert bytes(big[10:10 + len(text)]) == text, "Struct.pack_into at offset 10"

print("pack_into_writes_at_offset OK")
