"""Hot-loop bench for `idna.encode` / `idna.decode` /
`idna.IDNAError` / `idna.alabel` module-attribute reads (#1486).

End-user scenario: URL normalization and DNS-handling code (`requests`,
`httpx`, `urllib3`, async stacks, etc.) re-resolves `idna.encode` /
`idna.decode` / `idna.alabel` on every hostname punycode round-trip
and `idna.IDNAError` on every error catch. Wrapper code that
normalizes hostnames per-request re-resolves these names through
the module's attribute table on each call site. That per-call
module-attribute quad-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 `idna.encode` / `idna.decode` / `idna.alabel` are
pure-Python functions and `idna.IDNAError` is a top-level exception
class, all routed through the `idna` module dict). Mamba's shim
returns the same identity-stable sentinels directly from a dense
constant table in the `idna` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
sentinels.

Workload: 20_000 paired reads of `encode`, `decode`, `IDNAError`,
and `alabel` per iteration (ITERS scaled to 20_000 so 4 attrs x 20k
= ~80k attr-reads per run, matching the per-spawn budget of the
8-attr fixtures at 10_000 iters, the 2-attr fixtures at 40_000
iters, and the 1-attr fixtures at 80_000 iters). All four values
are re-resolved from the `idna` module-attribute table on every
iter (not hoisted to a local) and identity-compared against the
hoisted baseline references; the accumulator increments when all
four reads resolve to identical objects.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import idna as _idna

_ENCODE_BASELINE = _idna.encode
_DECODE_BASELINE = _idna.decode
_IDNA_ERROR_BASELINE = _idna.IDNAError
_ALABEL_BASELINE = _idna.alabel

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = _idna.encode
    b = _idna.decode
    c = _idna.IDNAError
    d = _idna.alabel
    if (a is _ENCODE_BASELINE
            and b is _DECODE_BASELINE
            and c is _IDNA_ERROR_BASELINE
            and d is _ALABEL_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"idna module-attribute read acc drift: acc={acc} expected={ITERS}"
print("idna_type_read_hot:", acc)
