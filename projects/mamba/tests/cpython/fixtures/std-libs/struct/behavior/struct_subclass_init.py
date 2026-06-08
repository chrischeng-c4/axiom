# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "struct_subclass_init"
# subject = "struct.Struct"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.Struct: Struct can be subclassed with the format set in __init__ via super().__init__('>h'); the subclass packs/unpacks like the base"""
import struct


# Struct can be subclassed and configured in __init__.
class BigShort(struct.Struct):
    def __init__(self):
        super().__init__(">h")


bs = BigShort()
assert bs.pack(12345) == b"09", f"subclass pack = {bs.pack(12345)!r}"
assert bs.unpack(b"09") == (12345,), "subclass unpack"

print("struct_subclass_init OK")
