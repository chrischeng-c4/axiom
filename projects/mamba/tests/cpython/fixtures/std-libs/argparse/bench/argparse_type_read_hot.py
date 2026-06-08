"""Hot-loop bench for `argparse.ArgumentParser` module-attribute
reads (#1442).

End-user scenario: CLI-driver glue (release scripts, wrapper
binaries, dev-tooling shims) typically reads
`argparse.ArgumentParser` on every entry-point site rather than
caching a local alias. Wrapper code that constructs
`parser = argparse.ArgumentParser(prog=..., description=...)` on
each invocation re-resolves the name through the `argparse`
module's attribute table at every call site. That per-call
module-attribute read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x --
CPython's `argparse.ArgumentParser` is a top-level module-dict
probe on 3.12 returning a class). Mamba's shim returns the same
identity-stable sentinel directly from a dense constant table in
the `argparse` module-attribute resolver, short-circuiting
CPython's module-dict probe chain for the read-only
`ArgumentParser` sentinel.

Workload: 80_000 reads of `argparse.ArgumentParser` per iteration
(ITERS scaled to 80_000 so 1 attr x 80k = ~80k attr-reads per
run, matching the per-spawn budget of the 8-attr fixtures at
10_000 iters, the 4-attr fixtures at 20_000 iters, and the 2-attr
fixtures at 40_000 iters). The value is re-resolved from the
`argparse` module-attribute table on every iter (not hoisted to a
local) and identity-compared against the hoisted baseline
reference; the accumulator increments on every match.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import argparse as _ap

_ARGUMENT_PARSER_BASELINE = _ap.ArgumentParser

ITERS = 80_000

acc = 0
for _ in range(ITERS):
    a = _ap.ArgumentParser
    if a is _ARGUMENT_PARSER_BASELINE:
        acc = acc + 1

assert acc - ITERS == 0, f"argparse module-attribute read acc drift: acc={acc} expected={ITERS}"
print("argparse_type_read_hot:", acc)
