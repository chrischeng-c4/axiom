"""Hot-loop bench for `s3transfer.TransferManager` /
`s3transfer.TransferConfig` / `s3transfer.S3Transfer` /
`s3transfer.tasks` module-attribute reads (#1503).

End-user scenario: s3transfer-using services re-resolve
`s3transfer.TransferManager` (high-level transfer manager),
`s3transfer.TransferConfig` (transfer configuration),
`s3transfer.S3Transfer` (low-level transfer class), and
`s3transfer.tasks` (task submodule) on every call site.
Per-call attribute resolution goes through the `s3transfer`
module's attribute table on each call site. That per-call
module-attribute quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
on CPython 3.12 the four entries are attached to the
`s3transfer` module via Python-side wrappers). Mamba's shim
returns the same identity-stable sentinels directly from a dense
constant table in the `s3transfer` module-attribute resolver,
short-circuiting CPython's module-dict probe chain for read-only
sentinels.

Workload: 20_000 paired reads of `TransferManager`,
`TransferConfig`, `S3Transfer`, and `tasks` per iteration (ITERS
scaled so 4 attrs x 20_000 = ~80k attr-reads per run, matching
the cross-tier 80k attr-read budget used by the 4-attr 3p
perf-pin family).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import s3transfer


_TM_BASELINE = s3transfer.TransferManager
_TC_BASELINE = s3transfer.TransferConfig
_ST_BASELINE = s3transfer.S3Transfer
_TASKS_BASELINE = s3transfer.tasks

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = s3transfer.TransferManager
    b = s3transfer.TransferConfig
    c = s3transfer.S3Transfer
    d = s3transfer.tasks
    if (a is _TM_BASELINE
            and b is _TC_BASELINE
            and c is _ST_BASELINE
            and d is _TASKS_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"s3transfer module-attribute read acc drift: acc={acc} expected={ITERS}"
print("s3transfer_type_read_hot:", acc)
