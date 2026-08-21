# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_zlib_gzip_zipfile_bz2_lzma_html_xmldom_colorsys_value_ops"
# subject = "cpython321.test_zlib_gzip_zipfile_bz2_lzma_html_xmldom_colorsys_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_zlib_gzip_zipfile_bz2_lzma_html_xmldom_colorsys_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_zlib_gzip_zipfile_bz2_lzma_html_xmldom_colorsys_value_ops: execute CPython 3.12 seed test_zlib_gzip_zipfile_bz2_lzma_html_xmldom_colorsys_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 243 pass conformance — zlib / gzip / zipfile partial / tarfile
# partial / bz2 / lzma / html / xml.dom.minidom / wave / colorsys / quopri
# surface + roundtrip value ops that match between CPython 3.12 and mamba.
import zlib
import gzip
import zipfile
import tarfile
import bz2
import lzma
import html
import xml.dom.minidom as xmd
import wave
import colorsys
import quopri


_ledger: list[int] = []

# 1) zlib partial surface + value ops
assert hasattr(zlib, "compress") == True; _ledger.append(1)
assert hasattr(zlib, "decompress") == True; _ledger.append(1)
assert hasattr(zlib, "crc32") == True; _ledger.append(1)
assert hasattr(zlib, "adler32") == True; _ledger.append(1)
assert zlib.crc32(b"hello") == 907060870; _ledger.append(1)
assert zlib.adler32(b"hello") == 103547413; _ledger.append(1)
assert zlib.decompress(zlib.compress(b"hello world")) == b"hello world"; _ledger.append(1)

# 2) gzip partial surface + roundtrip
assert hasattr(gzip, "compress") == True; _ledger.append(1)
assert hasattr(gzip, "decompress") == True; _ledger.append(1)
assert hasattr(gzip, "open") == True; _ledger.append(1)
assert hasattr(gzip, "GzipFile") == True; _ledger.append(1)
assert hasattr(gzip, "BadGzipFile") == True; _ledger.append(1)
assert gzip.decompress(gzip.compress(b"hello world")) == b"hello world"; _ledger.append(1)

# 3) zipfile partial surface
assert hasattr(zipfile, "ZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "is_zipfile") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_STORED") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_DEFLATED") == True; _ledger.append(1)

# 4) tarfile partial surface
assert hasattr(tarfile, "open") == True; _ledger.append(1)
assert hasattr(tarfile, "is_tarfile") == True; _ledger.append(1)

# 5) bz2 full surface + roundtrip
assert hasattr(bz2, "compress") == True; _ledger.append(1)
assert hasattr(bz2, "decompress") == True; _ledger.append(1)
assert hasattr(bz2, "open") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2File") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Compressor") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Decompressor") == True; _ledger.append(1)
assert bz2.decompress(bz2.compress(b"hello world")) == b"hello world"; _ledger.append(1)

# 6) lzma full surface + roundtrip
assert hasattr(lzma, "compress") == True; _ledger.append(1)
assert hasattr(lzma, "decompress") == True; _ledger.append(1)
assert hasattr(lzma, "open") == True; _ledger.append(1)
assert hasattr(lzma, "LZMAFile") == True; _ledger.append(1)
assert hasattr(lzma, "LZMACompressor") == True; _ledger.append(1)
assert hasattr(lzma, "LZMADecompressor") == True; _ledger.append(1)
assert hasattr(lzma, "LZMAError") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_XZ") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_RAW") == True; _ledger.append(1)
assert lzma.decompress(lzma.compress(b"hello world")) == b"hello world"; _ledger.append(1)

# 7) html surface + value ops
assert hasattr(html, "escape") == True; _ledger.append(1)
assert hasattr(html, "unescape") == True; _ledger.append(1)
assert html.escape("<a href=\"x\">&") == "&lt;a href=&quot;x&quot;&gt;&amp;"; _ledger.append(1)
assert html.unescape("&lt;a&gt;") == "<a>"; _ledger.append(1)

# 8) xml.dom.minidom partial surface (aliased import dodges the dotted-access quirk)
assert hasattr(xmd, "Document") == True; _ledger.append(1)
assert hasattr(xmd, "Element") == True; _ledger.append(1)
assert hasattr(xmd, "parseString") == True; _ledger.append(1)

# 9) wave surface
assert hasattr(wave, "open") == True; _ledger.append(1)
assert hasattr(wave, "Error") == True; _ledger.append(1)

# 10) colorsys value ops
assert colorsys.rgb_to_hls(1.0, 0.0, 0.0) == (0.0, 0.5, 1.0); _ledger.append(1)
assert colorsys.hls_to_rgb(0.0, 0.5, 1.0) == (1.0, 0.0, 0.0); _ledger.append(1)
assert colorsys.rgb_to_hsv(1.0, 0.0, 0.0) == (0.0, 1.0, 1.0); _ledger.append(1)
assert colorsys.hsv_to_rgb(0.0, 1.0, 1.0) == (1.0, 0.0, 0.0); _ledger.append(1)

# 11) quopri value ops on alphanumeric input
assert quopri.encodestring(b"hello") == b"hello"; _ledger.append(1)
assert quopri.decodestring(b"hello") == b"hello"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_zlib_gzip_zipfile_bz2_lzma_html_xmldom_colorsys_value_ops {sum(_ledger)} asserts")
