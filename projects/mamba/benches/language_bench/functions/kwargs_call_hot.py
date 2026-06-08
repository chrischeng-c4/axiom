"""Keyword-arg function call hot-loop bench — functions perf.

End-user scenario: `req = make_request(url=u, timeout=10, retries=3)`
inside a tight loop, the canonical named-args dispatch primitive that
backs every API-call site / factory constructor with config kwargs /
options-bag passing / configurator-style fn boundary. CPython routes
through CALL_FUNCTION_KW + co_varnames-walk for kw lookup; mamba's
kw-arg dispatch should hit a typed-arg-bind path through its native
bridge.

Distinct from `call_overhead_hot.py` (positional-only call cost).
kwargs dispatch is strictly more work — each keyword must be matched
to a parameter slot, and the keyword-tuple must be built at the call
site (FORMAT_VALUE-style allocation per call).

Bounded context (DDD): language_bench/functions.

Tier: compute (with per-call keyword-tuple build + kw bind walk).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: a plain `def` fn is a module-level free fn; safe to call
directly each iter.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time


def make_record(_name, age, score, tier):
    return age + score + tier


N = 1000
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for i in range(N):
        s = s + make_record(_name="row", age=i, score=i * 2, tier=3)
    acc = acc + s
_t1 = time.perf_counter()

print("kwargs_call_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for i in range(N):
    per_iter = per_iter + make_record(_name="row", age=i, score=i * 2, tier=3)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
