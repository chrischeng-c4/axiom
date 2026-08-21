# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "iter_unpack_buffer_inputs"
# subject = "struct.iter_unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.iter_unpack: Struct.iter_unpack accepts any buffer-protocol input (memoryview, bytearray) and produces the same record list as from bytes"""
import struct

data = bytes(range(1, 16))  # three ">IB" records of 5 bytes each
s = struct.Struct(">IB")
expected = [(16909060, 5), (101124105, 10), (185339150, 15)]

# Any buffer-like object works as input and yields the same records.
for view in (memoryview(data), bytearray(data)):
    records = list(s.iter_unpack(view))
    assert records == expected, f"buffer {type(view).__name__}"

print("iter_unpack_buffer_inputs OK")
