# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_zlib_gzip_bz2_lzma_compress_ops"
# subject = "cpython321.test_zlib_gzip_bz2_lzma_compress_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_zlib_gzip_bz2_lzma_compress_ops.py"
# status = "filled"
# ///
"""cpython321.test_zlib_gzip_bz2_lzma_compress_ops: execute CPython 3.12 seed test_zlib_gzip_bz2_lzma_compress_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# four bundled compression / decompression codecs used by every
# storage and network transport layer: `zlib` (the deflate
# checksum + compress primitive), `gzip` (the GNU gzip
# stream codec), `bz2` (the bzip2 block codec), and `lzma`
# (the XZ / LZMA stream codec). No fixture coverage yet for any
# of the four at this level of detail.
#
# The matching subset between mamba and CPython is the byte-exact
# transform layer: zlib.compress / decompress and zlib.crc32 /
# adler32 return the documented bytes / integers; gzip.compress /
# decompress / bz2.compress / decompress / lzma.compress /
# decompress all round-trip arbitrary bytes; the documented
# lzma.FORMAT_XZ / FORMAT_ALONE / CHECK_CRC32 / CHECK_CRC64
# integer sentinels have the documented values; the documented
# http.HTTPStatus integer values are byte-exact for the
# commonly-relied-upon HTTP status codes (200 / 201 / 204 / 400
# / 401 / 403 / 404 / 500); configparser.ConfigParser.__name__ ==
# "ConfigParser" class-name identity.
#
# Surface in this fixture:
#   • zlib.compress(b"hello world") round-trips through
#     zlib.decompress(...) == b"hello world";
#   • zlib.crc32(b"hi") == 3633523372 (documented CRC-32 value);
#   • zlib.adler32(b"hi") == 20644050 (documented Adler-32 value);
#   • type(zlib.compress(b"")).__name__ == "bytes";
#   • gzip.compress / decompress round-trip arbitrary bytes;
#   • bz2.compress / decompress round-trip arbitrary bytes;
#   • lzma.compress / decompress round-trip arbitrary bytes;
#   • lzma.FORMAT_XZ == 1, FORMAT_ALONE == 2 (documented format
#     sentinels);
#   • lzma.CHECK_CRC32 == 1, CHECK_CRC64 == 4 (documented integrity
#     check sentinels);
#   • int(http.HTTPStatus.OK) == 200 / .CREATED == 201 / .NO_CONTENT
#     == 204 / .BAD_REQUEST == 400 / .UNAUTHORIZED == 401 /
#     .FORBIDDEN == 403 / .NOT_FOUND == 404 /
#     .INTERNAL_SERVER_ERROR == 500 (documented HTTP status codes);
#   • hasattr(configparser, "ConfigParser") is True — the class
#     binding is exposed on the module.
#
# Behavioral edges that DIVERGE on mamba (zlib.Z_DEFAULT_COMPRESSION
# / Z_BEST_COMPRESSION / Z_BEST_SPEED / Z_NO_COMPRESSION /
# MAX_WBITS integer sentinels, gzip.GzipFile / READ / WRITE /
# FNAME class identity + sentinels, bz2.BZ2File / BZ2Compressor
# class identity, lzma.LZMAFile class identity, http.HTTPStatus
# .value / .name / .phrase per-member attributes, type(http.
# HTTPStatus) == EnumType, configparser.ConfigParser.__name__
# class identity + instance lifecycle, configparser.DEFAULTSECT
# sentinel) are covered in
# `lang_http_status_configparser_zlib_const_silent`.
import zlib
import gzip
import bz2
import lzma
import http
import configparser

_ledger: list[int] = []

# 1) zlib.compress / decompress — byte round-trip
_zc = zlib.compress(b"hello world")
assert isinstance(_zc, bytes); _ledger.append(1)
assert len(_zc) > 0; _ledger.append(1)
assert zlib.decompress(_zc) == b"hello world"; _ledger.append(1)
_zc2 = zlib.compress(b"alpha beta gamma")
assert zlib.decompress(_zc2) == b"alpha beta gamma"; _ledger.append(1)

# 2) zlib.crc32 / adler32 — documented numeric values
assert zlib.crc32(b"hi") == 3633523372; _ledger.append(1)
assert zlib.adler32(b"hi") == 20644050; _ledger.append(1)
assert isinstance(zlib.crc32(b"hi"), int); _ledger.append(1)
assert isinstance(zlib.adler32(b"hi"), int); _ledger.append(1)

# 3) gzip.compress / decompress — byte round-trip
_gc = gzip.compress(b"hello world")
assert isinstance(_gc, bytes); _ledger.append(1)
assert len(_gc) > 0; _ledger.append(1)
assert gzip.decompress(_gc) == b"hello world"; _ledger.append(1)
assert gzip.decompress(gzip.compress(b"alpha")) == b"alpha"; _ledger.append(1)

# 4) bz2.compress / decompress — byte round-trip
_bc = bz2.compress(b"hello world")
assert isinstance(_bc, bytes); _ledger.append(1)
assert len(_bc) > 0; _ledger.append(1)
assert bz2.decompress(_bc) == b"hello world"; _ledger.append(1)
assert bz2.decompress(bz2.compress(b"alpha")) == b"alpha"; _ledger.append(1)

# 5) lzma.compress / decompress — byte round-trip
_lc = lzma.compress(b"hello world")
assert isinstance(_lc, bytes); _ledger.append(1)
assert len(_lc) > 0; _ledger.append(1)
assert lzma.decompress(_lc) == b"hello world"; _ledger.append(1)
assert lzma.decompress(lzma.compress(b"alpha")) == b"alpha"; _ledger.append(1)

# 6) lzma.FORMAT_* / CHECK_* — documented integer sentinels
assert lzma.FORMAT_XZ == 1; _ledger.append(1)
assert lzma.FORMAT_ALONE == 2; _ledger.append(1)
assert lzma.CHECK_CRC32 == 1; _ledger.append(1)
assert lzma.CHECK_CRC64 == 4; _ledger.append(1)

# 7) http.HTTPStatus — documented integer values
assert int(http.HTTPStatus.OK) == 200; _ledger.append(1)
assert int(http.HTTPStatus.CREATED) == 201; _ledger.append(1)
assert int(http.HTTPStatus.NO_CONTENT) == 204; _ledger.append(1)
assert int(http.HTTPStatus.BAD_REQUEST) == 400; _ledger.append(1)
assert int(http.HTTPStatus.UNAUTHORIZED) == 401; _ledger.append(1)
assert int(http.HTTPStatus.FORBIDDEN) == 403; _ledger.append(1)
assert int(http.HTTPStatus.NOT_FOUND) == 404; _ledger.append(1)
assert int(http.HTTPStatus.INTERNAL_SERVER_ERROR) == 500; _ledger.append(1)

# 8) hasattr surface — module-level helpers
assert hasattr(zlib, "compress"); _ledger.append(1)
assert hasattr(zlib, "decompress"); _ledger.append(1)
assert hasattr(zlib, "crc32"); _ledger.append(1)
assert hasattr(zlib, "adler32"); _ledger.append(1)
assert hasattr(gzip, "compress"); _ledger.append(1)
assert hasattr(gzip, "decompress"); _ledger.append(1)
assert hasattr(bz2, "compress"); _ledger.append(1)
assert hasattr(bz2, "decompress"); _ledger.append(1)
assert hasattr(lzma, "compress"); _ledger.append(1)
assert hasattr(lzma, "decompress"); _ledger.append(1)
assert hasattr(http, "HTTPStatus"); _ledger.append(1)
assert hasattr(configparser, "ConfigParser"); _ledger.append(1)

# NB: zlib.Z_DEFAULT_COMPRESSION / Z_BEST_COMPRESSION / Z_BEST_SPEED
# / Z_NO_COMPRESSION / MAX_WBITS, gzip.GzipFile / READ / WRITE /
# FNAME, bz2.BZ2File / BZ2Compressor, lzma.LZMAFile, http.HTTPStatus
# per-member .value / .name / .phrase, type(http.HTTPStatus) ==
# EnumType, configparser.ConfigParser.__name__ class identity +
# instance lifecycle, configparser.DEFAULTSECT all DIVERGE on
# mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_zlib_gzip_bz2_lzma_compress_ops {sum(_ledger)} asserts")
