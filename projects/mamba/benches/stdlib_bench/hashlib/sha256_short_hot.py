"""hashlib.sha256 — short-message digest perf bench.

End-user scenario: `hashlib.sha256(s).hexdigest()` over many
short byte strings, the canonical content-addressed lookup /
ETag / id-from-payload primitive that backs every CDN cache
key, every transactional dedupe table. CPython routes through
_sha256.SHA256_Final from openssl; mamba's hashlib.sha256
calls through the integer-handle pattern into the Rust
sha2 crate.

Bounded context (DDD): stdlib_bench/hashlib.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `sha256` to a local before the hot loop.
"""

import hashlib
import sys
import time

_sha256 = hashlib.sha256

N = 1000
payloads = [(f"row-{i}").encode("ascii") for i in range(N)]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for p in payloads:
        total = total + len(_sha256(p).hexdigest())
_t1 = time.perf_counter()

print("sha256_short_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# hexdigest() always 64 chars for sha256.
expected = ITERS * N * 64
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
