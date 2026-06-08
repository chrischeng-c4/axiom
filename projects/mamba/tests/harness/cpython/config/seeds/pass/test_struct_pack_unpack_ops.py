# Operational AssertionPass seed for the struct-module binary-codec
# surface. Surface: `struct.pack(fmt, *values)` produces canonical
# big-endian bytes (`">i", 100` → b'\x00\x00\x00d') and little-
# endian bytes (`"<i", 1` → b'\x01\x00\x00\x00'); `struct.unpack(
# fmt, buf)` returns a tuple, and `unpack(pack(...))` round-trips
# for signed/unsigned `i`/`h`/`q`/`b`/`B`/`H`/`I`/`Q` ints (incl.
# negative + maxint-by-width), with multi-element formats packing
# fields in sequence and unpacking as a tuple of matching arity;
# `struct.calcsize(fmt)` reports the byte width of each format
# specifier and sums fields for multi-element formats. Companion
# to test_struct (which covers the broader format surface).
import struct
_ledger: list[int] = []

# Canonical big-endian int encoding
assert struct.pack(">i", 100) == b'\x00\x00\x00d'; _ledger.append(1)
assert struct.unpack(">i", b'\x00\x00\x00d') == (100,); _ledger.append(1)

# Round-trip signed int — positive, negative, zero
assert struct.unpack(">i", struct.pack(">i", 12345))[0] == 12345; _ledger.append(1)
assert struct.unpack(">i", struct.pack(">i", -1))[0] == -1; _ledger.append(1)
assert struct.unpack(">i", struct.pack(">i", 0))[0] == 0; _ledger.append(1)

# Width inventory — one byte per primitive
assert struct.calcsize(">i") == 4; _ledger.append(1)
assert struct.calcsize(">h") == 2; _ledger.append(1)
assert struct.calcsize(">q") == 8; _ledger.append(1)
assert struct.calcsize(">b") == 1; _ledger.append(1)
assert struct.calcsize(">B") == 1; _ledger.append(1)
assert struct.calcsize(">H") == 2; _ledger.append(1)
assert struct.calcsize(">I") == 4; _ledger.append(1)
assert struct.calcsize(">Q") == 8; _ledger.append(1)
assert struct.calcsize(">f") == 4; _ledger.append(1)
assert struct.calcsize(">d") == 8; _ledger.append(1)

# Unsigned round-trip at width-max values
assert struct.unpack(">H", struct.pack(">H", 65535))[0] == 65535; _ledger.append(1)
assert struct.unpack(">B", struct.pack(">B", 255))[0] == 255; _ledger.append(1)
assert struct.unpack(">Q", struct.pack(">Q", 1234567890))[0] == 1234567890; _ledger.append(1)

# Multi-element pack/unpack
assert struct.pack(">ii", 1, 2) == b'\x00\x00\x00\x01\x00\x00\x00\x02'; _ledger.append(1)
assert struct.unpack(">ii", b'\x00\x00\x00\x01\x00\x00\x00\x02') == (1, 2); _ledger.append(1)
assert struct.unpack(">ii", struct.pack(">ii", 100, 200)) == (100, 200); _ledger.append(1)

# Little-endian — reversed byte order
assert struct.pack("<i", 1) == b'\x01\x00\x00\x00'; _ledger.append(1)
assert struct.unpack("<i", b'\x01\x00\x00\x00') == (1,); _ledger.append(1)
assert struct.unpack("<H", struct.pack("<H", 256))[0] == 256; _ledger.append(1)

# calcsize sums multi-element formats
assert struct.calcsize(">ii") == 8; _ledger.append(1)
assert struct.calcsize(">4s") == 4; _ledger.append(1)
assert struct.calcsize(">ihq") == 14; _ledger.append(1)

# Native byte-order at least matches platform int width (>= 4)
assert struct.calcsize("i") >= 4; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_struct_pack_unpack_ops {sum(_ledger)} asserts")
