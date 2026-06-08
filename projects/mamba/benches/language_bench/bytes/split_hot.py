"""bytes.split — byte-stream tokenizer perf bench.

End-user scenario: `payload.split(b",")` inside a tight loop, the
canonical wire-format tokenize primitive that backs every CSV-bytes
row splitter / HTTP header line dissector / TLV record demultiplexer
/ log-shipper column parser. CPython routes through bytes_split
(C-level scan + per-slice Bytes new); mamba's bytes should hit a
native impl through its typed bridge.

Bounded context (DDD): language_bench/bytes.

Tier: compute (with new-Bytes allocation per slice).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `split` is a bytes method; DO NOT hoist `_split = LINE.split`
— bound-method hoist returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

LINE = b"2026-05-27T10:14:33Z,INFO,api,handler.py:142,request_id=abc123,user=42,latency_ms=87,status=200"
SEP = b","
ITERS = 30000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    parts = LINE.split(SEP)
    s = 0
    for p in parts:
        s = s + len(p)
    acc = acc + s
_t1 = time.perf_counter()

print("split_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for p in LINE.split(SEP):
    per_iter = per_iter + len(p)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
