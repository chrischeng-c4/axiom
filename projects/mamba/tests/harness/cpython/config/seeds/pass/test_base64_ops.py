# Operational AssertionPass seed for the `base64` stdlib module.
# Surface: b64encode round-trip, urlsafe variant, hex-ish strings.
# Companion to stub/test_base64.py — vendored unittest seed.
import base64
_ledger: list[int] = []
assert base64.b64encode(b"hello") == b"aGVsbG8="; _ledger.append(1)
assert base64.b64decode(b"aGVsbG8=") == b"hello"; _ledger.append(1)
assert base64.b64encode(b"") == b""; _ledger.append(1)
assert base64.b64decode(b"") == b""; _ledger.append(1)
data = b"the quick brown fox jumps over the lazy dog"
assert base64.b64decode(base64.b64encode(data)) == data; _ledger.append(1)
assert base64.b64encode(b"\x00\x01\x02\x03") == b"AAECAw=="; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_base64_ops {sum(_ledger)} asserts")
