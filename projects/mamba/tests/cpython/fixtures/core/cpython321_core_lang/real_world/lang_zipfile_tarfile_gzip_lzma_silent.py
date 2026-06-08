# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_zipfile_tarfile_gzip_lzma_silent"
# subject = "cpython321.lang_zipfile_tarfile_gzip_lzma_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_zipfile_tarfile_gzip_lzma_silent.py"
# status = "filled"
# ///
"""cpython321.lang_zipfile_tarfile_gzip_lzma_silent: execute CPython 3.12 seed lang_zipfile_tarfile_gzip_lzma_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across
# the `zipfile` module class / compression-sentinel /
# exception / path-class identifier surface +
# `tarfile` module class / exception / typeflag /
# format-sentinel / encoding identifier surface +
# `tarfile.USTAR_FORMAT` / `tarfile.GNU_FORMAT` /
# `tarfile.PAX_FORMAT` integer-constant value contract
# + `gzip` module flag-sentinel / mode-sentinel
# identifier surface + `lzma.is_check_supported`
# helper identifier pinned by atomic 208: `zipfile`
# (the documented class / compression-sentinel /
# exception / path-class identifier surface —
# `ZipInfo` / `ZIP_BZIP2` / `ZIP_LZMA` / `PyZipFile` /
# `BadZipFile` / `BadZipfile` / `LargeZipFile` /
# `ZIP_FILECOUNT_LIMIT` / `ZIP_MAX_COMMENT` / `Path`),
# `tarfile` (the documented class / exception /
# typeflag / format-sentinel / encoding identifier
# surface — `TarFile` / `TarInfo` / `TarError` /
# `ReadError` / `CompressionError` / `StreamError` /
# `ExtractError` / `REGTYPE` / `AREGTYPE` / `LNKTYPE`
# / `SYMTYPE` / `DIRTYPE` / `FIFOTYPE` / `CONTTYPE` /
# `CHRTYPE` / `BLKTYPE` / `GNUTYPE_SPARSE` /
# `USTAR_FORMAT` / `GNU_FORMAT` / `PAX_FORMAT` /
# `DEFAULT_FORMAT` / `ENCODING` + the documented
# `tarfile.USTAR_FORMAT == 0` / `tarfile.GNU_FORMAT
# == 1` / `tarfile.PAX_FORMAT == 2` /
# `type(tarfile.USTAR_FORMAT).__name__ == "int"`
# integer-constant value contract — mamba: all
# None / NoneType), `gzip` (the documented
# flag-sentinel / mode-sentinel identifier surface —
# `FNAME` / `FCOMMENT` / `FEXTRA` / `FHCRC` / `FTEXT`
# / `READ` / `WRITE`), and `lzma` (the documented
# `is_check_supported` helper identifier).
#
# The matching subset (partial zipfile hasattr +
# integer-sentinel value contract, partial tarfile
# hasattr, partial gzip hasattr + round-trip value
# contract, full bz2 hasattr + round-trip value
# contract, partial lzma hasattr + round-trip +
# integer-sentinel value contract) is covered by
# `test_zipfile_tarfile_gzip_bz2_lzma_value_ops`;
# this fixture pins the CPython-only contracts that
# mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(zipfile, "ZipInfo") is True — documented
#     class identifier (mamba: False);
#   • hasattr(zipfile, "ZIP_BZIP2") is True —
#     documented compression-sentinel identifier
#     (mamba: False);
#   • hasattr(zipfile, "ZIP_LZMA") is True —
#     documented compression-sentinel identifier
#     (mamba: False);
#   • hasattr(zipfile, "PyZipFile") is True —
#     documented class identifier (mamba: False);
#   • hasattr(zipfile, "BadZipFile") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(zipfile, "BadZipfile") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(zipfile, "LargeZipFile") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(zipfile, "ZIP_FILECOUNT_LIMIT") is True
#     — documented integer-sentinel identifier
#     (mamba: False);
#   • hasattr(zipfile, "ZIP_MAX_COMMENT") is True —
#     documented integer-sentinel identifier
#     (mamba: False);
#   • hasattr(zipfile, "Path") is True — documented
#     path-class identifier (mamba: False);
#   • hasattr(tarfile, "TarFile") is True —
#     documented class identifier (mamba: False);
#   • hasattr(tarfile, "TarInfo") is True —
#     documented class identifier (mamba: False);
#   • hasattr(tarfile, "TarError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(tarfile, "ReadError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(tarfile, "CompressionError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(tarfile, "StreamError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(tarfile, "ExtractError") is True —
#     documented exception identifier (mamba: False);
#   • hasattr(tarfile, "REGTYPE") is True — documented
#     typeflag identifier (mamba: False);
#   • hasattr(tarfile, "AREGTYPE") is True —
#     documented typeflag identifier (mamba: False);
#   • hasattr(tarfile, "LNKTYPE") is True — documented
#     typeflag identifier (mamba: False);
#   • hasattr(tarfile, "SYMTYPE") is True — documented
#     typeflag identifier (mamba: False);
#   • hasattr(tarfile, "DIRTYPE") is True — documented
#     typeflag identifier (mamba: False);
#   • hasattr(tarfile, "FIFOTYPE") is True —
#     documented typeflag identifier (mamba: False);
#   • hasattr(tarfile, "CONTTYPE") is True —
#     documented typeflag identifier (mamba: False);
#   • hasattr(tarfile, "CHRTYPE") is True —
#     documented typeflag identifier (mamba: False);
#   • hasattr(tarfile, "BLKTYPE") is True —
#     documented typeflag identifier (mamba: False);
#   • hasattr(tarfile, "GNUTYPE_SPARSE") is True —
#     documented typeflag identifier (mamba: False);
#   • hasattr(tarfile, "USTAR_FORMAT") is True —
#     documented format-sentinel identifier
#     (mamba: False);
#   • hasattr(tarfile, "GNU_FORMAT") is True —
#     documented format-sentinel identifier
#     (mamba: False);
#   • hasattr(tarfile, "PAX_FORMAT") is True —
#     documented format-sentinel identifier
#     (mamba: False);
#   • hasattr(tarfile, "DEFAULT_FORMAT") is True —
#     documented format-sentinel identifier
#     (mamba: False);
#   • hasattr(tarfile, "ENCODING") is True —
#     documented encoding identifier (mamba: False);
#   • tarfile.USTAR_FORMAT == 0 — documented
#     integer-constant value (mamba: None);
#   • tarfile.GNU_FORMAT == 1 — documented
#     integer-constant value (mamba: None);
#   • tarfile.PAX_FORMAT == 2 — documented
#     integer-constant value (mamba: None);
#   • type(tarfile.USTAR_FORMAT).__name__ == "int" —
#     documented constant-type contract
#     (mamba: "NoneType");
#   • hasattr(gzip, "FNAME") is True — documented
#     flag-sentinel identifier (mamba: False);
#   • hasattr(gzip, "FCOMMENT") is True — documented
#     flag-sentinel identifier (mamba: False);
#   • hasattr(gzip, "FEXTRA") is True — documented
#     flag-sentinel identifier (mamba: False);
#   • hasattr(gzip, "FHCRC") is True — documented
#     flag-sentinel identifier (mamba: False);
#   • hasattr(gzip, "FTEXT") is True — documented
#     flag-sentinel identifier (mamba: False);
#   • hasattr(gzip, "READ") is True — documented
#     mode-sentinel identifier (mamba: False);
#   • hasattr(gzip, "WRITE") is True — documented
#     mode-sentinel identifier (mamba: False);
#   • hasattr(lzma, "is_check_supported") is True —
#     documented helper identifier (mamba: False).
import zipfile as _zipfile_mod
import tarfile as _tarfile_mod
import gzip as _gzip_mod
import lzma as _lzma_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identity / module-attribute identifier behavior that
# mamba's bundled type stubs do not surface accurately.
zipfile: Any = _zipfile_mod
tarfile: Any = _tarfile_mod
gzip: Any = _gzip_mod
lzma: Any = _lzma_mod


_ledger: list[int] = []

# 1) zipfile — class / compression-sentinel / exception / path-class identifier surface
assert hasattr(zipfile, "ZipInfo") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_BZIP2") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_LZMA") == True; _ledger.append(1)
assert hasattr(zipfile, "PyZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "BadZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "BadZipfile") == True; _ledger.append(1)
assert hasattr(zipfile, "LargeZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_FILECOUNT_LIMIT") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_MAX_COMMENT") == True; _ledger.append(1)
assert hasattr(zipfile, "Path") == True; _ledger.append(1)

# 2) tarfile — class / exception / typeflag / format-sentinel / encoding identifier surface
assert hasattr(tarfile, "TarFile") == True; _ledger.append(1)
assert hasattr(tarfile, "TarInfo") == True; _ledger.append(1)
assert hasattr(tarfile, "TarError") == True; _ledger.append(1)
assert hasattr(tarfile, "ReadError") == True; _ledger.append(1)
assert hasattr(tarfile, "CompressionError") == True; _ledger.append(1)
assert hasattr(tarfile, "StreamError") == True; _ledger.append(1)
assert hasattr(tarfile, "ExtractError") == True; _ledger.append(1)
assert hasattr(tarfile, "REGTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "AREGTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "LNKTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "SYMTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "DIRTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "FIFOTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "CONTTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "CHRTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "BLKTYPE") == True; _ledger.append(1)
assert hasattr(tarfile, "GNUTYPE_SPARSE") == True; _ledger.append(1)
assert hasattr(tarfile, "USTAR_FORMAT") == True; _ledger.append(1)
assert hasattr(tarfile, "GNU_FORMAT") == True; _ledger.append(1)
assert hasattr(tarfile, "PAX_FORMAT") == True; _ledger.append(1)
assert hasattr(tarfile, "DEFAULT_FORMAT") == True; _ledger.append(1)
assert hasattr(tarfile, "ENCODING") == True; _ledger.append(1)

# 3) tarfile — integer-constant value contract
assert tarfile.USTAR_FORMAT == 0; _ledger.append(1)
assert tarfile.GNU_FORMAT == 1; _ledger.append(1)
assert tarfile.PAX_FORMAT == 2; _ledger.append(1)
assert type(tarfile.USTAR_FORMAT).__name__ == "int"; _ledger.append(1)

# 4) gzip — flag-sentinel / mode-sentinel identifier surface
assert hasattr(gzip, "FNAME") == True; _ledger.append(1)
assert hasattr(gzip, "FCOMMENT") == True; _ledger.append(1)
assert hasattr(gzip, "FEXTRA") == True; _ledger.append(1)
assert hasattr(gzip, "FHCRC") == True; _ledger.append(1)
assert hasattr(gzip, "FTEXT") == True; _ledger.append(1)
assert hasattr(gzip, "READ") == True; _ledger.append(1)
assert hasattr(gzip, "WRITE") == True; _ledger.append(1)

# 5) lzma — helper identifier
assert hasattr(lzma, "is_check_supported") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_zipfile_tarfile_gzip_lzma_silent {sum(_ledger)} asserts")
