"""int.from_bytes — bytes-to-int parse perf bench.

End-user scenario: `seq_num = int.from_bytes(payload[:4], "big")` inside
a tight loop, the canonical bytes-to-int decode primitive that backs
every binary record reader (TLV/length-prefix parser) / network packet
sequence-number extractor / file-format chunk-header decoder / protocol
buffer varint reader. CPython routes through int_from_bytes (C-level
digit-buffer build from byteorder); mamba's int should hit a native
impl through its typed bridge.

Inverse of `to_bytes_hot.py` — parse instead of serialize. Same
fixed-width big-endian path as the dominant wire-protocol case.

Bounded context (DDD): language_bench/integers.

Tier: compute (with per-call new-PyLong; for small ints CPython
interns 0..256 but bench values cross the boundary).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `from_bytes` is a class method on int; DO NOT hoist
`_fb = int.from_bytes` — bound-method hoist returns None silently
under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

BUFFERS = (b"\x00\x00\x00\x01", b"\x00\x00\x00\xff", b"\x00\x00\x01\x00",
           b"\x00\x00\xff\xff", b"\x00\xff\xff\xff", b"\xff\xff\xff\xff",
           b"\x01\x02\x03\x04", b"\x10\x20\x30\x40", b"\x7f\xff\xff\xff",
           b"\x80\x00\x00\x00", b"\xab\xcd\xef\x01", b"\x12\x34\x56\x78",
           b"\x00\x00\x00\x00", b"\x55\x55\x55\x55", b"\xaa\xaa\xaa\xaa",
           b"\xde\xad\xbe\xef")
ORDER = "big"
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for b in BUFFERS:
        s = s + int.from_bytes(b, ORDER)
    acc = acc + s
_t1 = time.perf_counter()

print("from_bytes_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for b in BUFFERS:
    per_iter = per_iter + int.from_bytes(b, ORDER)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
