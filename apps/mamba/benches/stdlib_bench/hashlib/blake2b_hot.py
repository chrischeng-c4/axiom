"""hashlib.blake2b — modern fast-hash perf bench.

End-user scenario: `blake2b(msg).hexdigest()` inside a tight loop, the
canonical modern-hash primitive that backs every content-addressable
storage key / cache invalidation tag / shard-key compute / Merkle-leaf
hash. CPython routes through C-level _blake2; mamba's hashlib should
hit a native impl through its typed bridge.

Bounded context (DDD): stdlib_bench/hashlib.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `blake2b` to a local.
"""

import hashlib
import sys
import time

_blake2b = hashlib.blake2b

MSG = b"the quick brown fox jumps over the lazy dog " * 4
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    h = _blake2b(MSG).hexdigest()
    acc = acc + len(h)
_t1 = time.perf_counter()

print("blake2b_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref = len(_blake2b(MSG).hexdigest())
expected = ITERS * ref
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
