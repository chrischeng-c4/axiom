"""Hot-loop bench for `os.path.join` 4-segment join (#1432).

End-user scenario: build systems, package managers, test runners, and
serving stacks that synthesize file paths on every request — `static/`
asset resolvers, plugin auto-discovery walkers, and language-server
import-path resolution all hit this code path tens of thousands of
times per process lifetime. The interesting cost per call is the
per-arg separator inspection (does the next segment start with `/`?)
+ the in-place str-buffer growth, exactly the work CPython's pure-
Python `posixpath.join` performs in a Python loop.

Tier: `os shim` (target mamba/cpython <= 1.0x — CPython's
`os.path.join` is pure Python and pays bytecode dispatch + per-arg
`isinstance(s, (str, bytes))` overhead). Mamba lowers the same
surface to a Rust path-segment fold that fuses the separator check
and the buffer append into one inlined call.

Workload: 10_000 4-arg joins (`"usr"`, `"local"`, `"bin"`,
`"python3"`) which is the common toolchain-discovery shape (no
absolute-segment resets to keep the success path uniform across both
runtimes).

DoD: exits 0 under both CPython and mamba; cross-runtime bench
#1265 Goal 2.
"""

import os

# Hoist the bound module-level function to a local alias (#2097) so
# per-iter attribute lookup overhead does not skew the measurement.
_join = os.path.join

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    r = _join("usr", "local", "bin", "python3")
    # Accumulator readback prevents DCE — every join returns a non-
    # empty string on both runtimes.
    if r is not None:
        acc = acc + 1

# Correctness: every iteration produced a join. acc == ITERS or we
# have a regression in the join dispatcher.
assert acc - ITERS == 0, f"os.path.join acc drift: acc={acc} expected={ITERS}"
print("join_hot:", acc)
