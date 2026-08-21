"""keyword.iskeyword — reserved-word check perf bench.

End-user scenario: `iskeyword(token)` inside a tight loop, the canonical
identifier-validator primitive that backs every code linter / template
sanitiser / generated-code namer / column-name dedup. CPython routes
through keyword.iskeyword (a frozenset membership test); mamba's
keyword should hit the same set test through its typed bridge.

Bounded context (DDD): stdlib_bench/keyword.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `iskeyword` to a local before the hot loop.
"""

import keyword
import sys
import time

_iskeyword = keyword.iskeyword

# Half keywords / half identifiers — exercises both true and false branches.
TOKENS = (
    ["if", "while", "for", "return", "class", "def", "import", "yield", "global", "lambda"]
    + ["x", "name", "value", "result", "data", "user", "row", "id", "score", "count"]
)
ITERS = 100_000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for t in TOKENS:
        if _iskeyword(t):
            total = total + 1
_t1 = time.perf_counter()

print("iskeyword_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for t in TOKENS:
    if _iskeyword(t):
        per_iter = per_iter + 1
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
