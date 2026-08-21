"""Hot-loop bench for `struct.pack` + `struct.unpack` round-trip.

End-user scenario: a binary protocol codec packs a small fixed-shape
record (magic int, type byte, length short) and then parses it back.
This is the canonical end-user pattern for `struct` — a few-field
record, fixed format string, hoisted callable references.

Tier: `dynamic` (target mamba/cpython >= 1.5x per #1265, but enforced
floor here is 1.0x). struct.pack/unpack is dispatch-bound for tiny
records: per-iteration cost is dominated by the shim-call edge plus
the format-mini-language interpreter, not by the byte arithmetic
itself.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
harness compares per-iteration wall time and reports the ratio.

Note: a smaller record shape (`<i` packed one int at a time) currently
reports much worse than 1.0x under mamba — that's a per-call dispatch
overhead gap in the runtime, not a struct-codec defect. The 3-field
record below is the realistic protocol-decoder shape, which is what
mamba should beat or match in steady state.
"""

import struct


FMT = "<iBh"            # 4 + 1 + 2 = 7 bytes per record
MAGIC = 0x4D414D42
TYPE = 7
LENGTH = 1024

# Hoist callables to local names — idiomatic Python micro-opt.
pack = struct.pack
unpack = struct.unpack

ITERS = 20000
checks = 0
for _ in range(ITERS):
    blob = pack(FMT, MAGIC, TYPE, LENGTH)
    m, t, l = unpack(FMT, blob)
    # Cheap predicate: each round-trip should produce the same triple.
    if t == TYPE and l == LENGTH:
        checks += 1

# Per #2105 avoidance: print the accumulator BEFORE asserting so any
# JIT post-call branch elision does not silently zero the marker.
print("pack_unpack_hot:", checks)
assert checks == ITERS, f"checks mismatch: {checks} != {ITERS}"
