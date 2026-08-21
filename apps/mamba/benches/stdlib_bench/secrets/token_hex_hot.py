"""secrets.token_hex — crypto-random hex string perf bench.

End-user scenario: `secrets.token_hex(16)` inside a tight loop, the
canonical session-token / api-key / csrf-token / signed-cookie nonce
primitive. CPython routes through secrets.token_hex -> os.urandom +
binascii.hexlify; mamba's secrets should hit the same urandom call
through its typed bridge.

Bounded context (DDD): stdlib_bench/secrets.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `token_hex` to a local before the hot loop.
"""

import secrets
import sys
import time

_token_hex = secrets.token_hex

ITERS = 100_000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    s = _token_hex(16)
    total = total + len(s)
_t1 = time.perf_counter()

print("token_hex_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# token_hex(16) returns exactly 32 hex chars per call.
expected = ITERS * 32
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
