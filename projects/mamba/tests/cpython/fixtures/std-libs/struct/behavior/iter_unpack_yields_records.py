# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "struct"
# dimension = "behavior"
# case = "iter_unpack_yields_records"
# subject = "struct.iter_unpack"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""struct.iter_unpack: both Struct.iter_unpack and module struct.iter_unpack yield successive '>IB' records from a flat byte buffer and then raise StopIteration (stays stopped on repeat next())"""
import struct

data = bytes(range(1, 16))  # 15 bytes -> three ">IB" records of 5 bytes each

# Struct.iter_unpack yields successive records, then StopIteration.
s = struct.Struct(">IB")
it = s.iter_unpack(data)
assert next(it) == (16909060, 5), "record 0"
assert next(it) == (101124105, 10), "record 1"
assert next(it) == (185339150, 15), "record 2"
for _ in range(2):
    try:
        next(it)
        raise AssertionError("expected StopIteration")
    except StopIteration:
        pass

# The module-level struct.iter_unpack behaves the same.
it2 = struct.iter_unpack(">IB", bytes(range(1, 11)))
assert next(it2) == (16909060, 5), "module iter record 0"
assert next(it2) == (101124105, 10), "module iter record 1"
try:
    next(it2)
    raise AssertionError("expected StopIteration")
except StopIteration:
    pass

print("iter_unpack_yields_records OK")
