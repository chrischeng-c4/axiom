"""hmac.new(sha256).hexdigest — keyed-hash perf bench.

End-user scenario: `hmac.new(key, msg, sha256).hexdigest()` inside a
tight loop, the canonical signing primitive that backs every webhook-
signature verify / JWT HS256 sign / token-bucket nonce HMAC / cookie-
MAC. CPython routes through _hashlib HMAC + the same _sha256 backend;
mamba's hmac should ride the same handle-table pattern through its
typed bridge ([[project-mamba-integer-handle-pattern]]).

Bounded context (DDD): stdlib_bench/hmac.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `new` and the digestmod to locals.
"""

import hashlib
import hmac
import sys
import time

_new = hmac.new
_sha256 = hashlib.sha256

KEY = b"super-secret-key-bytes"
N = 1000
msgs = [(f"event-{i:06d}-payload").encode("ascii") for i in range(N)]
ITERS = 100

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for m in msgs:
        total = total + len(_new(KEY, m, _sha256).hexdigest())
_t1 = time.perf_counter()

print("hmac_sha256_hexdigest_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Each hexdigest is always 64 chars for SHA-256.
expected = ITERS * N * 64
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
