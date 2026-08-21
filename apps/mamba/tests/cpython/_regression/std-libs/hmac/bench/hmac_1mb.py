"""Hot-loop bench for `hmac.new(key, msg, digestmod).hexdigest()` round-trip.

End-user scenario: keyed message authentication (cookie HMAC, API request
signature, content-addressed cache key with a secret) where the message
is large enough that bulk-work inside the RustCrypto HMAC dominates total
time. The fixture runs many iterations of new-hmac + update(1MB) +
hexdigest so the bench harness measures both per-iter wall time and
steady-state memory.

Tier: `native-shim compute` (target mamba/cpython >= 1.0x — CPython's
hmac is a thin Python wrapper around OpenSSL's HMAC, so the absolute
ceiling is bounded by the underlying digest speed; mamba's edge over
CPython is removing the interpreter overhead around the keying +
update + finalize calls). See Task #19 hypothesis branches.

Hoist convention (per #2097): `hmac_new = hmac.new` outside the loop.
Without hoisting, mamba's module-attr lookup at the call site is ~5x
slower than the hoisted form — and the published cross-runtime ratio
would understate the underlying HMAC speed.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
harness compares per-iteration wall time and reports the ratio.
"""

import hmac
import time

# 1 MiB payload via 4-byte pattern (matches hashlib/digest_1mb.py convention).
PAYLOAD: bytes = b"abcd" * 262144  # 1,048,576 bytes (1 MiB)
KEY: bytes = b"a-32-byte-shared-hmac-test-key!!"  # 32 bytes, > sha256 digest size

ITERS = 8

# Hoist module attributes outside the loop — see CLAUDE.md note + #2097.
hmac_new = hmac.new

# Warmup correctness probe (per Task #15 pattern) — establish that the
# JIT keeps the assertion path live AFTER the hot call. If mamba's
# JIT-branch-drop bug (#2099) fires, this assert would silently no-op
# and the bench would be measuring nothing meaningful.
warmup = hmac_new(KEY, PAYLOAD, "sha256")
warmup_hex = warmup.hexdigest()
assert len(warmup_hex) == 64, f"warmup digest len mismatch: {len(warmup_hex)}"

total_len = 0
# Internal-time marker for Task #22 — see hashlib/digest_1mb.py rationale.
# wall-time ratio is biased by Python startup overhead, marker is not.
for _ in range(ITERS):
    h = hmac_new(KEY, PAYLOAD, "sha256")
    d = h.hexdigest()
    total_len += len(d)

# sha256 hexdigest is always 64 hex chars; 64 * ITERS is invariant.
# Use subtraction-equals-zero (per #boxed-accumulator-int-equality memory
# entry — `==` on a `+=` accumulator can mismatch a multiplied int).
expected = 64 * ITERS
diff = total_len - expected
assert diff == 0, f"digest length mismatch: total={total_len} expected={expected} diff={diff}"
print("hmac_1mb:", total_len)
