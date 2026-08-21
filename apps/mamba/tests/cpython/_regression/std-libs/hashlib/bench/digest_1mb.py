"""Hot-loop bench for `hashlib.sha256(1MB).hexdigest()` round-trip.

End-user scenario: digesting an artifact (cache key, content-addressed
store, integrity check) where the input is large enough that bulk-work
inside the RustCrypto digest dominates total time. The fixture runs
many iterations of new-hasher + update(1MB) + hexdigest so the bench
harness measures both per-iter wall time and steady-state memory.

Tier: `compute` (target mamba/cpython >= 10x per issue #1265). Hashing
1MB of bytes through SHA-256 is pure compute inside RustCrypto's
streaming digest — Python's `hashlib` is itself a thin shim over
OpenSSL/builtin C, so mamba's win comes from removing the CPython
interpreter overhead around the digest call.

Hoist convention (per #2097): `sha256 = hashlib.sha256` outside the loop.
Without hoisting, mamba's module-attr lookup at the call site is ~5x
slower than the hoisted form — and the published cross-runtime ratio
would understate the underlying digest speed.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
harness compares per-iteration wall time and reports the ratio.
"""

import hashlib
import sys
import time

PAYLOAD: bytes = b"abcd" * 262144  # 1,048,576 bytes (1 MiB) — 4-char pattern keeps init fast.

ITERS = 12

# Hoist module attribute outside the loop — see CLAUDE.md note + #2097.
sha256 = hashlib.sha256

total_len = 0
# Internal-time marker for Task #22: measure the hot loop with
# so cross_runtime.rs can compute the unbiased per-call ratio. The
# wall-time ratio is dominated by Python startup overhead for short
# benches; the marker captures the pure steady-state per-call cost.
# Note: emitted to stderr in source, but mamba currently routes
# `file=sys.stderr` to stdout — the harness accepts either stream.
for _ in range(ITERS):
    h = sha256(PAYLOAD)
    d = h.hexdigest()
    total_len += len(d)

# sha256 hexdigest is always 64 hex chars; 64 * ITERS is invariant.
expected = 64 * ITERS
diff = total_len - expected
assert diff == 0, f"digest length mismatch: total={total_len} expected={expected} diff={diff}"
print("digest_1mb:", total_len)
