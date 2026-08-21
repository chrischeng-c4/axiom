# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "unpack_from_buffer_protocol"
# subject = "struct.unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.unpack: unpack reads from any buffer-protocol object (memoryview, bytearray), not just bytes: '>I' on b'\\x12\\x34\\x56\\x78' yields 0x12345678 from each"""
import struct

# unpack reads from any buffer-protocol object, not just bytes.
for buf in (memoryview(b"\x12\x34\x56\x78"), bytearray(b"\x12\x34\x56\x78")):
    (value,) = struct.unpack(">I", buf)
    assert value == 0x12345678, f"buffer unpack from {type(buf).__name__}"

print("unpack_from_buffer_protocol OK")
