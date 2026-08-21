# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "real_world"
# case = "session_token_signing"
# subject = "secrets"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets: auth flow: mint an opaque session token via secrets, derive a server-side lookup key via hashlib.sha256, then per-request HMAC-sign the body and constant-time-verify via compare_digest; random output checked by shape only"""
import hashlib
import hmac
import secrets

# -- 1. Mint side: secrets token shape ---------------------------------------

# token_bytes: opaque server-side material.
raw_token = secrets.token_bytes(32)
assert isinstance(raw_token, bytes), f"token_bytes must return bytes, got {type(raw_token).__name__}"
assert len(raw_token) == 32, f"token_bytes(32) must be 32 bytes, got {len(raw_token)}"

# token_bytes(0) is a defined corner: returns b"".
empty = secrets.token_bytes(0)
assert empty == b"", f"token_bytes(0) must be empty bytes, got {empty!r}"

# token_hex: hex-encoded variant, 2x nbytes characters.
hex_token = secrets.token_hex(16)
assert isinstance(hex_token, str), f"token_hex must return str, got {type(hex_token).__name__}"
assert len(hex_token) == 32, f"token_hex(16) must be 32 chars, got {len(hex_token)}"
assert all(c in "0123456789abcdef" for c in hex_token), "token_hex output must be lowercase hex"

# token_urlsafe: URL-safe base64 without padding; len is ceil(4*nbytes/3) >= nbytes.
url_token = secrets.token_urlsafe(32)
assert isinstance(url_token, str), f"token_urlsafe must return str, got {type(url_token).__name__}"
assert len(url_token) >= 32, f"token_urlsafe(32) must be >= 32 chars, got {len(url_token)}"
assert "=" not in url_token, "token_urlsafe must not contain padding"
assert "+" not in url_token and "/" not in url_token, (
    "token_urlsafe must not contain non-URL-safe base64 chars"
)

# Disjointness sanity: two mints in a row must not collide (2**-256).
second_raw = secrets.token_bytes(32)
assert raw_token != second_raw, "two consecutive token_bytes(32) values must differ"

# -- 2. Server-side lookup key via hashlib.sha256 -----------------------------

# The auth backend stores hash(token), not the raw token, so a database
# compromise does not leak live session material.
lookup_key = hashlib.sha256(raw_token).hexdigest()
assert len(lookup_key) == 64, f"sha256 hex digest must be 64 chars, got {len(lookup_key)}"
assert hashlib.sha256(raw_token).hexdigest() == lookup_key, "sha256 must be deterministic"
assert hashlib.sha256(second_raw).hexdigest() != lookup_key, "different inputs differ"

# Known-answer test on a fixed input guards against a silently-wrong digest.
kat_input = b"The quick brown fox jumps over the lazy dog"
kat_expected = "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
assert hashlib.sha256(kat_input).hexdigest() == kat_expected, "sha256 KAT must match"

# -- 3. Per-request signing via hmac + constant-time verify -------------------

# The raw token is the HMAC key; the request body is signed.
body = b'{"event":"order.created","order_id":"42-deadbeef"}'
signature = hmac.new(raw_token, body, "sha256").hexdigest()
assert len(signature) == 64, f"hmac-sha256 hex must be 64 chars, got {len(signature)}"

# Receiver-side: rebuild HMAC with the same key and compare_digest.
receiver_sig = hmac.new(raw_token, body, "sha256").hexdigest()
assert hmac.compare_digest(signature, receiver_sig), "compare_digest accepts matching signatures"

# Tampered signature: flip the first hex nibble. Constant-time compare rejects.
tampered = ("1" if signature[0] != "1" else "2") + signature[1:]
assert not hmac.compare_digest(signature, tampered), "compare_digest rejects tampered signatures"

# Wrong-key path: another freshly-minted token signs the same body differently.
attacker_sig = hmac.new(second_raw, body, "sha256").hexdigest()
assert not hmac.compare_digest(signature, attacker_sig), "different keys produce different signatures"

# secrets.compare_digest is the user-facing alias of hmac.compare_digest.
assert secrets.compare_digest(signature, receiver_sig), "secrets.compare_digest agrees on equal inputs"
assert not secrets.compare_digest(signature, tampered), "secrets.compare_digest rejects tampered"

# Bytes operands also compare correctly (auth libs route both kinds).
sig_bytes = bytes.fromhex(signature)
recv_bytes = bytes.fromhex(receiver_sig)
assert hmac.compare_digest(sig_bytes, recv_bytes), "compare_digest accepts matching bytes signatures"

print("session_token_signing OK")
