# Operational AssertionPass seed for `base64.a85encode` /
# `base64.a85decode` (ASCII-85) and long-payload round-trip identity
# through the four base-N encoders. Surface: `a85encode(b"hello")`
# returns a bytes blob distinct from the standard b64 encoding, and
# `a85decode(a85encode(x)) == x`. A 19-byte ASCII payload survives a
# round trip through all four encoders (`b64`, `b32`, `b16`,
# `urlsafe_b64`).
import base64
_ledger: list[int] = []

# a85encode produces bytes
enc_a85 = base64.a85encode(b"hello")
assert isinstance(enc_a85, bytes); _ledger.append(1)
assert len(enc_a85) > 0; _ledger.append(1)

# a85 round-trip identity
assert base64.a85decode(base64.a85encode(b"hello")) == b"hello"; _ledger.append(1)
assert base64.a85decode(base64.a85encode(b"world")) == b"world"; _ledger.append(1)
assert base64.a85decode(base64.a85encode(b"")) == b""; _ledger.append(1)

# Long payload round-trip through each base-N encoder
payload = b"The quick brown fox"
assert base64.b64decode(base64.b64encode(payload)) == payload; _ledger.append(1)
assert base64.b32decode(base64.b32encode(payload)) == payload; _ledger.append(1)
assert base64.b16decode(base64.b16encode(payload)) == payload; _ledger.append(1)
assert base64.urlsafe_b64decode(base64.urlsafe_b64encode(payload)) == payload; _ledger.append(1)
assert base64.a85decode(base64.a85encode(payload)) == payload; _ledger.append(1)

# Binary payload (non-printable bytes) round-trips
binp = b"\x00\x01\x02\xff\xfe\xfd"
assert base64.b64decode(base64.b64encode(binp)) == binp; _ledger.append(1)
assert base64.b32decode(base64.b32encode(binp)) == binp; _ledger.append(1)
assert base64.b16decode(base64.b16encode(binp)) == binp; _ledger.append(1)

# urlsafe variant uses '-' / '_' in the output (no '+' or '/')
url_enc = base64.urlsafe_b64encode(b"???>>>")
assert b"+" not in url_enc; _ledger.append(1)
assert b"/" not in url_enc; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_base64_a85_roundtrip_ops {sum(_ledger)} asserts")
