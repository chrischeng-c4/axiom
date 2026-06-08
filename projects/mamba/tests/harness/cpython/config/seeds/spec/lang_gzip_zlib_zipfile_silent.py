# Operational AssertionPass seed for SILENT divergences across
# the `gzip` extended constant identifier surface + `zlib`
# extended class / constant / exception identifier surface +
# `lzma` extended helper surface + `zipfile` extended class
# / constant / exception identifier surface pinned by atomic
# 195: `gzip` (the documented `READ` / `WRITE` extended
# constant identifier surface), `zlib` (the documented
# `compressobj` / `decompressobj` / `Z_BEST_COMPRESSION` /
# `Z_BEST_SPEED` / `Z_DEFAULT_COMPRESSION` /
# `Z_NO_COMPRESSION` / `Z_FINISH` / `Z_FULL_FLUSH` /
# `Z_SYNC_FLUSH` / `Z_NO_FLUSH` / `Z_PARTIAL_FLUSH` /
# `DEF_BUF_SIZE` / `DEFLATED` / `MAX_WBITS` / `error`
# extended function / constant / exception identifier
# surface), `lzma` (the documented `is_check_supported`
# extended function identifier surface), and `zipfile` (the
# documented `ZipInfo` / `ZIP_BZIP2` / `ZIP_LZMA` /
# `BadZipFile` / `BadZipfile` / `LargeZipFile` / `Path`
# extended class / constant / exception identifier surface).
#
# The matching subset (partial gzip hasattr + bytes-returning
# + round-trip, partial zlib hasattr + crc32 value + bytes-
# returning + round-trip, full bz2 hasattr + bytes-returning
# + round-trip, full lzma hasattr minus is_check_supported +
# bytes-returning + round-trip, partial zipfile hasattr +
# ZIP_STORED / ZIP_DEFLATED integer values) is covered by
# `test_gzip_zlib_bz2_lzma_zipfile_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(gzip, "READ") is True — documented constant
#     identifier (mamba: False);
#   • hasattr(gzip, "WRITE") is True — documented constant
#     identifier (mamba: False);
#   • hasattr(zlib, "compressobj") is True — documented
#     function identifier (mamba: False);
#   • hasattr(zlib, "decompressobj") is True — documented
#     function identifier (mamba: False);
#   • hasattr(zlib, "Z_BEST_COMPRESSION") is True —
#     documented constant identifier (mamba: False);
#   • hasattr(zlib, "Z_BEST_SPEED") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(zlib, "Z_DEFAULT_COMPRESSION") is True —
#     documented constant identifier (mamba: False);
#   • hasattr(zlib, "Z_NO_COMPRESSION") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(zlib, "Z_FINISH") is True — documented constant
#     identifier (mamba: False);
#   • hasattr(zlib, "Z_FULL_FLUSH") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(zlib, "Z_SYNC_FLUSH") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(zlib, "Z_NO_FLUSH") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(zlib, "Z_PARTIAL_FLUSH") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(zlib, "DEF_BUF_SIZE") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(zlib, "DEFLATED") is True — documented constant
#     identifier (mamba: False);
#   • hasattr(zlib, "MAX_WBITS") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(zlib, "error") is True — documented exception
#     identifier (mamba: False);
#   • hasattr(lzma, "is_check_supported") is True —
#     documented function identifier (mamba: False);
#   • hasattr(zipfile, "ZipInfo") is True — documented class
#     identifier (mamba: False);
#   • hasattr(zipfile, "ZIP_BZIP2") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(zipfile, "ZIP_LZMA") is True — documented
#     constant identifier (mamba: False);
#   • hasattr(zipfile, "BadZipFile") is True — documented
#     exception identifier (mamba: False);
#   • hasattr(zipfile, "BadZipfile") is True — documented
#     exception identifier alias (mamba: False);
#   • hasattr(zipfile, "LargeZipFile") is True — documented
#     exception identifier (mamba: False);
#   • hasattr(zipfile, "Path") is True — documented class
#     identifier (mamba: False).
import gzip as _gzip_mod
import zlib as _zlib_mod
import lzma as _lzma_mod
import zipfile as _zipfile_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# function / class / constant / exception identifier behavior
# that mamba's bundled type stubs do not surface accurately.
gzip: Any = _gzip_mod
zlib: Any = _zlib_mod
lzma: Any = _lzma_mod
zipfile: Any = _zipfile_mod


_ledger: list[int] = []

# 1) gzip — extended constant identifier surface
assert hasattr(gzip, "READ") == True; _ledger.append(1)
assert hasattr(gzip, "WRITE") == True; _ledger.append(1)

# 2) zlib — extended function / constant / exception surface
assert hasattr(zlib, "compressobj") == True; _ledger.append(1)
assert hasattr(zlib, "decompressobj") == True; _ledger.append(1)
assert hasattr(zlib, "Z_BEST_COMPRESSION") == True; _ledger.append(1)
assert hasattr(zlib, "Z_BEST_SPEED") == True; _ledger.append(1)
assert hasattr(zlib, "Z_DEFAULT_COMPRESSION") == True; _ledger.append(1)
assert hasattr(zlib, "Z_NO_COMPRESSION") == True; _ledger.append(1)
assert hasattr(zlib, "Z_FINISH") == True; _ledger.append(1)
assert hasattr(zlib, "Z_FULL_FLUSH") == True; _ledger.append(1)
assert hasattr(zlib, "Z_SYNC_FLUSH") == True; _ledger.append(1)
assert hasattr(zlib, "Z_NO_FLUSH") == True; _ledger.append(1)
assert hasattr(zlib, "Z_PARTIAL_FLUSH") == True; _ledger.append(1)
assert hasattr(zlib, "DEF_BUF_SIZE") == True; _ledger.append(1)
assert hasattr(zlib, "DEFLATED") == True; _ledger.append(1)
assert hasattr(zlib, "MAX_WBITS") == True; _ledger.append(1)
assert hasattr(zlib, "error") == True; _ledger.append(1)

# 3) lzma — extended function identifier surface
assert hasattr(lzma, "is_check_supported") == True; _ledger.append(1)

# 4) zipfile — extended class / constant / exception surface
assert hasattr(zipfile, "ZipInfo") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_BZIP2") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_LZMA") == True; _ledger.append(1)
assert hasattr(zipfile, "BadZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "BadZipfile") == True; _ledger.append(1)
assert hasattr(zipfile, "LargeZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "Path") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_gzip_zlib_zipfile_silent {sum(_ledger)} asserts")
