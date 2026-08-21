# Atomic 293 pass conformance — bz2 module (hasattr compress/
# decompress/BZ2File/BZ2Compressor/BZ2Decompressor/open + compress
# returns bytes + round-trip) + lzma module (hasattr compress/
# decompress/LZMAFile/LZMACompressor/LZMADecompressor/LZMAError/
# FORMAT_XZ/FORMAT_ALONE/FORMAT_RAW/CHECK_NONE/CHECK_CRC32/CHECK_
# SHA256/PRESET_DEFAULT/PRESET_EXTREME + compress returns bytes +
# round-trip) + gzip module (hasattr compress/decompress/GzipFile/
# open/BadGzipFile + compress returns bytes + round-trip) +
# tarfile module (hasattr open/is_tarfile) + zipfile module
# (hasattr ZipFile/is_zipfile/ZIP_STORED/ZIP_DEFLATED +
# ZIP_STORED==0 + ZIP_DEFLATED==8).
# All asserts match between CPython 3.12 and mamba.
import bz2
import lzma
import gzip
import tarfile
import zipfile


_ledger: list[int] = []

# 1) bz2 — hasattr core surface
assert hasattr(bz2, "compress") == True; _ledger.append(1)
assert hasattr(bz2, "decompress") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2File") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Compressor") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Decompressor") == True; _ledger.append(1)
assert hasattr(bz2, "open") == True; _ledger.append(1)

# 2) bz2 — value contracts
assert isinstance(bz2.compress(b"hello"), bytes) == True; _ledger.append(1)
assert bz2.decompress(bz2.compress(b"hello world")) == b"hello world"; _ledger.append(1)

# 3) lzma — hasattr core surface
assert hasattr(lzma, "compress") == True; _ledger.append(1)
assert hasattr(lzma, "decompress") == True; _ledger.append(1)
assert hasattr(lzma, "LZMAFile") == True; _ledger.append(1)
assert hasattr(lzma, "LZMACompressor") == True; _ledger.append(1)
assert hasattr(lzma, "LZMADecompressor") == True; _ledger.append(1)
assert hasattr(lzma, "LZMAError") == True; _ledger.append(1)

# 4) lzma — hasattr format/check constants
assert hasattr(lzma, "FORMAT_XZ") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_ALONE") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_RAW") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_NONE") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_CRC32") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_SHA256") == True; _ledger.append(1)
assert hasattr(lzma, "PRESET_DEFAULT") == True; _ledger.append(1)
assert hasattr(lzma, "PRESET_EXTREME") == True; _ledger.append(1)

# 5) lzma — value contracts
assert isinstance(lzma.compress(b"hello"), bytes) == True; _ledger.append(1)
assert lzma.decompress(lzma.compress(b"hello world")) == b"hello world"; _ledger.append(1)

# 6) gzip — hasattr core surface
assert hasattr(gzip, "compress") == True; _ledger.append(1)
assert hasattr(gzip, "decompress") == True; _ledger.append(1)
assert hasattr(gzip, "GzipFile") == True; _ledger.append(1)
assert hasattr(gzip, "open") == True; _ledger.append(1)
assert hasattr(gzip, "BadGzipFile") == True; _ledger.append(1)

# 7) gzip — value contracts
assert isinstance(gzip.compress(b"hello"), bytes) == True; _ledger.append(1)
assert gzip.decompress(gzip.compress(b"hello world")) == b"hello world"; _ledger.append(1)

# 8) tarfile — hasattr conformant surface
assert hasattr(tarfile, "open") == True; _ledger.append(1)
assert hasattr(tarfile, "is_tarfile") == True; _ledger.append(1)

# 9) zipfile — hasattr conformant surface
assert hasattr(zipfile, "ZipFile") == True; _ledger.append(1)
assert hasattr(zipfile, "is_zipfile") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_STORED") == True; _ledger.append(1)
assert hasattr(zipfile, "ZIP_DEFLATED") == True; _ledger.append(1)

# 10) zipfile — constant values
assert zipfile.ZIP_STORED == 0; _ledger.append(1)
assert zipfile.ZIP_DEFLATED == 8; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_bz2_lzma_gzip_tarfile_zipfile_value_ops {sum(_ledger)} asserts")
