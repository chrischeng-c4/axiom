"""Hot-loop bench for `tarfile.open` / `tarfile.is_tarfile`
module-attribute reads (#1461).

End-user scenario: archive-tooling glue (release packagers, log
rotators, sdist builders, CI artifact uploaders) typically reads
`tarfile.open` and `tarfile.is_tarfile` on every archive site
rather than caching a local alias. Wrapper code that probes a path
via `if tarfile.is_tarfile(path): tar = tarfile.open(path, 'r:gz')`
re-resolves these two names through the `tarfile` module's
attribute table on each call site. That per-call module-attribute
pair-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
CPython's `tarfile.open` and `tarfile.is_tarfile` are top-level
module-dict probes on 3.12 returning a function and a callable
class respectively). Mamba's shim returns the same identity-stable
sentinels directly from a dense constant table in the `tarfile`
module-attribute resolver, short-circuiting CPython's module-dict
probe chain for read-only tarfile sentinels.

Workload: 40_000 paired reads of `tarfile.open` and
`tarfile.is_tarfile` per iteration (ITERS quadrupled to 40_000 so
2 attrs x 40k = ~80k attr-reads per run, matching the per-spawn
budget of the 8-attr fixtures at 10_000 iters and the 4-attr
fixtures at 20_000 iters). Both values are re-resolved from the
`tarfile` module-attribute table on every iter (not hoisted to a
local) and identity-compared against the hoisted baseline
references; the accumulator increments when both reads resolve to
identical objects.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import tarfile as _tarfile

_OPEN_BASELINE = _tarfile.open
_IS_TARFILE_BASELINE = _tarfile.is_tarfile

ITERS = 40_000

acc = 0
for _ in range(ITERS):
    a = _tarfile.open
    b = _tarfile.is_tarfile
    if a is _OPEN_BASELINE and b is _IS_TARFILE_BASELINE:
        acc = acc + 1

assert acc - ITERS == 0, f"tarfile module-attribute read acc drift: acc={acc} expected={ITERS}"
print("tarfile_type_read_hot:", acc)
