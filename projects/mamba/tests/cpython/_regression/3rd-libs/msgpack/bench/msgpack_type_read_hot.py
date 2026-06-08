"""Hot-loop bench for `msgpack.pack` / `msgpack.unpack` /
`msgpack.Packer` / `msgpack.Unpacker` module-attribute reads (#1499).

End-user scenario: msgpack-using services re-resolve
`msgpack.pack` (one-shot encode entry),
`msgpack.unpack` (one-shot decode entry),
`msgpack.Packer` (streaming encoder class), and
`msgpack.Unpacker` (streaming decoder class) on every call site.
Per-call attribute resolution goes through the `msgpack`
module's attribute table on each call site. That per-call
module-attribute quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the `msgpack`
module via Python-side wrappers). Mamba's shim returns the same
identity-stable sentinels directly from a dense constant table
in the `msgpack` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for read-only sentinels.

Workload: 20_000 paired reads of `pack`, `unpack`, `Packer`, and
`Unpacker` per iteration (ITERS scaled so 4 attrs x 20_000 = ~80k
attr-reads per run, matching the cross-tier 80k attr-read budget
used by the 4-attr 3p perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import msgpack


_PACK_BASELINE = msgpack.pack
_UNPACK_BASELINE = msgpack.unpack
_PACKER_BASELINE = msgpack.Packer
_UNPACKER_BASELINE = msgpack.Unpacker

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = msgpack.pack
    b = msgpack.unpack
    c = msgpack.Packer
    d = msgpack.Unpacker
    if (a is _PACK_BASELINE
            and b is _UNPACK_BASELINE
            and c is _PACKER_BASELINE
            and d is _UNPACKER_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"msgpack module-attribute read acc drift: acc={acc} expected={ITERS}"
print("msgpack_type_read_hot:", acc)
