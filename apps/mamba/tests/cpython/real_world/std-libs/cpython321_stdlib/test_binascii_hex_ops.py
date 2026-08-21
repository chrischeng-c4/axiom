# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_binascii_hex_ops"
# subject = "cpython321.test_binascii_hex_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_binascii_hex_ops.py"
# status = "filled"
# ///
"""cpython321.test_binascii_hex_ops: execute CPython 3.12 seed test_binascii_hex_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `binascii` module — the
# stdlib hex / binary-ASCII conversion utilities (`hexlify`,
# `unhexlify`, `b2a_hex`, `a2b_hex`). Used by hex-dump utilities,
# binary-data inspectors, hash-value displays, low-level protocol
# encoders, and any code that needs to round-trip between raw
# bytes and hexadecimal-ASCII representations. Surface focuses
# on the matching subset between mamba and CPython on the
# bytes-to-hex / hex-to-bytes round-trip. `binascii.crc32` is
# missing on mamba (AttributeError on the dict-backed module
# object), and `b2a_base64` / `a2b_base64` are excluded — only
# the hex family is exercised here. No fixture coverage yet for
# binascii.
#
# Surface:
#   • binascii.hexlify(b: bytes[, sep: bytes[, bytes_per_sep: int]])
#       → bytes (lowercase hex digits, no separator by default);
#       — empty bytes → empty hex;
#       — `hexlify(b'\xff')` → `b'ff'`;
#   • binascii.unhexlify(s: str|bytes) → bytes
#       — accepts str or bytes;
#       — case-insensitive (`'FF'` and `'ff'` decode the same);
#       — empty input → empty bytes;
#   • binascii.b2a_hex — alias of `hexlify`;
#   • binascii.a2b_hex — alias of `unhexlify`.
import binascii
_ledger: list[int] = []

# hexlify — lowercase hex output for every byte
assert binascii.hexlify(b'\x00') == b'00'; _ledger.append(1)
assert binascii.hexlify(b'\xff') == b'ff'; _ledger.append(1)
assert binascii.hexlify(b'\x00\xff') == b'00ff'; _ledger.append(1)
assert binascii.hexlify(b'hello') == b'68656c6c6f'; _ledger.append(1)
assert binascii.hexlify(b'') == b''; _ledger.append(1)
assert binascii.hexlify(b'\x01\x02\x03\x04') == b'01020304'; _ledger.append(1)
assert binascii.hexlify(b'\x7f') == b'7f'; _ledger.append(1)
assert binascii.hexlify(b'A') == b'41'; _ledger.append(1)
assert binascii.hexlify(b'Z') == b'5a'; _ledger.append(1)
assert binascii.hexlify(bytes([0, 1, 2, 3])) == b'00010203'; _ledger.append(1)

# unhexlify — bytes input
assert binascii.unhexlify(b'00') == b'\x00'; _ledger.append(1)
assert binascii.unhexlify(b'ff') == b'\xff'; _ledger.append(1)
assert binascii.unhexlify(b'00ff') == b'\x00\xff'; _ledger.append(1)
assert binascii.unhexlify(b'48656c6c6f') == b'Hello'; _ledger.append(1)
assert binascii.unhexlify(b'') == b''; _ledger.append(1)
assert binascii.unhexlify(b'01020304') == b'\x01\x02\x03\x04'; _ledger.append(1)
assert binascii.unhexlify(b'7f') == b'\x7f'; _ledger.append(1)
assert binascii.unhexlify(b'41') == b'A'; _ledger.append(1)

# unhexlify — case-insensitive
assert binascii.unhexlify(b'FF') == b'\xff'; _ledger.append(1)
assert binascii.unhexlify(b'AB') == b'\xab'; _ledger.append(1)
assert binascii.unhexlify(b'aB') == b'\xab'; _ledger.append(1)
assert binascii.unhexlify(b'Ff') == b'\xff'; _ledger.append(1)

# unhexlify — accepts str input
assert binascii.unhexlify('00ff') == b'\x00\xff'; _ledger.append(1)
assert binascii.unhexlify('48656c6c6f') == b'Hello'; _ledger.append(1)
assert binascii.unhexlify('') == b''; _ledger.append(1)
assert binascii.unhexlify('FF') == b'\xff'; _ledger.append(1)

# Round-trip — hexlify followed by unhexlify is identity
for _data in [b'', b'\x00', b'\xff', b'hello', b'\x01\x02\x03\x04',
              b'The quick brown fox', b'\x00' * 100, bytes(range(256))]:
    assert binascii.unhexlify(binascii.hexlify(_data)) == _data; _ledger.append(1)

# b2a_hex — alias of hexlify
assert binascii.b2a_hex(b'hi') == b'6869'; _ledger.append(1)
assert binascii.b2a_hex(b'') == b''; _ledger.append(1)
assert binascii.b2a_hex(b'\xff') == b'ff'; _ledger.append(1)
assert binascii.b2a_hex(b'\x00') == b'00'; _ledger.append(1)
assert binascii.b2a_hex(b'A') == b'41'; _ledger.append(1)

# a2b_hex — alias of unhexlify
assert binascii.a2b_hex(b'6869') == b'hi'; _ledger.append(1)
assert binascii.a2b_hex(b'') == b''; _ledger.append(1)
assert binascii.a2b_hex(b'ff') == b'\xff'; _ledger.append(1)
assert binascii.a2b_hex(b'00') == b'\x00'; _ledger.append(1)
assert binascii.a2b_hex(b'41') == b'A'; _ledger.append(1)

# Aliases produce identical output to canonical names
for _data in [b'hello', b'\x00\xff', b'\x01\x02', b'', b'world']:
    assert binascii.b2a_hex(_data) == binascii.hexlify(_data); _ledger.append(1)
for _hex in [b'6869', b'00ff', b'41', b'', b'48656c6c6f']:
    assert binascii.a2b_hex(_hex) == binascii.unhexlify(_hex); _ledger.append(1)

# Output length — hex output is exactly 2× input length
for _data in [b'', b'a', b'ab', b'abc', b'abcd', b'\x00' * 16]:
    assert len(binascii.hexlify(_data)) == 2 * len(_data); _ledger.append(1)

# Output is bytes (not str)
assert isinstance(binascii.hexlify(b'a'), bytes); _ledger.append(1)
assert isinstance(binascii.unhexlify(b'61'), bytes); _ledger.append(1)
assert isinstance(binascii.b2a_hex(b'a'), bytes); _ledger.append(1)
assert isinstance(binascii.a2b_hex(b'61'), bytes); _ledger.append(1)

# Module-level attribute discipline
for _name in ["hexlify", "unhexlify", "b2a_hex", "a2b_hex"]:
    assert hasattr(binascii, _name); _ledger.append(1)
    assert callable(getattr(binascii, _name)); _ledger.append(1)

# Idempotence — same input, same result
assert binascii.hexlify(b'test') == binascii.hexlify(b'test'); _ledger.append(1)
assert binascii.unhexlify(b'74657374') == binascii.unhexlify(b'74657374'); _ledger.append(1)

# Hex output is composed of digit / lowercase-letter byte values
# (codepoints 0x30..0x39 or 0x61..0x66). Each byte is in the valid
# hex-character range — checked numerically rather than via
# `set(b'0123456789abcdef')` membership.
for _data in [b'\x00\x01\x02\x7f\xff', b'hello', b'\x00' * 20]:
    _h = binascii.hexlify(_data)
    for _c in _h:
        _is_digit = 0x30 <= _c <= 0x39
        _is_hexlower = 0x61 <= _c <= 0x66
        assert _is_digit or _is_hexlower; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_binascii_hex_ops {sum(_ledger)} asserts")
