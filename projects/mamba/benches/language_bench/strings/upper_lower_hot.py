"""str.upper / str.lower — case-fold perf bench.

End-user scenario: `email.lower()` / `header.upper()` inside a tight
loop, the canonical case-normalize primitive that backs every
case-insensitive comparison precursor / email/username canonicalizer /
HTTP-header name normalizer / SQL identifier case-folder. CPython
routes through unicode_upper_impl / unicode_lower_impl (C-level
scan + new-string build with Unicode tables); mamba's str should hit
a native impl through its typed bridge.

Bounded context (DDD): language_bench/strings.

Tier: compute (with new-string allocation per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `upper`/`lower` are str methods; DO NOT hoist `_up = TEXT.upper`
— bound-method hoist returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

TEXT = "The Quick Brown Fox Jumps Over The Lazy Dog And Then The Fox Jumps Again"
ITERS = 30000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    u = TEXT.upper()
    l = TEXT.lower()
    acc = acc + len(u) + len(l)
_t1 = time.perf_counter()

print("upper_lower_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = len(TEXT.upper()) + len(TEXT.lower())
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
