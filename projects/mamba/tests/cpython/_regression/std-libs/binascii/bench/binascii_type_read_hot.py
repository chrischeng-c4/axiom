"""Hot-loop bench for `binascii.hexlify` / `binascii.unhexlify` /
`binascii.b2a_base64` / `binascii.a2b_base64` module-attribute
reads (#1261).

End-user scenario: binascii-using encoding code re-resolves
`hexlify` / `unhexlify` (hex codec) and `b2a_base64` /
`a2b_base64` (base64 codec) on every call site. Per-call
attribute resolution goes through the `binascii` module's
attribute table on each call site. That per-call module-attribute
quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `hexlify`, `unhexlify`,
`b2a_base64`, and `a2b_base64` per iteration (ITERS scaled so
4 attrs x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import binascii


_HX_BASELINE = binascii.hexlify
_UHX_BASELINE = binascii.unhexlify
_B2A_BASELINE = binascii.b2a_base64
_A2B_BASELINE = binascii.a2b_base64

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = binascii.hexlify
    b = binascii.unhexlify
    c = binascii.b2a_base64
    d = binascii.a2b_base64
    if (a is _HX_BASELINE
            and b is _UHX_BASELINE
            and c is _B2A_BASELINE
            and d is _A2B_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"binascii module-attribute read acc drift: acc={acc} expected={ITERS}"
print("binascii_type_read_hot:", acc)
