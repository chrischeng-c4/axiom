# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_binascii_hex_base64_roundtrip_ops"
# subject = "cpython321.test_binascii_hex_base64_roundtrip_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_binascii_hex_base64_roundtrip_ops.py"
# status = "filled"
# ///
"""cpython321.test_binascii_hex_base64_roundtrip_ops: execute CPython 3.12 seed test_binascii_hex_base64_roundtrip_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `binascii` module — the
# stdlib hex / base64 / quoted-printable / uuencode codec primitives
# used by `hashlib` (downstream `hexdigest` helpers), `base64` (which
# wraps `binascii.b2a_base64`), `ssl` (DER<->PEM conversions), and
# protocol parsers (HTTP digest, MIME, SMTP). Surface focuses on the
# round-trip invariants of the four most-used pairs:
# `hexlify`/`unhexlify`, `b2a_hex`/`a2b_hex`, `b2a_base64`/
# `a2b_base64`. Mamba silently drops the separator argument to
# `hexlify(data, sep)` (returns no-separator output regardless), so
# this fixture only exercises the no-separator overload. Mamba also
# omits `binascii.Error` / `binascii.Incomplete` exception classes —
# those would be checked in a spec fixture, not here.  No fixture
# coverage yet for binascii beyond hashlib-adjacent hex output.
#
# Surface:
#   • binascii.hexlify(data: bytes) → bytes
#       — lowercase hex digits, two chars per byte;
#       — `hexlify(b'') == b''`;
#       — `hexlify(b'abc') == b'616263'`;
#   • binascii.unhexlify(hex_str) → bytes
#       — inverse of `hexlify`; accepts bytes or str input;
#       — `unhexlify('') == b''`;
#       — `unhexlify('616263') == b'abc'`;
#       — round-trip invariant: `unhexlify(hexlify(x)) == x` for
#         any bytes x;
#   • binascii.b2a_hex(data: bytes) → bytes
#       — alias for `hexlify` (same surface);
#   • binascii.a2b_hex(hex_str) → bytes
#       — alias for `unhexlify`;
#   • binascii.b2a_base64(data: bytes) → bytes
#       — standard base64 encoding with trailing `\n`;
#       — `b2a_base64(b'hi') == b'aGk=\n'`;
#   • binascii.a2b_base64(b64_bytes) → bytes
#       — inverse of `b2a_base64`;
#       — round-trip invariant: `a2b_base64(b2a_base64(x)) == x`.
import binascii
_ledger: list[int] = []

# hexlify — lowercase, two chars per byte
assert binascii.hexlify(b"") == b""; _ledger.append(1)
assert binascii.hexlify(b"a") == b"61"; _ledger.append(1)
assert binascii.hexlify(b"abc") == b"616263"; _ledger.append(1)
assert binascii.hexlify(b"hello") == b"68656c6c6f"; _ledger.append(1)
assert binascii.hexlify(b"\x00\x01\xff") == b"0001ff"; _ledger.append(1)
assert binascii.hexlify(b"\xde\xad\xbe\xef") == b"deadbeef"; _ledger.append(1)

# unhexlify — inverse, accepts str or bytes
assert binascii.unhexlify("") == b""; _ledger.append(1)
assert binascii.unhexlify("61") == b"a"; _ledger.append(1)
assert binascii.unhexlify("616263") == b"abc"; _ledger.append(1)
assert binascii.unhexlify("68656c6c6f") == b"hello"; _ledger.append(1)
assert binascii.unhexlify("0001ff") == b"\x00\x01\xff"; _ledger.append(1)
assert binascii.unhexlify("deadbeef") == b"\xde\xad\xbe\xef"; _ledger.append(1)
assert binascii.unhexlify(b"616263") == b"abc"; _ledger.append(1)

# b2a_hex — alias for hexlify
assert binascii.b2a_hex(b"") == b""; _ledger.append(1)
assert binascii.b2a_hex(b"abc") == b"616263"; _ledger.append(1)
assert binascii.b2a_hex(b"\x00\x01\xff") == b"0001ff"; _ledger.append(1)

# a2b_hex — alias for unhexlify
assert binascii.a2b_hex("") == b""; _ledger.append(1)
assert binascii.a2b_hex("616263") == b"abc"; _ledger.append(1)
assert binascii.a2b_hex(b"6869") == b"hi"; _ledger.append(1)

# Round-trip — hexlify/unhexlify across diverse byte payloads
_inputs = [
    b"",
    b"a",
    b"hello",
    b"The quick brown fox",
    b"\x00\x01\x02\x03",
    b"\xff\xfe\xfd\xfc",
    bytes(range(256)),
    b"\x00" * 100,
    b"\xff" * 100,
]
for _x in _inputs:
    assert binascii.unhexlify(binascii.hexlify(_x)) == _x; _ledger.append(1)
    assert binascii.a2b_hex(binascii.b2a_hex(_x)) == _x; _ledger.append(1)

# b2a_base64 — standard base64, trailing newline
assert binascii.b2a_base64(b"") == b"\n"; _ledger.append(1)
assert binascii.b2a_base64(b"hi") == b"aGk=\n"; _ledger.append(1)
assert binascii.b2a_base64(b"abc") == b"YWJj\n"; _ledger.append(1)
assert binascii.b2a_base64(b"hello world") == b"aGVsbG8gd29ybGQ=\n"; _ledger.append(1)

# a2b_base64 — inverse
assert binascii.a2b_base64(b"") == b""; _ledger.append(1)
assert binascii.a2b_base64(b"aGk=\n") == b"hi"; _ledger.append(1)
assert binascii.a2b_base64(b"YWJj\n") == b"abc"; _ledger.append(1)
assert binascii.a2b_base64(b"aGVsbG8gd29ybGQ=\n") == b"hello world"; _ledger.append(1)

# Round-trip — b2a_base64/a2b_base64 across diverse byte payloads
for _x in _inputs:
    assert binascii.a2b_base64(binascii.b2a_base64(_x)) == _x; _ledger.append(1)

# Return type discipline — hexlify/unhexlify always bytes
assert isinstance(binascii.hexlify(b"abc"), bytes); _ledger.append(1)
assert isinstance(binascii.unhexlify("616263"), bytes); _ledger.append(1)
assert isinstance(binascii.b2a_hex(b"abc"), bytes); _ledger.append(1)
assert isinstance(binascii.a2b_hex("616263"), bytes); _ledger.append(1)
assert isinstance(binascii.b2a_base64(b"hi"), bytes); _ledger.append(1)
assert isinstance(binascii.a2b_base64(b"aGk=\n"), bytes); _ledger.append(1)

# hexlify output is always lowercase ascii hex
_hx = binascii.hexlify(b"\xab\xcd\xef")
assert _hx == b"abcdef"; _ledger.append(1)
# Every byte in hexlify output is in [0-9a-f]
for _ch in binascii.hexlify(bytes(range(256))):
    assert _ch in b"0123456789abcdef"; _ledger.append(1)

# Length invariants
assert len(binascii.hexlify(b"abc")) == 6; _ledger.append(1)
assert len(binascii.hexlify(b"")) == 0; _ledger.append(1)
assert len(binascii.hexlify(bytes(range(256)))) == 512; _ledger.append(1)

# Idempotent — same input, same output
assert binascii.hexlify(b"abc") == binascii.hexlify(b"abc"); _ledger.append(1)
assert binascii.unhexlify("616263") == binascii.unhexlify("616263"); _ledger.append(1)
assert binascii.b2a_base64(b"hi") == binascii.b2a_base64(b"hi"); _ledger.append(1)
assert binascii.a2b_base64(b"aGk=\n") == binascii.a2b_base64(b"aGk=\n"); _ledger.append(1)

# Uppercase / mixed-case hex input for unhexlify also works
assert binascii.unhexlify("ABCDEF") == b"\xab\xcd\xef"; _ledger.append(1)
assert binascii.unhexlify("AbCdEf") == b"\xab\xcd\xef"; _ledger.append(1)

# All-bytes round-trip — full domain coverage
_all = bytes(range(256))
assert binascii.unhexlify(binascii.hexlify(_all)) == _all; _ledger.append(1)
assert binascii.a2b_base64(binascii.b2a_base64(_all)) == _all; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_binascii_hex_base64_roundtrip_ops {sum(_ledger)} asserts")
