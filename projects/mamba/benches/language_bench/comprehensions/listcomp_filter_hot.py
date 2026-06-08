"""List comprehension with `if` filter — predicate-filter perf bench.

End-user scenario: tight loop `[x for x in seq if pred(x)]`, the
canonical "keep matching items" primitive that backs every
"non-empty line" filter / "positive value" selector / "active row"
WHERE clause / log-grep-by-level reducer. CPython routes through
the listcomp opcode with COMPARE_OP + POP_JUMP_IF_FALSE (C-level
opcode dispatch); mamba should fuse the loop + predicate + alloc
through its typed bridge.

Distinct from `listcomp_square_hot.py` (no-filter map variant) — the
filter case adds a branch per element and produces a variable-length
output list (alloc-bound based on selectivity).

Bounded context (DDD): language_bench/comprehensions.

Tier: compute + allocation (per-iter new-list + per-kept-element
ref-add; CPython has list_append C path).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module attrs to hoist; the loop body is pure syntax.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars. `x % 2` arithmetic must stay outside the comp
expression IF mamba parser complains — but `>` predicate is fine.
"""

import sys
import time

VALS = (3, 7, 1, 9, 2, 8, 4, 6, 5, 0,
        11, 14, 12, 18, 15, 13, 19, 16, 17, 10,
        25, 22, 28, 24, 21, 27, 23, 29, 26, 20)
THRESH = 10
ITERS = 10000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    kept = [v for v in VALS if v > THRESH]
    acc = acc + len(kept)
_t1 = time.perf_counter()

print("listcomp_filter_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = len([v for v in VALS if v > THRESH])
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
