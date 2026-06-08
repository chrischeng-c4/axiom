"""String concatenation hot-loop bench — language-core string perf.

End-user scenario: tight loop over `s = s + small_str`, the naive
form every Python tutorial warns about but which still shows up in
log formatters, code generators, and template engines that haven't
adopted str.join. CPython has a special optimisation for the
single-reference case where it mutates the underlying PyUnicode
buffer in place; mamba's MbValue::Str carries an owned String and
clones on rebind.

Bounded context (DDD): language_bench/strings — first member of
the language-core string perf suite.

Tier: compute. Mamba doesn't currently do CPython's in-place
mutation trick, so this bench is honest about a known regression
domain — exactly the kind of perf surface the test corpus is meant
to keep visible.

#2105: print of `len(s)` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

ITERS = 10_000  # bounded — s grows linearly so total work is O(ITERS^2)
CHUNK = "x"

s = ""
_t0 = time.perf_counter()
for _ in range(ITERS):
    s = s + CHUNK
_t1 = time.perf_counter()

print("concat_hot len:", len(s))
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Cheap checksum — final length must match the iteration count.
assert len(s) == ITERS, f"length mismatch: len(s)={len(s)} expected={ITERS}"
