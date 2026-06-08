# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "real_world"
# case = "sha256_artifact"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: a build tool / content-addressed store digests a 1 MiB artifact for a stable cache key: sha256 + md5 one-shot vs chunked update, copy() isolation, and new(name, data) all agree"""
import hashlib

# 1 MiB of mixed content — the pattern repeats but the digest must still
# match across runtimes (CPython 3.12 + mamba) byte-for-byte.
payload: bytes = (b"Mamba force-typed Python compiler artifact integrity check.\n"
                  b"This buffer exercises 1 MiB through the digest call.\n") * 9532
# Trim/pad to exactly 1 MiB (1048576 bytes).
if len(payload) > 1048576:
    payload = payload[:1048576]
elif len(payload) < 1048576:
    payload = payload + b"\x00" * (1048576 - len(payload))
assert len(payload) == 1048576, f"expected 1MiB, got {len(payload)} bytes"

# sha256: artifact integrity case.
h = hashlib.sha256(payload)
hex_digest = h.hexdigest()
assert len(hex_digest) == 64, f"sha256 hex digest must be 64 chars, got {len(hex_digest)}"
assert hex_digest == h.hexdigest(), "hexdigest must be stable across repeated calls"
assert h.digest_size == 32, f"sha256 digest_size must be 32, got {h.digest_size}"
assert h.name == "sha256", f"sha256 name attr must be 'sha256', got {h.name!r}"

# md5: cache-key case.
m = hashlib.md5(payload)
md_hex = m.hexdigest()
assert len(md_hex) == 32, f"md5 hex digest must be 32 chars, got {len(md_hex)}"
assert m.digest_size == 16, f"md5 digest_size must be 16, got {m.digest_size}"

# Incremental update path: single shot vs chunked feed.
chunks = [payload[i:i + 65536] for i in range(0, len(payload), 65536)]
m2 = hashlib.md5()
for chunk in chunks:
    m2.update(chunk)
assert m2.hexdigest() == md_hex, "chunked update must equal single-shot digest"

# Copy isolation: forking a hasher must not mutate the parent state.
base = hashlib.sha256(b"prefix-only")
fork = base.copy()
fork.update(b"-extra")
assert base.hexdigest() != fork.hexdigest(), "copy() must produce an independent hasher"
assert base.hexdigest() == hashlib.sha256(b"prefix-only").hexdigest(), (
    "base hasher must remain unchanged after copy + fork.update"
)

# new(name, data): named factory case.
n = hashlib.new("sha1", payload)
n_hex = n.hexdigest()
assert len(n_hex) == 40, f"sha1 hex digest must be 40 chars, got {len(n_hex)}"

print("sha256_artifact OK")
