# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_struct_ops"
# subject = "cpython321.test_struct_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_struct_ops.py"
# status = "filled"
# ///
"""cpython321.test_struct_ops: execute CPython 3.12 seed test_struct_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `struct` stdlib module.
# Surface: pack/unpack round-trip for int32 big-endian, int16
# little-endian, calcsize for common formats.
# Companion to stub/test_struct.py — vendored unittest seed.
import struct
_ledger: list[int] = []
assert struct.pack(">i", 42) == b"\x00\x00\x00\x2a"; _ledger.append(1)
assert struct.unpack(">i", b"\x00\x00\x00\x2a") == (42,); _ledger.append(1)
assert struct.pack("<h", 1) == b"\x01\x00"; _ledger.append(1)
assert struct.unpack("<h", b"\x01\x00") == (1,); _ledger.append(1)
assert struct.calcsize(">i") == 4; _ledger.append(1)
assert struct.calcsize("<h") == 2; _ledger.append(1)
assert struct.calcsize(">q") == 8; _ledger.append(1)
# Round-trip composite format
packed = struct.pack(">ihh", 1000, 2, 3)
assert struct.unpack(">ihh", packed) == (1000, 2, 3); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_struct_ops {sum(_ledger)} asserts")
