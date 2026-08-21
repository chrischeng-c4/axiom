# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_struct_unsigned_floats_ops"
# subject = "cpython321.test_struct_unsigned_floats_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_struct_unsigned_floats_ops.py"
# status = "filled"
# ///
"""cpython321.test_struct_unsigned_floats_ops: execute CPython 3.12 seed test_struct_unsigned_floats_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for struct module surfaces beyond
# test_struct_ops (which covers signed >i / <h / >q and a ihh
# composite).
# Surface: unsigned integer format codes B (1B), H (2B), I (4B);
# byte-order prefixes > and < flip the byte layout; pack returns
# bytes of the expected length; multi-field packs concatenate;
# float `f` (4B) and double `d` (8B) round-trip exact-binary values;
# calcsize agrees with the packed length for every common code.
import struct
_ledger: list[int] = []

# Big-endian unsigned int — most-significant byte first
assert struct.pack(">I", 1) == b"\x00\x00\x00\x01"; _ledger.append(1)
assert struct.pack(">I", 256) == b"\x00\x00\x01\x00"; _ledger.append(1)
# Little-endian flips the byte order
assert struct.pack("<I", 256) == b"\x00\x01\x00\x00"; _ledger.append(1)
# Packed length for >I is 4 bytes
assert len(struct.pack(">I", 1)) == 4; _ledger.append(1)

# Unpack returns a one-element tuple
assert struct.unpack(">I", b"\x00\x00\x01\x00") == (256,); _ledger.append(1)
# Indexing the tuple yields the scalar value
assert struct.unpack(">I", b"\x00\x00\x01\x00")[0] == 256; _ledger.append(1)

# Single unsigned byte 'B' — encodes 65 as b"A"
assert struct.pack(">B", 65) == b"A"; _ledger.append(1)
assert struct.unpack(">B", b"A") == (65,); _ledger.append(1)
assert len(struct.pack(">B", 0)) == 1; _ledger.append(1)

# Unsigned short 'H' — 2 bytes
assert struct.pack(">H", 1000) == b"\x03\xe8"; _ledger.append(1)
assert struct.unpack(">H", b"\x03\xe8") == (1000,); _ledger.append(1)
assert len(struct.pack(">H", 1)) == 2; _ledger.append(1)

# Multiple unsigned-int fields concatenate
assert struct.pack(">II", 1, 2) == b"\x00\x00\x00\x01\x00\x00\x00\x02"; _ledger.append(1)
assert struct.unpack(">II", b"\x00\x00\x00\x01\x00\x00\x00\x02") == (1, 2); _ledger.append(1)
assert len(struct.pack(">II", 1, 2)) == 8; _ledger.append(1)

# Float (4 bytes) — 1.5 is exactly representable
assert len(struct.pack(">f", 1.5)) == 4; _ledger.append(1)
assert struct.unpack(">f", struct.pack(">f", 1.5)) == (1.5,); _ledger.append(1)
# 0.5 and 0.25 are also exact
assert struct.unpack(">f", struct.pack(">f", 0.5)) == (0.5,); _ledger.append(1)
assert struct.unpack(">f", struct.pack(">f", 0.25)) == (0.25,); _ledger.append(1)

# Double (8 bytes) — 3.14 round-trips exactly through d
assert len(struct.pack(">d", 3.14)) == 8; _ledger.append(1)
assert struct.unpack(">d", struct.pack(">d", 3.14)) == (3.14,); _ledger.append(1)
# 1.0 and 0.5 are exact for d as well
assert struct.unpack(">d", struct.pack(">d", 1.0)) == (1.0,); _ledger.append(1)
assert struct.unpack(">d", struct.pack(">d", 0.5)) == (0.5,); _ledger.append(1)

# calcsize matches the packed length for each unsigned code
assert struct.calcsize(">I") == 4; _ledger.append(1)
assert struct.calcsize(">II") == 8; _ledger.append(1)
assert struct.calcsize(">B") == 1; _ledger.append(1)
assert struct.calcsize(">H") == 2; _ledger.append(1)
assert struct.calcsize(">f") == 4; _ledger.append(1)
assert struct.calcsize(">d") == 8; _ledger.append(1)

# calcsize and len(pack(...)) agree
assert struct.calcsize(">I") == len(struct.pack(">I", 0)); _ledger.append(1)
assert struct.calcsize(">II") == len(struct.pack(">II", 0, 0)); _ledger.append(1)
assert struct.calcsize(">d") == len(struct.pack(">d", 0.0)); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_struct_unsigned_floats_ops {sum(_ledger)} asserts")
