"""Hot-loop bench for `base64.b64encode` + `base64.b64decode` round-trip.

End-user scenario: a downstream tool encodes a binary payload to base64
for transport (e.g. HTTP header, JSON field) and the receiver decodes it
back. The fixture exercises encode + decode in lockstep, which together
form the dominant base64 cost in real workloads.

Tier: `compute` (target mamba/cpython >= 10x per #1265). Encode/decode
is byte-level arithmetic — pure compute, no allocations beyond the
output buffer.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
harness compares per-iteration wall time and reports the ratio.
"""

import base64


PAYLOAD: bytes = b"Mamba force-typed Python compiler base64 benchmark payload!" * 32

# Hoist module attrs to local aliases (#2097) so per-iter attribute
# lookup overhead does not skew the measurement.
b64encode = base64.b64encode
b64decode = base64.b64decode

ITERS = 2000
total_bytes = 0
for _ in range(ITERS):
    encoded = b64encode(PAYLOAD)
    decoded = b64decode(encoded)
    total_bytes += len(decoded)

# Print total + emit marker BEFORE the trailing assert per #2105 (avoid
# JIT post-call branch elision silently zeroing the marker on mamba).
print("encode_decode:", total_bytes)

assert total_bytes == ITERS * len(PAYLOAD), (
    f"byte total mismatch: {total_bytes} != {ITERS} * {len(PAYLOAD)}"
)
