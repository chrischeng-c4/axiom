"""Hot-loop bench for `sys.byteorder` module-attribute read (#1434).

End-user scenario: serialization libraries (struct packing wrappers,
binary protocol codecs, numpy-free array shims) that branch on host
endianness on every record. The cost is a single module-attribute
lookup returning a small interned string ("little" or "big"), but it
appears in *every* per-record fast path, so the per-access constant
factor is what matters. Mamba's edge is collapsing the lookup chain
(module dict -> string ref) into a direct constant-string return.

Tier: `runtime constant` (target mamba/cpython <= 1.0x — CPython's
`sys.byteorder` access is a `module.__dict__` lookup; the value is a
real `str` object whose identity is stable across the whole process).

Workload: 10_000 reads of `sys.byteorder`, comparing each read against
the expected platform value ("little" on the reference machine — both
runtimes are macOS aarch64). The accumulator is incremented on every
matching read, so a misread immediately fails the correctness assert
and a dead-code elimination of the read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
and reports the ratio. Floor is 1.0x per #1265 Goal 2.
"""

import sys

# Hoist the bound module to a local alias (#2097) so per-iter attribute
# lookup overhead is the *only* thing we measure — the LOAD_GLOBAL ->
# module-dict lookup chain is the hot path under test.
_sys = sys
EXPECTED = sys.byteorder  # snapshot once for correctness comparison

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    v = _sys.byteorder
    # Accumulator readback prevents DCE — `v` is the interned platform
    # endianness string ("little" on the reference aarch64 box), so the
    # equality always holds and the increment is always taken.
    if v == EXPECTED:
        acc = acc + 1

# Correctness: every iteration must read back the same endianness
# string. acc == ITERS or we have a regression in module-attribute
# stability.
assert acc - ITERS == 0, f"sys.byteorder acc drift: acc={acc} expected={ITERS}"
print("byteorder_hot:", acc)
