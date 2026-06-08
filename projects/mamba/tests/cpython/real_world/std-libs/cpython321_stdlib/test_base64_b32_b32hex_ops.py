# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_base64_b32_b32hex_ops"
# subject = "cpython321.test_base64_b32_b32hex_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_base64_b32_b32hex_ops.py"
# status = "filled"
# ///
"""cpython321.test_base64_b32_b32hex_ops: execute CPython 3.12 seed test_base64_b32_b32hex_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the base32 surface of the
# `base64` stdlib module — the RFC 4648 base32 alphabet
# (`A-Z2-7`) and the extended base32-hex alphabet (`0-9A-V`).
# Used by tooling that exchanges binary payloads in case-
# insensitive contexts (DNS-safe identifiers, OTP secrets like
# TOTP / HOTP keys, filename-safe bytes), since base32 is the
# only RFC 4648 form that survives case folding. Complementary
# to the b16 / b64 / urlsafe-b64 coverage in
# `test_base64_*_ops.py`; this seed pins the matching subset
# between mamba and CPython on the b32encode / b32decode /
# b32hexencode / b32hexdecode round-trip, padding length,
# casefold flag, and the symmetric inverse property.
#
# Surface:
#   • base64.b32encode(s: bytes) → bytes
#       — RFC 4648 base32 alphabet (A-Z, 2-7);
#       — output length is always a multiple of 8 (padded with =);
#       — b32encode(b'') → b'';
#   • base64.b32decode(s: bytes, casefold=False) → bytes
#       — inverse of b32encode;
#       — `casefold=True` accepts lowercase input;
#       — b32decode(b'') → b'';
#   • base64.b32hexencode(s: bytes) → bytes
#       — extended base32-hex alphabet (0-9, A-V);
#       — preserves byte-ordering;
#   • base64.b32hexdecode(s: bytes) → bytes
#       — inverse of b32hexencode;
#   • output is always `bytes`.
import base64
_ledger: list[int] = []

# b32encode — exact output values
assert base64.b32encode(b'') == b''; _ledger.append(1)
assert base64.b32encode(b'a') == b'ME======'; _ledger.append(1)
assert base64.b32encode(b'ab') == b'MFRA===='; _ledger.append(1)
assert base64.b32encode(b'abc') == b'MFRGG==='; _ledger.append(1)
assert base64.b32encode(b'abcd') == b'MFRGGZA='; _ledger.append(1)
assert base64.b32encode(b'abcde') == b'MFRGGZDF'; _ledger.append(1)
assert base64.b32encode(b'hello') == b'NBSWY3DP'; _ledger.append(1)
assert base64.b32encode(b'hello world') == b'NBSWY3DPEB3W64TMMQ======'; _ledger.append(1)

# b32encode — output type is bytes
assert isinstance(base64.b32encode(b'hello'), bytes); _ledger.append(1)
assert isinstance(base64.b32encode(b''), bytes); _ledger.append(1)

# b32encode — output length is multiple of 8
for _data in [b'', b'a', b'ab', b'abc', b'abcd', b'abcde',
              b'abcdef', b'abcdefg', b'abcdefgh',
              b'hello', b'hello world', b'X' * 100]:
    assert len(base64.b32encode(_data)) % 8 == 0; _ledger.append(1)

# b32decode — exact output values
assert base64.b32decode(b'') == b''; _ledger.append(1)
assert base64.b32decode(b'ME======') == b'a'; _ledger.append(1)
assert base64.b32decode(b'MFRA====') == b'ab'; _ledger.append(1)
assert base64.b32decode(b'MFRGG===') == b'abc'; _ledger.append(1)
assert base64.b32decode(b'MFRGGZA=') == b'abcd'; _ledger.append(1)
assert base64.b32decode(b'MFRGGZDF') == b'abcde'; _ledger.append(1)
assert base64.b32decode(b'NBSWY3DP') == b'hello'; _ledger.append(1)

# b32decode — output type is bytes
assert isinstance(base64.b32decode(b'NBSWY3DP'), bytes); _ledger.append(1)
assert isinstance(base64.b32decode(b''), bytes); _ledger.append(1)

# b32encode / b32decode — round-trip invariant
for _data in [b'', b'a', b'ab', b'abc', b'abcd', b'abcde',
              b'\x00\x01\x02\x03\x04', b'\xff\xfe\xfd',
              b'hello world', b'the quick brown fox',
              b'X' * 50, b'X' * 100]:
    assert base64.b32decode(base64.b32encode(_data)) == _data; _ledger.append(1)

# b32decode — casefold accepts lowercase
assert base64.b32decode(b'nbswy3dp', casefold=True) == b'hello'; _ledger.append(1)
assert base64.b32decode(b'mfrggzdf', casefold=True) == b'abcde'; _ledger.append(1)
assert base64.b32decode(b'me======', casefold=True) == b'a'; _ledger.append(1)

# b32hexencode — exact output values
assert base64.b32hexencode(b'hello') == b'D1IMOR3F'; _ledger.append(1)

# b32hexencode — output type / length
assert isinstance(base64.b32hexencode(b'hello'), bytes); _ledger.append(1)
for _data in [b'a', b'ab', b'abc', b'abcd', b'abcde', b'hello',
              b'X' * 10, b'X' * 20]:
    assert len(base64.b32hexencode(_data)) % 8 == 0; _ledger.append(1)

# b32hexdecode — inverse
assert base64.b32hexdecode(b'D1IMOR3F') == b'hello'; _ledger.append(1)
assert isinstance(base64.b32hexdecode(b'D1IMOR3F'), bytes); _ledger.append(1)

# b32hexencode / b32hexdecode — round-trip invariant
for _data in [b'a', b'ab', b'abc', b'abcd', b'abcde',
              b'hello', b'hello world', b'X' * 50]:
    assert base64.b32hexdecode(base64.b32hexencode(_data)) == _data; _ledger.append(1)

# Cross-check — b32encode and b32hexencode produce different output
# for non-trivial input (they use different alphabets)
assert base64.b32encode(b'hello') != base64.b32hexencode(b'hello'); _ledger.append(1)
assert base64.b32encode(b'abc') != base64.b32hexencode(b'abc'); _ledger.append(1)

# b32encode preserves the RFC 4648 alphabet (A-Z, 2-7, =) — decode
# back to bytes-via-int and check membership in the allowed set
_b32_alphabet = b'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567='
_encoded = base64.b32encode(b'hello world')
for _i in range(len(_encoded)):
    assert _encoded[_i] in _b32_alphabet; _ledger.append(1)

# b32hexencode preserves the RFC 4648 base32hex alphabet (0-9, A-V, =)
_b32hex_alphabet = b'0123456789ABCDEFGHIJKLMNOPQRSTUV='
_hex_encoded = base64.b32hexencode(b'hello world')
for _i in range(len(_hex_encoded)):
    assert _hex_encoded[_i] in _b32hex_alphabet; _ledger.append(1)

# Empty input — invariants
assert base64.b32encode(b'') == b''; _ledger.append(1)
assert base64.b32decode(b'') == b''; _ledger.append(1)
assert base64.b32hexencode(b'') == b''; _ledger.append(1)
assert base64.b32hexdecode(b'') == b''; _ledger.append(1)

# Large input round-trip
_big = b'\x42' * 1000
assert base64.b32decode(base64.b32encode(_big)) == _big; _ledger.append(1)
assert base64.b32hexdecode(base64.b32hexencode(_big)) == _big; _ledger.append(1)

# Module-level attribute discipline
for _name in ['b32encode', 'b32decode', 'b32hexencode', 'b32hexdecode']:
    assert hasattr(base64, _name); _ledger.append(1)
    assert callable(getattr(base64, _name)); _ledger.append(1)

# Module name discipline
assert base64.__name__ == 'base64'; _ledger.append(1)

# b32 length contracts — 5 bytes → 8 chars, 10 bytes → 16 chars
assert len(base64.b32encode(b'\x00' * 5)) == 8; _ledger.append(1)
assert len(base64.b32encode(b'\x00' * 10)) == 16; _ledger.append(1)
assert len(base64.b32encode(b'\x00' * 15)) == 24; _ledger.append(1)
assert len(base64.b32hexencode(b'\x00' * 5)) == 8; _ledger.append(1)
assert len(base64.b32hexencode(b'\x00' * 10)) == 16; _ledger.append(1)

# Idempotence — same input, same output
assert base64.b32encode(b'hello') == base64.b32encode(b'hello'); _ledger.append(1)
assert base64.b32hexencode(b'hello') == base64.b32hexencode(b'hello'); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_base64_b32_b32hex_ops {sum(_ledger)} asserts")
