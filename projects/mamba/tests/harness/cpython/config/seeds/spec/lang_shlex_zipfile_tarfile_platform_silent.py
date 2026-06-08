# Operational AssertionPass seed for SILENT divergences across the
# shell-quoting / archive / compression / platform-info quintet
# pinned by atomic 150 (the 50-tick reorg checkpoint): `shlex`
# (the documented split single-quote handling + `shlex.shlex`
# bare class identity), `reprlib` (the documented `Repr` class
# identity + bound instance `repr` method), `getpass` (the
# documented `GetPassWarning` exception class), `platform` (the
# documented `system()` returning "Darwin" on macOS + the
# `python_implementation()` helper), `zipfile` (the documented
# `ZipFile` / `BadZipFile` / `ZipInfo` / `ZIP_BZIP2` /
# `ZIP_LZMA` class identity surface), `tarfile` (the documented
# `TarFile` / `TarInfo` class identity + `REGTYPE` / `DIRTYPE`
# / `LNKTYPE` byte constants), and `gzip` / `bz2` / `lzma`
# (the documented file-wrapping class identity — `GzipFile` /
# `BZ2File` / `LZMAFile`).
#
# The matching subset (posixpath / ntpath join / split / splitext
# / basename / dirname / normpath / sep / extsep / pathsep /
# isabs, glob.glob / iglob / escape, gzip / bz2 / lzma module-
# level compress / decompress byte-round-trip) is covered by
# `test_posixpath_ntpath_compression_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • shlex.split("a 'b c' d") == ["a", "b c", "d"] — single-
#     quote tokenization (mamba: returns ["a", "'b", "c'", "d"],
#     single quotes are not paired);
#   • shlex.shlex.__name__ == "shlex" — bare class identity
#     (mamba: hasattr returns False, mamba doesn't expose the
#     class);
#   • reprlib.Repr.__name__ == "Repr" — bare class identity
#     (mamba: None);
#   • reprlib.Repr().repr("test") == "'test'" — bound instance
#     method (mamba: AttributeError, 'dict' object has no
#     attribute 'repr');
#   • hasattr(getpass, "GetPassWarning") is True — exception
#     class surface (mamba: returns False);
#   • platform.system() == "Darwin" — CPython platform name
#     (mamba: returns "macos", a lowercase friendly name);
#   • platform.python_implementation() == "CPython" (mamba:
#     AttributeError, 'dict' object has no attribute
#     'python_implementation');
#   • zipfile.ZipFile.__name__ == "ZipFile" — archive class
#     identity (mamba: None);
#   • zipfile.ZIP_BZIP2 == 12 — bz2 method constant (mamba:
#     None);
#   • zipfile.ZIP_LZMA == 14 — lzma method constant (mamba:
#     None);
#   • zipfile.BadZipFile.__name__ == "BadZipFile" — exception
#     class identity (mamba: None);
#   • hasattr(zipfile, "ZipInfo") is True — info-record class
#     surface (mamba: returns False);
#   • tarfile.TarFile.__name__ == "TarFile" (mamba: None);
#   • tarfile.TarInfo.__name__ == "TarInfo" (mamba: None);
#   • tarfile.REGTYPE == b"0" — regular-file type byte (mamba:
#     None);
#   • tarfile.DIRTYPE == b"5" — directory type byte (mamba:
#     None);
#   • tarfile.LNKTYPE == b"1" — hardlink type byte (mamba:
#     None);
#   • gzip.GzipFile.__name__ == "GzipFile" — wrapper class
#     identity (mamba: None);
#   • bz2.BZ2File.__name__ == "BZ2File" (mamba: None);
#   • lzma.LZMAFile.__name__ == "LZMAFile" (mamba: None).
import shlex as _shlex_mod
import reprlib as _reprlib_mod
import getpass as _getpass_mod
import platform as _platform_mod
import zipfile as _zipfile_mod
import tarfile as _tarfile_mod
import gzip as _gzip_mod
import bz2 as _bz2_mod
import lzma as _lzma_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
shlex: Any = _shlex_mod
reprlib: Any = _reprlib_mod
getpass: Any = _getpass_mod
platform: Any = _platform_mod
zipfile: Any = _zipfile_mod
tarfile: Any = _tarfile_mod
gzip: Any = _gzip_mod
bz2: Any = _bz2_mod
lzma: Any = _lzma_mod


_ledger: list[int] = []

# 1) shlex.split — single-quote tokenization
assert shlex.split("a 'b c' d") == ["a", "b c", "d"]; _ledger.append(1)

# 2) shlex.shlex — bare class identity
assert shlex.shlex.__name__ == "shlex"; _ledger.append(1)

# 3) reprlib.Repr — bare class identity + bound instance method
assert reprlib.Repr.__name__ == "Repr"; _ledger.append(1)
assert reprlib.Repr().repr("test") == "'test'"; _ledger.append(1)

# 4) getpass.GetPassWarning — exception class surface
assert hasattr(getpass, "GetPassWarning") == True; _ledger.append(1)

# 5) platform.system — CPython OS name
assert platform.system() == "Darwin"; _ledger.append(1)

# 6) platform.python_implementation
assert platform.python_implementation() == "CPython"; _ledger.append(1)

# 7) zipfile — archive + exception class identity
assert zipfile.ZipFile.__name__ == "ZipFile"; _ledger.append(1)
assert zipfile.ZIP_BZIP2 == 12; _ledger.append(1)
assert zipfile.ZIP_LZMA == 14; _ledger.append(1)
assert zipfile.BadZipFile.__name__ == "BadZipFile"; _ledger.append(1)

# 8) zipfile.ZipInfo — info-record class surface
assert hasattr(zipfile, "ZipInfo") == True; _ledger.append(1)

# 9) tarfile — archive + info-record class identity
assert tarfile.TarFile.__name__ == "TarFile"; _ledger.append(1)
assert tarfile.TarInfo.__name__ == "TarInfo"; _ledger.append(1)

# 10) tarfile — file-type byte constants
assert tarfile.REGTYPE == b"0"; _ledger.append(1)
assert tarfile.DIRTYPE == b"5"; _ledger.append(1)
assert tarfile.LNKTYPE == b"1"; _ledger.append(1)

# 11) gzip / bz2 / lzma — wrapper class identity
assert gzip.GzipFile.__name__ == "GzipFile"; _ledger.append(1)
assert bz2.BZ2File.__name__ == "BZ2File"; _ledger.append(1)
assert lzma.LZMAFile.__name__ == "LZMAFile"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_shlex_zipfile_tarfile_platform_silent {sum(_ledger)} asserts")
