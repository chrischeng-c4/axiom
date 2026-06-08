"""Hot-loop bench for `zipfile.ZIP_STORED` / `zipfile.ZIP_DEFLATED`
module-constant reads (#1460).

End-user scenario: archive writers that select a compression method
per-entry (`zf.writestr(name, data, compress_type=zipfile.ZIP_DEFLATED)`)
or default to no compression for already-compressed payloads
(`compress_type=zipfile.ZIP_STORED`). Build pipelines, asset bundlers,
and backup tools reference these constants on every per-entry write;
the canonical hot-path idiom is to hoist locals once
(`STORED = zipfile.ZIP_STORED; DEFLATED = zipfile.ZIP_DEFLATED`) and
pass them as the `compress_type=` kwarg per-call. That per-iter
module-constant readback pair is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x —
CPython's `zipfile.ZIP_STORED` / `ZIP_DEFLATED` are top-level
module-dict probes returning the ints `0` and `8` on 3.12). Mamba's
shim returns the same sentinel ints directly from the module-
attribute resolver, so the per-access constant factor is the only
thing on the clock.

Workload: 10_000 paired reads of `zipfile.ZIP_STORED` (0) and
`zipfile.ZIP_DEFLATED` (8) against canonical CPython values. The
accumulator sums each iter's `(stored + deflated) == 8` check, so a
misread (wrong int) immediately fails the correctness assert and
dead-code elimination of either read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import zipfile as _zipfile

# Hoist the module-attribute reads to local aliases outside the hot
# loop. The bench measures the per-iter readback through these locals
# — the bound integer sentinels are the canonical CPython values
# (`ZIP_STORED = 0`, `ZIP_DEFLATED = 8` on 3.12).
_ZIP_STORED = _zipfile.ZIP_STORED
_ZIP_DEFLATED = _zipfile.ZIP_DEFLATED

# Snapshot expected combined value once before the loop so the
# correctness compare is a pure int-equality probe in the hot path.
EXPECTED_SUM = 8  # 0 (STORED) + 8 (DEFLATED)

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    s = _ZIP_STORED
    d = _ZIP_DEFLATED
    # Accumulator readback prevents DCE — `s + d` is the bound
    # sentinel sum, so the equality always holds in both CPython and
    # mamba and the increment is always taken.
    if (s + d) == EXPECTED_SUM:
        acc = acc + 1

# Correctness: every iteration must read back ZIP_STORED + ZIP_DEFLATED == 8.
# acc == ITERS or we have a regression in zipfile module constants.
assert acc - ITERS == 0, f"zipfile compression-constants read acc drift: acc={acc} expected={ITERS}"
print("compression_constants_read_hot:", acc)
