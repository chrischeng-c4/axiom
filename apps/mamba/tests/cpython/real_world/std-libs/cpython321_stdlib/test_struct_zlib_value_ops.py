# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_struct_zlib_value_ops"
# subject = "cpython321.test_struct_zlib_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_struct_zlib_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_struct_zlib_value_ops: execute CPython 3.12 seed test_struct_zlib_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 298 pass conformance — struct module (hasattr pack/unpack/
# pack_into/unpack_from/calcsize/Struct/error/iter_unpack + pack big/
# little-endian int + unpack big-endian int + calcsize '>i'/'>q'
# + pack '>h' + Struct() type) + zlib module (hasattr compress/
# decompress/crc32/adler32 + crc32/adler32 known-vectors + compress/
# decompress bytes round-trip + compress returns bytes) + array
# module (hasattr array/typecodes) + io module (hasattr StringIO/
# BytesIO).
# All asserts match between CPython 3.12 and mamba.
import struct
import zlib
import array
import io


_ledger: list[int] = []

# 1) struct — hasattr core surface
assert hasattr(struct, "pack") == True; _ledger.append(1)
assert hasattr(struct, "unpack") == True; _ledger.append(1)
assert hasattr(struct, "pack_into") == True; _ledger.append(1)
assert hasattr(struct, "unpack_from") == True; _ledger.append(1)
assert hasattr(struct, "calcsize") == True; _ledger.append(1)
assert hasattr(struct, "Struct") == True; _ledger.append(1)
assert hasattr(struct, "error") == True; _ledger.append(1)
assert hasattr(struct, "iter_unpack") == True; _ledger.append(1)

# 2) struct — value contracts
assert struct.pack(">i", 1) == b"\x00\x00\x00\x01"; _ledger.append(1)
assert struct.pack("<i", 1) == b"\x01\x00\x00\x00"; _ledger.append(1)
assert struct.unpack(">i", b"\x00\x00\x00\x01") == (1,); _ledger.append(1)
assert struct.calcsize(">i") == 4; _ledger.append(1)
assert struct.calcsize(">q") == 8; _ledger.append(1)
assert struct.pack(">h", 0x1234) == b"\x124"; _ledger.append(1)
assert type(struct.Struct(">i")).__name__ == "Struct"; _ledger.append(1)

# 3) zlib — hasattr core surface (conformant subset)
assert hasattr(zlib, "compress") == True; _ledger.append(1)
assert hasattr(zlib, "decompress") == True; _ledger.append(1)
assert hasattr(zlib, "crc32") == True; _ledger.append(1)
assert hasattr(zlib, "adler32") == True; _ledger.append(1)

# 4) zlib — value contracts (CRC + Adler-32 + round-trip)
assert zlib.crc32(b"abc") == 891568578; _ledger.append(1)
assert zlib.adler32(b"abc") == 38600999; _ledger.append(1)
assert isinstance(zlib.compress(b"hello"), bytes) == True; _ledger.append(1)
assert zlib.decompress(zlib.compress(b"hi")) == b"hi"; _ledger.append(1)
assert zlib.decompress(zlib.compress(b"hello world")) == b"hello world"; _ledger.append(1)

# 5) array — hasattr core surface
assert hasattr(array, "array") == True; _ledger.append(1)
assert hasattr(array, "typecodes") == True; _ledger.append(1)

# 6) io — hasattr StringIO/BytesIO (conformant subset)
assert hasattr(io, "StringIO") == True; _ledger.append(1)
assert hasattr(io, "BytesIO") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_struct_zlib_value_ops {sum(_ledger)} asserts")
