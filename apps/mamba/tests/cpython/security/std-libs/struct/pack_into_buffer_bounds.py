# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "security"
# case = "pack_into_buffer_bounds"
# subject = "struct.pack_into"
# kind = "semantic"
# xfail = "struct shim silently clips out-of-range pack_into writes instead of raising struct.error (WI #3929; struct_mod.rs mb_struct_pack_into)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.pack_into: pack_into/unpack_from past the end of, or before the start of, a mutable buffer raise struct.error with descriptive messages that quote the required size and the offset (incl. negative-offset 'no space'/'out of range' wording)"""
import struct

# Packing a field that does not fit raises struct.error.
s = struct.Struct("21s")
small = bytearray(10)
try:
    s.pack_into(small, 0, b"Reykjavik rocks, eow!")
    raise AssertionError("expected struct.error for oversized field")
except struct.error:
    pass

# unpack_from past the end raises struct.error.
field = struct.Struct("4s")
data = bytearray(b"abcd01234")
try:
    field.unpack_from(data, 6)
    raise AssertionError("expected struct.error past the end")
except struct.error:
    pass

# Boundary error messages are descriptive and quote the required size and offset.
try:
    struct.pack_into("b", bytearray(1), 5, 1)
    raise AssertionError("expected struct.error")
except struct.error as e:
    assert "at least 6 bytes" in str(e), f"pack_into msg = {str(e)!r}"
    assert "offset 5" in str(e), f"pack_into msg = {str(e)!r}"

try:
    struct.unpack_from("b", bytearray(1), 5)
    raise AssertionError("expected struct.error")
except struct.error as e:
    assert "at least 6 bytes" in str(e), f"unpack_from msg = {str(e)!r}"

# Negative offsets produce their own dedicated messages.
ten = bytearray(10)
try:
    struct.pack_into("<I", ten, -2, 123)
    raise AssertionError("expected struct.error")
except struct.error as e:
    assert "no space to pack 4 bytes at offset -2" in str(e), f"neg msg = {str(e)!r}"
try:
    struct.pack_into("<B", ten, -11, 123)
    raise AssertionError("expected struct.error")
except struct.error as e:
    assert "offset -11 out of range for 10-byte buffer" in str(e), f"oor msg = {str(e)!r}"

print("pack_into_buffer_bounds OK")
