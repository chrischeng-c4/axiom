"""str.strip — whitespace-strip perf bench.

End-user scenario: `clean = raw.strip()` inside a tight loop, the
canonical input-normalization primitive that backs every CLI-arg
trimmer / form-input cleaner / file-line whitespace strip / CSV-cell
trim / config-key normalizer. CPython routes through unicode_strip
(C-level both-ends whitespace scan + new-str slice); mamba's str
should hit a native impl through its typed bridge.

Distinct from `replace`/`split` — strip is bounded by the prefix and
suffix only (no full-buffer scan), so on long strings with short
whitespace runs it's nearly O(prefix+suffix).

Bounded context (DDD): language_bench/strings.

Tier: compute (with per-call new-str slice alloc; cheap when no
strip needed since CPython returns self).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `strip` is a str method; DO NOT hoist `_st = RAW.strip` —
bound-method hoist returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

RAW = "   \t  hello world  \t\n  "
ITERS = 50000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    acc = acc + len(RAW.strip())
_t1 = time.perf_counter()

print("strip_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = len(RAW.strip())
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
