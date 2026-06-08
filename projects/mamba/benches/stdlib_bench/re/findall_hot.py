"""re.findall — multi-match collect perf bench.

End-user scenario: `re.findall(pat, text)` inside a tight loop, the
canonical extract-all primitive that backs every link-extract / email-
harvest / mention-parse / log-field-extract. CPython routes through
_sre.SRE_Pattern.findall; mamba's re hits a Rust regex impl through
its typed bridge — known allocation-bound per
[[project-mamba-re-findall-allocation-bound]].

Bounded context (DDD): stdlib_bench/re.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `findall` to a local before the hot loop.
"""

import re
import sys
import time

_findall = re.findall

PATTERN = r"\b\w+\b"
N = 1000
texts = [f"alpha beta-{i} gamma delta-{i} epsilon zeta-{i}" for i in range(N)]
ITERS = 100

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for t in texts:
        total = total + len(_findall(PATTERN, t))
_t1 = time.perf_counter()

print("findall_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for t in texts:
    per_iter = per_iter + len(_findall(PATTERN, t))
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
