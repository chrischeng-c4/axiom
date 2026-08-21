# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_zlib_binascii_email_value_ops"
# subject = "cpython321.test_zlib_binascii_email_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_zlib_binascii_email_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_zlib_binascii_email_value_ops: execute CPython 3.12 seed test_zlib_binascii_email_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of three
# bootstrap stdlib modules used by every compression / binary-
# transit / email path: `zlib` (the documented `compress` /
# `decompress` round-trip + the `adler32` / `crc32` checksum
# helpers — both fixed against the canonical "hello world"
# payload), `binascii` (the documented hex / base64 encode-
# decode round-trips — hexlify / unhexlify / b2a_base64 /
# a2b_base64 / b2a_hex / a2b_hex), and `email` (the documented
# top-level `message_from_string` / `message_from_bytes`
# attribute surface).
#
# The matching subset between mamba and CPython is the
# compression-round-trip layer + checksum layer + binary-ASCII
# layer + email top-level surface layer:
# zlib.decompress(zlib.compress(b"hello world")) == b"hello
# world"; zlib.adler32(b"hello world") == 436929629;
# zlib.crc32(b"hello world") == 222957957;
# binascii.hexlify(b"hello") == b"68656c6c6f";
# binascii.unhexlify(b"68656c6c6f") == b"hello";
# binascii.b2a_base64(b"hello") == b"aGVsbG8=\n";
# binascii.a2b_base64(b"aGVsbG8=") == b"hello"; hasattr(email,
# "message_from_string") + hasattr(email, "message_from_bytes").
#
# Surface in this fixture:
#   • zlib.compress / decompress — byte round-trip;
#   • zlib.adler32 — Adler-32 checksum on "hello world";
#   • zlib.crc32 — CRC-32 checksum on "hello world";
#   • binascii.hexlify / unhexlify — hex round-trip;
#   • binascii.b2a_base64 / a2b_base64 — base64 round-trip;
#   • binascii.b2a_hex / a2b_hex — hex round-trip (legacy
#     names);
#   • email — hasattr message_from_string / message_from_bytes;
#   • hasattr surface — zlib.compress / decompress / adler32 /
#     crc32, binascii.hexlify / unhexlify.
#
# Behavioral edges that DIVERGE on mamba (email.EmailMessage /
# Message / Parser class identity, EmailMessage().set_content
# AttributeError, string.Template / Formatter class identity +
# substitute AttributeError, copyreg.pickle / constructor /
# __reduce_ex__ / _reconstructor attribute surface,
# pickletools.optimize returning empty string, shelve.Shelf /
# DbfilenameShelf class identity, marshal.dumps returning
# empty bytes + marshal.loads returning None,
# zlib.MAX_WBITS / DEF_BUF_SIZE / Z_DEFAULT_COMPRESSION
# constants, zlib.ZLIB_VERSION class identity, zlib.error
# class identity, binascii.crc32 AttributeError) are covered
# in the matching spec fixture `lang_email_marshal_zlib_silent`.
import zlib
import binascii
import email


_ledger: list[int] = []

# 1) zlib — byte round-trip
_payload = b"hello world"
assert zlib.decompress(zlib.compress(_payload)) == _payload; _ledger.append(1)

# 2) zlib — Adler-32 checksum on canonical payload
assert zlib.adler32(_payload) == 436929629; _ledger.append(1)

# 3) zlib — CRC-32 checksum on canonical payload
assert zlib.crc32(_payload) == 222957957; _ledger.append(1)

# 4) binascii — hex round-trip
assert binascii.hexlify(b"hello") == b"68656c6c6f"; _ledger.append(1)
assert binascii.unhexlify(b"68656c6c6f") == b"hello"; _ledger.append(1)

# 5) binascii — base64 round-trip
assert binascii.b2a_base64(b"hello") == b"aGVsbG8=\n"; _ledger.append(1)
assert binascii.a2b_base64(b"aGVsbG8=") == b"hello"; _ledger.append(1)

# 6) binascii — hex round-trip (legacy names)
assert binascii.b2a_hex(b"abc") == b"616263"; _ledger.append(1)
assert binascii.a2b_hex(b"616263") == b"abc"; _ledger.append(1)

# 7) email — top-level helper attribute surface
assert hasattr(email, "message_from_string"); _ledger.append(1)
assert hasattr(email, "message_from_bytes"); _ledger.append(1)

# 8) hasattr surface — module-level helpers
assert hasattr(zlib, "compress"); _ledger.append(1)
assert hasattr(zlib, "decompress"); _ledger.append(1)
assert hasattr(zlib, "adler32"); _ledger.append(1)
assert hasattr(zlib, "crc32"); _ledger.append(1)
assert hasattr(binascii, "hexlify"); _ledger.append(1)
assert hasattr(binascii, "unhexlify"); _ledger.append(1)
assert hasattr(binascii, "b2a_base64"); _ledger.append(1)
assert hasattr(binascii, "a2b_base64"); _ledger.append(1)

# NB: email.EmailMessage / Message / Parser class identity,
# EmailMessage().set_content AttributeError, string.Template /
# Formatter class identity + substitute AttributeError,
# copyreg.pickle / constructor / __reduce_ex__ /
# _reconstructor attribute surface, pickletools.optimize
# returning empty string, shelve.Shelf / DbfilenameShelf class
# identity, marshal.dumps returning empty bytes +
# marshal.loads returning None, zlib.MAX_WBITS /
# DEF_BUF_SIZE / Z_DEFAULT_COMPRESSION constants,
# zlib.ZLIB_VERSION class identity, zlib.error class identity,
# binascii.crc32 AttributeError all DIVERGE on mamba — moved
# to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_zlib_binascii_email_value_ops {sum(_ledger)} asserts")
