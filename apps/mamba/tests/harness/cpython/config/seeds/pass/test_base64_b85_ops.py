# Operational AssertionPass seed for `base64.b85encode` and
# `base64.b85decode` (RFC 1924 / Z85-style base-85 encoding).
# Existing base64 seeds (test_base64_ops, test_base64_a85_roundtrip_ops,
# test_binascii_hex_base64_ops) cover b16 / b32 / b64 / a85, but skip
# b85. mamba 0.3.60 already produces byte-identical output to CPython
# on every probed b85 form (empty bytes, 1/3/4/5-byte payloads, the
# common 'hello world' fixture).
#
# Surface:
#   • b85encode round-trips an empty bytes payload to empty bytes;
#   • b85encode(b'abc') → b'VPaz' — the canonical short-string example;
#   • b85encode(b'\x00\x00\x00\x00') → b'00000' — all-zero 4-byte
#     block encodes as the '0' character five times;
#   • b85encode(b'\xff') → b'{{' — single-byte payload pads to two
#     base-85 chars;
#   • b85encode(b'\x01\x02\x03\x04\x05') → b'0RjUA1p' — 5-byte
#     payload spans a block boundary;
#   • b85encode(b'hello world') → b'Xk~0{Zy<MXa%^M' — multi-block
#     payload;
#   • b85decode is the inverse of b85encode on every payload above;
#   • full encode→decode round-trip on a 10-byte digit payload.
import base64
_ledger: list[int] = []

# Empty bytes encodes to empty bytes
assert base64.b85encode(b'') == b''; _ledger.append(1)
assert base64.b85decode(b'') == b''; _ledger.append(1)

# Canonical short-string fixture — b85encode(b'abc') == b'VPaz'
assert base64.b85encode(b'abc') == b'VPaz'; _ledger.append(1)
assert base64.b85decode(b'VPaz') == b'abc'; _ledger.append(1)

# All-zero 4-byte block — '0' is the lowest base-85 digit, so a
# 32-bit zero word encodes to five '0' characters
assert base64.b85encode(b'\x00\x00\x00\x00') == b'00000'; _ledger.append(1)
assert base64.b85decode(b'00000') == b'\x00\x00\x00\x00'; _ledger.append(1)

# Single-byte (\xff) payload — needs two output characters to encode
assert base64.b85encode(b'\xff') == b'{{'; _ledger.append(1)
assert base64.b85decode(b'{{') == b'\xff'; _ledger.append(1)

# 5-byte payload spans a 4-byte block boundary, producing 7 output
# chars (5 for the full block + 2 for the trailing partial-block byte)
assert base64.b85encode(b'\x01\x02\x03\x04\x05') == b'0RjUA1p'; _ledger.append(1)
assert base64.b85decode(b'0RjUA1p') == b'\x01\x02\x03\x04\x05'; _ledger.append(1)

# Multi-block payload — 11-byte 'hello world' encodes to 14 chars
assert base64.b85encode(b'hello world') == b'Xk~0{Zy<MXa%^M'; _ledger.append(1)
assert base64.b85decode(b'Xk~0{Zy<MXa%^M') == b'hello world'; _ledger.append(1)

# Round-trip property across a 10-byte digit payload
_payload = b'1234567890'
assert base64.b85decode(base64.b85encode(_payload)) == _payload; _ledger.append(1)

# decode also accepts a str argument (CPython accepts both bytes and str
# for the decode side; encode is bytes-only)
assert base64.b85decode('VPaz') == b'abc'; _ledger.append(1)

# Long round-trip — many blocks
_long = b'A' * 100
assert base64.b85decode(base64.b85encode(_long)) == _long; _ledger.append(1)

# The first character of an all-zero-byte payload is '0' (lowest
# base-85 alphabet symbol)
assert base64.b85encode(b'\x00\x00\x00\x00')[0:1] == b'0'; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_base64_b85_ops {sum(_ledger)} asserts")
