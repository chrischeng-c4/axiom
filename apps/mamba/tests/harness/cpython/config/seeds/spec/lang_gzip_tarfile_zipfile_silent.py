# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(gzip, 'READ')` (the
# documented "gzip exposes the READ mode constant" — mamba returns
# False), `hasattr(gzip, 'WRITE')` (the documented "gzip exposes the
# WRITE mode constant" — mamba returns False), `hasattr(tarfile,
# 'TarFile')` (the documented "tarfile exposes the TarFile class" —
# mamba returns False), `hasattr(tarfile, 'TarInfo')` (the
# documented "tarfile exposes the TarInfo class" — mamba returns
# False), `hasattr(tarfile, 'USTAR_FORMAT')` (the documented
# "tarfile exposes the USTAR_FORMAT format constant" — mamba returns
# False), `tarfile.USTAR_FORMAT == 0` (the documented "USTAR_FORMAT
# is the integer 0 format code" — mamba returns None — attribute
# resolves to None placeholder), `hasattr(zipfile, 'ZipInfo')` (the
# documented "zipfile exposes the ZipInfo class" — mamba returns
# False), `hasattr(zipfile, 'BadZipFile')` (the documented "zipfile
# exposes the BadZipFile exception" — mamba returns False),
# `hasattr(zipfile, 'ZIP_BZIP2')` (the documented "zipfile exposes
# the ZIP_BZIP2 compression constant" — mamba returns False), and
# `zipfile.ZIP_BZIP2 == 12` (the documented "ZIP_BZIP2 is the
# integer 12 compression code" — mamba returns None — attribute
# resolves to None placeholder).
# Ten-pack pinned to atomic 293.
#
# Behavioral edges that CONFORM on mamba (bz2 — hasattr compress/
# decompress/BZ2File/BZ2Compressor/BZ2Decompressor/open + compress
# bytes + round-trip. lzma — hasattr compress/decompress/LZMAFile/
# LZMACompressor/LZMADecompressor/LZMAError/FORMAT_XZ/FORMAT_ALONE/
# FORMAT_RAW/CHECK_NONE/CHECK_CRC32/CHECK_SHA256/PRESET_DEFAULT/
# PRESET_EXTREME + compress bytes + round-trip. gzip — hasattr
# compress/decompress/GzipFile/open/BadGzipFile + compress bytes +
# round-trip. tarfile — hasattr open/is_tarfile. zipfile — hasattr
# ZipFile/is_zipfile/ZIP_STORED/ZIP_DEFLATED + ZIP_STORED==0 +
# ZIP_DEFLATED==8) are covered in the matching pass fixture `test_
# bz2_lzma_gzip_tarfile_zipfile_value_ops`.
import gzip
import tarfile
import zipfile


_ledger: list[int] = []

# 1) hasattr(gzip, 'READ') — gzip READ mode constant
#    (mamba: returns False)
assert hasattr(gzip, "READ") == True; _ledger.append(1)

# 2) hasattr(gzip, 'WRITE') — gzip WRITE mode constant
#    (mamba: returns False)
assert hasattr(gzip, "WRITE") == True; _ledger.append(1)

# 3) hasattr(tarfile, 'TarFile') — TarFile class
#    (mamba: returns False)
assert hasattr(tarfile, "TarFile") == True; _ledger.append(1)

# 4) hasattr(tarfile, 'TarInfo') — TarInfo class
#    (mamba: returns False)
assert hasattr(tarfile, "TarInfo") == True; _ledger.append(1)

# 5) hasattr(tarfile, 'USTAR_FORMAT') — USTAR_FORMAT format constant
#    (mamba: returns False)
assert hasattr(tarfile, "USTAR_FORMAT") == True; _ledger.append(1)

# 6) tarfile.USTAR_FORMAT == 0 — USTAR_FORMAT integer code
#    (mamba: attribute resolves to None placeholder)
assert tarfile.USTAR_FORMAT == 0; _ledger.append(1)

# 7) hasattr(zipfile, 'ZipInfo') — ZipInfo class
#    (mamba: returns False)
assert hasattr(zipfile, "ZipInfo") == True; _ledger.append(1)

# 8) hasattr(zipfile, 'BadZipFile') — BadZipFile exception
#    (mamba: returns False)
assert hasattr(zipfile, "BadZipFile") == True; _ledger.append(1)

# 9) hasattr(zipfile, 'ZIP_BZIP2') — ZIP_BZIP2 compression constant
#    (mamba: returns False)
assert hasattr(zipfile, "ZIP_BZIP2") == True; _ledger.append(1)

# 10) zipfile.ZIP_BZIP2 == 12 — ZIP_BZIP2 integer compression code
#     (mamba: attribute resolves to None placeholder)
assert zipfile.ZIP_BZIP2 == 12; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_gzip_tarfile_zipfile_silent {sum(_ledger)} asserts")
