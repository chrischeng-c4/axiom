# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_zipfile_tarfile_gzip_bz2_lzma_value_ops"
# subject = "cpython321.test_zipfile_tarfile_gzip_bz2_lzma_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_zipfile_tarfile_gzip_bz2_lzma_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_zipfile_tarfile_gzip_bz2_lzma_value_ops: execute CPython 3.12 seed test_zipfile_tarfile_gzip_bz2_lzma_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `zipfile` / `tarfile` / `gzip` / `bz2` / `lzma` five-pack
# pinned to atomic 208: `zipfile` (the documented partial
# module-level class / helper / compression-sentinel
# identifier hasattr surface — `ZipFile` / `is_zipfile` /
# `ZIP_STORED` / `ZIP_DEFLATED` + the documented
# `ZIP_STORED == 0` / `ZIP_DEFLATED == 8` /
# `type(zipfile.ZIP_STORED).__name__ == "int"`
# integer-sentinel value contract), `tarfile` (the
# documented partial module-level helper identifier
# hasattr surface — `is_tarfile` / `open`), `gzip` (the
# documented partial module-level class / helper /
# exception identifier hasattr surface — `GzipFile` /
# `open` / `compress` / `decompress` / `BadGzipFile` +
# the documented `gzip.decompress(gzip.compress(data))
# == data` / `type(gzip.compress(data)).__name__ ==
# "bytes"` / `len(gzip.compress(data)) <
# len(data) * 2` round-trip value contract), `bz2` (the
# documented full module-level class / helper
# identifier hasattr surface — `BZ2File` /
# `BZ2Compressor` / `BZ2Decompressor` / `open` /
# `compress` / `decompress` + the documented
# `bz2.decompress(bz2.compress(data)) == data` /
# `type(bz2.compress(data)).__name__ == "bytes"`
# round-trip value contract), and `lzma` (the
# documented partial module-level class / helper /
# exception / check-sentinel / format-sentinel /
# preset-sentinel / match-finder-sentinel /
# mode-sentinel identifier hasattr surface — `LZMAFile`
# / `LZMACompressor` / `LZMADecompressor` / `LZMAError`
# / `open` / `compress` / `decompress` / `CHECK_NONE` /
# `CHECK_CRC32` / `CHECK_CRC64` / `CHECK_SHA256` /
# `CHECK_ID_MAX` / `CHECK_UNKNOWN` / `FORMAT_XZ` /
# `FORMAT_ALONE` / `FORMAT_RAW` / `FORMAT_AUTO` /
# `PRESET_DEFAULT` / `PRESET_EXTREME` / `MF_HC3` /
# `MF_HC4` / `MF_BT2` / `MF_BT3` / `MF_BT4` /
# `MODE_FAST` / `MODE_NORMAL` + the documented
# `lzma.decompress(lzma.compress(data)) == data` /
# `type(lzma.compress(data)).__name__ == "bytes"` /
# `FORMAT_XZ == 1` / `CHECK_CRC64 == 4` /
# `type(lzma.FORMAT_XZ).__name__ == "int"`
# round-trip / integer-sentinel value contract).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(zipfile, "ZipInfo") / "ZIP_BZIP2" /
# "ZIP_LZMA" / "PyZipFile" / "BadZipFile" /
# "BadZipfile" / "LargeZipFile" /
# "ZIP_FILECOUNT_LIMIT" / "ZIP_MAX_COMMENT" / "Path"
# all False on mamba, hasattr(tarfile, "TarFile") /
# "TarInfo" / "TarError" / "ReadError" /
# "CompressionError" / "StreamError" /
# "ExtractError" / "REGTYPE" / "AREGTYPE" / "LNKTYPE"
# / "SYMTYPE" / "DIRTYPE" / "FIFOTYPE" / "CONTTYPE"
# / "CHRTYPE" / "BLKTYPE" / "GNUTYPE_SPARSE" /
# "USTAR_FORMAT" / "GNU_FORMAT" / "PAX_FORMAT" /
# "DEFAULT_FORMAT" / "ENCODING" all False on mamba +
# tarfile.USTAR_FORMAT/GNU_FORMAT/PAX_FORMAT all
# None on mamba, hasattr(gzip, "FNAME") /
# "FCOMMENT" / "FEXTRA" / "FHCRC" / "FTEXT" /
# "READ" / "WRITE" all False on mamba,
# hasattr(lzma, "is_check_supported") False on
# mamba) are covered in the matching spec fixture
# `lang_zipfile_tarfile_gzip_lzma_silent`.
import zipfile
import tarfile
import gzip
import bz2
import lzma


_ledger: list[int] = []

# 1) zipfile — partial module hasattr surface
#    (ZipInfo / ZIP_BZIP2 / ZIP_LZMA / PyZipFile /
#    BadZipFile / BadZipfile / LargeZipFile /
#    ZIP_FILECOUNT_LIMIT / ZIP_MAX_COMMENT / Path all
#    DIVERGE on mamba — moved to spec)
assert hasattr(zipfile, "ZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "is_zipfile") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_STORED") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_DEFLATED") == True; _ledger.append(1)

# 2) zipfile — integer-sentinel value contract
assert zipfile.ZIP_STORED == 0; _ledger.append(1)
assert zipfile.ZIP_DEFLATED == 8; _ledger.append(1)
assert type(zipfile.ZIP_STORED).__name__ == "int"; _ledger.append(1)

# 3) tarfile — partial module hasattr surface
#    (TarFile / TarInfo / TarError / ReadError /
#    CompressionError / StreamError / ExtractError /
#    REGTYPE / AREGTYPE / LNKTYPE / SYMTYPE /
#    DIRTYPE / FIFOTYPE / CONTTYPE / CHRTYPE /
#    BLKTYPE / GNUTYPE_SPARSE / USTAR_FORMAT /
#    GNU_FORMAT / PAX_FORMAT / DEFAULT_FORMAT /
#    ENCODING all DIVERGE on mamba — moved to spec)
assert hasattr(tarfile, "is_tarfile") == True; _ledger.append(1)
assert hasattr(tarfile, "open") == True; _ledger.append(1)

# 4) gzip — partial module hasattr surface
#    (FNAME / FCOMMENT / FEXTRA / FHCRC / FTEXT /
#    READ / WRITE all DIVERGE on mamba — moved to spec)
assert hasattr(gzip, "GzipFile") == True; _ledger.append(1)
assert hasattr(gzip, "open") == True; _ledger.append(1)
assert hasattr(gzip, "compress") == True; _ledger.append(1)
assert hasattr(gzip, "decompress") == True; _ledger.append(1)
assert hasattr(gzip, "BadGzipFile") == True; _ledger.append(1)

# 5) gzip — round-trip value contract
_gdata = b"hello world hello world hello world"
_gc = gzip.compress(_gdata)
assert type(_gc).__name__ == "bytes"; _ledger.append(1)
assert gzip.decompress(_gc) == _gdata; _ledger.append(1)
assert len(_gc) < len(_gdata) * 2; _ledger.append(1)

# 6) bz2 — full module hasattr surface
assert hasattr(bz2, "BZ2File") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Compressor") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Decompressor") == True; _ledger.append(1)
assert hasattr(bz2, "open") == True; _ledger.append(1)
assert hasattr(bz2, "compress") == True; _ledger.append(1)
assert hasattr(bz2, "decompress") == True; _ledger.append(1)

# 7) bz2 — round-trip value contract
_bdata = b"hello world hello world hello world"
_bc = bz2.compress(_bdata)
assert type(_bc).__name__ == "bytes"; _ledger.append(1)
assert bz2.decompress(_bc) == _bdata; _ledger.append(1)

# 8) lzma — partial module hasattr surface
#    (is_check_supported DIVERGES on mamba — moved to spec)
assert hasattr(lzma, "LZMAFile") == True; _ledger.append(1)
assert hasattr(lzma, "LZMACompressor") == True; _ledger.append(1)
assert hasattr(lzma, "LZMADecompressor") == True; _ledger.append(1)
assert hasattr(lzma, "LZMAError") == True; _ledger.append(1)
assert hasattr(lzma, "open") == True; _ledger.append(1)
assert hasattr(lzma, "compress") == True; _ledger.append(1)
assert hasattr(lzma, "decompress") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_NONE") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_CRC32") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_CRC64") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_SHA256") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_ID_MAX") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_UNKNOWN") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_XZ") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_ALONE") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_RAW") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_AUTO") == True; _ledger.append(1)
assert hasattr(lzma, "PRESET_DEFAULT") == True; _ledger.append(1)
assert hasattr(lzma, "PRESET_EXTREME") == True; _ledger.append(1)
assert hasattr(lzma, "MF_HC3") == True; _ledger.append(1)
assert hasattr(lzma, "MF_HC4") == True; _ledger.append(1)
assert hasattr(lzma, "MF_BT2") == True; _ledger.append(1)
assert hasattr(lzma, "MF_BT3") == True; _ledger.append(1)
assert hasattr(lzma, "MF_BT4") == True; _ledger.append(1)
assert hasattr(lzma, "MODE_FAST") == True; _ledger.append(1)
assert hasattr(lzma, "MODE_NORMAL") == True; _ledger.append(1)

# 9) lzma — round-trip + integer-sentinel value contract
_ldata = b"hello world hello world hello world"
_lc = lzma.compress(_ldata)
assert type(_lc).__name__ == "bytes"; _ledger.append(1)
assert lzma.decompress(_lc) == _ldata; _ledger.append(1)
assert lzma.FORMAT_XZ == 1; _ledger.append(1)
assert lzma.CHECK_CRC64 == 4; _ledger.append(1)
assert type(lzma.FORMAT_XZ).__name__ == "int"; _ledger.append(1)

# NB: hasattr(zipfile, "ZipInfo") / "ZIP_BZIP2" /
# "ZIP_LZMA" / "PyZipFile" / "BadZipFile" /
# "BadZipfile" / "LargeZipFile" /
# "ZIP_FILECOUNT_LIMIT" / "ZIP_MAX_COMMENT" / "Path"
# all False on mamba, hasattr(tarfile, "TarFile") /
# "TarInfo" / "TarError" / "ReadError" /
# "CompressionError" / "StreamError" /
# "ExtractError" / "REGTYPE" / "AREGTYPE" / "LNKTYPE"
# / "SYMTYPE" / "DIRTYPE" / "FIFOTYPE" / "CONTTYPE"
# / "CHRTYPE" / "BLKTYPE" / "GNUTYPE_SPARSE" /
# "USTAR_FORMAT" / "GNU_FORMAT" / "PAX_FORMAT" /
# "DEFAULT_FORMAT" / "ENCODING" all False on mamba +
# tarfile.USTAR_FORMAT/GNU_FORMAT/PAX_FORMAT all
# None on mamba, hasattr(gzip, "FNAME") /
# "FCOMMENT" / "FEXTRA" / "FHCRC" / "FTEXT" /
# "READ" / "WRITE" all False on mamba,
# hasattr(lzma, "is_check_supported") False on
# mamba — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_zipfile_tarfile_gzip_bz2_lzma_value_ops {sum(_ledger)} asserts")
