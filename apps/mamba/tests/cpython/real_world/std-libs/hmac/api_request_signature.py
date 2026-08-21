# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "real_world"
# case = "api_request_signature"
# subject = "hmac"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac: a service-to-service client signs a 1 MiB request body with a shared secret across sha256/sha1/md5, verifies single-shot vs chunked-update parity, copy() isolation, the hmac.digest one-shot path, and compare_digest accept/reject of the canonical signature"""
import hmac

# 1 MiB of structured-looking payload (JSON-ish repeating pattern).
payload: bytes = (b"{\"event\":\"order.created\",\"order_id\":\"42-deadbeef\","
                  b"\"amount\":\"199.99\",\"currency\":\"USD\",\"ts\":1715600000}\n"
                  ) * 9532
if len(payload) > 1048576:
    payload = payload[:1048576]
elif len(payload) < 1048576:
    payload = payload + b"\x00" * (1048576 - len(payload))
assert len(payload) == 1048576, f"expected 1 MiB, got {len(payload)} bytes"

SECRET: bytes = b"shared-32-byte-rotating-secret-x"
assert len(SECRET) == 32, "secret must be 32 bytes for the scenario"

# Primary case: HMAC-SHA256 of the body (modern signature default).
h = hmac.new(SECRET, payload, "sha256")
hex_digest = h.hexdigest()
assert len(hex_digest) == 64, f"hmac-sha256 hex digest must be 64 chars, got {len(hex_digest)}"
assert hex_digest == h.hexdigest(), "hexdigest must be stable across repeated calls"
assert h.digest_size == 32, f"hmac-sha256 digest_size must be 32, got {h.digest_size}"
assert h.name == "hmac-sha256", f"hmac-sha256 name attr must be 'hmac-sha256', got {h.name!r}"

# Legacy compat case: HMAC-SHA1 (still used by some webhook protocols).
h_sha1 = hmac.new(SECRET, payload, "sha1")
sha1_hex = h_sha1.hexdigest()
assert len(sha1_hex) == 40, f"hmac-sha1 hex digest must be 40 chars, got {len(sha1_hex)}"
assert h_sha1.digest_size == 20

# Cookie-fingerprint case: HMAC-MD5 (deprecated but still seen).
h_md5 = hmac.new(SECRET, payload, "md5")
md5_hex = h_md5.hexdigest()
assert len(md5_hex) == 32, f"hmac-md5 hex digest must be 32 chars, got {len(md5_hex)}"

# Incremental update path: same digest from single-shot vs chunked feed.
chunks = [payload[i:i + 65536] for i in range(0, len(payload), 65536)]
h2 = hmac.new(SECRET, None, "sha256")
for chunk in chunks:
    h2.update(chunk)
assert h2.hexdigest() == hex_digest, "chunked update must equal single-shot digest"

# Copy isolation: forking a keyed HMAC must not mutate the parent state.
base = hmac.new(SECRET, b"prefix-only", "sha256")
fork = base.copy()
fork.update(b"-extra")
assert base.hexdigest() != fork.hexdigest(), "copy() must produce an independent hmac"
assert base.hexdigest() == hmac.new(SECRET, b"prefix-only", "sha256").hexdigest(), (
    "base hmac must remain unchanged after copy + fork.update"
)

# compare_digest: constant-time equality on the canonical signature.
assert hmac.compare_digest(hex_digest, hex_digest), "compare_digest must accept equal hex"
tampered = "0" + hex_digest[1:]
assert not hmac.compare_digest(hex_digest, tampered), "compare_digest must reject tampered hex"

# hmac.digest(...) one-shot fast path: equals object-path digest bytes.
one_shot = hmac.digest(SECRET, payload, "sha256")
assert one_shot == h.digest(), "one-shot hmac.digest must match object-path digest"

print("api_request_signature OK")
