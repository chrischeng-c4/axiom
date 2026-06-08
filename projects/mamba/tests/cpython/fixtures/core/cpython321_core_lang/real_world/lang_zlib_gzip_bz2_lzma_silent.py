# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_zlib_gzip_bz2_lzma_silent"
# subject = "cpython321.lang_zlib_gzip_bz2_lzma_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_zlib_gzip_bz2_lzma_silent.py"
# status = "filled"
# ///
"""cpython321.lang_zlib_gzip_bz2_lzma_silent: execute CPython 3.12 seed lang_zlib_gzip_bz2_lzma_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `zlib.Z_BEST_COMPRESSION` (the
# documented "zlib exposes Z_BEST_COMPRESSION == 9" — mamba returns
# None), `zlib.MAX_WBITS` (the documented "zlib exposes MAX_WBITS
# == 15 — the maximum window-size base-2 exponent" — mamba returns
# None), `zlib.DEFLATED` (the documented "zlib exposes DEFLATED ==
# 8 — the only allowed compression method" — mamba returns None),
# `zlib.decompress(b'not zlib data')` (the documented "decompress
# raises zlib.error on malformed input" — mamba returns b'' silently
# without raising), `gzip.GzipFile.__name__` (the documented "gzip
# exposes the GzipFile class — type name 'GzipFile'" — mamba
# returns None because GzipFile is not a class), `type(gzip
# .BadGzipFile).__name__` (the documented "BadGzipFile is an
# exception class — its type is 'type'" — mamba returns 'str'),
# `bz2.BZ2File.__name__` (the documented "bz2 exposes the BZ2File
# class — type name 'BZ2File'" — mamba returns None), `bz2
# .decompress(b'not bz2')` (the documented "decompress raises
# OSError on malformed input" — mamba returns b'' silently without
# raising), `lzma.LZMAError.__name__` (the documented "lzma exposes
# the LZMAError exception class — type name 'LZMAError'" — mamba
# returns None), and `lzma.decompress(b'not lzma')` (the documented
# "decompress raises LZMAError on malformed input" — mamba returns
# b'' silently without raising).
# Ten-pack pinned to atomic 265.
#
# Behavioral edges that CONFORM on mamba (zlib — hasattr compress/
# decompress/crc32/adler32 + compress->decompress roundtrip eq +
# bytes type + crc32(b'')==0 / crc32(b'hello')==907060870 / crc32
# (b'abc')==891568578 / adler32(b'')==1 + crc32 seed=0 / adler32
# seed=1 match no-seed + crc32 deterministic + compress shrinks
# repeated input + level 1/9 produce non-empty bytes. gzip —
# hasattr compress/decompress/open/GzipFile/BadGzipFile + roundtrip
# eq + bytes type + gzip magic 0x1f/0x8b + level= / mtime= kwargs.
# bz2 — hasattr compress/decompress/open/BZ2File/BZ2Compressor/
# BZ2Decompressor + roundtrip eq + bytes type + level 1/9. lzma —
# hasattr compress/decompress/open/LZMAFile/LZMACompressor/LZMA
# Decompressor/LZMAError + hasattr FORMAT_XZ/FORMAT_ALONE/PRESET_
# DEFAULT + roundtrip eq + bytes type + format= kwarg) are covered
# in the matching pass fixture `test_zlib_gzip_bz2_lzma_value_ops`.
import zlib
import gzip
import bz2
import lzma
from typing import Any


_ledger: list[int] = []

# 1) zlib.Z_BEST_COMPRESSION == 9
#    (mamba: returns None — constant not exposed as int value)
assert zlib.Z_BEST_COMPRESSION == 9; _ledger.append(1)

# 2) zlib.MAX_WBITS == 15
#    (mamba: returns None)
assert zlib.MAX_WBITS == 15; _ledger.append(1)

# 3) zlib.DEFLATED == 8
#    (mamba: returns None)
assert zlib.DEFLATED == 8; _ledger.append(1)

# 4) zlib.decompress(b'not zlib data') raises zlib.error
#    (mamba: returns b'' silently — no error raised)
def _zlib_decompress_invalid() -> Any:
    try:
        return zlib.decompress(b"not zlib data")
    except Exception:
        return "raised"
assert _zlib_decompress_invalid() == "raised"; _ledger.append(1)

# 5) gzip.GzipFile.__name__ == 'GzipFile'
#    (mamba: returns None — GzipFile is not a class)
assert gzip.GzipFile.__name__ == "GzipFile"; _ledger.append(1)

# 6) type(gzip.BadGzipFile).__name__ == 'type'
#    (mamba: returns 'str' — BadGzipFile is a string name not a class)
assert type(gzip.BadGzipFile).__name__ == "type"; _ledger.append(1)

# 7) bz2.BZ2File.__name__ == 'BZ2File'
#    (mamba: returns None — BZ2File is not a class)
assert bz2.BZ2File.__name__ == "BZ2File"; _ledger.append(1)

# 8) bz2.decompress(b'not bz2') raises OSError
#    (mamba: returns b'' silently)
def _bz2_decompress_invalid() -> Any:
    try:
        return bz2.decompress(b"not bz2")
    except Exception:
        return "raised"
assert _bz2_decompress_invalid() == "raised"; _ledger.append(1)

# 9) lzma.LZMAError.__name__ == 'LZMAError'
#    (mamba: returns None — LZMAError is not a class)
assert lzma.LZMAError.__name__ == "LZMAError"; _ledger.append(1)

# 10) lzma.decompress(b'not lzma') raises LZMAError
#     (mamba: returns b'' silently)
def _lzma_decompress_invalid() -> Any:
    try:
        return lzma.decompress(b"not lzma")
    except Exception:
        return "raised"
assert _lzma_decompress_invalid() == "raised"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_zlib_gzip_bz2_lzma_silent {sum(_ledger)} asserts")
