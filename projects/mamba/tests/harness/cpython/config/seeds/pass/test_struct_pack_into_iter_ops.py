# Operational AssertionPass seed for `struct` buffer-oriented APIs
# not covered by existing struct seeds. The seeds
# `test_struct_ops`, `test_struct_pack_unpack_ops`,
# `test_struct_unsigned_floats_ops`, `test_struct_calcsize_pad_float_ops`
# cover `pack`/`unpack`/`calcsize` only. This seed asserts:
#   * `pack_into(fmt, buffer, offset, *values)` — writes packed bytes
#     into a pre-allocated mutable bytearray at the given offset.
#   * `unpack_from(fmt, buffer, offset)` — reads them back.
#   * `iter_unpack(fmt, data)` — yields a tuple per format-chunk
#     contained in the data.
import struct
_ledger: list[int] = []

# pack_into / unpack_from — write into and read from a bytearray
buf = bytearray(8)
struct.pack_into(">I", buf, 0, 0xDEADBEEF)
assert bytes(buf[:4]) == b"\xde\xad\xbe\xef"; _ledger.append(1)

val, = struct.unpack_from(">I", buf, 0)
assert val == 0xDEADBEEF; _ledger.append(1)

# pack_into respects offset
buf2 = bytearray(8)
struct.pack_into(">I", buf2, 4, 0xCAFEBABE)
assert bytes(buf2[:4]) == b"\x00\x00\x00\x00"; _ledger.append(1)
assert bytes(buf2[4:8]) == b"\xca\xfe\xba\xbe"; _ledger.append(1)

val2, = struct.unpack_from(">I", buf2, 4)
assert val2 == 0xCAFEBABE; _ledger.append(1)

# pack_into with multiple values
buf3 = bytearray(8)
struct.pack_into(">HH", buf3, 0, 0x1234, 0x5678)
assert bytes(buf3[:2]) == b"\x12\x34"; _ledger.append(1)
assert bytes(buf3[2:4]) == b"\x56\x78"; _ledger.append(1)

# unpack_from with multiple values
a, b = struct.unpack_from(">HH", buf3, 0)
assert a == 0x1234; _ledger.append(1)
assert b == 0x5678; _ledger.append(1)

# iter_unpack — yield one tuple per format-chunk
data = struct.pack(">III", 1, 2, 3)
result = list(struct.iter_unpack(">I", data))
assert result == [(1,), (2,), (3,)]; _ledger.append(1)
assert len(result) == 3; _ledger.append(1)
assert result[0] == (1,); _ledger.append(1)
assert result[1] == (2,); _ledger.append(1)
assert result[2] == (3,); _ledger.append(1)

# iter_unpack on multi-field format
data2 = struct.pack(">HH HH HH", 10, 20, 30, 40, 50, 60)
result2 = list(struct.iter_unpack(">HH", data2))
assert result2 == [(10, 20), (30, 40), (50, 60)]; _ledger.append(1)

# pack_into / unpack_from round-trip for signed int
buf4 = bytearray(4)
struct.pack_into(">i", buf4, 0, -42)
val4, = struct.unpack_from(">i", buf4, 0)
assert val4 == -42; _ledger.append(1)

# pack_into preserves zero-fill in surrounding bytes
buf5 = bytearray(10)
struct.pack_into(">H", buf5, 2, 0xABCD)
assert bytes(buf5[0:2]) == b"\x00\x00"; _ledger.append(1)
assert bytes(buf5[2:4]) == b"\xab\xcd"; _ledger.append(1)
assert bytes(buf5[4:10]) == b"\x00\x00\x00\x00\x00\x00"; _ledger.append(1)

# iter_unpack count matches data-length / format-size
data3 = struct.pack(">5H", 1, 2, 3, 4, 5)
result3 = list(struct.iter_unpack(">H", data3))
assert len(result3) == 5; _ledger.append(1)
assert [t[0] for t in result3] == [1, 2, 3, 4, 5]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_struct_pack_into_iter_ops {sum(_ledger)} asserts")
