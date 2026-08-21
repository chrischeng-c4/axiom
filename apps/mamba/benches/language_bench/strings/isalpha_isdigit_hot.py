"""str.isalpha / str.isalnum / str.isdigit — char-class predicate bench.

End-user scenario: per-token classification via `s.isalpha()` etc.
inside a tight loop, the canonical input-sanitation primitive that
backs every lexer character-class check / form-field validator /
identifier-vs-literal disambiguator / numeric-string guard. CPython
routes through PyUnicode_IsAlpha and friends (C-level Unicode tables);
mamba's str should hit a native impl through its typed bridge.

Bounded context (DDD): language_bench/strings.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module-level attrs to hoist; these are methods.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

TOKENS = ["hello", "world42", "12345", "x", "abc!", "ABC", "0xFF", "snake_case", "", "go"]
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for t in TOKENS:
        if t.isalpha():
            s = s + 1
        if t.isalnum():
            s = s + 2
        if t.isdigit():
            s = s + 4
    acc = acc + s
_t1 = time.perf_counter()

print("isalpha_isdigit_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for t in TOKENS:
    if t.isalpha():
        per_iter = per_iter + 1
    if t.isalnum():
        per_iter = per_iter + 2
    if t.isdigit():
        per_iter = per_iter + 4
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
