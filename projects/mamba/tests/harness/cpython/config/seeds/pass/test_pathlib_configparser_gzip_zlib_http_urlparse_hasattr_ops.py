# Operational AssertionPass seed for the value contract of the
# `pathlib` / `configparser` / `gzip` / `zlib` / `http` /
# `urllib.parse` six-pack pinned to atomic 183: `pathlib` (the
# documented full module-level helper hasattr surface — `Path`
# / `PurePath` / `PurePosixPath` / `PureWindowsPath` /
# `PosixPath` / `WindowsPath` + the documented Path() / PurePath()
# class-identity contract), `configparser` (the documented
# `ConfigParser` class identifier), `gzip` (the documented full
# module-level helper hasattr surface — `GzipFile` / `open` /
# `compress` / `decompress` / `BadGzipFile` + the documented
# gzip.compress/decompress round-trip value contract), `zlib`
# (the documented partial module-level helper hasattr surface —
# `compress` / `decompress` / `crc32` / `adler32` + the
# documented zlib.compress/decompress round-trip value contract
# + the documented zlib.crc32 / zlib.adler32 integer value
# contract), `http` (the documented `HTTPStatus` class
# identifier + the documented HTTPStatus.OK / NOT_FOUND integer
# value contract), and `urllib.parse` (the documented `urlparse`
# / `urlencode` / `quote` / `unquote` / `urljoin` value
# contracts).
#
# The matching subset between mamba and CPython is the full
# `pathlib` module hasattr surface + the `type(Path(...)).__name__
# == "PosixPath"` class-identity layer (the Path instance value
# layer DIVERGES — every documented attribute returns None and
# str() returns "<PosixPath instance>"), the partial
# `configparser` module hasattr surface (`ConfigParser` —
# `RawConfigParser` / `Error` / `NoSectionError` / `NoOptionError`
# / `DuplicateSectionError` / `DuplicateOptionError` /
# `InterpolationError` / `ParsingError` DIVERGE + the
# ConfigParser instance value layer DIVERGES), the full `gzip`
# module hasattr surface + the gzip.compress/decompress round-
# trip value layer (compressed-byte-length differs but round-
# trip holds), the partial `zlib` module hasattr surface
# (compress / decompress / crc32 / adler32 — compressobj /
# decompressobj / Z_BEST_COMPRESSION / Z_BEST_SPEED /
# Z_DEFAULT_COMPRESSION / Z_FINISH DIVERGE) + the
# zlib.compress/decompress round-trip value layer + the
# zlib.crc32 / zlib.adler32 integer value layer, the partial
# `http` module hasattr surface (HTTPStatus — HTTPMethod
# DIVERGES) + the HTTPStatus.OK / NOT_FOUND integer value layer,
# and the full `urllib.parse` urlparse / urlencode / quote /
# unquote / urljoin value layer.
#
# Surface in this fixture:
#   • pathlib — full module hasattr surface (Path / PurePath /
#     PurePosixPath / PureWindowsPath / PosixPath /
#     WindowsPath);
#   • pathlib — type(Path(...)).__name__ class identity;
#   • configparser — partial module hasattr surface
#     (ConfigParser);
#   • gzip — full module hasattr surface (GzipFile / open /
#     compress / decompress / BadGzipFile);
#   • gzip.compress + gzip.decompress — round-trip value
#     contract;
#   • zlib — partial module hasattr surface (compress /
#     decompress / crc32 / adler32);
#   • zlib.compress + zlib.decompress — round-trip value
#     contract;
#   • zlib.crc32 / zlib.adler32 — integer value contract;
#   • http — partial module hasattr surface (HTTPStatus);
#   • http.HTTPStatus.OK / NOT_FOUND — integer value contract;
#   • urllib.parse — urlparse / urlencode / quote / unquote /
#     urljoin value contracts.
#
# Behavioral edges that DIVERGE on mamba (Path() str returns
# "<PosixPath instance>" not "/tmp/foo/bar.txt", Path.name /
# .suffix / .stem / .parent / .parts all return None, PurePath()
# str returns "<PurePosixPath instance>", PurePath.name / .stem
# return None, hasattr(configparser, "RawConfigParser") /
# "Error" / "NoSectionError" / "NoOptionError" /
# "DuplicateSectionError" / "DuplicateOptionError" /
# "InterpolationError" / "ParsingError" all False,
# type(ConfigParser()).__name__ returns "dict" not
# "ConfigParser", ConfigParser.add_section raises
# AttributeError, hasattr(zlib, "compressobj") /
# "decompressobj" / "Z_BEST_COMPRESSION" / "Z_BEST_SPEED" /
# "Z_DEFAULT_COMPRESSION" / "Z_FINISH" all False,
# hasattr(http, "HTTPMethod") False) are covered in the
# matching spec fixture
# `lang_pathlib_configparser_zlib_consts_silent`.
import pathlib
import configparser
import gzip
import zlib
import http
from urllib.parse import urlparse, urlencode, quote, unquote, urljoin


_ledger: list[int] = []

# 1) pathlib — full module hasattr surface
assert hasattr(pathlib, "Path") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePath") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PureWindowsPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "WindowsPath") == True; _ledger.append(1)

# 2) pathlib — type(Path(...)).__name__ class identity
_p = pathlib.Path("/tmp/foo/bar.txt")
assert type(_p).__name__ == "PosixPath"; _ledger.append(1)

# 3) configparser — partial module hasattr surface
#    (RawConfigParser / Error / NoSectionError / NoOptionError
#    / DuplicateSectionError / DuplicateOptionError /
#    InterpolationError / ParsingError DIVERGE — moved to
#    spec fixture)
assert hasattr(configparser, "ConfigParser") == True; _ledger.append(1)

# 4) gzip — full module hasattr surface
assert hasattr(gzip, "GzipFile") == True; _ledger.append(1)
assert hasattr(gzip, "open") == True; _ledger.append(1)
assert hasattr(gzip, "compress") == True; _ledger.append(1)
assert hasattr(gzip, "decompress") == True; _ledger.append(1)
assert hasattr(gzip, "BadGzipFile") == True; _ledger.append(1)

# 5) gzip.compress + gzip.decompress — round-trip value
_data_g = b"hello world" * 10
_compressed_g = gzip.compress(_data_g)
assert type(_compressed_g).__name__ == "bytes"; _ledger.append(1)
assert gzip.decompress(_compressed_g) == _data_g; _ledger.append(1)

# 6) zlib — partial module hasattr surface
#    (compressobj / decompressobj / Z_BEST_COMPRESSION /
#    Z_BEST_SPEED / Z_DEFAULT_COMPRESSION / Z_FINISH DIVERGE
#    — moved to spec fixture)
assert hasattr(zlib, "compress") == True; _ledger.append(1)
assert hasattr(zlib, "decompress") == True; _ledger.append(1)
assert hasattr(zlib, "crc32") == True; _ledger.append(1)
assert hasattr(zlib, "adler32") == True; _ledger.append(1)

# 7) zlib.compress + zlib.decompress — round-trip value
_data_z = b"hello world" * 10
_compressed_z = zlib.compress(_data_z)
assert zlib.decompress(_compressed_z) == _data_z; _ledger.append(1)

# 8) zlib.crc32 / zlib.adler32 — integer value contract
assert zlib.crc32(b"hello") == 907060870; _ledger.append(1)
assert zlib.adler32(b"hello") == 103547413; _ledger.append(1)

# 9) http — partial module hasattr surface
#    (HTTPMethod DIVERGES — moved to spec fixture)
assert hasattr(http, "HTTPStatus") == True; _ledger.append(1)

# 10) http.HTTPStatus.OK / NOT_FOUND — integer value contract
assert http.HTTPStatus.OK == 200; _ledger.append(1)
assert http.HTTPStatus.NOT_FOUND == 404; _ledger.append(1)

# 11) urllib.parse — urlparse value contract
_u = urlparse("https://example.com/path?q=1&w=2")
assert _u.scheme == "https"; _ledger.append(1)
assert _u.netloc == "example.com"; _ledger.append(1)
assert _u.path == "/path"; _ledger.append(1)
assert _u.query == "q=1&w=2"; _ledger.append(1)

# 12) urllib.parse — urlencode / quote / unquote / urljoin
assert urlencode({"a": "1", "b": "2"}) == "a=1&b=2"; _ledger.append(1)
assert quote("hello world") == "hello%20world"; _ledger.append(1)
assert unquote("hello%20world") == "hello world"; _ledger.append(1)
assert urljoin("http://a/b/c", "d") == "http://a/b/d"; _ledger.append(1)

# NB: Path() str returns "<PosixPath instance>" on mamba,
# Path.name / .suffix / .stem / .parent / .parts all return
# None, PurePath() str returns "<PurePosixPath instance>",
# PurePath.name / .stem return None, hasattr(configparser,
# "RawConfigParser") / "Error" / "NoSectionError" /
# "NoOptionError" / "DuplicateSectionError" /
# "DuplicateOptionError" / "InterpolationError" /
# "ParsingError" all False, type(ConfigParser()).__name__
# returns "dict" not "ConfigParser", ConfigParser.add_section
# raises AttributeError, hasattr(zlib, "compressobj") /
# "decompressobj" / "Z_BEST_COMPRESSION" / "Z_BEST_SPEED" /
# "Z_DEFAULT_COMPRESSION" / "Z_FINISH" all False, hasattr
# (http, "HTTPMethod") False — all DIVERGE on mamba — moved
# to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_pathlib_configparser_gzip_zlib_http_urlparse_hasattr_ops {sum(_ledger)} asserts")
