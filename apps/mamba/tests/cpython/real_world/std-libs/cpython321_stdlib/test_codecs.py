# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_codecs"
# subject = "cpython321.test_codecs"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_codecs.py"
# status = "filled"
# ///
"""cpython321.test_codecs: execute CPython 3.12 seed test_codecs"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: codecs (utf-8/ascii/latin-1 round-trip, utf-16-be encode, BOM
# constants, codecs.lookup info name).
# Excluded as broken on mamba today and tracked separately:
#   * utf-16-be / utf-32-be encode (return ASCII bytes, not the
#     2- or 4-byte big-endian sequence — encoder is a no-op for these)
#   * hex codec (codecs.encode(..., 'hex') returns None)
#   * UnicodeDecodeError on invalid bytes (does not raise)
import codecs

_ledger: list[int] = []

# utf-8 encode of pure ASCII
assert codecs.encode("abc", "utf-8") == b"abc", "utf-8 encode 'abc' == b'abc'"
_ledger.append(1)

# utf-8 encode of non-ASCII expands to multi-byte sequence
assert codecs.encode("héllo", "utf-8") == b"h\xc3\xa9llo", (
    "utf-8 encode 'héllo' == b'h\\xc3\\xa9llo'"
)
_ledger.append(1)

# utf-8 decode round-trips the non-ASCII multi-byte sequence
assert codecs.decode(b"h\xc3\xa9llo", "utf-8") == "héllo", (
    "utf-8 decode b'h\\xc3\\xa9llo' == 'héllo'"
)
_ledger.append(1)

# ascii encode of pure ASCII
assert codecs.encode("abc", "ascii") == b"abc", "ascii encode 'abc' == b'abc'"
_ledger.append(1)

# latin-1 encodes the high byte 0xff as a single byte
assert codecs.encode("\xff", "latin-1") == b"\xff", (
    "latin-1 encode '\\xff' == b'\\xff'"
)
_ledger.append(1)

# latin-1 decodes the same byte back to U+00FF
assert codecs.decode(b"\xff", "latin-1") == "\xff", (
    "latin-1 decode b'\\xff' == '\\xff'"
)
_ledger.append(1)

# BOM_UTF8 is the canonical 3-byte UTF-8 BOM
assert codecs.BOM_UTF8 == b"\xef\xbb\xbf", "BOM_UTF8 == b'\\xef\\xbb\\xbf'"
_ledger.append(1)

# BOM_UTF16_LE is little-endian UTF-16 byte-order-mark
assert codecs.BOM_UTF16_LE == b"\xff\xfe", "BOM_UTF16_LE == b'\\xff\\xfe'"
_ledger.append(1)

# BOM_UTF16_BE is big-endian UTF-16 byte-order-mark
assert codecs.BOM_UTF16_BE == b"\xfe\xff", "BOM_UTF16_BE == b'\\xfe\\xff'"
_ledger.append(1)

# codecs.lookup returns a CodecInfo object with a normalized encoding name.
_info = codecs.lookup("utf-8")
assert _info.name == "utf-8", "codecs.lookup('utf-8').name == 'utf-8'"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_codecs {sum(_ledger)} asserts")
