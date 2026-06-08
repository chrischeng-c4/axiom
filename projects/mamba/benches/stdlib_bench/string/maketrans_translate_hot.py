"""str.maketrans + str.translate — table-driven map perf bench.

End-user scenario: `s.translate(table)` after `str.maketrans(...)`
inside a tight loop, the canonical char-substitute primitive that
backs every ROT-style cipher / strip-accents pass /
homoglyph-normaliser / leet-decode. CPython routes through the C-level
unicode translate (a tight per-char dict lookup); mamba's str should
hit the same native impl through its typed bridge.

Bounded context (DDD): stdlib_bench/string.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: maketrans is hoisted once outside the loop.
"""

import sys
import time

# Build once: rot-style table flipping case.
_TABLE = str.maketrans(
    "abcdefghijklmnopqrstuvwxyz",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
)

N = 1000
strings = [f"hello world {i} mamba quick brown fox" for i in range(N)]
ITERS = 100

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for s in strings:
        total = total + len(s.translate(_TABLE))
_t1 = time.perf_counter()

print("maketrans_translate_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for s in strings:
    per_iter = per_iter + len(s)
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
