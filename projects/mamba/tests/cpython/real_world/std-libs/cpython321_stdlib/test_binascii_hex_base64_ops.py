# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_binascii_hex_base64_ops"
# subject = "cpython321.test_binascii_hex_base64_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_binascii_hex_base64_ops.py"
# status = "filled"
# ///
"""cpython321.test_binascii_hex_base64_ops: execute CPython 3.12 seed test_binascii_hex_base64_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the binascii hex- and
# base64-codec surface. Surface: `binascii.hexlify(b)` returns the
# lowercase ASCII-hex bytes form of b ("abc" -> b"616263",
# b"\xff" -> b"ff"), `binascii.unhexlify(s)` is its inverse and
# accepts both bytes and str, b2a_hex/a2b_hex are aliases for the
# same operation; `binascii.b2a_base64(b)` returns the canonical
# base64 byte-form with a single trailing newline (empty input
# returns b"\n"), `binascii.a2b_base64(s)` is its inverse and
# accepts both a trailing newline and a non-newline-terminated
# form; hexlify/unhexlify and b2a_base64/a2b_base64 round-trip
# across empty / short / NUL-byte / high-byte / long inputs.
import binascii
_ledger: list[int] = []

# hexlify — short / empty / NUL bytes / high bytes / round-trip
assert binascii.hexlify(b"abc") == b"616263"; _ledger.append(1)
assert binascii.hexlify(b"") == b""; _ledger.append(1)
assert binascii.hexlify(b"\x00\x01\x02") == b"000102"; _ledger.append(1)
assert binascii.hexlify(b"\xff") == b"ff"; _ledger.append(1)
assert binascii.hexlify(b"\xab\xcd\xef") == b"abcdef"; _ledger.append(1)
assert binascii.hexlify(b"\x10\x20\x30") == b"102030"; _ledger.append(1)

# Long-input hexlify
assert binascii.hexlify(b"x" * 10) == b"78" * 10; _ledger.append(1)

# unhexlify — bytes inverse
assert binascii.unhexlify(b"616263") == b"abc"; _ledger.append(1)
assert binascii.unhexlify(b"") == b""; _ledger.append(1)
assert binascii.unhexlify(b"000102") == b"\x00\x01\x02"; _ledger.append(1)
assert binascii.unhexlify(b"102030") == b"\x10\x20\x30"; _ledger.append(1)
assert binascii.unhexlify(b"78" * 10) == b"x" * 10; _ledger.append(1)

# unhexlify accepts str argument
assert binascii.unhexlify("616263") == b"abc"; _ledger.append(1)
assert binascii.unhexlify("ff") == b"\xff"; _ledger.append(1)

# Round-trip through hexlify/unhexlify
assert binascii.unhexlify(binascii.hexlify(b"hello")) == b"hello"; _ledger.append(1)
assert binascii.unhexlify(binascii.hexlify(b"\xff\xee\xdd")) == b"\xff\xee\xdd"; _ledger.append(1)

# b2a_hex / a2b_hex aliases
assert binascii.b2a_hex(b"abc") == b"616263"; _ledger.append(1)
assert binascii.a2b_hex(b"616263") == b"abc"; _ledger.append(1)

# base64 — canonical with trailing newline, empty -> b"\n"
assert binascii.b2a_base64(b"abc") == b"YWJj\n"; _ledger.append(1)
assert binascii.b2a_base64(b"") == b"\n"; _ledger.append(1)

# a2b_base64 accepts both newline-terminated and bare form
assert binascii.a2b_base64(b"YWJj\n") == b"abc"; _ledger.append(1)
assert binascii.a2b_base64(b"YWJj") == b"abc"; _ledger.append(1)

# base64 round-trip — short ASCII and NUL bytes
assert binascii.a2b_base64(binascii.b2a_base64(b"hello")) == b"hello"; _ledger.append(1)
assert binascii.a2b_base64(binascii.b2a_base64(b"\x00\x01\x02")) == b"\x00\x01\x02"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_binascii_hex_base64_ops {sum(_ledger)} asserts")
