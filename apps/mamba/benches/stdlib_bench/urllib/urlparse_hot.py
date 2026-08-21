"""urllib.parse.urlparse — URL split perf bench.

End-user scenario: `urlparse(url)` inside a tight loop, the canonical
URL decomposition primitive that backs every router / proxy / link
rewriter / referrer logger. CPython routes through urlparse.py (pure
Python with an LRU cache around _splitnetloc); mamba's urllib.parse
should hit the same logic but may pay per-call SplitResult NamedTuple
boxing.

Bounded context (DDD): stdlib_bench/urllib.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `urlparse` to a local before the hot loop.

Mamba quirk: dotted submodule access doesn't bind — use
`from urllib.parse import urlparse` not `import urllib.parse`.
"""

from urllib.parse import urlparse as _urlparse
import sys
import time

N = 1000
urls = [
    f"https://example-{i % 10}.com:8080/path/to/resource-{i}?q={i}&lang=en#frag-{i}"
    for i in range(N)
]
ITERS = 100

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for u in urls:
        total = total + len(_urlparse(u).netloc)
_t1 = time.perf_counter()

print("urlparse_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for u in urls:
    per_iter = per_iter + len(_urlparse(u).netloc)
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
