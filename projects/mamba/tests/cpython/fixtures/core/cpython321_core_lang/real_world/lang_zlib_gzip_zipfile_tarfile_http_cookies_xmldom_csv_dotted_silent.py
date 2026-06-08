# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_zlib_gzip_zipfile_tarfile_http_cookies_xmldom_csv_dotted_silent"
# subject = "cpython321.lang_zlib_gzip_zipfile_tarfile_http_cookies_xmldom_csv_dotted_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_zlib_gzip_zipfile_tarfile_http_cookies_xmldom_csv_dotted_silent.py"
# status = "filled"
# ///
"""cpython321.lang_zlib_gzip_zipfile_tarfile_http_cookies_xmldom_csv_dotted_silent: execute CPython 3.12 seed lang_zlib_gzip_zipfile_tarfile_http_cookies_xmldom_csv_dotted_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `zlib` deep surface / `gzip` flag constants / `zipfile` deep
# surface / `tarfile` deep surface / `csv.DictReader /
# DictWriter` instance ops / `http.cookies` / `xml.dom`
# top-level surface / `xml.dom.minidom` dotted-access value
# contract seven-pack pinned to atomic 243:
# `zlib.compressobj / decompressobj / Z_BEST_SPEED /
# Z_BEST_COMPRESSION / Z_DEFAULT_COMPRESSION / DEFLATED /
# DEF_BUF_SIZE / error` (the documented deep surface — mamba's
# `zlib` module dict only exposes `compress / decompress /
# crc32 / adler32`), `gzip.READ / gzip.WRITE` (the documented
# mode flag constants — mamba does not expose them),
# `zipfile.ZipInfo / BadZipFile / BadZipfile / LargeZipFile /
# ZIP_BZIP2 / ZIP_LZMA` (the documented deep surface — mamba's
# `zipfile` module dict only exposes `ZipFile / is_zipfile /
# ZIP_STORED / ZIP_DEFLATED`), `tarfile.TarFile / TarInfo /
# TarError / ReadError / CompressionError` (the documented
# deep surface — mamba's `tarfile` module dict only exposes
# `open / is_tarfile`), `csv.DictReader(io.StringIO("a,b\n1,2\n3,4\n"))`
# (the documented "iterate the reader to yield ordered dicts
# keyed by the header row" value contract — mamba's
# DictReader instance silently yields the empty iterator) and
# `csv.DictWriter(...).writeheader()` (the documented "write
# the fieldnames as the first row" value contract — mamba's
# DictWriter instance is a bare string and raises
# AttributeError at the call site), `http.cookies.SimpleCookie
# / BaseCookie / Morsel / CookieError` (the documented
# top-level cookie surface — mamba does not expose any of
# them), `xml.dom.Node / DOMException / getDOMImplementation`
# (the documented top-level DOM surface — mamba's `xml.dom`
# module dict does not expose any of them), and
# `xml.dom.minidom.Document / Element / parseString` via the
# dotted `xml.dom.minidom.X` access path (the documented
# canonical dotted-attribute access pattern — mamba silently
# returns False for every dotted access through
# `xml.dom.minidom.<name>`, the same quirk that affects
# `urllib.parse.<name>` access).
#
# Behavioral edges that CONFORM on mamba (zlib compress/
# decompress/crc32/adler32 hasattr + crc32/adler32 + zlib
# roundtrip; gzip compress/decompress/open/GzipFile/
# BadGzipFile hasattr + roundtrip; zipfile ZipFile/is_zipfile/
# ZIP_STORED/ZIP_DEFLATED hasattr; tarfile open/is_tarfile
# hasattr; bz2 6-name hasattr + roundtrip; lzma 9-name hasattr
# + roundtrip; html escape/unescape hasattr + value ops;
# xml.dom.minidom Document/Element/parseString via aliased
# import; wave open/Error hasattr; colorsys 4 value ops;
# quopri encodestring/decodestring) are covered in the
# matching pass fixture
# `test_zlib_gzip_zipfile_bz2_lzma_html_xmldom_colorsys_value_ops`.
from typing import Any
import zlib as _zlib_mod
import gzip as _gzip_mod
import zipfile as _zipfile_mod
import tarfile as _tarfile_mod
import csv as _csv_mod
import io as _io_mod
import http.cookies as _http_cookies_mod
import xml.dom as _xml_dom_mod
import xml.dom.minidom  # imported for dotted access path divergence below
import xml as _xml_root_mod

zlib_mod: Any = _zlib_mod
gzip_mod: Any = _gzip_mod
zipfile_mod: Any = _zipfile_mod
tarfile_mod: Any = _tarfile_mod
csv_mod: Any = _csv_mod
io_mod: Any = _io_mod
http_cookies_mod: Any = _http_cookies_mod
xml_dom_mod: Any = _xml_dom_mod
xml_mod: Any = _xml_root_mod


_ledger: list[int] = []

# 1) zlib deep surface
#    (mamba: missing — only compress/decompress/crc32/adler32 exposed)
assert hasattr(zlib_mod, "compressobj") == True; _ledger.append(1)
assert hasattr(zlib_mod, "decompressobj") == True; _ledger.append(1)
assert hasattr(zlib_mod, "Z_BEST_SPEED") == True; _ledger.append(1)
assert hasattr(zlib_mod, "Z_BEST_COMPRESSION") == True; _ledger.append(1)
assert hasattr(zlib_mod, "Z_DEFAULT_COMPRESSION") == True; _ledger.append(1)
assert hasattr(zlib_mod, "DEFLATED") == True; _ledger.append(1)
assert hasattr(zlib_mod, "DEF_BUF_SIZE") == True; _ledger.append(1)
assert hasattr(zlib_mod, "error") == True; _ledger.append(1)

# 2) gzip mode flag constants
#    (mamba: missing)
assert hasattr(gzip_mod, "READ") == True; _ledger.append(1)
assert hasattr(gzip_mod, "WRITE") == True; _ledger.append(1)

# 3) zipfile deep surface
#    (mamba: missing — only ZipFile/is_zipfile/ZIP_STORED/ZIP_DEFLATED exposed)
assert hasattr(zipfile_mod, "ZipInfo") == True; _ledger.append(1)
assert hasattr(zipfile_mod, "BadZipFile") == True; _ledger.append(1)
assert hasattr(zipfile_mod, "BadZipfile") == True; _ledger.append(1)
assert hasattr(zipfile_mod, "LargeZipFile") == True; _ledger.append(1)
assert hasattr(zipfile_mod, "ZIP_BZIP2") == True; _ledger.append(1)
assert hasattr(zipfile_mod, "ZIP_LZMA") == True; _ledger.append(1)

# 4) tarfile deep surface
#    (mamba: missing — only open/is_tarfile exposed)
assert hasattr(tarfile_mod, "TarFile") == True; _ledger.append(1)
assert hasattr(tarfile_mod, "TarInfo") == True; _ledger.append(1)
assert hasattr(tarfile_mod, "TarError") == True; _ledger.append(1)
assert hasattr(tarfile_mod, "ReadError") == True; _ledger.append(1)
assert hasattr(tarfile_mod, "CompressionError") == True; _ledger.append(1)

# 5) csv.DictReader iteration value contract
#    (mamba: silently yields the empty iterator)
assert list(csv_mod.DictReader(io_mod.StringIO("a,b\n1,2\n3,4\n"))) == [{"a": "1", "b": "2"}, {"a": "3", "b": "4"}]; _ledger.append(1)

# 6) csv.DictWriter.writeheader instance method
#    (mamba: DictWriter instance is a bare string — AttributeError at call site)
_buf = io_mod.StringIO()
_w = csv_mod.DictWriter(_buf, fieldnames=["a", "b"])
_w.writeheader()
assert _buf.getvalue().strip() == "a,b"; _ledger.append(1)

# 7) http.cookies top-level surface
#    (mamba: missing — module dict does not expose the documented classes)
assert hasattr(http_cookies_mod, "SimpleCookie") == True; _ledger.append(1)
assert hasattr(http_cookies_mod, "BaseCookie") == True; _ledger.append(1)
assert hasattr(http_cookies_mod, "Morsel") == True; _ledger.append(1)
assert hasattr(http_cookies_mod, "CookieError") == True; _ledger.append(1)

# 8) xml.dom top-level surface
#    (mamba: missing — module dict does not expose the documented constants/classes)
assert hasattr(xml_dom_mod, "Node") == True; _ledger.append(1)
assert hasattr(xml_dom_mod, "DOMException") == True; _ledger.append(1)
assert hasattr(xml_dom_mod, "getDOMImplementation") == True; _ledger.append(1)

# 9) xml.dom.minidom dotted-access path
#    (mamba: silently returns False for every dotted access through
#    `xml.dom.minidom.<name>` — same quirk as urllib.parse.<name>)
assert hasattr(xml_mod.dom.minidom, "Document") == True; _ledger.append(1)
assert hasattr(xml_mod.dom.minidom, "Element") == True; _ledger.append(1)
assert hasattr(xml_mod.dom.minidom, "parseString") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_zlib_gzip_zipfile_tarfile_http_cookies_xmldom_csv_dotted_silent {sum(_ledger)} asserts")
