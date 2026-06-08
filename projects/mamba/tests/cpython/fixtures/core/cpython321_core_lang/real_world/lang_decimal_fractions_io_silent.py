# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_decimal_fractions_io_silent"
# subject = "cpython321.lang_decimal_fractions_io_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_decimal_fractions_io_silent.py"
# status = "filled"
# ///
"""cpython321.lang_decimal_fractions_io_silent: execute CPython 3.12 seed lang_decimal_fractions_io_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `type(decimal.Decimal('1')).__name__
# == 'Decimal'` (the documented "decimal.Decimal constructs a Decimal
# instance" — mamba returns 'int' — constructor degrades to integer
# handle), `decimal.Decimal('1') == 1` (the documented "Decimal('1')
# compares equal to int 1" — mamba returns False — integer-handle
# equality is broken), `str(decimal.Decimal('1.50')) == '1.50'` (the
# documented "str(Decimal('1.50')) preserves trailing zero" — mamba
# returns '70368744177671' — handle id leaks through __str__), `
# hasattr(decimal, 'getcontext')` (the documented "decimal exposes the
# getcontext helper" — mamba returns False), `hasattr(decimal, '
# ROUND_HALF_EVEN')` (the documented "decimal exposes the
# ROUND_HALF_EVEN constant" — mamba returns False), `type(fractions.
# Fraction(1, 2)).__name__ == 'Fraction'` (the documented "fractions.
# Fraction(1, 2) constructs a Fraction instance" — mamba returns 'int'
# — constructor degrades to integer handle), `fractions.Fraction(1, 2)
# == 0.5` (the documented "Fraction(1, 2) compares equal to float 0.5
# " — mamba returns False — integer-handle equality is broken), `str(
# fractions.Fraction(1, 2)) == '1/2'` (the documented "str(Fraction(1,
# 2)) returns '1/2'" — mamba returns '1099511627784' — handle id leaks
# through __str__), `type(io.StringIO()).__name__ == 'StringIO'` (the
# documented "io.StringIO() constructs a StringIO instance" — mamba
# returns 'dict' — constructor degrades to plain dict), and `io.
# StringIO('abc').read() == 'abc'` (the documented "StringIO('abc').
# read() returns the initial-value string" — mamba returns '' — read
# loses the seed value).
# Ten-pack pinned to atomic 308.
#
# Behavioral edges that CONFORM on mamba (decimal — hasattr Decimal
# only. fractions — hasattr Fraction + Fraction(1,2).numerator/
# denominator. io — hasattr StringIO/BytesIO + StringIO().getvalue()
# empty. gzip — hasattr GzipFile/open/compress/decompress/BadGzipFile
# + compress/decompress round-trip on b'hello'. bz2 — hasattr BZ2File/
# BZ2Compressor/BZ2Decompressor/compress/decompress/open + round-trip
# on b'hello'. lzma — hasattr LZMAFile/LZMACompressor/LZMADecompressor
# /compress/decompress/open/FORMAT_XZ/FORMAT_ALONE/CHECK_NONE/CHECK_
# CRC32/CHECK_CRC64/CHECK_SHA256 + round-trip on b'hello') are covered
# in the matching pass fixture `test_gzip_bz2_lzma_value_ops`.
import decimal
import fractions
import io


_ledger: list[int] = []

# 1) type(decimal.Decimal('1')).__name__ == 'Decimal' — Decimal instance
#    (mamba: returns 'int' — constructor degrades to integer handle)
assert type(decimal.Decimal("1")).__name__ == "Decimal"; _ledger.append(1)

# 2) decimal.Decimal('1') == 1 — int equality
#    (mamba: returns False — integer-handle equality is broken)
assert (decimal.Decimal("1") == 1) == True; _ledger.append(1)

# 3) str(decimal.Decimal('1.50')) == '1.50' — preserved trailing zero
#    (mamba: returns '70368744177671' — handle id leaks through __str__)
assert str(decimal.Decimal("1.50")) == "1.50"; _ledger.append(1)

# 4) hasattr(decimal, 'getcontext') — getcontext helper
#    (mamba: returns False)
assert hasattr(decimal, "getcontext") == True; _ledger.append(1)

# 5) hasattr(decimal, 'ROUND_HALF_EVEN') — ROUND_HALF_EVEN constant
#    (mamba: returns False)
assert hasattr(decimal, "ROUND_HALF_EVEN") == True; _ledger.append(1)

# 6) type(fractions.Fraction(1, 2)).__name__ == 'Fraction' — Fraction instance
#    (mamba: returns 'int' — constructor degrades to integer handle)
assert type(fractions.Fraction(1, 2)).__name__ == "Fraction"; _ledger.append(1)

# 7) fractions.Fraction(1, 2) == 0.5 — float equality
#    (mamba: returns False — integer-handle equality is broken)
assert (fractions.Fraction(1, 2) == 0.5) == True; _ledger.append(1)

# 8) str(fractions.Fraction(1, 2)) == '1/2' — slash notation
#    (mamba: returns '1099511627784' — handle id leaks through __str__)
assert str(fractions.Fraction(1, 2)) == "1/2"; _ledger.append(1)

# 9) type(io.StringIO()).__name__ == 'StringIO' — StringIO instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(io.StringIO()).__name__ == "StringIO"; _ledger.append(1)

# 10) io.StringIO('abc').read() == 'abc' — initial-value read
#     (mamba: returns '' — read loses the seed value)
assert io.StringIO("abc").read() == "abc"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_decimal_fractions_io_silent {sum(_ledger)} asserts")
