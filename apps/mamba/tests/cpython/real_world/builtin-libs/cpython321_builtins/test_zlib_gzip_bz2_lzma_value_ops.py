# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_zlib_gzip_bz2_lzma_value_ops"
# subject = "cpython321.test_zlib_gzip_bz2_lzma_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_zlib_gzip_bz2_lzma_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_zlib_gzip_bz2_lzma_value_ops: execute CPython 3.12 seed test_zlib_gzip_bz2_lzma_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 265 pass conformance — zlib module (hasattr compress/
# decompress/crc32/adler32 + type(zlib.compress(b'hello')) is bytes,
# compress->decompress roundtrip eq for 'hello world' and 'foo bar
# baz', crc32(b'') == 0, crc32(b'hello') == 907060870, crc32(b'abc')
# == 891568578, adler32(b'') == 1, crc32 with seed=0 matches no-seed
# form, adler32 with seed=1 matches no-seed form, crc32(b'a') is
# deterministic, compress of 1000 'a' chars shrinks below 1000
# bytes, compress level 1 and 9 produce non-empty bytes) + gzip
# module (hasattr compress/decompress/open/GzipFile/BadGzipFile +
# type(gzip.compress(b'hello')) is bytes, compress->decompress
# roundtrip eq for 'hello world' and empty input, compress nonempty
# > 0 bytes, compress starts with 0x1f/0x8b gzip magic, compress
# level= and mtime= kwargs both produce non-empty output) + bz2
# module (hasattr compress/decompress/open/BZ2File/BZ2Compressor/
# BZ2Decompressor + type is bytes, compress->decompress roundtrip
# eq for 'hello world' and empty input, compress nonempty, compress
# level 1 and 9 produce non-empty bytes) + lzma module (hasattr
# compress/decompress/open/LZMAFile/LZMACompressor/LZMADecompressor/
# LZMAError + hasattr FORMAT_XZ/FORMAT_ALONE/PRESET_DEFAULT + type
# is bytes, compress->decompress roundtrip eq for 'hello world' and
# empty input, compress nonempty, compress format= kwarg produces
# non-empty bytes).
# All asserts match between CPython 3.12 and mamba.
import zlib
import gzip
import bz2
import lzma


_ledger: list[int] = []

# 1) zlib — hasattr surface (the conform subset)
assert hasattr(zlib, "compress") == True; _ledger.append(1)
assert hasattr(zlib, "decompress") == True; _ledger.append(1)
assert hasattr(zlib, "crc32") == True; _ledger.append(1)
assert hasattr(zlib, "adler32") == True; _ledger.append(1)

# 2) zlib — compress type + roundtrip value contracts
assert type(zlib.compress(b"hello")).__name__ == "bytes"; _ledger.append(1)
assert zlib.decompress(zlib.compress(b"hello world")) == b"hello world"; _ledger.append(1)
assert zlib.decompress(zlib.compress(b"foo bar baz")) == b"foo bar baz"; _ledger.append(1)

# 3) zlib — crc32 / adler32 numeric contracts
assert zlib.crc32(b"") == 0; _ledger.append(1)
assert zlib.crc32(b"hello") == 907060870; _ledger.append(1)
assert zlib.crc32(b"abc") == 891568578; _ledger.append(1)
assert zlib.adler32(b"") == 1; _ledger.append(1)
assert zlib.crc32(b"hello", 0) == zlib.crc32(b"hello"); _ledger.append(1)
assert zlib.adler32(b"hello", 1) == zlib.adler32(b"hello"); _ledger.append(1)
assert zlib.crc32(b"a") == zlib.crc32(b"a"); _ledger.append(1)

# 4) zlib — compression actually shrinks repeated input
assert (len(zlib.compress(b"a" * 1000)) < 1000) == True; _ledger.append(1)
assert (len(zlib.compress(b"abc", 1)) > 0) == True; _ledger.append(1)
assert (len(zlib.compress(b"abc", 9)) > 0) == True; _ledger.append(1)

# 5) gzip — hasattr surface
assert hasattr(gzip, "compress") == True; _ledger.append(1)
assert hasattr(gzip, "decompress") == True; _ledger.append(1)
assert hasattr(gzip, "open") == True; _ledger.append(1)
assert hasattr(gzip, "GzipFile") == True; _ledger.append(1)
assert hasattr(gzip, "BadGzipFile") == True; _ledger.append(1)

# 6) gzip — type + roundtrip + magic prefix
assert type(gzip.compress(b"hello")).__name__ == "bytes"; _ledger.append(1)
assert gzip.decompress(gzip.compress(b"hello world")) == b"hello world"; _ledger.append(1)
assert gzip.decompress(gzip.compress(b"")) == b""; _ledger.append(1)
assert (len(gzip.compress(b"hello")) > 0) == True; _ledger.append(1)
assert gzip.compress(b"hello")[:2] == b"\x1f\x8b"; _ledger.append(1)
assert (len(gzip.compress(b"hello world", 6)) > 0) == True; _ledger.append(1)
assert (len(gzip.compress(b"hello", mtime=0)) > 0) == True; _ledger.append(1)

# 7) bz2 — hasattr surface
assert hasattr(bz2, "compress") == True; _ledger.append(1)
assert hasattr(bz2, "decompress") == True; _ledger.append(1)
assert hasattr(bz2, "open") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2File") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Compressor") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Decompressor") == True; _ledger.append(1)

# 8) bz2 — type + roundtrip + level
assert type(bz2.compress(b"hello")).__name__ == "bytes"; _ledger.append(1)
assert bz2.decompress(bz2.compress(b"hello world")) == b"hello world"; _ledger.append(1)
assert bz2.decompress(bz2.compress(b"")) == b""; _ledger.append(1)
assert (len(bz2.compress(b"hello")) > 0) == True; _ledger.append(1)
assert (len(bz2.compress(b"abc", 1)) > 0) == True; _ledger.append(1)
assert (len(bz2.compress(b"abc", 9)) > 0) == True; _ledger.append(1)

# 9) lzma — hasattr surface
assert hasattr(lzma, "compress") == True; _ledger.append(1)
assert hasattr(lzma, "decompress") == True; _ledger.append(1)
assert hasattr(lzma, "open") == True; _ledger.append(1)
assert hasattr(lzma, "LZMAFile") == True; _ledger.append(1)
assert hasattr(lzma, "LZMACompressor") == True; _ledger.append(1)
assert hasattr(lzma, "LZMADecompressor") == True; _ledger.append(1)
assert hasattr(lzma, "LZMAError") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_XZ") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_ALONE") == True; _ledger.append(1)
assert hasattr(lzma, "PRESET_DEFAULT") == True; _ledger.append(1)

# 10) lzma — type + roundtrip + format kwarg
assert type(lzma.compress(b"hello")).__name__ == "bytes"; _ledger.append(1)
assert lzma.decompress(lzma.compress(b"hello world")) == b"hello world"; _ledger.append(1)
assert lzma.decompress(lzma.compress(b"")) == b""; _ledger.append(1)
assert (len(lzma.compress(b"hello")) > 0) == True; _ledger.append(1)
assert (len(lzma.compress(b"hello", format=lzma.FORMAT_XZ)) > 0) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_zlib_gzip_bz2_lzma_value_ops {sum(_ledger)} asserts")
