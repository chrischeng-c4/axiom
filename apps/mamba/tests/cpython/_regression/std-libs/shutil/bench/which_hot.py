"""Hot-loop bench for `shutil.which` lookup (#1465).

End-user scenario: build / CI / launcher scripts that resolve an
executable by name on every invocation — `make`-style toolchain
discovery, language version managers (pyenv shims), and per-task
runner spawn. The interesting cost is the PATH walk + per-entry
`os.path.join` + `os.access(X_OK)` round trip per lookup, where
mamba's edge is a thin Rust implementation that fuses the path
split, join, and access check into a single syscall sweep.

Tier: `os shim` (target mamba/cpython <= 1.0x — CPython's
`shutil.which` is pure-Python and iterates `os.environ['PATH'].split(os.pathsep)`
in a Python loop, calling `os.path.join` and `os.access` per entry).

Workload: 10_000 lookups of an existing executable name ("python3")
to exercise the success path (every iteration walks PATH until it
hits the first match — pyenv shims tend to live early in PATH, but
the work is still dominated by per-iteration Python-level glue).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker) and reports the ratio. Floor is 1.0x per #1265 Goal 2.
"""

import shutil

# Hoist the bound module-level function to a local alias (#2097) so
# per-iter attribute lookup overhead does not skew the measurement.
_which = shutil.which

ITERS = 10_000
NAME = "python3"

acc = 0
for _ in range(ITERS):
    r = _which(NAME)
    # Accumulator readback prevents DCE — `r` is a non-empty string
    # path on both runtimes, so `r is not None` always holds.
    if r is not None:
        acc = acc + 1

# Correctness: every iteration should have resolved python3 to a
# non-empty path. acc == ITERS or we have a regression in the
# success-path branch.
assert acc - ITERS == 0, f"shutil.which acc drift: acc={acc} expected={ITERS}"
print("which_hot:", acc)
