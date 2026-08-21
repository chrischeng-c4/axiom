"""urllib.parse.quote — URL percent-encode perf bench.

End-user scenario: `quote(value)` inside a tight loop, the canonical
URL-safe string emit that backs every query-string builder / link
generator / OAuth signer / shareable-link composer. CPython routes
through urlparse.quote (pure Python + a precomputed always-safe
table); mamba's urllib.parse should hit the same logic.

Bounded context (DDD): stdlib_bench/urllib.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `quote` to a local before the hot loop.

Mamba quirk: use `from urllib.parse import quote` not
`import urllib.parse` (dotted-attr resolves to None).
"""

from urllib.parse import quote as _quote
import sys
import time

N = 1000
# Mix of safe and unsafe chars so the encode path actually runs.
values = [f"hello world & friends #{i} = ¡§{i}" for i in range(N)]
ITERS = 100

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for v in values:
        total = total + len(_quote(v))
_t1 = time.perf_counter()

print("quote_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for v in values:
    per_iter = per_iter + len(_quote(v))
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
