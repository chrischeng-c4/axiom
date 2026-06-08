"""str.format — positional-args templating perf bench.

End-user scenario: `"{}/{}/{}".format(a, b, c)` inside a tight loop,
the canonical templated-string builder primitive that backs every
log-line formatter / URL path constructor / CSV row emitter / SQL
parameter renderer. CPython routes through unicode_format (C-level
parse + per-field str() + arena join); mamba's str should hit a
native impl through its typed bridge.

Bounded context (DDD): language_bench/strings.

Tier: compute (with new-string allocation per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `format` is a bound method on a str literal; DO NOT hoist
`_fmt = TEMPLATE.format` — bound-method hoist returns None silently
under mamba (see Random.sample / re.Pattern.search quirks).

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

TEMPLATE = "id={}/score={}/tag={}"
N = 100
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for i in range(N):
        s = s + len(TEMPLATE.format(i, i * 2, i * 3))
    acc = acc + s
_t1 = time.perf_counter()

print("format_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for i in range(N):
    per_iter = per_iter + len(TEMPLATE.format(i, i * 2, i * 3))
# Subtraction-style (mamba accumulator-vs-arith int comparison bug).
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
