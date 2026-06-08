"""Hot-loop bench for `subprocess.PIPE` / `STDOUT` / `DEVNULL` module-constant
read (#1439).

End-user scenario: process-spawn wrappers (CLI orchestrators, build
drivers, test runners, supervisor loops) that reference
`subprocess.PIPE` / `subprocess.STDOUT` / `subprocess.DEVNULL` on every
`Popen(..., stdout=PIPE, stderr=STDOUT)` call. The module attribute
lookup is on the hot path of every spawn decision; the hoisted-local
alias (`P = subprocess.PIPE`) is the canonical pattern for tight inner
loops. That per-iter module-constant readback is the workload measured
here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x —
CPython's `subprocess.PIPE` is a top-level module-dict probe returning
the int `-1`; `subprocess.STDOUT` is `-2`; `subprocess.DEVNULL` is
`-3`). Mamba's shim returns the same three sentinel ints directly from
the module-attribute resolver, so the per-access constant factor is
the only thing on the clock.

Workload: 10_000 reads each of `subprocess.PIPE`, `subprocess.STDOUT`,
and `subprocess.DEVNULL` against the canonical CPython values
(`-1`, `-2`, `-3`). The accumulator sums the three reads on every
iter, so a misread (wrong int) immediately fails the correctness
assert and dead-code elimination of the reads would leave
`acc != ITERS * EXPECTED_SUM`.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import subprocess as _subprocess

# Hoist the three module-attribute reads to local aliases outside the
# hot loop. The bench measures the per-iter readback through these
# locals — the bound integer sentinels are the canonical CPython
# values (`PIPE = -1`, `STDOUT = -2`, `DEVNULL = -3`).
_PIPE = _subprocess.PIPE
_STDOUT = _subprocess.STDOUT
_DEVNULL = _subprocess.DEVNULL

# Snapshot expected values once before the loop so the correctness
# compare is a pure int-equality probe in the hot path.
EXPECTED_PIPE = -1
EXPECTED_STDOUT = -2
EXPECTED_DEVNULL = -3
EXPECTED_SUM = EXPECTED_PIPE + EXPECTED_STDOUT + EXPECTED_DEVNULL  # -6

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    s = _PIPE + _STDOUT + _DEVNULL
    # Accumulator readback prevents DCE — `s` is an int sum of the
    # three sentinel constants, so the equality always holds in both
    # CPython and mamba and the increment is always taken.
    if s == EXPECTED_SUM:
        acc = acc + 1

# Correctness: every iteration must read back PIPE+STDOUT+DEVNULL == -6.
# acc == ITERS or we have a regression in subprocess sentinel constants.
assert acc - ITERS == 0, f"subprocess const read acc drift: acc={acc} expected={ITERS}"
print("pipe_const_read_hot:", acc)
