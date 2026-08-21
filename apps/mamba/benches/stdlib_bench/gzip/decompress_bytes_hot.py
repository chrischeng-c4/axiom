"""gzip.decompress — gzipped-bytes-to-bytes perf bench.

End-user scenario: `gzip.decompress(blob)` inside a tight loop, the
inverse of compress — every Content-Encoding: gzip ingress / log-tail
unpacker / object-storage download lands here. CPython routes through
zlibmodule.zlib_decompress (a C-level zlib `inflate` call); mamba's
gzip should hit the same native decompressor through its typed bridge.

Bounded context (DDD): stdlib_bench/gzip.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `decompress` to a local before the hot loop.
"""

import gzip
import sys
import time

_compress = gzip.compress
_decompress = gzip.decompress

PAYLOAD = b"\n".join(
    (f"line-{i:03d}: " + ("x" * 80)).encode("ascii") for i in range(10)
)
BLOB = _compress(PAYLOAD)
ITERS = 5000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    total = total + len(_decompress(BLOB))
_t1 = time.perf_counter()

print("decompress_bytes_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Round-trip back to the original payload length each iter.
expected = ITERS * len(PAYLOAD)
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
