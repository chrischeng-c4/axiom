# Atomic 308 pass conformance — decimal module (hasattr Decimal only)
# + fractions module (hasattr Fraction + Fraction(1,2).numerator == 1
# + Fraction(1,2).denominator == 2) + io module (hasattr StringIO/
# BytesIO + StringIO().getvalue() == '') + gzip module (hasattr
# GzipFile/open/compress/decompress/BadGzipFile + compress/decompress
# round-trip on b'hello') + bz2 module (hasattr BZ2File/BZ2Compressor/
# BZ2Decompressor/compress/decompress/open + compress/decompress round
# -trip on b'hello') + lzma module (hasattr LZMAFile/LZMACompressor/
# LZMADecompressor/compress/decompress/open/FORMAT_XZ/FORMAT_ALONE/
# CHECK_NONE/CHECK_CRC32/CHECK_CRC64/CHECK_SHA256 + compress/decompress
# round-trip on b'hello').
# All asserts match between CPython 3.12 and mamba.
import decimal
import fractions
import io
import gzip
import bz2
import lzma


_ledger: list[int] = []

# 1) decimal — hasattr Decimal only (conformant subset)
assert hasattr(decimal, "Decimal") == True; _ledger.append(1)

# 2) fractions — hasattr Fraction + numerator/denominator (conformant subset)
assert hasattr(fractions, "Fraction") == True; _ledger.append(1)
assert fractions.Fraction(1, 2).numerator == 1; _ledger.append(1)
assert fractions.Fraction(1, 2).denominator == 2; _ledger.append(1)

# 3) io — hasattr StringIO/BytesIO + StringIO empty getvalue (conformant subset)
assert hasattr(io, "StringIO") == True; _ledger.append(1)
assert hasattr(io, "BytesIO") == True; _ledger.append(1)
assert io.StringIO().getvalue() == ""; _ledger.append(1)

# 4) gzip — hasattr core surface + round-trip
assert hasattr(gzip, "GzipFile") == True; _ledger.append(1)
assert hasattr(gzip, "open") == True; _ledger.append(1)
assert hasattr(gzip, "compress") == True; _ledger.append(1)
assert hasattr(gzip, "decompress") == True; _ledger.append(1)
assert hasattr(gzip, "BadGzipFile") == True; _ledger.append(1)
assert gzip.decompress(gzip.compress(b"hello")) == b"hello"; _ledger.append(1)

# 5) bz2 — hasattr core surface + round-trip
assert hasattr(bz2, "BZ2File") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Compressor") == True; _ledger.append(1)
assert hasattr(bz2, "BZ2Decompressor") == True; _ledger.append(1)
assert hasattr(bz2, "compress") == True; _ledger.append(1)
assert hasattr(bz2, "decompress") == True; _ledger.append(1)
assert hasattr(bz2, "open") == True; _ledger.append(1)
assert bz2.decompress(bz2.compress(b"hello")) == b"hello"; _ledger.append(1)

# 6) lzma — hasattr core surface + round-trip
assert hasattr(lzma, "LZMAFile") == True; _ledger.append(1)
assert hasattr(lzma, "LZMACompressor") == True; _ledger.append(1)
assert hasattr(lzma, "LZMADecompressor") == True; _ledger.append(1)
assert hasattr(lzma, "compress") == True; _ledger.append(1)
assert hasattr(lzma, "decompress") == True; _ledger.append(1)
assert hasattr(lzma, "open") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_XZ") == True; _ledger.append(1)
assert hasattr(lzma, "FORMAT_ALONE") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_NONE") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_CRC32") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_CRC64") == True; _ledger.append(1)
assert hasattr(lzma, "CHECK_SHA256") == True; _ledger.append(1)
assert lzma.decompress(lzma.compress(b"hello")) == b"hello"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_gzip_bz2_lzma_value_ops {sum(_ledger)} asserts")
