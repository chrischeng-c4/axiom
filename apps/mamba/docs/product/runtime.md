# Runtime

The compiler and runtime side of mamba: CPython 3.12 parity, CPU and memory
under CPython, and the mambalibs replacements. This area is the README
capabilities `cpython-312-parity`, `cpu-and-memory-under-cpython`, and
`mambalibs-end-to-end`. It is second in delivery order: the package manager
ships first, and nothing here changes its contract.

## CPython runtime replacement

- Problem: a CPython 3.12 program compiled by mamba does not yet give the
  CPython result across the conformance corpus, and the memory gate measures
  mamba above CPython on small programs: `cargo test -p mamba --test
  conformance_contract` and `cargo test -p mamba --release --test perf_pin`
  both exit non-zero today. The seven tiers in the README section
  [Runtime replacement order](../../README.md#runtime-replacement-order)
  order that work, and their exit gates are not written.
- Who: a developer who compiles an existing CPython 3.12 program with mamba
  and expects the same observable result with less CPU time and less memory.
- Promise: tier by tier from T1 to T7, the compiled program gives the same
  observable result as under CPython 3.12 for the tier's scope and uses less
  CPU time and less memory than CPython on the tier's fixtures, with each
  tier's exit gate written and named in the README before the tier is
  claimed.
- Non-goals: a mamba runtime as a prerequisite for the package manager; C
  extensions built from sdist.
- Open: which tier opens the first release Milestone after
  `uv-workflow-parity`. Whether each tier's exit gate is a new `[[test]]`
  target or a case set inside the existing conformance targets.
- Neighbours: starts after
  [package-manager.md](package-manager.md) § uv workflow parity; a compiled
  `mamba run <file>` stays the opt-in path that outcome defines.
- Outcome: `cpython-runtime-replacement`. Tracking: Not assigned.

## Non-goals in this area

- `sdist-c-extension-builds`: the runtime replaces C-backed stdlib with
  native paths; it does not compile third-party C extensions from source.
