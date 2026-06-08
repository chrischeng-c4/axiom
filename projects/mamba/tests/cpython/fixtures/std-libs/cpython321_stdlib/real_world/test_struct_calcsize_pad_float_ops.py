# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_struct_calcsize_pad_float_ops"
# subject = "cpython321.test_struct_calcsize_pad_float_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_struct_calcsize_pad_float_ops.py"
# status = "filled"
# ///
"""cpython321.test_struct_calcsize_pad_float_ops: execute CPython 3.12 seed test_struct_calcsize_pad_float_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `struct` corners not covered
# by `test_struct_ops`, `test_struct_pack_unpack_ops`, or
# `test_struct_unsigned_floats_ops`. Surface: `struct.calcsize`
# reports the byte width of the float (`d` → 8, `f` → 4) and bool
# (`?` → 1) format codes that aren't asserted elsewhere; the `x`
# pad-byte format contributes 1 byte; repeat-count syntax (`>3i`)
# pre-multiplies width. Endian markers `>` and `<` both keep the
# inner format width — `>H` and `<H` both calcsize to 2 — and
# produce mirrored byte streams (`>H` for 1 → `\x00\x01`, `<H` for
# 1 → `\x01\x00`).
import struct
_ledger: list[int] = []

# calcsize for float / bool / pad format codes
assert struct.calcsize(">d") == 8; _ledger.append(1)
assert struct.calcsize(">f") == 4; _ledger.append(1)
assert struct.calcsize(">?") == 1; _ledger.append(1)
assert struct.calcsize(">x") == 1; _ledger.append(1)

# calcsize with repeat count
assert struct.calcsize(">3i") == 12; _ledger.append(1)
assert struct.calcsize(">4h") == 8; _ledger.append(1)
assert struct.calcsize(">2q") == 16; _ledger.append(1)

# calcsize with pad mixed in
assert struct.calcsize(">ix") == 5; _ledger.append(1)
assert struct.calcsize(">2xi") == 6; _ledger.append(1)

# Endian markers preserve width
assert struct.calcsize(">H") == 2; _ledger.append(1)
assert struct.calcsize("<H") == 2; _ledger.append(1)
assert struct.calcsize(">I") == 4; _ledger.append(1)
assert struct.calcsize("<I") == 4; _ledger.append(1)

# Endian produces mirrored bytes for the same value
assert struct.pack(">H", 1) == b"\x00\x01"; _ledger.append(1)
assert struct.pack("<H", 1) == b"\x01\x00"; _ledger.append(1)
assert struct.pack(">I", 256) == b"\x00\x00\x01\x00"; _ledger.append(1)
assert struct.pack("<I", 256) == b"\x00\x01\x00\x00"; _ledger.append(1)

# Repeat-count round-trip
packed = struct.pack(">3i", 10, 20, 30)
assert struct.unpack(">3i", packed) == (10, 20, 30); _ledger.append(1)
assert len(packed) == 12; _ledger.append(1)

# x-pad produces null byte
assert struct.pack(">x") == b"\x00"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_struct_calcsize_pad_float_ops {sum(_ledger)} asserts")
