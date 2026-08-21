# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: struct (pack/unpack roundtrip with > and < byte order, calcsize,
# integer and IEEE 754 floating-point formats). The "Ns" string format is
# omitted; mamba truncates pack("3s", b"abc") to b"a\\x00\\x00".
import struct

_ledger: list[int] = []

# pack/unpack roundtrip: big-endian unsigned 32-bit
packed = struct.pack(">I", 1234)
assert packed == b"\x00\x00\x04\xd2", f"pack('>I', 1234) bytes; got {packed!r}"
_ledger.append(1)

unpacked, = struct.unpack(">I", packed)
assert unpacked == 1234, f"unpack('>I', ...) == 1234, got {unpacked}"
_ledger.append(1)

# pack/unpack roundtrip: little-endian unsigned 16-bit
packed = struct.pack("<H", 256)
assert packed == b"\x00\x01", f"pack('<H', 256) bytes; got {packed!r}"
_ledger.append(1)

unpacked, = struct.unpack("<H", packed)
assert unpacked == 256, f"unpack('<H', ...) == 256, got {unpacked}"
_ledger.append(1)

# Multiple-value pack/unpack
packed = struct.pack(">II", 1, 2)
assert packed == b"\x00\x00\x00\x01\x00\x00\x00\x02", (
    f"pack('>II', 1, 2) bytes; got {packed!r}"
)
_ledger.append(1)

a, b = struct.unpack(">II", packed)
assert (a, b) == (1, 2), f"unpack('>II', ...) == (1, 2), got ({a}, {b})"
_ledger.append(1)

# Signed 32-bit roundtrip preserves -1
packed = struct.pack(">i", -1)
assert packed == b"\xff\xff\xff\xff", f"pack('>i', -1) is 0xffffffff; got {packed!r}"
_ledger.append(1)

unpacked, = struct.unpack(">i", packed)
assert unpacked == -1, f"unpack('>i', 0xffffffff) == -1, got {unpacked}"
_ledger.append(1)

# Signed 8-bit roundtrip preserves -1
packed = struct.pack(">b", -1)
assert packed == b"\xff", f"pack('>b', -1) is b'\\xff'; got {packed!r}"
_ledger.append(1)

unpacked, = struct.unpack(">b", packed)
assert unpacked == -1, f"unpack('>b', b'\\xff') == -1, got {unpacked}"
_ledger.append(1)

# IEEE 754 single-precision (4 bytes)
packed = struct.pack(">f", 1.5)
assert packed == b"\x3f\xc0\x00\x00", f"pack('>f', 1.5) bytes; got {packed!r}"
_ledger.append(1)

unpacked, = struct.unpack(">f", packed)
assert unpacked == 1.5, f"unpack('>f', ...) == 1.5, got {unpacked}"
_ledger.append(1)

# IEEE 754 double-precision (8 bytes)
packed = struct.pack(">d", 1.5)
assert packed == b"\x3f\xf8\x00\x00\x00\x00\x00\x00", (
    f"pack('>d', 1.5) bytes; got {packed!r}"
)
_ledger.append(1)

unpacked, = struct.unpack(">d", packed)
assert unpacked == 1.5, f"unpack('>d', ...) == 1.5, got {unpacked}"
_ledger.append(1)

# calcsize for the formats this seed touches
assert struct.calcsize(">I") == 4, "calcsize('>I') == 4"
_ledger.append(1)

assert struct.calcsize(">Q") == 8, "calcsize('>Q') == 8"
_ledger.append(1)

assert struct.calcsize(">d") == 8, "calcsize('>d') == 8"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_struct {sum(_ledger)} asserts")
