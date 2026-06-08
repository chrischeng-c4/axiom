"""str.count — substring tally perf bench.

End-user scenario: `text.count(needle)` inside a tight loop, the
canonical substring-tally primitive that backs every newline counter
for line-numbering / pattern-density gauge in log analysis / template
placeholder count / motif frequency in DNA-like strings. CPython
routes through stringlib_count (C-level); mamba's str should hit a
native impl through its typed bridge.

Bounded context (DDD): language_bench/strings.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; str.count is a method.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

TEXT = "the quick brown fox jumps over the lazy dog\n" * 50
NEEDLE = "the"
ITERS = 100000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    acc = acc + TEXT.count(NEEDLE)
_t1 = time.perf_counter()

print("str_count_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * TEXT.count(NEEDLE)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
