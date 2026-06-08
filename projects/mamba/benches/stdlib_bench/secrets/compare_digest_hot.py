"""secrets.compare_digest — constant-time equality perf bench.

End-user scenario: `secrets.compare_digest(submitted_token, stored_token)`
inside a tight loop, the canonical timing-attack-safe equality
primitive that backs every CSRF-token verifier / HMAC signature
comparator / API-key validator / session-token check. CPython routes
through compare_digest (C-level constant-time XOR-or-accumulate);
mamba's secrets should hit a native impl through its typed bridge.

This bench probes the HAPPY-PATH (equal strings) cost. compare_digest
intentionally takes the SAME time on equal vs unequal of equal length
to defeat timing attacks — so we measure the floor cost only.

Bounded context (DDD): stdlib_bench/secrets.

Tier: compute (no allocation; pure pairwise byte XOR + accumulate).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `compare_digest` is a module-level free fn; safe to hoist.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import secrets
import sys
import time

_cmp = secrets.compare_digest
TOKEN_A = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
TOKEN_B = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
ITERS = 30000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    if _cmp(TOKEN_A, TOKEN_B):
        acc = acc + 1
_t1 = time.perf_counter()

print("compare_digest_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Strings are identical → every cmp returns True → acc == ITERS.
expected = ITERS
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
