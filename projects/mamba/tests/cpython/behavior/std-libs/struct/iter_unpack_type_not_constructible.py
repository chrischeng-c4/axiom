# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "iter_unpack_type_not_constructible"
# subject = "struct.iter_unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.iter_unpack: the unpack-iterator type itself cannot be instantiated directly; calling type(it)() raises TypeError"""
import struct

# The iterator type itself cannot be constructed directly.
s = struct.Struct(">IB")
iter_type = type(s.iter_unpack(b""))
try:
    iter_type()
    raise AssertionError("expected TypeError")
except TypeError:
    pass

print("iter_unpack_type_not_constructible OK")
