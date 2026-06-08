# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "unpack_from_slides_window"
# subject = "struct.unpack_from"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.unpack_from: unpack_from slides a fixed-size '4s' window across bytes and bytearray inputs at each offset, defaults offset to 0, and accepts keyword args buffer=/offset="""
import struct

field = struct.Struct("4s")
for cls in (bytes, bytearray):
    data = cls(b"abcd01234")
    # Defaults offset to 0, then slides the window across the buffer.
    assert field.unpack_from(data) == (b"abcd",), f"default offset ({cls.__name__})"
    assert field.unpack_from(data, 2) == (b"cd01",), f"offset 2 ({cls.__name__})"
    for i in range(6):
        assert field.unpack_from(data, i) == (bytes(data[i:i + 4]),), f"window at {i}"

# unpack_from accepts keyword arguments.
assert field.unpack_from(buffer=b"abcd01234", offset=2) == (b"cd01",), "keyword args"

print("unpack_from_slides_window OK")
