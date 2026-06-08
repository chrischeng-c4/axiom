# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_posixpath_ntpath_compression_value_ops"
# subject = "cpython321.test_posixpath_ntpath_compression_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_posixpath_ntpath_compression_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_posixpath_ntpath_compression_value_ops: execute CPython 3.12 seed test_posixpath_ntpath_compression_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of three
# bootstrap stdlib modules used by every path-manipulation /
# archive / compression path: `posixpath` (the documented
# join / split / splitext / basename / dirname / normpath +
# sep / extsep / pathsep / isabs surface), `ntpath` (the
# Windows-flavoured equivalent surface — join / split / splitext
# / basename / dirname / sep / isabs), `glob` (the hasattr surface
# + escape contract), and `gzip` / `bz2` / `lzma` (the module-
# level compress / decompress byte-round-trip contract).
#
# The matching subset between mamba and CPython is the path-
# composition layer + path-decomposition layer + glob-escape
# layer + compression-round-trip layer: posixpath join / split /
# splitext / basename / dirname / normpath / isabs all return the
# documented strings; the `sep` / `extsep` / `pathsep` module
# constants resolve to "/" / "." / ":"; ntpath uses `\\` as
# sep + handles drive letters; glob exposes `glob` / `iglob` /
# `escape` + escape inserts `[*]` and `[?]`; gzip / bz2 / lzma
# all round-trip bytes via module-level `compress` / `decompress`.
#
# Surface in this fixture:
#   • posixpath.join("a", "b", "c") == "a/b/c";
#   • posixpath.split("/a/b/c.txt") == ("/a/b", "c.txt");
#   • posixpath.splitext("/a/b/c.txt") == ("/a/b/c", ".txt");
#   • posixpath.basename / dirname / normpath;
#   • posixpath.sep == "/", extsep == ".", pathsep == ":";
#   • posixpath.isabs;
#   • ntpath.join / split / splitext / basename / dirname / sep
#     == "\\" / isabs (drive-letter style);
#   • glob.escape("a*b?c") == "a[*]b[?]c";
#   • gzip.compress + decompress byte-round-trip;
#   • bz2.compress + decompress byte-round-trip;
#   • lzma.compress + decompress byte-round-trip.
#
# Behavioral edges that DIVERGE on mamba (shlex.split single-quote
# handling, shlex.shlex class identity, reprlib.Repr class +
# instance method, getpass.GetPassWarning, platform.system
# "Darwin" → "macos" + platform.python_implementation
# AttributeError, zipfile.ZipFile / ZIP_BZIP2 / ZIP_LZMA /
# BadZipFile / ZipInfo class identity, tarfile.TarFile / TarInfo
# / REGTYPE / DIRTYPE / LNKTYPE class identity + constants,
# gzip.GzipFile / bz2.BZ2File / lzma.LZMAFile class identity)
# are covered in the matching spec fixture
# `lang_shlex_zipfile_tarfile_platform_silent`.
import posixpath
import ntpath
import glob
import gzip
import bz2
import lzma


_ledger: list[int] = []

# 1) posixpath.join — slash composition
assert posixpath.join("a", "b", "c") == "a/b/c"; _ledger.append(1)

# 2) posixpath.split / splitext — path decomposition
assert posixpath.split("/a/b/c.txt") == ("/a/b", "c.txt"); _ledger.append(1)
assert posixpath.splitext("/a/b/c.txt") == ("/a/b/c", ".txt"); _ledger.append(1)

# 3) posixpath.basename / dirname — component access
assert posixpath.basename("/a/b/c.txt") == "c.txt"; _ledger.append(1)
assert posixpath.dirname("/a/b/c.txt") == "/a/b"; _ledger.append(1)

# 4) posixpath.normpath — collapsed canonical form
assert posixpath.normpath("/a/./b/../c") == "/a/c"; _ledger.append(1)

# 5) posixpath module constants
assert posixpath.sep == "/"; _ledger.append(1)
assert posixpath.extsep == "."; _ledger.append(1)
assert posixpath.pathsep == ":"; _ledger.append(1)

# 6) posixpath.isabs
assert posixpath.isabs("/a") == True; _ledger.append(1)
assert posixpath.isabs("a") == False; _ledger.append(1)

# 7) ntpath.join / split / splitext — backslash composition
assert ntpath.join("a", "b", "c") == "a\\b\\c"; _ledger.append(1)
assert ntpath.split("C:\\a\\b\\c.txt") == ("C:\\a\\b", "c.txt"); _ledger.append(1)
assert ntpath.splitext("C:\\a\\b\\c.txt") == ("C:\\a\\b\\c", ".txt"); _ledger.append(1)

# 8) ntpath.basename / dirname
assert ntpath.basename("C:\\a\\b\\c.txt") == "c.txt"; _ledger.append(1)
assert ntpath.dirname("C:\\a\\b\\c.txt") == "C:\\a\\b"; _ledger.append(1)

# 9) ntpath module constants + isabs
assert ntpath.sep == "\\"; _ledger.append(1)
assert ntpath.isabs("C:\\a") == True; _ledger.append(1)

# 10) glob — hasattr surface + escape
assert hasattr(glob, "glob"); _ledger.append(1)
assert hasattr(glob, "iglob"); _ledger.append(1)
assert hasattr(glob, "escape"); _ledger.append(1)
assert glob.escape("a*b?c") == "a[*]b[?]c"; _ledger.append(1)

# 11) gzip — module-level compress / decompress round-trip
_payload = b"hello world"
assert gzip.decompress(gzip.compress(_payload)) == _payload; _ledger.append(1)

# 12) bz2 — module-level compress / decompress round-trip
assert bz2.decompress(bz2.compress(_payload)) == _payload; _ledger.append(1)

# 13) lzma — module-level compress / decompress round-trip
assert lzma.decompress(lzma.compress(_payload)) == _payload; _ledger.append(1)

# 14) hasattr surface — compression module helpers
assert hasattr(gzip, "compress"); _ledger.append(1)
assert hasattr(gzip, "decompress"); _ledger.append(1)
assert hasattr(bz2, "compress"); _ledger.append(1)
assert hasattr(bz2, "decompress"); _ledger.append(1)
assert hasattr(lzma, "compress"); _ledger.append(1)
assert hasattr(lzma, "decompress"); _ledger.append(1)

# NB: shlex.split single-quote handling, shlex.shlex class
# identity, reprlib.Repr class + instance method,
# getpass.GetPassWarning, platform.system "Darwin" / "macos" +
# python_implementation AttributeError, zipfile.ZipFile /
# ZIP_BZIP2 / ZIP_LZMA / BadZipFile / ZipInfo class identity,
# tarfile.TarFile / TarInfo / REGTYPE / DIRTYPE / LNKTYPE class
# identity + constants, gzip.GzipFile / bz2.BZ2File / lzma.LZMAFile
# class identity all DIVERGE on mamba — moved to the divergence-
# spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_posixpath_ntpath_compression_value_ops {sum(_ledger)} asserts")
