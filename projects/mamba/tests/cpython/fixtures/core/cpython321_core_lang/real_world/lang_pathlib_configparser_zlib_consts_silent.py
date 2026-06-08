# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_pathlib_configparser_zlib_consts_silent"
# subject = "cpython321.lang_pathlib_configparser_zlib_consts_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_pathlib_configparser_zlib_consts_silent.py"
# status = "filled"
# ///
"""cpython321.lang_pathlib_configparser_zlib_consts_silent: execute CPython 3.12 seed lang_pathlib_configparser_zlib_consts_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# pathlib.Path str / instance-attribute surface + pathlib.PurePath
# str / instance-attribute surface + configparser extended class
# / exception identifier surface + configparser.ConfigParser
# class-identity + ConfigParser add_section/set/get value contract
# + zlib extended module helper surface + http.HTTPMethod class
# identifier pinned by atomic 183: `pathlib` (the documented
# `str(Path(...))` / `Path.name` / `Path.suffix` / `Path.stem` /
# `Path.parent` / `Path.parts` instance value contract + the
# documented `str(PurePath(...))` / `PurePath.name` /
# `PurePath.stem` instance value contract), `configparser` (the
# documented `RawConfigParser` / `Error` / `NoSectionError` /
# `NoOptionError` / `DuplicateSectionError` /
# `DuplicateOptionError` / `InterpolationError` / `ParsingError`
# class / exception identifiers + the documented
# `type(ConfigParser()).__name__ == "ConfigParser"` class-
# identity contract + the documented ConfigParser add_section/
# set/get value contract), `zlib` (the documented `compressobj`
# / `decompressobj` / `Z_BEST_COMPRESSION` / `Z_BEST_SPEED` /
# `Z_DEFAULT_COMPRESSION` / `Z_FINISH` extended class /
# function / sentinel identifiers), and `http` (the documented
# `HTTPMethod` class identifier).
#
# The matching subset (full pathlib module hasattr surface +
# pathlib type identity + partial configparser module hasattr
# surface (ConfigParser) + full gzip module hasattr surface +
# gzip round-trip + partial zlib module hasattr surface
# (compress / decompress / crc32 / adler32) + zlib round-trip
# + zlib crc32/adler32 integer values + partial http module
# hasattr surface (HTTPStatus) + HTTPStatus.OK/NOT_FOUND +
# full urllib.parse value contracts) is covered by
# `test_pathlib_configparser_gzip_zlib_http_urlparse_hasattr_ops`;
# this fixture pins the CPython-only contracts that mamba
# currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • str(Path("/tmp/foo/bar.txt")) == "/tmp/foo/bar.txt" —
#     documented value contract (mamba: returns
#     "<PosixPath instance>" — the Path instance str()
#     surface is broken);
#   • Path("/tmp/foo/bar.txt").name == "bar.txt" —
#     documented instance attribute (mamba: returns None);
#   • Path("/tmp/foo/bar.txt").suffix == ".txt" —
#     documented instance attribute (mamba: returns None);
#   • Path("/tmp/foo/bar.txt").stem == "bar" — documented
#     instance attribute (mamba: returns None);
#   • str(Path("/tmp/foo/bar.txt").parent) == "/tmp/foo" —
#     documented instance attribute (mamba: returns None);
#   • Path("/tmp/foo/bar.txt").parts == ("/", "tmp", "foo",
#     "bar.txt") — documented instance attribute (mamba:
#     returns None);
#   • str(PurePath("/a/b/c.txt")) == "/a/b/c.txt" —
#     documented value contract (mamba: returns
#     "<PurePosixPath instance>");
#   • PurePath("/a/b/c.txt").name == "c.txt" — documented
#     instance attribute (mamba: returns None);
#   • PurePath("/a/b/c.txt").stem == "c" — documented
#     instance attribute (mamba: returns None);
#   • hasattr(configparser, "RawConfigParser") is True —
#     documented class identifier (mamba: False);
#   • hasattr(configparser, "Error") is True — documented
#     exception identifier (mamba: False);
#   • hasattr(configparser, "NoSectionError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(configparser, "NoOptionError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(configparser, "DuplicateSectionError") is True
#     — documented exception identifier (mamba: False);
#   • hasattr(configparser, "DuplicateOptionError") is True
#     — documented exception identifier (mamba: False);
#   • hasattr(configparser, "InterpolationError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(configparser, "ParsingError") is True —
#     documented exception identifier (mamba: False);
#   • type(ConfigParser()).__name__ == "ConfigParser" —
#     documented class-identity contract (mamba: returns
#     "dict" — the ConfigParser constructor returns a plain
#     dict not a ConfigParser instance);
#   • ConfigParser().add_section + set + get value contract
#     (mamba: add_section raises AttributeError);
#   • hasattr(zlib, "compressobj") is True — documented
#     class identifier (mamba: False);
#   • hasattr(zlib, "decompressobj") is True — documented
#     class identifier (mamba: False);
#   • hasattr(zlib, "Z_BEST_COMPRESSION") is True —
#     documented integer sentinel (mamba: False);
#   • hasattr(zlib, "Z_BEST_SPEED") is True — documented
#     integer sentinel (mamba: False);
#   • hasattr(zlib, "Z_DEFAULT_COMPRESSION") is True —
#     documented integer sentinel (mamba: False);
#   • hasattr(zlib, "Z_FINISH") is True — documented
#     integer sentinel (mamba: False);
#   • hasattr(http, "HTTPMethod") is True — documented
#     class identifier (mamba: False).
import pathlib as _pathlib_mod
import configparser as _configparser_mod
import zlib as _zlib_mod
import http as _http_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance attribute / value-contract behavior
# that mamba's bundled type stubs do not surface accurately.
pathlib: Any = _pathlib_mod
configparser: Any = _configparser_mod
zlib: Any = _zlib_mod
http: Any = _http_mod


_ledger: list[int] = []

# 1) pathlib.Path — str + instance attribute value contract
_p = pathlib.Path("/tmp/foo/bar.txt")
assert str(_p) == "/tmp/foo/bar.txt"; _ledger.append(1)
assert _p.name == "bar.txt"; _ledger.append(1)
assert _p.suffix == ".txt"; _ledger.append(1)
assert _p.stem == "bar"; _ledger.append(1)
assert str(_p.parent) == "/tmp/foo"; _ledger.append(1)
assert _p.parts == ("/", "tmp", "foo", "bar.txt"); _ledger.append(1)

# 2) pathlib.PurePath — str + instance attribute
_pp = pathlib.PurePath("/a/b/c.txt")
assert str(_pp) == "/a/b/c.txt"; _ledger.append(1)
assert _pp.name == "c.txt"; _ledger.append(1)
assert _pp.stem == "c"; _ledger.append(1)

# 3) configparser — extended class / exception identifiers
assert hasattr(configparser, "RawConfigParser") == True; _ledger.append(1)
assert hasattr(configparser, "Error") == True; _ledger.append(1)
assert hasattr(configparser, "NoSectionError") == True; _ledger.append(1)
assert hasattr(configparser, "NoOptionError") == True; _ledger.append(1)
assert hasattr(configparser, "DuplicateSectionError") == True; _ledger.append(1)
assert hasattr(configparser, "DuplicateOptionError") == True; _ledger.append(1)
assert hasattr(configparser, "InterpolationError") == True; _ledger.append(1)
assert hasattr(configparser, "ParsingError") == True; _ledger.append(1)

# 4) configparser.ConfigParser — class identity +
#    add_section / set / get value contract
_cp = configparser.ConfigParser()
assert type(_cp).__name__ == "ConfigParser"; _ledger.append(1)
_cp.add_section("section1")
_cp.set("section1", "key1", "value1")
assert _cp.get("section1", "key1") == "value1"; _ledger.append(1)
assert _cp.sections() == ["section1"]; _ledger.append(1)

# 5) zlib — extended class / function / sentinel identifiers
assert hasattr(zlib, "compressobj") == True; _ledger.append(1)
assert hasattr(zlib, "decompressobj") == True; _ledger.append(1)
assert hasattr(zlib, "Z_BEST_COMPRESSION") == True; _ledger.append(1)
assert hasattr(zlib, "Z_BEST_SPEED") == True; _ledger.append(1)
assert hasattr(zlib, "Z_DEFAULT_COMPRESSION") == True; _ledger.append(1)
assert hasattr(zlib, "Z_FINISH") == True; _ledger.append(1)

# 6) http — HTTPMethod class identifier
assert hasattr(http, "HTTPMethod") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pathlib_configparser_zlib_consts_silent {sum(_ledger)} asserts")
