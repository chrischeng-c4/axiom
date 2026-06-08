# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_http_status_configparser_zlib_const_silent"
# subject = "cpython321.lang_http_status_configparser_zlib_const_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_http_status_configparser_zlib_const_silent.py"
# status = "filled"
# ///
"""cpython321.lang_http_status_configparser_zlib_const_silent: execute CPython 3.12 seed lang_http_status_configparser_zlib_const_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences in
# `zlib` (the documented Z_DEFAULT_COMPRESSION / Z_BEST_COMPRESSION
# / Z_BEST_SPEED / Z_NO_COMPRESSION integer sentinels and MAX_WBITS),
# `gzip` (the GzipFile class identity plus the documented READ /
# WRITE / FNAME mode-flag integer sentinels), `bz2` (the BZ2File /
# BZ2Compressor class identity), `lzma` (the LZMAFile class
# identity), `http.HTTPStatus` (the per-member .value / .name /
# .phrase attributes plus type(http.HTTPStatus) == EnumType class
# identity), and `configparser` (ConfigParser.__name__ bare class
# identity + read_string / sections / has_section / has_option
# instance lifecycle + DEFAULTSECT sentinel).
#
# The matching subset (zlib.compress / decompress / crc32 / adler32,
# gzip / bz2 / lzma compress / decompress round-trip, lzma.FORMAT_XZ
# / FORMAT_ALONE / CHECK_CRC32 / CHECK_CRC64 sentinels, integer-form
# http.HTTPStatus values 200 / 201 / 204 / 400 / 401 / 403 / 404 /
# 500, hasattr surface) is covered by
# `test_zlib_gzip_bz2_lzma_compress_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • zlib.Z_DEFAULT_COMPRESSION == -1 (mamba: None);
#   • zlib.Z_BEST_COMPRESSION == 9 (mamba: None);
#   • zlib.Z_BEST_SPEED == 1 (mamba: None);
#   • zlib.Z_NO_COMPRESSION == 0 (mamba: None);
#   • zlib.MAX_WBITS == 15 (mamba: None);
#   • gzip.GzipFile.__name__ == "GzipFile" (mamba: None — the
#     binding surfaces as a bare string-ish stub);
#   • gzip.READ == 1 (mamba: None);
#   • gzip.WRITE == 2 (mamba: None);
#   • gzip.FNAME == 8 (mamba: None);
#   • bz2.BZ2File.__name__ == "BZ2File" (mamba: None);
#   • bz2.BZ2Compressor.__name__ == "BZ2Compressor" (mamba: None);
#   • lzma.LZMAFile.__name__ == "LZMAFile" (mamba: None);
#   • http.HTTPStatus.OK.value == 200 — per-member integer view
#     (mamba: returns None);
#   • http.HTTPStatus.OK.name == "OK" — per-member symbol name
#     (mamba: None);
#   • http.HTTPStatus.OK.phrase == "OK" — per-member English
#     phrase (mamba: None);
#   • http.HTTPStatus.NOT_FOUND.name == "NOT_FOUND" (mamba: None);
#   • http.HTTPStatus.INTERNAL_SERVER_ERROR.value == 500 (mamba:
#     None);
#   • type(http.HTTPStatus).__name__ == "EnumType" — class identity
#     of the enum metaclass (mamba: returns "HTTPStatus");
#   • configparser.ConfigParser.__name__ == "ConfigParser" — bare
#     class identity (mamba: returns None, the binding is a
#     `<lambda>`);
#   • configparser.ConfigParser() — instance constructor +
#     read_string / sections / has_section / has_option (mamba:
#     returns a `dict`, AttributeError on .read_string);
#   • configparser.DEFAULTSECT == "DEFAULT" (mamba: None).
import zlib as _zlib_mod
import gzip as _gzip_mod
import bz2 as _bz2_mod
import lzma as _lzma_mod
import http as _http_mod
import configparser as _configparser_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# constants / class identifiers / instance methods that mamba's
# bundled type stubs do not surface accurately.
zlib: Any = _zlib_mod
gzip: Any = _gzip_mod
bz2: Any = _bz2_mod
lzma: Any = _lzma_mod
http: Any = _http_mod
configparser: Any = _configparser_mod

_ledger: list[int] = []

# 1) zlib — documented compression-level integer sentinels
assert zlib.Z_DEFAULT_COMPRESSION == -1; _ledger.append(1)
assert zlib.Z_BEST_COMPRESSION == 9; _ledger.append(1)
assert zlib.Z_BEST_SPEED == 1; _ledger.append(1)
assert zlib.Z_NO_COMPRESSION == 0; _ledger.append(1)
assert zlib.MAX_WBITS == 15; _ledger.append(1)

# 2) gzip — class identity + documented mode-flag sentinels
assert gzip.GzipFile.__name__ == "GzipFile"; _ledger.append(1)
assert gzip.READ == 1; _ledger.append(1)
assert gzip.WRITE == 2; _ledger.append(1)
assert gzip.FNAME == 8; _ledger.append(1)

# 3) bz2 — class identity
assert bz2.BZ2File.__name__ == "BZ2File"; _ledger.append(1)
assert bz2.BZ2Compressor.__name__ == "BZ2Compressor"; _ledger.append(1)

# 4) lzma — class identity
assert lzma.LZMAFile.__name__ == "LZMAFile"; _ledger.append(1)

# 5) http.HTTPStatus — per-member .value / .name / .phrase
assert http.HTTPStatus.OK.value == 200; _ledger.append(1)
assert http.HTTPStatus.OK.name == "OK"; _ledger.append(1)
assert http.HTTPStatus.OK.phrase == "OK"; _ledger.append(1)
assert http.HTTPStatus.NOT_FOUND.name == "NOT_FOUND"; _ledger.append(1)
assert http.HTTPStatus.INTERNAL_SERVER_ERROR.value == 500; _ledger.append(1)

# 6) type(http.HTTPStatus) — enum metaclass identity
assert type(http.HTTPStatus).__name__ == "EnumType"; _ledger.append(1)

# 7) configparser.ConfigParser — bare class identity
assert configparser.ConfigParser.__name__ == "ConfigParser"; _ledger.append(1)

# 8) configparser.ConfigParser — instance constructor + lifecycle
_cp: Any = configparser.ConfigParser()
assert type(_cp).__name__ == "ConfigParser"; _ledger.append(1)
_cp.read_string("[main]\nkey = value\n")
assert _cp['main']['key'] == "value"; _ledger.append(1)
assert _cp.sections() == ["main"]; _ledger.append(1)
assert _cp.has_section("main") is True; _ledger.append(1)
assert _cp.has_option("main", "key") is True; _ledger.append(1)

# 9) configparser.DEFAULTSECT — documented sentinel string
assert configparser.DEFAULTSECT == "DEFAULT"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_http_status_configparser_zlib_const_silent {sum(_ledger)} asserts")
