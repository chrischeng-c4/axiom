"""hashlib.md5 — short-payload hash perf bench.

End-user scenario: `md5(payload).hexdigest()` inside a tight loop, the
canonical short-message fingerprint primitive that backs every cache-
key derivation / ETag computation / dedup-token builder / weak content-
hash for non-crypto bucketing. CPython routes through openssl_md5 /
md5module (C-level OpenSSL block-loop); mamba's hashlib should hit a
native impl through its typed bridge.

Distinct from `sha256_short_hot.py` (SHA-256 block size + rounds) and
`blake2b_hot.py` (BLAKE2 wide-state). MD5 is the cheapest classic hash
and a useful per-call dispatch cost probe.

NOTE: MD5 is NOT a security primitive here — it is benchmarked solely
as a non-crypto bucketing/dedup hash, the established CPython
hashlib-perf reference point.

Bounded context (DDD): stdlib_bench/hashlib.

Tier: compute (with per-call new-Hash + hexdigest str alloc).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `hashlib.md5` is a module-level constructor; safe to hoist.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import hashlib
import sys
import time

_md5 = hashlib.md5
PAYLOAD = b"user_id=42&session=abc123&ts=1716798000&op=read&path=/v1/items/9876"
ITERS = 10000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    h = _md5(PAYLOAD).hexdigest()
    acc = acc + len(h)
_t1 = time.perf_counter()

print("md5_short_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = len(hashlib.md5(PAYLOAD).hexdigest())
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
