# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_zlib_gzip_ops"
# subject = "cpython321.test_zlib_gzip_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_zlib_gzip_ops.py"
# status = "filled"
# ///
"""cpython321.test_zlib_gzip_ops: execute CPython 3.12 seed test_zlib_gzip_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `zlib` + `gzip` compression
# round-trip and checksum surface.
# Surface: zlib.compress / zlib.decompress round-trip; zlib.crc32
# and zlib.adler32 canonical-vector checksums for b"abc"; gzip.compress
# / gzip.decompress round-trip. Compressed output shorter than input
# on repetitive payloads.
# Companion to stub/test_zlib.py + stub/test_gzip.py — vendored
# unittest seeds.
import zlib
import gzip
_ledger: list[int] = []
data = b"hello world hello world hello world"
# zlib round-trip identity
c = zlib.compress(data)
assert zlib.decompress(c) == data; _ledger.append(1)
# Compression makes repetitive payloads strictly smaller
assert len(c) < len(data); _ledger.append(1)
# Canonical-vector checksums for b"abc"
assert zlib.crc32(b"abc") == 891568578; _ledger.append(1)
assert zlib.adler32(b"abc") == 38600999; _ledger.append(1)
# Empty input has well-defined checksums (CRC-32 of empty = 0;
# Adler-32 of empty = 1)
assert zlib.crc32(b"") == 0; _ledger.append(1)
assert zlib.adler32(b"") == 1; _ledger.append(1)
# gzip round-trip identity
g = gzip.compress(data)
assert len(g) > 0; _ledger.append(1)
assert gzip.decompress(g) == data; _ledger.append(1)
# gzip round-trip on empty payload
assert gzip.decompress(gzip.compress(b"")) == b""; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_zlib_gzip_ops {sum(_ledger)} asserts")
