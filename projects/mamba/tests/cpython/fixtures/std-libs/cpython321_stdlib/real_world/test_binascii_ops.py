# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_binascii_ops"
# subject = "cpython321.test_binascii_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_binascii_ops.py"
# status = "filled"
# ///
"""cpython321.test_binascii_ops: execute CPython 3.12 seed test_binascii_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `binascii` hex codec.
# Surface: hexlify / unhexlify and the b2a_hex / a2b_hex aliases
# round-trip cleanly for ASCII byte payloads.
# Companion to stub/test_binascii.py — vendored unittest seed.
import binascii
_ledger: list[int] = []
assert binascii.hexlify(b"abc") == b"616263"; _ledger.append(1)
assert binascii.unhexlify("616263") == b"abc"; _ledger.append(1)
assert binascii.b2a_hex(b"hello") == b"68656c6c6f"; _ledger.append(1)
assert binascii.a2b_hex("68656c6c6f") == b"hello"; _ledger.append(1)
# Round-trip preserves arbitrary ASCII content
payload = b"mamba-binascii-roundtrip"
assert binascii.unhexlify(binascii.hexlify(payload)) == payload; _ledger.append(1)
# Empty input is identity
assert binascii.hexlify(b"") == b""; _ledger.append(1)
assert binascii.unhexlify("") == b""; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_binascii_ops {sum(_ledger)} asserts")
