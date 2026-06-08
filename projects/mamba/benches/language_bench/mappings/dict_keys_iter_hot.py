"""dict.keys() — keys iteration perf bench.

End-user scenario: tight loop `for k in d.keys()` (or `for k in d:`),
the canonical key-by-key dict walk that backs every config-key
enumerator / cache-key invalidator / dict-to-list collector / lookup-
table inspector. CPython routes through dict_iter / dict_keys
(C-level slot walk over the dk_entries array); mamba's dict should
hit a native impl through its typed bridge.

Distinct from `dict_items_iter_hot.py` (key+value tuples) — keys()
returns just the key per step (no per-iter 2-tuple alloc), so on
CPython it's strictly cheaper. The ratio here probes mamba's key-only
iterator path.

Bounded context (DDD): language_bench/mappings.

Tier: compute (per-iter dict-slot walk; no per-key alloc since keys
are interned strings).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `keys` is a dict method; DO NOT hoist `_keys = D.keys` —
bound-method hoist returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

D = {"name": 1, "age": 2, "city": 3, "email": 4, "phone": 5,
     "addr": 6, "zip": 7, "country": 8, "lang": 9, "tz": 10,
     "dob": 11, "ssn": 12, "tel": 13, "fax": 14, "url": 15,
     "ip": 16, "mac": 17, "uid": 18, "gid": 19, "pid": 20}
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for k in D.keys():
        s = s + len(k)
    acc = acc + s
_t1 = time.perf_counter()

print("dict_keys_iter_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for k in D.keys():
    per_iter = per_iter + len(k)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
