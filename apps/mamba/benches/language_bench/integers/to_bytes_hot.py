"""int.to_bytes — int-to-bytes serialize perf bench.

End-user scenario: `seq_num.to_bytes(4, "big")` inside a tight loop,
the canonical int-to-network-bytes primitive that backs every binary
record builder (TLV/length-prefix) / protocol-buffer varint emitter /
file-format chunk-header writer / network packet sequence-number serializer.
CPython routes through long_to_bytes_impl (C-level memcpy of the digit
buffer in chosen byteorder); mamba's int should hit a native impl
through its typed bridge.

This bench probes the FIXED-WIDTH big-endian path (the dominant
wire-protocol case). Distinct from `bit_length_count_hot.py` which
covers introspection of digit count.

Bounded context (DDD): language_bench/integers.

Tier: compute (with per-call new-Bytes alloc).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `to_bytes` is an int method; DO NOT hoist `_tb = (0).to_bytes`
— bound-method hoist returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

VALS = (0, 1, 255, 256, 65535, 65536, 16777215, 16777216,
        100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 999999999)
WIDTH = 4
ORDER = "big"
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for v in VALS:
        b = v.to_bytes(WIDTH, ORDER)
        s = s + len(b)
    acc = acc + s
_t1 = time.perf_counter()

print("to_bytes_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for v in VALS:
    per_iter = per_iter + len(v.to_bytes(WIDTH, ORDER))
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
