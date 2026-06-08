# Operational AssertionPass seed for integer reflection methods:
# `bit_count`, `to_bytes`, `from_bytes`, and the base-conversion
# argument forms of `int()`. Surface: `.bit_count()` returns the
# number of 1-bits in the absolute value (PEP 657-era addition);
# `.to_bytes(length, byteorder)` serialises a non-negative integer
# to a fixed-width byte sequence in `"big"` or `"little"` endian;
# the inverse classmethod `int.from_bytes(b, byteorder)` reads back
# the same integer. Round-tripping through `to_bytes` then
# `from_bytes` is value-preserving. `int(str, base)` parses an
# integer literal in an arbitrary base (2 through 36 plus 0 for
# "infer from `0b`/`0o`/`0x` prefix"). `divmod(a, b)` returns
# `(a // b, a % b)` as a 2-tuple, and the negative-dividend case
# follows Python's floor-division convention where the quotient
# rounds toward negative infinity and the remainder takes the sign
# of the divisor.
_ledger: list[int] = []

# bit_count counts 1-bits in the absolute value
assert (7).bit_count() == 3; _ledger.append(1)  # 0b111
assert (0).bit_count() == 0; _ledger.append(1)
assert (255).bit_count() == 8; _ledger.append(1)
assert (256).bit_count() == 1; _ledger.append(1)  # 0b100000000

# to_bytes serialises with explicit endianness
assert (256).to_bytes(2, "big") == b"\x01\x00"; _ledger.append(1)
assert (256).to_bytes(2, "little") == b"\x00\x01"; _ledger.append(1)
assert (1).to_bytes(4, "big") == b"\x00\x00\x00\x01"; _ledger.append(1)
assert (255).to_bytes(1, "big") == b"\xff"; _ledger.append(1)

# from_bytes is the inverse
assert int.from_bytes(b"\x01\x00", "big") == 256; _ledger.append(1)
assert int.from_bytes(b"\x00\x01", "little") == 256; _ledger.append(1)
assert int.from_bytes(b"\xff", "big") == 255; _ledger.append(1)
assert int.from_bytes(b"\x00\x00\x00\x01", "big") == 1; _ledger.append(1)

# Round-trip: to_bytes then from_bytes is value-preserving
val = 12345
assert int.from_bytes(val.to_bytes(4, "big"), "big") == val; _ledger.append(1)
assert int.from_bytes(val.to_bytes(4, "little"), "little") == val; _ledger.append(1)

# int(str, base) parses arbitrary radix
assert int("ff", 16) == 255; _ledger.append(1)
assert int("101", 2) == 5; _ledger.append(1)
assert int("777", 8) == 511; _ledger.append(1)

# hex / oct / bin formatters produce the canonical prefixed string
assert hex(255) == "0xff"; _ledger.append(1)
assert oct(8) == "0o10"; _ledger.append(1)
assert bin(5) == "0b101"; _ledger.append(1)

# divmod returns the (quotient, remainder) 2-tuple
assert divmod(17, 5) == (3, 2); _ledger.append(1)
assert divmod(-17, 5) == (-4, 3); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_int_bit_count_to_from_bytes_ops {sum(_ledger)} asserts")
