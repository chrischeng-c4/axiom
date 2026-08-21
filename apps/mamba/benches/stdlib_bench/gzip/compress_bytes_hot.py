"""gzip.compress — bytes-to-gzipped-bytes perf bench.

End-user scenario: `gzip.compress(payload)` inside a tight loop, the
canonical wire/at-rest squeeze that backs every Content-Encoding:
gzip response / log-shipping packer / object-storage upload. CPython
routes through zlibmodule.zlib_compress (a C-level zlib `deflate`
call); mamba's gzip should ride the same native compressor through
its typed bridge.

Bounded context (DDD): stdlib_bench/gzip.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `compress` to a local before the hot loop.
"""

import gzip
import sys
import time

_compress = gzip.compress

# ~1 KiB representative log-line payload (10 lines × ~100 chars).
PAYLOAD = b"\n".join(
    (f"line-{i:03d}: " + ("x" * 80)).encode("ascii") for i in range(10)
)
ITERS = 5000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    total = total + len(_compress(PAYLOAD))
_t1 = time.perf_counter()

print("compress_bytes_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = len(_compress(PAYLOAD))
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
