"""unicodedata.normalize — NFC canonicalize perf bench.

End-user scenario: `normalize('NFC', s)` inside a tight loop, the
canonical user-input canonicalisation primitive that backs every
search-index tokeniser / username uniqueness check / filename
deduper. CPython routes through unicodedata.normalize (a C-level
ICU-style decomposer); mamba's unicodedata should hit a Rust unicode-
normalisation impl through its typed bridge.

Bounded context (DDD): stdlib_bench/unicodedata.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `normalize` to a local before the hot loop.
"""

import unicodedata
import sys
import time

_normalize = unicodedata.normalize

N = 1000
# Mix of pure ASCII, accented Latin, CJK — typical multilingual input.
strings = [
    f"user-{i}-café-naïve-fiancée-Ωμέγα-日本語"
    for i in range(N)
]
ITERS = 100

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for s in strings:
        total = total + len(_normalize("NFC", s))
_t1 = time.perf_counter()

print("normalize_nfc_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for s in strings:
    per_iter = per_iter + len(_normalize("NFC", s))
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
