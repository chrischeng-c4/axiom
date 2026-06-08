"""Hot-loop bench for `pathlib.PurePath` / `pathlib.PurePosixPath` /
`pathlib.PureWindowsPath` / `pathlib.Path` module-attribute reads
(#1433).

End-user scenario: cross-platform path-handling code that branches
on `isinstance(p, pathlib.PurePosixPath)` /
`isinstance(p, pathlib.PureWindowsPath)` to pick the right OS-aware
helper, plus generic dispatch through `pathlib.PurePath` and
`pathlib.Path`. The canonical hot-path idiom is to read the type
families directly off the `pathlib` module each call rather than
caching a local — keeps the call site robust against late-binding
patterns (test monkey-patching, plugin reloads). That per-iter
module-attribute quad-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x —
CPython's `pathlib.PurePath` family are top-level module-dict probes
returning the class objects on 3.12). Mamba's shim returns the
same callable objects directly from a dense constant table in the
`pathlib` module-attribute resolver, short-circuiting CPython's
module-dict probe chain for read-only class sentinels.

Workload: 10_000 paired reads of `pathlib.PurePath`,
`pathlib.PurePosixPath`, `pathlib.PureWindowsPath`, and
`pathlib.Path` per iteration, compared by identity (`is`) against
the hoisted baseline references taken once before the loop. The
accumulator increments when all four reads resolve to the identical
class objects; a misread (different identity / wrong binding) would
immediately fail the correctness assert and dead-code elimination of
any read would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import pathlib as _pathlib

# Hoist baseline references to the canonical class objects once
# before the loop. The hot path re-reads the module attribute on
# every iter so the bench actually exercises the module-attribute
# resolver — the `is` compare against the hoisted baseline is the
# correctness probe.
_PUREPATH_BASELINE = _pathlib.PurePath
_PUREPOSIX_BASELINE = _pathlib.PurePosixPath
_PUREWIN_BASELINE = _pathlib.PureWindowsPath
_PATH_BASELINE = _pathlib.Path

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    pp = _pathlib.PurePath
    pposix = _pathlib.PurePosixPath
    pwin = _pathlib.PureWindowsPath
    p = _pathlib.Path
    # Accumulator readback prevents DCE — every iteration must
    # resolve to the identical class objects bound at the
    # `pathlib.PurePath` / `pathlib.PurePosixPath` /
    # `pathlib.PureWindowsPath` / `pathlib.Path` module slots.
    if (pp is _PUREPATH_BASELINE
            and pposix is _PUREPOSIX_BASELINE
            and pwin is _PUREWIN_BASELINE
            and p is _PATH_BASELINE):
        acc = acc + 1

# Correctness: every iteration must read back the canonical class
# objects via the module-attribute resolver. acc == ITERS or we have
# a regression in mamba's pathlib module-attribute table.
assert acc - ITERS == 0, f"pathlib module-attribute read acc drift: acc={acc} expected={ITERS}"
print("pure_path_type_read_hot:", acc)
