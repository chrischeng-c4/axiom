# Operational AssertionPass seed for the value contract of the
# `gzip` / `zlib` / `bz2` / `lzma` / `zipfile` five-pack pinned
# to atomic 195: `gzip` (the documented partial module-level
# helper hasattr surface — `compress` / `decompress` / `open`
# / `GzipFile` / `BadGzipFile` + the documented gzip.compress
# bytes-returning + gzip.compress/decompress round-trip value
# contract), `zlib` (the documented partial module-level
# helper hasattr surface — `compress` / `decompress` / `crc32`
# / `adler32` + the documented zlib.crc32(b"hello") ==
# 907060870 integer-value contract + the documented
# zlib.compress bytes-returning + zlib.compress/decompress
# round-trip value contract), `bz2` (the documented full
# module-level helper hasattr surface — `compress` /
# `decompress` / `open` / `BZ2File` / `BZ2Compressor` /
# `BZ2Decompressor` + the documented bz2.compress bytes-
# returning + bz2.compress/decompress round-trip value
# contract), `lzma` (the documented full module-level helper
# hasattr surface minus `is_check_supported` — `compress` /
# `decompress` / `open` / `LZMAFile` / `LZMACompressor` /
# `LZMADecompressor` / `FORMAT_XZ` / `FORMAT_ALONE` /
# `FORMAT_RAW` / `FORMAT_AUTO` / `CHECK_NONE` / `CHECK_CRC32`
# / `CHECK_CRC64` / `CHECK_SHA256` / `PRESET_DEFAULT` /
# `PRESET_EXTREME` / `MF_HC3` / `MF_HC4` / `MF_BT2` /
# `MF_BT3` / `MF_BT4` / `MODE_FAST` / `MODE_NORMAL` /
# `LZMAError` + the documented lzma.compress bytes-returning
# + lzma.compress/decompress round-trip value contract), and
# `zipfile` (the documented partial module-level helper
# hasattr surface — `ZipFile` / `ZIP_STORED` / `ZIP_DEFLATED`
# / `is_zipfile` + the documented ZIP_STORED == 0 /
# ZIP_DEFLATED == 8 integer-value contract).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(gzip, "READ") / "WRITE" False on mamba,
# hasattr(zlib, "compressobj") / "decompressobj" /
# "Z_BEST_COMPRESSION" / "Z_BEST_SPEED" /
# "Z_DEFAULT_COMPRESSION" / "Z_NO_COMPRESSION" / "Z_FINISH"
# / "Z_FULL_FLUSH" / "Z_SYNC_FLUSH" / "Z_NO_FLUSH" /
# "Z_PARTIAL_FLUSH" / "DEF_BUF_SIZE" / "DEFLATED" /
# "MAX_WBITS" / "error" all False on mamba,
# hasattr(lzma, "is_check_supported") False on mamba,
# hasattr(zipfile, "ZipInfo") / "ZIP_BZIP2" / "ZIP_LZMA" /
# "BadZipFile" / "BadZipfile" / "LargeZipFile" / "Path" all
# False on mamba) are covered in the matching spec fixture
# `lang_gzip_zlib_zipfile_silent`.
import gzip
import zlib
import bz2
import lzma
import zipfile


_ledger: list[int] = []

# 1) gzip — partial module hasattr surface
#    (READ / WRITE DIVERGE — moved to spec fixture)
assert hasattr(gzip, "compress") == True; _ledger.append(1)
assert hasattr(gzip, "decompress") == True; _ledger.append(1)
assert hasattr(gzip, "open") == True; _ledger.append(1)
assert hasattr(gzip, "GzipFile") == True; _ledger.append(1)
assert hasattr(gzip, "BadGzipFile") == True; _ledger.append(1)

# 2) gzip — bytes-returning compress + round-trip
_gz = gzip.compress(b"hello world")
assert type(_gz).__name__ == "bytes"; _ledger.append(1)
assert gzip.decompress(_gz) == b"hello world"; _ledger.append(1)

# 3) zlib — partial module hasattr surface
#    (compressobj / decompressobj / Z_BEST_COMPRESSION /
#    Z_BEST_SPEED / Z_DEFAULT_COMPRESSION / Z_NO_COMPRESSION
#    / Z_FINISH / Z_FULL_FLUSH / Z_SYNC_FLUSH / Z_NO_FLUSH /
#    Z_PARTIAL_FLUSH / DEF_BUF_SIZE / DEFLATED / MAX_WBITS /
#    error DIVERGE — moved to spec fixture)
assert hasattr(zlib, "compress") == True; _ledger.append(1)
assert hasattr(zlib, "decompress") == True; _ledger.append(1)
assert hasattr(zlib, "crc32") == True; _ledger.append(1)
assert hasattr(zlib, "adler32") == True; _ledger.append(1)

# 4) zlib — integer-value contract + bytes-returning compress + round-trip
assert zlib.crc32(b"hello") == 907060870; _ledger.append(1)
_zl = zlib.compress(b"hello world")
assert type(_zl).__name__ == "bytes"; _ledger.append(1)
assert zlib.decompress(_zl) == b"hello world"; _ledger.append(1)

# 5) bz2 — full module hasattr surface
assert hasattr(bz2, "compress") == True; _ledger.append(1)
assert hasattr(bz2, "decompress") == True; _ledger.append(1)
assert hasattr(bz2, "open") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2File") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Compressor") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Decompressor") == True; _ledger.append(1)

# 6) bz2 — bytes-returning compress + round-trip
_bz = bz2.compress(b"hello world")
assert type(_bz).__name__ == "bytes"; _ledger.append(1)
assert bz2.decompress(_bz) == b"hello world"; _ledger.append(1)

# 7) lzma — full module hasattr surface minus is_check_supported
#    (is_check_supported DIVERGES — moved to spec fixture)
assert hasattr(lzma, "compress") == True; _ledger.append(1)
assert hasattr(lzma, "decompress") == True; _ledger.append(1)
assert hasattr(lzma, "open") == True; _ledger.append(1)
assert hasattr(lzma, "LZMAFile") == True; _ledger.append(1)
assert hasattr(lzma, "LZMACompressor") == True; _ledger.append(1)
assert hasattr(lzma, "LZMADecompressor") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_XZ") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_ALONE") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_RAW") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_AUTO") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_NONE") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_CRC32") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_CRC64") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_SHA256") == True; _ledger.append(1)
assert hasattr(lzma, "PRESET_DEFAULT") == True; _ledger.append(1)
assert hasattr(lzma, "PRESET_EXTREME") == True; _ledger.append(1)
assert hasattr(lzma, "MF_HC3") == True; _ledger.append(1)
assert hasattr(lzma, "MF_HC4") == True; _ledger.append(1)
assert hasattr(lzma, "MF_BT2") == True; _ledger.append(1)
assert hasattr(lzma, "MF_BT3") == True; _ledger.append(1)
assert hasattr(lzma, "MF_BT4") == True; _ledger.append(1)
assert hasattr(lzma, "MODE_FAST") == True; _ledger.append(1)
assert hasattr(lzma, "MODE_NORMAL") == True; _ledger.append(1)
assert hasattr(lzma, "LZMAError") == True; _ledger.append(1)

# 8) lzma — bytes-returning compress + round-trip
_lz = lzma.compress(b"hello world")
assert type(_lz).__name__ == "bytes"; _ledger.append(1)
assert lzma.decompress(_lz) == b"hello world"; _ledger.append(1)

# 9) zipfile — partial module hasattr surface
#    (ZipInfo / ZIP_BZIP2 / ZIP_LZMA / BadZipFile / BadZipfile
#    / LargeZipFile / Path DIVERGE — moved to spec fixture)
assert hasattr(zipfile, "ZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_STORED") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_DEFLATED") == True; _ledger.append(1)
assert hasattr(zipfile, "is_zipfile") == True; _ledger.append(1)

# 10) zipfile — integer-value contract
assert zipfile.ZIP_STORED == 0; _ledger.append(1)
assert zipfile.ZIP_DEFLATED == 8; _ledger.append(1)

# NB: hasattr(gzip, "READ") / "WRITE" False on mamba,
# hasattr(zlib, "compressobj") / "decompressobj" /
# "Z_BEST_COMPRESSION" / "Z_BEST_SPEED" /
# "Z_DEFAULT_COMPRESSION" / "Z_NO_COMPRESSION" / "Z_FINISH"
# / "Z_FULL_FLUSH" / "Z_SYNC_FLUSH" / "Z_NO_FLUSH" /
# "Z_PARTIAL_FLUSH" / "DEF_BUF_SIZE" / "DEFLATED" /
# "MAX_WBITS" / "error" all False on mamba,
# hasattr(lzma, "is_check_supported") False on mamba,
# hasattr(zipfile, "ZipInfo") / "ZIP_BZIP2" / "ZIP_LZMA" /
# "BadZipFile" / "BadZipfile" / "LargeZipFile" / "Path" all
# False on mamba — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_gzip_zlib_bz2_lzma_zipfile_value_ops {sum(_ledger)} asserts")
