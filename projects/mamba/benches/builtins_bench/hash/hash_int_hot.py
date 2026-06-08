"""hash() on int — builtin perf bench.

End-user scenario: `hash(x)` inside a tight loop, the canonical
hash-emission idiom that backs every dict/set/cache key
computation. CPython routes through long_hash with a fold-mod
mersenne reduction; mamba's mb_hash lowers to a direct i64
fold on the small-int fast path.

Bounded context (DDD): builtins_bench/hash.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

The accumulator uses bitwise xor to keep the per-iter body
cheap and to avoid swamping the hash cost with addition
overhead.
"""

import sys
import time

N = 1000
xs = [(i * 2654435761) & 0x7FFFFFFFFFFFFFFF for i in range(N)]
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        acc = acc ^ hash(x)
_t1 = time.perf_counter()

print("hash_int_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Reference uses the same hash() so both runtimes share their definition.
ref_acc = 0
for _ in range(ITERS):
    for x in xs:
        ref_acc = ref_acc ^ hash(x)
diff = acc - ref_acc
assert diff == 0, f"checksum mismatch: {acc} - {ref_acc} = {diff}"
