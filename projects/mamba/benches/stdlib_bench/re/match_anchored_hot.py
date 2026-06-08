"""re.match — anchored prefix match perf bench.

End-user scenario: `re.match(r"^v(\\d+)\\.(\\d+)\\.(\\d+)$", tag)` inside a
tight loop, the canonical fixed-prefix shape-check primitive that backs
every semver / build-tag validator / log-level prefix dispatcher /
config-key route classifier / API-version router. CPython routes
through pattern_match (C-level SRE engine on a precompiled pattern,
anchored at pos 0 — no scan); mamba's re should hit a native impl
through its typed bridge.

Distinct from `findall_hot.py` (unanchored bulk find, exercises the
SRE scan + multi-match collect path) and `sub_replace_hot.py` (substitute).
match is the cheapest re entry point: anchored, single attempt.

Bounded context (DDD): stdlib_bench/re.

Tier: compute (with new-Match + groups allocation per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: precompile pattern + hoist `pat.match` is bound-method-hoist —
DO NOT do that under mamba; call via `pat.match(...)` each time.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import re
import sys
import time

PAT = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")
TAGS = ("v1.0.0", "v2.13.4", "v0.0.99", "v10.20.30",
        "not-a-version", "v3.7.11", "v100.200.300", "almost-v1.2")
# mamba re paths are alloc-bound (see #2178 + re-findall memo); cap ITERS
# low enough that mamba run-time stays under ~5s in CI.
ITERS = 2000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for t in TAGS:
        m = PAT.match(t)
        if m is not None:
            s = s + 1
    acc = acc + s
_t1 = time.perf_counter()

print("match_anchored_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for t in TAGS:
    if PAT.match(t) is not None:
        per_iter = per_iter + 1
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
