"""re.compile + p.search — pre-compiled pattern + groups perf bench.

End-user scenario: pre-compile a pattern once then call `p.search(text)`
inside a tight loop, the canonical extract-N-fields-per-line primitive
that backs every log parser / email harvester / phone-number scrubber /
metric line extractor / CSV-with-quoted-fields tokenizer. CPython routes
through C-level _sre.SRE_Pattern.search; mamba routes through Rust
regex.

Bounded context (DDD): stdlib_bench/re.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `re` to a local; do NOT hoist `_PAT.search` to a local
       (mamba quirk: hoisted bound-method form returns None on Pattern).
"""

import re
import sys
import time

_PAT = re.compile(r"(\w+)@(\w+)\.(\w+)")

LINES = [
    f"user{i}@host{i % 7}.example contact line {i}" for i in range(100)
]
# Mamba ~1400x slower on this path (pattern likely re-parses per call
# despite pre-compile); cap ITERS to keep wall under ~5s.
ITERS = 50

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for line in LINES:
        m = _PAT.search(line)
        if m is not None:
            g = m.groups()
            total = total + len(g[0]) + len(g[1]) + len(g[2])
_t1 = time.perf_counter()

print("compiled_search_groups_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for line in LINES:
    m = _PAT.search(line)
    if m is not None:
        g = m.groups()
        per_iter = per_iter + len(g[0]) + len(g[1]) + len(g[2])
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
