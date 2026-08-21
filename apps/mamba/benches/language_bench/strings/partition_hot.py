"""str.partition — first-delimiter 3-tuple split perf bench.

End-user scenario: `key, sep, value = line.partition("=")` inside a
tight loop, the canonical "split-once-on-first-delim" primitive that
backs every key=value config-line parser / "name: value" header
splitter / scheme://rest URL prefix peel / first-colon log-level
extractor. CPython routes through unicode_partition (C-level single
forward scan + 3-tuple of new-str views); mamba's str should hit a
native impl through its typed bridge.

Distinct from `split_hot.py` (str.split — N-way, allocates a list).
partition allocates a fixed 3-tuple (no list, no per-element loop)
and stops at the first match — so it should be cheaper than a
full split when only the first delim matters.

Bounded context (DDD): language_bench/strings.

Tier: compute (with new-tuple + 3 new-str slice alloc per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `partition` is a str method; DO NOT hoist `_p = LINE.partition`
— bound-method hoist returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

LINE = "method=POST path=/api/v1/users status=200 duration_ms=42"
SEP = " "
ITERS = 30000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    head, _sep, tail = LINE.partition(SEP)
    acc = acc + len(head) + len(tail)
_t1 = time.perf_counter()

print("partition_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

h, _s, t = LINE.partition(SEP)
per_iter = len(h) + len(t)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
