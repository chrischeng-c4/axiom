"""re.sub — pattern-based string replace perf bench.

End-user scenario: `re.sub(pat, repl, s)` inside a tight loop, the
canonical string-rewrite primitive that backs every log scrubber /
mask-PII filter / template renderer / config normaliser. CPython
routes through _sre.SRE_Pattern.sub (a C-level state machine + new
str alloc); mamba's re should hit a Rust regex impl through its
typed bridge — known to be allocation-bound on per-match new_str
per the re.findall memory.

Bounded context (DDD): stdlib_bench/re.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `sub` to a local before the hot loop.
"""

import re
import sys
import time

_sub = re.sub

N = 1000
texts = [f"user=alice-{i} pw=secret-{i} ip=10.0.{i % 256}.{i % 256}" for i in range(N)]
ITERS = 100

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for t in texts:
        total = total + len(_sub(r"secret-\d+", "***", t))
_t1 = time.perf_counter()

print("sub_replace_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for t in texts:
    per_iter = per_iter + len(_sub(r"secret-\d+", "***", t))
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
