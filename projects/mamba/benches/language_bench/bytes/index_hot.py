"""bytes[i] — per-byte index access perf bench.

End-user scenario: tight loop `for i in range(len(buf)): b = buf[i]`
or its functional equivalent, the canonical byte-by-byte read
primitive that backs every hand-rolled binary parser / checksum
accumulator / byte-frequency histogram / hex-dump renderer. CPython
routes through bytes_subscript (C-level array bounds-check + int
unbox-to-Python-int); mamba's bytes should hit a native i8 load
through its typed bridge.

Distinct from `bytes/find_hot.py` (whole-buffer scan via C) — this
exercises the per-element subscript dispatch, which on CPython
returns a per-call Python int (boxed via small-int cache for 0–255).

Bounded context (DDD): language_bench/bytes.

Tier: compute (per-call int unbox; on CPython 0–255 is cached so no
new-PyLong, on mamba native i8 → i64 promotion is free).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: indexing is a syntax op — no hoisting concern.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

BUF = (b"The quick brown fox jumps over the lazy dog "
       b"0123456789 ABCDEFGHIJKLMNOPQRSTUVWXYZ "
       b"abcdefghijklmnopqrstuvwxyz!@#$%^&*()_+-=")
BUF_LEN = len(BUF)
ITERS = 5000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for i in range(BUF_LEN):
        s = s + BUF[i]
    acc = acc + s
_t1 = time.perf_counter()

print("bytes_index_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for i in range(BUF_LEN):
    per_iter = per_iter + BUF[i]
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
