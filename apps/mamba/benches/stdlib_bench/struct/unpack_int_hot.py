"""struct.unpack — binary parse perf bench.

End-user scenario: `struct.unpack(">IHB", header_bytes)` inside a
tight loop, the canonical binary-record decoder primitive that backs
every wire-protocol header parser / binary file-format reader (PNG/
RIFF/WAV chunks) / network packet field extractor / on-disk index
entry decoder. CPython routes through unpack_impl (C-level switch
over format chars + scalar reads into a PyTuple); mamba's struct
should hit a native impl through its typed bridge.

Distinct from `pack_int_hot.py` which is the encode direction;
unpack exercises the byteswap + PyLong-from-bytes + tuple-build path,
not the format-driven write path.

Bounded context (DDD): stdlib_bench/struct.

Tier: compute (with new-tuple allocation per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `unpack` is a module-level free fn; safe to hoist locally.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import struct
import sys
import time

_unpack = struct.unpack
FMT = ">IHB"
# Header: 0x01020304 (u32) | 0x0506 (u16) | 0x07 (u8) — 7 bytes total.
HDR = b"\x01\x02\x03\x04\x05\x06\x07"
ITERS = 30000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    a, b, c = _unpack(FMT, HDR)
    acc = acc + a + b + c
_t1 = time.perf_counter()

print("unpack_int_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

a, b, c = struct.unpack(FMT, HDR)
per_iter = a + b + c
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
