# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_zlib_compress_crc_ops"
# subject = "cpython321.test_zlib_compress_crc_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_zlib_compress_crc_ops.py"
# status = "filled"
# ///
"""cpython321.test_zlib_compress_crc_ops: execute CPython 3.12 seed test_zlib_compress_crc_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the zlib compression + checksum
# surface. Surface: zlib.compress(b"...") returns non-empty bytes;
# zlib.decompress is the exact inverse over plain ASCII bytes,
# repeated-pattern bytes, an explicit sentence, and the empty bytes;
# explicit compression levels 1 (fastest), 9 (best), and -1 (default)
# all round-trip; zlib.crc32 produces stable canonical fingerprints
# for "" / "abc" / "hello" with both the implicit-seed and explicit-
# seed=0 forms; zlib.adler32 produces the canonical 1 / 38600999
# fingerprints for "" / "abc". Companion to test_zlib_gzip_ops
# (which covers the gzip-frame wrapper).
import zlib
_ledger: list[int] = []

# compress returns non-empty bytes
data = b"hello world"
c = zlib.compress(data)
assert isinstance(c, bytes); _ledger.append(1)
assert len(c) > 0; _ledger.append(1)

# decompress is the inverse of compress
assert zlib.decompress(c) == data; _ledger.append(1)
assert zlib.decompress(zlib.compress(b"abc")) == b"abc"; _ledger.append(1)
big = b"x" * 100
assert zlib.decompress(zlib.compress(big)) == big; _ledger.append(1)
sentence = b"the quick brown fox"
assert zlib.decompress(zlib.compress(sentence)) == sentence; _ledger.append(1)
assert zlib.decompress(zlib.compress(b"")) == b""; _ledger.append(1)
long = b"abcdef" * 100
assert zlib.decompress(zlib.compress(long)) == long; _ledger.append(1)

# Explicit compression levels — 1 (fastest), 9 (best), -1 (default)
assert zlib.decompress(zlib.compress(b"data", 1)) == b"data"; _ledger.append(1)
assert zlib.decompress(zlib.compress(b"data", 9)) == b"data"; _ledger.append(1)
assert zlib.decompress(zlib.compress(b"data", -1)) == b"data"; _ledger.append(1)

# crc32 — canonical fingerprints for known inputs
assert zlib.crc32(b"") == 0; _ledger.append(1)
assert zlib.crc32(b"abc") == 891568578; _ledger.append(1)
assert zlib.crc32(b"hello") == 907060870; _ledger.append(1)
assert zlib.crc32(b"hello", 0) == 907060870; _ledger.append(1)

# adler32 — canonical fingerprints (empty input is the identity 1)
assert zlib.adler32(b"") == 1; _ledger.append(1)
assert zlib.adler32(b"abc") == 38600999; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_zlib_compress_crc_ops {sum(_ledger)} asserts")
