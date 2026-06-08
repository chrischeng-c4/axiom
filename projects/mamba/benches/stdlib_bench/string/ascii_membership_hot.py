"""string.ascii_letters / digits membership hot-loop bench.

End-user scenario: `if c in string.ascii_letters:` inside a tight
loop, the canonical "is-this-char-a-letter" predicate that backs
every identifier-character validator / username sanitizer / token
char-class classifier / simple lexer-table lookup. CPython routes
through `unicode_contains` (C-level char-scan over the 52-char
string constant); mamba's str-in-str should hit a native scan
through its typed bridge.

Distinct from `isalpha_isdigit_hot.py` (str.isalpha — operates on
the whole string). Here we test the per-character predicate via
`in` membership against a small fixed alphabet — the building
block of any hand-rolled char-class loop.

Bounded context (DDD): stdlib_bench/string.

Tier: compute (linear scan over 52 chars per `in`; no allocation).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `string.ascii_letters` is a module constant; hoist to a local
to dodge per-call module-attr lookup (free var, no bound-method).

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import string
import sys
import time

_LETTERS = string.ascii_letters
_DIGITS = string.digits
TEXT = "Hello123World456Foo789Bar0!@#"
ITERS = 10000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for c in TEXT:
        if c in _LETTERS:
            s = s + 1
        elif c in _DIGITS:
            s = s + 2
    acc = acc + s
_t1 = time.perf_counter()

print("ascii_membership_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for c in TEXT:
    if c in _LETTERS:
        per_iter = per_iter + 1
    elif c in _DIGITS:
        per_iter = per_iter + 2
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
