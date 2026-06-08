# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_int_signed_bytes_round_trip_ops"
# subject = "cpython321.test_int_signed_bytes_round_trip_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_int_signed_bytes_round_trip_ops.py"
# status = "filled"
# ///
"""cpython321.test_int_signed_bytes_round_trip_ops: execute CPython 3.12 seed test_int_signed_bytes_round_trip_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `signed=True` keyword of
# `int.to_bytes` / `int.from_bytes` on the matching subset of mamba +
# CPython behavior. The existing `test_int_bit_count_to_from_bytes_ops`
# fixture only covers the unsigned (default signed=False) form of these
# methods, and only one `(-1).to_bytes(2, 'big', signed=True)` assert
# leaks into `test_dict_views_set_inplace_ops`. That leaves the
# two's-complement signed encoding contract for both endiannesses + the
# round-trip identity essentially untested across the (INT8 .. INT32)
# range that mamba's int representation natively supports.
#
# Surface (the matching subset between mamba and CPython):
#   • int.to_bytes(length, byteorder, signed=True) for INT8 / INT16 /
#     INT32 boundaries (MIN, -1, 0, 1, MAX) on big and little endian;
#   • int.from_bytes(bytes, byteorder, signed=True) on the matching
#     boundary byte patterns;
#   • round-trip identity: `from_bytes(v.to_bytes(L, BO, signed=True),
#     BO, signed=True) == v` for every endian + length combination;
#   • narrow length (1, 2, 4, 8 bytes) on positive AND negative ints;
#   • bit_length() on signed int boundaries — confirms `(MIN).bit_length`
#     is `n` (not `n+1`) and the two's-complement rule that `~x ==
#     -x - 1` holds for boundary values.
_ledger: list[int] = []

# INT8 boundaries — to_bytes signed=True
assert (-128).to_bytes(1, "big", signed=True) == b"\x80"; _ledger.append(1)
assert (127).to_bytes(1, "big", signed=True) == b"\x7f"; _ledger.append(1)
assert (-1).to_bytes(1, "big", signed=True) == b"\xff"; _ledger.append(1)
assert (0).to_bytes(1, "big", signed=True) == b"\x00"; _ledger.append(1)
assert (1).to_bytes(1, "big", signed=True) == b"\x01"; _ledger.append(1)
# little endian — single byte is the same as big
assert (-128).to_bytes(1, "little", signed=True) == b"\x80"; _ledger.append(1)
assert (127).to_bytes(1, "little", signed=True) == b"\x7f"; _ledger.append(1)
assert (-1).to_bytes(1, "little", signed=True) == b"\xff"; _ledger.append(1)

# INT16 boundaries — to_bytes signed=True big endian
assert (-32768).to_bytes(2, "big", signed=True) == b"\x80\x00"; _ledger.append(1)
assert (32767).to_bytes(2, "big", signed=True) == b"\x7f\xff"; _ledger.append(1)
assert (-1).to_bytes(2, "big", signed=True) == b"\xff\xff"; _ledger.append(1)
assert (0).to_bytes(2, "big", signed=True) == b"\x00\x00"; _ledger.append(1)
assert (256).to_bytes(2, "big", signed=True) == b"\x01\x00"; _ledger.append(1)
# INT16 boundaries — to_bytes signed=True little endian
assert (-32768).to_bytes(2, "little", signed=True) == b"\x00\x80"; _ledger.append(1)
assert (32767).to_bytes(2, "little", signed=True) == b"\xff\x7f"; _ledger.append(1)
assert (-1).to_bytes(2, "little", signed=True) == b"\xff\xff"; _ledger.append(1)
assert (256).to_bytes(2, "little", signed=True) == b"\x00\x01"; _ledger.append(1)

# INT32 boundaries — to_bytes signed=True big endian
assert (-2147483648).to_bytes(4, "big", signed=True) == b"\x80\x00\x00\x00"; _ledger.append(1)
assert (2147483647).to_bytes(4, "big", signed=True) == b"\x7f\xff\xff\xff"; _ledger.append(1)
assert (-1).to_bytes(4, "big", signed=True) == b"\xff\xff\xff\xff"; _ledger.append(1)
assert (0).to_bytes(4, "big", signed=True) == b"\x00\x00\x00\x00"; _ledger.append(1)
# INT32 boundaries — to_bytes signed=True little endian
assert (-2147483648).to_bytes(4, "little", signed=True) == b"\x00\x00\x00\x80"; _ledger.append(1)
assert (2147483647).to_bytes(4, "little", signed=True) == b"\xff\xff\xff\x7f"; _ledger.append(1)
assert (-1).to_bytes(4, "little", signed=True) == b"\xff\xff\xff\xff"; _ledger.append(1)

# 8-byte padding for small positive / negative values
assert (0).to_bytes(8, "big", signed=True) == b"\x00\x00\x00\x00\x00\x00\x00\x00"; _ledger.append(1)
assert (-1).to_bytes(8, "big", signed=True) == b"\xff\xff\xff\xff\xff\xff\xff\xff"; _ledger.append(1)
assert (1).to_bytes(8, "big", signed=True) == b"\x00\x00\x00\x00\x00\x00\x00\x01"; _ledger.append(1)

# INT8 from_bytes signed=True
assert int.from_bytes(b"\x80", "big", signed=True) == -128; _ledger.append(1)
assert int.from_bytes(b"\x7f", "big", signed=True) == 127; _ledger.append(1)
assert int.from_bytes(b"\xff", "big", signed=True) == -1; _ledger.append(1)
assert int.from_bytes(b"\x00", "big", signed=True) == 0; _ledger.append(1)
assert int.from_bytes(b"\x01", "big", signed=True) == 1; _ledger.append(1)
# Same byte → same answer regardless of byteorder for length 1
assert int.from_bytes(b"\x80", "little", signed=True) == -128; _ledger.append(1)
assert int.from_bytes(b"\xff", "little", signed=True) == -1; _ledger.append(1)

# INT16 from_bytes signed=True
assert int.from_bytes(b"\x80\x00", "big", signed=True) == -32768; _ledger.append(1)
assert int.from_bytes(b"\x7f\xff", "big", signed=True) == 32767; _ledger.append(1)
assert int.from_bytes(b"\xff\xff", "big", signed=True) == -1; _ledger.append(1)
assert int.from_bytes(b"\x00\x00", "big", signed=True) == 0; _ledger.append(1)
# little endian
assert int.from_bytes(b"\x00\x80", "little", signed=True) == -32768; _ledger.append(1)
assert int.from_bytes(b"\xff\x7f", "little", signed=True) == 32767; _ledger.append(1)
assert int.from_bytes(b"\xff\xff", "little", signed=True) == -1; _ledger.append(1)
assert int.from_bytes(b"\x00\x01", "little", signed=True) == 256; _ledger.append(1)

# INT32 from_bytes signed=True
assert int.from_bytes(b"\x80\x00\x00\x00", "big", signed=True) == -2147483648; _ledger.append(1)
assert int.from_bytes(b"\x7f\xff\xff\xff", "big", signed=True) == 2147483647; _ledger.append(1)
assert int.from_bytes(b"\xff\xff\xff\xff", "big", signed=True) == -1; _ledger.append(1)
assert int.from_bytes(b"\x00\x00\x00\x00", "big", signed=True) == 0; _ledger.append(1)
# little endian
assert int.from_bytes(b"\x00\x00\x00\x80", "little", signed=True) == -2147483648; _ledger.append(1)
assert int.from_bytes(b"\xff\xff\xff\x7f", "little", signed=True) == 2147483647; _ledger.append(1)

# Round-trip identity over INT8 + INT16 + INT32 boundaries (big endian)
for _v in [-128, -1, 0, 1, 127, -32768, 32767, -2147483648, 2147483647]:
    _bs = _v.to_bytes(8, "big", signed=True)
    assert int.from_bytes(_bs, "big", signed=True) == _v
    _ledger.append(1)

# Round-trip identity (little endian)
for _v in [-128, -1, 0, 1, 127, -32768, 32767, -2147483648, 2147483647]:
    _bs = _v.to_bytes(8, "little", signed=True)
    assert int.from_bytes(_bs, "little", signed=True) == _v
    _ledger.append(1)

# Round-trip with minimal-length encodings
for _v in [-128, -1, 1, 127]:
    _bs = _v.to_bytes(1, "big", signed=True)
    assert int.from_bytes(_bs, "big", signed=True) == _v
    _ledger.append(1)
for _v in [-32768, -1, 1, 32767]:
    _bs = _v.to_bytes(2, "big", signed=True)
    assert int.from_bytes(_bs, "big", signed=True) == _v
    _ledger.append(1)
for _v in [-2147483648, -1, 1, 2147483647]:
    _bs = _v.to_bytes(4, "big", signed=True)
    assert int.from_bytes(_bs, "big", signed=True) == _v
    _ledger.append(1)

# bit_length() on signed int boundaries
# (-128).bit_length() == 8 — the unsigned magnitude fits in 8 bits
assert (-128).bit_length() == 8; _ledger.append(1)
assert (127).bit_length() == 7; _ledger.append(1)
assert (128).bit_length() == 8; _ledger.append(1)
assert (-1).bit_length() == 1; _ledger.append(1)
assert (1).bit_length() == 1; _ledger.append(1)
assert (-32768).bit_length() == 16; _ledger.append(1)
assert (32767).bit_length() == 15; _ledger.append(1)
assert (-2147483648).bit_length() == 32; _ledger.append(1)
assert (2147483647).bit_length() == 31; _ledger.append(1)

# Two's-complement invariant: ~x == -x - 1 on every boundary
assert ~0 == -1; _ledger.append(1)
assert ~-1 == 0; _ledger.append(1)
assert ~127 == -128; _ledger.append(1)
assert ~-128 == 127; _ledger.append(1)
assert ~32767 == -32768; _ledger.append(1)
assert ~-32768 == 32767; _ledger.append(1)

# Cross-form consistency: -1 encoded N bytes is always all-0xFF
assert (-1).to_bytes(1, "big", signed=True) == b"\xff"; _ledger.append(1)
assert (-1).to_bytes(2, "big", signed=True) == b"\xff\xff"; _ledger.append(1)
assert (-1).to_bytes(4, "big", signed=True) == b"\xff" * 4; _ledger.append(1)
assert (-1).to_bytes(8, "big", signed=True) == b"\xff" * 8; _ledger.append(1)

# Cross-form consistency: 0 encoded N bytes is always all-0x00
assert (0).to_bytes(1, "big", signed=True) == b"\x00"; _ledger.append(1)
assert (0).to_bytes(2, "big", signed=True) == b"\x00\x00"; _ledger.append(1)
assert (0).to_bytes(4, "big", signed=True) == b"\x00\x00\x00\x00"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_int_signed_bytes_round_trip_ops {sum(_ledger)} asserts")
