"""str.replace — substring substitute perf bench.

End-user scenario: `text.replace("\\r\\n", "\\n")` inside a tight loop,
the canonical text-cleaning primitive that backs every line-ending
normalizer / template variable substituter / log-line redactor /
markdown-to-plain converter. CPython routes through unicode_replace
(C-level scan + copy on a single arena); mamba's str should hit a
native impl through its typed bridge.

Bounded context (DDD): language_bench/strings.

Tier: compute (with new-string allocation per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `replace` is a str method; no module-attr hoisting needed.
DO NOT hoist `_rep = TEXT.replace` — bound-method hoist under mamba
silently returns None on some receivers.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

TEXT = "the quick brown fox jumps over the lazy dog and then the fox jumps again the the the"
OLD = "the"
NEW = "THE"
ITERS = 30000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = TEXT.replace(OLD, NEW)
    acc = acc + len(s)
_t1 = time.perf_counter()

print("replace_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = len(TEXT.replace(OLD, NEW))
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
