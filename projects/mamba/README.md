# Mamba

## Brief

Force-typed Python compiler. Lexes Python source with `logos`, lowers through HIR/MIR, and emits native machine code via Cranelift JIT/AOT. Not a transpiler, not an interpreter — produces real binaries.

For implementation map, see [llms.txt](llms.txt).

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| C1. Py3.12 functional parity — Axis 1 | #3331 | partial | planned | conformance | not_ready | confirmed README promise; CPython oracle gate remains open |
| C2. Less CPU time AND less memory than CPython — Axis 2 | #3880 | planned | planned | conformance | not_ready | confirmed README promise; CPU/RSS ratio gates remain open |
| C3. mambalibs end-to-end — Axis 3 | #3457 | partial | planned | conformance | not_ready | confirmed README promise; native module coverage remains open |
| C4. Package manager — uv-like | #459 | implemented | verified | conformance | ready | uv-like offline workflow coverage is green across init/auth/index/add/remove/lock/export/tree/version/pip/venv/python/workspace/shell/sync/run/install/tool/hash/cache |

### C1. Py3.12 functional parity — Axis 1

ID: c1-py3-12-functional-parity-axis-1
Type: RuntimeTool
Surfaces: CLI: `mamba build` + `mamba check` + `mamba run` + `mamba test` + `mamba test-batch` + `mamba pytest` + `mamba surface-report` - compile, type-check, run, batch, pytest, and surface-conformance entrypoints
EC Dimensions: behavior: `cargo test -p mamba --test conformance_cpython_lib_test --release` - CPython 3.12 Lib/test oracle; stability: `cargo test -p mamba --test conformance_runtime_shutdown` - runtime shutdown and crash-boundary checks
Root WI: #3331
Status: confirmed
Required Verification: conformance, corpus, negative
Promise:
Run real Python 3.12 programs without semantic divergence across language core, PEP syntax/semantics, builtins and stdlib, plus selected 3rd-party libraries. CPython `Lib/test` and typeshed are the authoritative denominators; declared force-typing divergences must be explicit rather than hidden as ordinary behavior failures.
Gate Inventory: `cargo test -p mamba --test conformance_cpython_lib_test --release`; `cargo test -p mamba --test conformance_contract`; `cargo test -p mamba --test conformance_real_world`; `cargo test -p mamba --test conformance_runtime_shutdown`; projects/mamba/tests/PRODUCTION-GATE.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Python 3.12 parity gate | epic | #3331 | partial | planned | conformance | `cargo test -p mamba --test conformance_cpython_lib_test --release`; `cargo test -p mamba --test conformance_contract`; `cargo test -p mamba --test conformance_real_world`; `cargo test -p mamba --test conformance_runtime_shutdown`; projects/mamba/tests/PRODUCTION-GATE.md |

### C2. Less CPU time AND less memory than CPython — Axis 2

ID: c2-less-cpu-time-and-less-memory-than-cpython-axis-2
Type: RuntimeTool
Surfaces: CLI: `mamba bench --compare cpython` + `mamba bench --fixtures` + `mamba bench --check` - benchmark and regression gate entrypoints
EC Dimensions: efficiency: `cargo test -p mamba --release --test perf_pin -- perf_pin` - CPU/RSS ratio pins against CPython; behavior: `mamba bench` - benchmark report generation contract
Root WI: #3880
Status: confirmed
Required Verification: conformance
Promise:
Performance is a committed capability: for the same program, mamba targets strictly less CPU time and strictly less peak RSS than CPython 3.12. The v1 bar is staged, not one-shot: at least 1.5x where force typing pays, no worse than roughly 0.8x on CPython-tuned C hot paths, and both CPU/RSS measured externally before claiming progress.
Gate Inventory: `cargo test -p mamba --release --test perf_pin -- perf_pin`; `cargo bench -p mamba --bench mamba_bench`; projects/mamba/benches/3p/cross_runtime.rs; projects/mamba/tests/harness/cpython/config/perf/pins

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| CPython CPU/RSS ratio gate | epic | #3880 | planned | planned | conformance | `cargo test -p mamba --release --test perf_pin -- perf_pin`; `cargo bench -p mamba --bench mamba_bench`; projects/mamba/benches/3p/cross_runtime.rs; projects/mamba/tests/harness/cpython/config/perf/pins |

### C3. mambalibs end-to-end — Axis 3

ID: c3-mambalibs-end-to-end-axis-3
Type: RuntimeTool
Surfaces: Python: import `mambalibs.*` through `mamba run` - Rust-native modules exposed inside the mamba runtime; CLI: `mamba run` + `mamba <file>.py` - execute programs that import native kits
EC Dimensions: behavior: `cargo test -p mamba --test mambalibs` - native module registration, import, and callable coverage
Root WI: #3457
Status: confirmed
Required Verification: conformance
Promise:
A statically linked set of Rust-native libraries exposed as importable Python modules inside mamba. Each kit registers via `MambaModule` plus the `linkme` distributed slice and is force-linked into the final mamba binary, with import/callable coverage for native kits instead of a separate ABI or dynamic plugin layer.
Gate Inventory: `cargo test -p mamba --test mambalibs`; projects/mamba/mambalibs; projects/mamba/src/pkgmanage/builder/force_link.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Native mambalibs import/callable surface | epic | #3457 | partial | planned | conformance | `cargo test -p mamba --test mambalibs`; projects/mamba/mambalibs; projects/mamba/src/pkgmanage/builder/force_link.rs |

### C4. Package manager — uv-like

ID: c4-package-manager-uv-like
Type: DeveloperTool
Surfaces: CLI: `mamba init` + `mamba auth` + `mamba index` + `mamba add` + `mamba remove` + `mamba lock` + `mamba audit` + `mamba export` + `mamba tree` + `mamba version` + `mamba pip` + `mamba venv` + `mamba python` + `mamba workspace` + `mamba shell` + `mamba sync` + `mamba install` + `mamba tool` + `mamba cache` + `mamba hash` + `mamba generate-shell-completion` + `mamba pkgmgr-validate` - project scaffold, credentials, frozen index, dependency, lockfile, audit, export, tree, version, pip inventory, venv, local Python discovery/pinning/install management, workspace inspection, shell integration, install, uv-style tool administration, cache, completion, and validation workflows; Config: `mamba.toml` + `mamba.lock` - manifest and resolved lockfile artifacts
EC Dimensions: behavior: `cargo test -p mamba --test pkgmgr` - uv-like workflow fixtures; stability: `cargo test -p mamba --test schema_gates pkgmgr` - schema, pin, and idempotence contracts
Root WI: #519
Status: partial
Required Verification: conformance, negative, uv-parity
Promise:
A built-in package manager surface for project scaffold, dependency add/remove, lockfile generation, sync/install, cache, and validation workflows. The product promise is `uv`-style ergonomics over the mamba runtime with `mamba.toml` and `mamba.lock` as the agent-readable project contract.
Gate Inventory: `cargo test -p mamba --test pkgmgr`; `cargo test -p mamba --test schema_gates pkgmgr`; projects/mamba/tests/pkgmgr; projects/mamba/src/pkgmanage

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Local-first package manager baseline | epic | #459 | implemented | verified | conformance | `cargo test -p mamba --test pkgmgr`; `cargo test -p mamba --test schema_gates pkgmgr`; `./target/debug/mamba pkgmgr-validate --json`; projects/mamba/tests/pkgmgr; projects/mamba/src/pkgmanage |
| Full uv package-manager parity and beyond | epic | #519 | partial | in_progress | uv-parity | `cargo test -p mamba --test pkgmgr`; `./target/debug/mamba pkgmgr-validate --json`; projects/mamba/src/pkgmanage/pkgmgr; projects/mamba/tests/pkgmgr |

Current state: `mamba init/auth/index/add/remove/lock/export/tree/version/pip/venv/python/workspace/shell/sync/run/install/tool/hash/cache`
plus `pkgmgr-validate` are wired through offline frozen-index gates, direct
local wheel paths, explicit registry URL tests, lockfile export to
requirements.txt / pylock.toml, dependency-tree rendering, PEP 621 version
bumping, and pip-compatible requirements compile plus installed-environment
install/sync/uninstall/list/freeze/show/tree/check inspection and
dependency-tree rendering. `mamba audit` checks `mamba.lock` against an offline
advisory database, and `mamba lock --check` / `mamba sync --check` provide
CI-friendly drift gates without mutating lockfiles or environments. `mamba venv` exposes create/remove safety around PEP 405
environments, and `mamba cache` now reports exact size/category info plus
dry-run, age, size, and package-targeted pruning. `mamba python` exposes local
interpreter list/find, `.python-version` pinning, managed Python directory
resolution, local-source install/download registration, uninstall, and shell
PATH setup for managed Python launchers. `mamba workspace list/dir/metadata` inspects uv-compatible
`[tool.uv.workspace]` membership, member paths, root paths, and exclusion
patterns. `mamba index build` can
materialize a frozen local index from wheel files or directories for
`mamba add --index` / `mamba lock --index`. `mamba shell path/init` emits
managed PATH snippets for mamba tool bin directories, and
`mamba generate-shell-completion` emits clap-derived bash/zsh/fish/powershell/elvish
completion scripts from the live command tree. `mamba auth dir/login/token/logout`
manages plaintext package-index credentials under an overrideable credentials
directory. `mamba tool run/install/upgrade/list/uninstall/dir/update-shell` wraps the
tool-install workflow behind a uv-style `tool` command family. The package-manager validation
profile requires nineteen offline workflow families and keeps live network
coverage opt-in/report-only. `mamba add` / `mamba lock` do not treat public
PyPI as an implicit default source; callers must provide a frozen local index,
direct local wheel file, or explicit registry URL when resolving dependencies.
Full uv parity remains open under #519; remaining command families include live
Python standalone downloads, build/publish package flows, stored credential use
by index/resolver flows, live-index pip compile/install/sync parity, and related
parity fixtures.

## Test Completeness — what we tested, against what authority

The question an ecosystem actually asks is not *"how many tests do you have"*
(test-case count is input, not coverage) but **"what authoritative standard did
you test against, and how far do you cover it?"** mamba's correctness is gated by
**four test dimensions**, each measured against an external denominator it does
not control — so the percentage cannot be gamed by adding fixtures. Full gate
spec: [tests/PRODUCTION-GATE.md](tests/PRODUCTION-GATE.md). Fixtures live under
[tests/cpython/](tests/cpython/); the harness under
[tests/harness/cpython/](tests/harness/cpython/).

| Dimension | What it proves | Authoritative denominator | Completeness = | Target | Status |
|-----------|----------------|---------------------------|----------------|--------|--------|
| **① Type** | force-typed contract — mamba MUST raise where CPython silently accepts a wrong type | **typeshed** signatures (`vendor/typeshed`, 5073 `.pyi`) | enforced signatures ÷ total typeshed signatures | 100% | **auto-measured** by `tools/type_enforce_matrix.py`: **74.1%** enforced of gradable + **100%** soundness (Capability status) |
| **② Behavior** | observable output identical to CPython 3.12 (except where types deliberately diverge) | **CPython 3.12 `Lib/test`** | AssertionPass methods ÷ total Lib/test methods | 100% − type divergences | live CPython oracle wired (no static goldens, D5.6); Lib/test denominator partial |
| **③ Performance** | CPU time *and* peak RSS beat CPython — the payoff of force typing | **pyperformance** (CPython's official real-workload suite) | benchmarks with mamba/cpython > 1.0 ÷ total | 100% | external getrusage/peak-RSS harness wired; pyperformance not yet imported |
| **④ Safety & stability** | no memory leak, no crash/hang, and error messages leak no secrets (paths, addresses, env, source) | composite — cargo-fuzz coverage + CPython crashers + an error-leak matrix | per axis (below) | 0 leak · 0 crash · 0 secret-leak | sandboxed crash/timeout verdicts only; leak-hunter / fuzz / leak-matrix TODO |

**Why these denominators:**

- **① Type → typeshed.** Force typing is mamba's headline feature, and typeshed
  *is* the authoritative type contract for the whole stdlib + builtins. Each
  signature (e.g. `def b64encode(s: ReadableBuffer) -> bytes`) is one rule;
  feeding it a wrong-typed argument must raise. The denominator is already in the
  repo, so type fixtures can be **generated** from `.pyi` and the enforce-rate
  computed automatically — no hand-writing.
- **② Behavior → CPython `Lib/test`.** The only authority on *what Python 3.12
  does* is CPython's own test suite. A behavior fixture that exits 0 against the
  live CPython oracle is one passing method.
- **③ Performance → pyperformance.** Winning micro-benches proves nothing;
  pyperformance is the community's real-workload bar. We gate on CPU **and**
  peak RSS, since force typing should win on memory too.
- **④ Safety & stability → composite.** No single authority exists, so it is
  three measurable axes: fuzz code-coverage with zero crashes; zero leaks under
  ASan over the runtime's allocating surface; and a matrix of *every error path ×
  every secret class* proving no message leaks internals.

A dimension is **done** when its completeness reaches its target — never when a
fixture count is hit. The numbers move only when the runtime actually improves,
which is exactly what makes them safe to quote to the ecosystem.

### Type enforcement vs CPython behavior — the divergence policy

Dimensions ① and ② deliberately **contradict** on one construct: a wrong-typed
annotated binding such as `x: int = "3"`, `def f(c: int = "3")`, or
`list[int] = [1, "two"]`. CPython treats annotations as hints and runs it; ①
requires mamba to **raise**, because force typing is mamba's product. mamba
cannot both enforce and not-enforce the same line. The policy:

- **Enforce — never relax.** Where the walls conflict, mamba keeps its
  stricter, force-typed behavior. Relaxing enforcement to make a ② fixture pass
  would weaken ① to move ② — forbidden. ① is *strengthened* (mamba must also
  raise on the cases it currently accepts — parameter defaults, container
  elements), not loosened.
- **① counts mamba's enforcement *form*.** mamba enforces at **compile time**
  (`x: int = "3"` → `error: type error: type mismatch: expected int, got str`,
  exit≠0), while type fixtures observe with a runtime `try/except TypeError`. So
  a non-zero exit whose diagnostic is a *type* error (`type error` /
  `type mismatch` / `TypeError`) counts as **enforced = pass** — distinct from a
  crash (panic / syntax error / missing name) which is a real failure.
- **② excludes *declared* type divergences.** A `Lib/test`/PEP fixture that is
  red *because* mamba correctly type-rejects what CPython silently accepts is
  recorded in [`config/type_divergences.txt`](tests/harness/cpython/config/type_divergences.txt)
  and removed from the ② passable denominator — exactly the `100% − type
  divergences` target above. Each entry is **verified genuine** (mamba emits a
  type-mismatch rejection; CPython runs clean), the same discipline as the
  CPython-fail exclusions in `behavior_gaps.txt`. It is not a dumping ground: a
  crash or a missing feature is a ② failure, never a divergence.

<!-- COVERAGE:BEGIN (generated by tests/harness/cpython/tools/coverage_matrix.py — do not edit by hand) -->

## Library coverage — what's implemented, what's stub

Per-**library** ② behavior status scoped to **CPython 3.12**. The unit is the **top-level importable module** — std-libs submodules are folded into their parent via each fixture's CPython `source`/`subject` metadata (all `asyncio.*` → `asyncio`, every codec page → `encodings`), so the denominator is CPython's own taxonomy, not an inflatable per-file split. Pass = mamba's stdout matches the **live CPython 3.12 oracle** (≤8 sampled fixtures/lib; fixtures CPython itself can't run are excluded, not graded). Core-language and built-in rows are mamba feature buckets, not modules, and are not collapsed. Regenerate: `MAMBA_BIN=… tools/coverage_matrix.py --sample 8 --write`; per-submodule detail in [COVERAGE.md](COVERAGE.md).

**Overall: ✅ 10 · 🟢 29 · 🟡 103 · 🔴 284** across 426 top-level libs.

### Core language features — 29 libs
✅ 0 · 🟢 3 · 🟡 8 · 🔴 18  (✅≥80% · 🟢 50–79% · 🟡 1–49% · 🔴 0%, top-level module, sampled ≤8/lib)

| lib | | pass | sub |
|-----|--|------|-----|
| `dictcomps` | 🟢 | 5/8 | 0 |
| `grammar` | 🟢 | 4/8 | 0 |
| `yield_from` | 🟢 | 4/8 | 0 |
| `compile` | 🟡 | 2/8 | 0 |
| `decorators` | 🟡 | 2/8 | 0 |
| `descr` | 🟡 | 1/8 | 0 |
| `exception_group` | 🟡 | 1/8 | 0 |
| `exceptions` | 🟡 | 1/8 | 0 |
| `fstring` | 🟡 | 2/8 | 0 |
| `funcattrs` | 🟡 | 2/8 | 0 |
| `pattern_matching` | 🟡 | 1/8 | 0 |
| `asdl_parser` | 🔴 | 0/1 | 0 |
| `asyncgen` | 🔴 | 0/8 | 0 |
| `bigmem` | 🔴 | 0/8 | 0 |
| `compare` | 🔴 | 0/8 | 0 |
| `compiler_assemble` | 🔴 | 0/1 | 0 |
| `compiler_codegen` | 🔴 | 0/2 | 0 |
| `coroutines` | 🔴 | 0/8 | 0 |
| `generators` | 🔴 | 0/8 | 0 |
| `global` | 🔴 | 0/4 | 0 |
| `import` | 🔴 | 0/8 | 0 |
| `iter` | 🔴 | 0/2 | 0 |
| `listcomps` | 🔴 | 0/8 | 0 |
| `opcode` | 🔴 | 0/3 | 0 |
| `peepholer` | 🔴 | 0/8 | 0 |
| `setcomps` | 🔴 | 0/1 | 0 |
| `syntax` | 🔴 | 0/8 | 0 |
| `sys_settrace` | 🔴 | 0/8 | 0 |
| `with` | 🔴 | 0/8 | 0 |

### Built-in libraries — 15 libs
✅ 0 · 🟢 2 · 🟡 10 · 🔴 3  (✅≥80% · 🟢 50–79% · 🟡 1–49% · 🔴 0%, top-level module, sampled ≤8/lib)

| lib | | pass | sub |
|-----|--|------|-----|
| `bool` | 🟢 | 5/8 | 0 |
| `set_methods` | 🟢 | 6/8 | 0 |
| `builtins` | 🟡 | 1/8 | 0 |
| `bytes` | 🟡 | 3/8 | 0 |
| `complex` | 🟡 | 2/8 | 0 |
| `dict_methods` | 🟡 | 1/8 | 0 |
| `enumerate` | 🟡 | 3/8 | 0 |
| `float_methods` | 🟡 | 2/8 | 0 |
| `int_methods` | 🟡 | 2/8 | 0 |
| `list_methods` | 🟡 | 2/8 | 0 |
| `slice` | 🟡 | 2/8 | 0 |
| `tuple_methods` | 🟡 | 2/8 | 0 |
| `hash` | 🔴 | 0/8 | 0 |
| `iter` | 🔴 | 0/8 | 0 |
| `range` | 🔴 | 0/8 | 0 |

### Standard library — 382 libs
✅ 10 · 🟢 24 · 🟡 85 · 🔴 263  (✅≥80% · 🟢 50–79% · 🟡 1–49% · 🔴 0%, top-level module, sampled ≤8/lib)

| lib | | pass | sub |
|-----|--|------|-----|
| `errno` | ✅ | 8/8 | 1 |
| `keyword` | ✅ | 7/8 | 1 |
| `longexp` | ✅ | 1/1 | 1 |
| `perfmaps` | ✅ | 1/1 | 1 |
| `spwd` | ✅ | 3/3 | 1 |
| `startfile` | ✅ | 3/3 | 1 |
| `str` | ✅ | 8/8 | 1 |
| `time` | ✅ | 7/8 | 1 |
| `token` | ✅ | 5/5 | 1 |
| `winapi` | ✅ | 3/3 | 1 |
| `abs` | 🟢 | 2/3 | 1 |
| `augassign` | 🟢 | 5/7 | 1 |
| `bisect` | 🟢 | 4/8 | 1 |
| `bufio` | 🟢 | 4/6 | 1 |
| `colorsys` | 🟢 | 5/8 | 1 |
| `copy` | 🟢 | 6/8 | 1 |
| `difflib` | 🟢 | 4/8 | 1 |
| `exception_variations` | 🟢 | 5/8 | 1 |
| `heapq` | 🟢 | 4/8 | 1 |
| `named_expressions` | 🟢 | 4/8 | 1 |
| `opcache` | 🟢 | 5/8 | 1 |
| `opcodes` | 🟢 | 4/8 | 1 |
| `operator` | 🟢 | 4/8 | 1 |
| `pickle` | 🟢 | 4/8 | 1 |
| `queue` | 🟢 | 4/8 | 1 |
| `richcmp` | 🟢 | 4/8 | 1 |
| `secrets` | 🟢 | 6/8 | 1 |
| `stat` | 🟢 | 4/8 | 1 |
| `statistics` | 🟢 | 4/8 | 1 |
| `strftime` | 🟢 | 3/4 | 1 |
| `subclassinit` | 🟢 | 4/8 | 1 |
| `textwrap` | 🟢 | 5/8 | 1 |
| `typechecks` | 🟢 | 3/6 | 1 |
| `unary` | 🟢 | 4/6 | 1 |
| `abc` | 🟡 | 2/8 | 2 |
| `argparse` | 🟡 | 1/8 | 1 |
| `array` | 🟡 | 1/8 | 1 |
| `atexit` | 🟡 | 1/8 | 1 |
| `base64` | 🟡 | 2/8 | 1 |
| `baseexception` | 🟡 | 1/8 | 1 |
| `binascii` | 🟡 | 3/8 | 1 |
| `calendar` | 🟡 | 3/8 | 1 |
| `cmath` | 🟡 | 3/8 | 1 |
| `code` | 🟡 | 1/8 | 2 |
| `codecs` | 🟡 | 1/8 | 1 |
| `collections` | 🟡 | 1/8 | 2 |
| `compileall` | 🟡 | 1/8 | 1 |
| `configparser` | 🟡 | 1/8 | 1 |
| `contextlib` | 🟡 | 1/8 | 2 |
| `csv` | 🟡 | 2/8 | 1 |
| `datetimetester` | 🟡 | 1/8 | 1 |
| `deque` | 🟡 | 1/8 | 1 |
| `dynamic` | 🟡 | 1/8 | 1 |
| `file` | 🟡 | 2/8 | 2 |
| `filecmp` | 🟡 | 2/8 | 1 |
| `fnmatch` | 🟡 | 3/8 | 1 |
| `functools` | 🟡 | 1/8 | 1 |
| `future_stmt` | 🟡 | 3/8 | 1 |
| `gc` | 🟡 | 1/8 | 1 |
| `genericclass` | 🟡 | 3/8 | 1 |
| `getopt` | 🟡 | 1/8 | 1 |
| `glob` | 🟡 | 3/8 | 1 |
| `graphlib` | 🟡 | 1/8 | 1 |
| `hmac` | 🟡 | 2/8 | 1 |
| `index` | 🟡 | 1/8 | 1 |
| `int` | 🟡 | 1/8 | 2 |
| `io` | 🟡 | 2/8 | 1 |
| `ipaddress` | 🟡 | 1/8 | 1 |
| `isinstance` | 🟡 | 1/8 | 1 |
| `iterlen` | 🟡 | 2/8 | 1 |
| `itertools` | 🟡 | 2/8 | 1 |
| `json` | 🟡 | 3/8 | 5 |
| `linecache` | 🟡 | 1/8 | 1 |
| `logging` | 🟡 | 2/8 | 1 |
| `math` | 🟡 | 3/8 | 2 |
| `memoryio` | 🟡 | 2/8 | 1 |
| `memoryview` | 🟡 | 1/8 | 1 |
| `mimetypes` | 🟡 | 2/8 | 1 |
| `minidom` | 🟡 | 1/8 | 1 |
| `module` | 🟡 | 1/8 | 1 |
| `numeric_tower` | 🟡 | 2/8 | 1 |
| `pathlib` | 🟡 | 3/8 | 1 |
| `platform` | 🟡 | 1/8 | 1 |
| `plistlib` | 🟡 | 1/8 | 1 |
| `posixpath` | 🟡 | 3/8 | 2 |
| `pow` | 🟡 | 1/7 | 1 |
| `pprint` | 🟡 | 1/8 | 1 |
| `property` | 🟡 | 2/8 | 1 |
| `quopri` | 🟡 | 1/8 | 1 |
| `raise` | 🟡 | 2/8 | 1 |
| `random` | 🟡 | 3/8 | 1 |
| `re` | 🟡 | 1/8 | 1 |
| `shlex` | 🟡 | 2/8 | 1 |
| `shutil` | 🟡 | 1/8 | 1 |
| `signal` | 🟡 | 2/8 | 1 |
| `sort` | 🟡 | 2/8 | 1 |
| `source_encoding` | 🟡 | 1/8 | 1 |
| `string` | 🟡 | 2/8 | 2 |
| `struct` | 🟡 | 3/8 | 1 |
| `structseq` | 🟡 | 1/8 | 1 |
| `subprocess` | 🟡 | 1/8 | 1 |
| `super` | 🟡 | 1/8 | 1 |
| `sys` | 🟡 | 3/8 | 2 |
| `tabnanny` | 🟡 | 2/8 | 1 |
| `threading` | 🟡 | 1/8 | 2 |
| `timeit` | 🟡 | 1/8 | 1 |
| `tomllib` | 🟡 | 2/8 | 4 |
| `tracemalloc` | 🟡 | 1/8 | 1 |
| `unicode` | 🟡 | 2/8 | 4 |
| `unicodedata` | 🟡 | 2/8 | 1 |
| `urllib` | 🟡 | 1/8 | 2 |
| `urllib2` | 🟡 | 1/8 | 4 |
| `userlist` | 🟡 | 2/8 | 1 |
| `utf8source` | 🟡 | 1/3 | 1 |
| `uuid` | 🟡 | 2/8 | 1 |
| `weakref` | 🟡 | 1/8 | 1 |
| `weakset` | 🟡 | 1/8 | 1 |
| `zlib` | 🟡 | 2/8 | 1 |
| `zoneinfo` | 🟡 | 1/8 | 2 |
| `ET` | 🔴 | 0/8 | 1 |
| `__all__` | 🔴 | 0/1 | 1 |
| `_encoded_words` | 🔴 | 0/4 | 1 |
| `_header_value_parser` | 🔴 | 0/1 | 1 |
| `_osx_support` | 🔴 | 0/8 | 2 |
| `_test_multiprocessing` | 🔴 | 0/8 | 1 |
| `_xxinterpchannels` | 🔴 | 0/8 | 1 |
| `_xxsubinterpreters` | 🔴 | 0/8 | 1 |
| `abstract_numbers` | 🔴 | 0/7 | 1 |
| `aifc` | 🔴 | 0/8 | 1 |
| `asian_codecs` | 🔴 | 0/1 | 1 |
| `ast` | 🔴 | 0/8 | 1 |
| `asyncio` | 🔴 | 0/8 | 23 |
| `audioop` | 🔴 | 0/8 | 1 |
| `audit` | 🔴 | 0/8 | 1 |
| `bdb` | 🔴 | 0/8 | 1 |
| `bigaddrspace` | 🔴 | 0/4 | 1 |
| `binop` | 🔴 | 0/8 | 1 |
| `buffer` | 🔴 | 0/8 | 1 |
| `builtin` | 🔴 | 0/8 | 1 |
| `bz2` | 🔴 | 0/8 | 1 |
| `c_locale_coercion` | 🔴 | 0/8 | 1 |
| `call` | 🔴 | 0/8 | 1 |
| `capi` | 🔴 | 0/8 | 21 |
| `cext` | 🔴 | 0/1 | 1 |
| `cgi` | 🔴 | 0/8 | 1 |
| `cgitb` | 🔴 | 0/6 | 1 |
| `charmapcodec` | 🔴 | 0/4 | 1 |
| `class` | 🔴 | 0/8 | 1 |
| `clinic` | 🔴 | 0/1 | 1 |
| `cmd` | 🔴 | 0/8 | 3 |
| `codeccallbacks` | 🔴 | 0/8 | 1 |
| `codecencodings_cn` | 🔴 | 0/1 | 1 |
| `codecencodings_hk` | 🔴 | 0/1 | 1 |
| `codecencodings_iso2022` | 🔴 | 0/2 | 1 |
| `codecencodings_jp` | 🔴 | 0/1 | 1 |
| `codecencodings_kr` | 🔴 | 0/1 | 1 |
| `codecencodings_tw` | 🔴 | 0/1 | 1 |
| `codecmaps_cn` | 🔴 | 0/1 | 1 |
| `codecmaps_hk` | 🔴 | 0/1 | 1 |
| `codecmaps_jp` | 🔴 | 0/1 | 1 |
| `codecmaps_kr` | 🔴 | 0/1 | 1 |
| `codecmaps_tw` | 🔴 | 0/1 | 1 |
| `codeop` | 🔴 | 0/8 | 1 |
| `concurrent` | 🔴 | 0/8 | 5 |
| `contains` | 🔴 | 0/4 | 1 |
| `context` | 🔴 | 0/8 | 1 |
| `contextvars` | 🔴 | 0/8 | 1 |
| `cookies` | 🔴 | 0/7 | 1 |
| `copyreg` | 🔴 | 0/6 | 1 |
| `cppext` | 🔴 | 0/1 | 1 |
| `cprofile` | 🔴 | 0/8 | 1 |
| `crashers` | 🔴 | 0/1 | 1 |
| `crypt` | 🔴 | 0/8 | 1 |
| `ctypes` | 🔴 | 0/8 | 51 |
| `curses` | 🔴 | 0/8 | 1 |
| `dataclasses` | 🔴 | 0/8 | 1 |
| `datetime` | 🔴 | 0/8 | 1 |
| `dbm` | 🔴 | 0/8 | 4 |
| `decimal` | 🔴 | 0/8 | 1 |
| `defaultdict` | 🔴 | 0/8 | 1 |
| `descrtut` | 🔴 | 0/1 | 1 |
| `devpoll` | 🔴 | 0/1 | 1 |
| `dict_version` | 🔴 | 0/8 | 1 |
| `dictviews` | 🔴 | 0/8 | 1 |
| `dis` | 🔴 | 0/8 | 1 |
| `doctest` | 🔴 | 0/3 | 2 |
| `docxmlrpc` | 🔴 | 0/8 | 1 |
| `dtrace` | 🔴 | 0/2 | 1 |
| `dynamicclassattribute` | 🔴 | 0/5 | 1 |
| `eintr` | 🔴 | 0/1 | 1 |
| `email` | 🔴 | 0/8 | 11 |
| `embed` | 🔴 | 0/8 | 1 |
| `ensurepip` | 🔴 | 0/8 | 1 |
| `enum` | 🔴 | 0/8 | 1 |
| `eof` | 🔴 | 0/6 | 1 |
| `epoll` | 🔴 | 0/1 | 1 |
| `except_star` | 🔴 | 0/8 | 1 |
| `exception_hierarchy` | 🔴 | 0/8 | 1 |
| `extcall` | 🔴 | 0/1 | 1 |
| `faulthandler` | 🔴 | 0/8 | 1 |
| `fcntl` | 🔴 | 0/8 | 1 |
| `fileinput` | 🔴 | 0/8 | 1 |
| `fileio` | 🔴 | 0/8 | 1 |
| `fileutils` | 🔴 | 0/1 | 1 |
| `finalization` | 🔴 | 0/8 | 1 |
| `float` | 🔴 | 0/8 | 1 |
| `flufl` | 🔴 | 0/4 | 1 |
| `fork1` | 🔴 | 0/2 | 1 |
| `format` | 🔴 | 0/8 | 1 |
| `fractions` | 🔴 | 0/8 | 2 |
| `frame` | 🔴 | 0/8 | 1 |
| `frozen` | 🔴 | 0/3 | 1 |
| `ftplib` | 🔴 | 0/8 | 1 |
| `gdb` | 🔴 | 0/1 | 1 |
| `generator_stop` | 🔴 | 0/2 | 1 |
| `genericalias` | 🔴 | 0/8 | 1 |
| `genericpath` | 🔴 | 0/8 | 1 |
| `genexps` | 🔴 | 0/1 | 1 |
| `getpass` | 🔴 | 0/8 | 1 |
| `getpath` | 🔴 | 0/8 | 1 |
| `gettext` | 🔴 | 0/8 | 1 |
| `grp` | 🔴 | 0/4 | 1 |
| `gzip` | 🔴 | 0/8 | 1 |
| `hashlib` | 🔴 | 0/8 | 1 |
| `html` | 🔴 | 0/8 | 2 |
| `htmlparser` | 🔴 | 0/6 | 1 |
| `http` | 🔴 | 0/8 | 3 |
| `httplib` | 🔴 | 0/8 | 3 |
| `httpservers` | 🔴 | 0/8 | 2 |
| `idle` | 🔴 | 0/1 | 1 |
| `imaplib` | 🔴 | 0/8 | 1 |
| `imghdr` | 🔴 | 0/8 | 1 |
| `importlib` | 🔴 | 0/8 | 19 |
| `inspect` | 🔴 | 0/8 | 1 |
| `interpreters` | 🔴 | 0/8 | 1 |
| `ioctl` | 🔴 | 0/1 | 1 |
| `keywordonlyarg` | 🔴 | 0/8 | 1 |
| `kqueue` | 🔴 | 0/8 | 1 |
| `largefile` | 🔴 | 0/3 | 1 |
| `launcher` | 🔴 | 0/1 | 1 |
| `lib2to3` | 🔴 | 0/8 | 8 |
| `lltrace` | 🔴 | 0/3 | 1 |
| `locale` | 🔴 | 0/8 | 2 |
| `long` | 🔴 | 0/8 | 1 |
| `lzma` | 🔴 | 0/8 | 1 |
| `mailbox` | 🔴 | 0/8 | 1 |
| `mailcap` | 🔴 | 0/8 | 1 |
| `marshal` | 🔴 | 0/8 | 1 |
| `metaclass` | 🔴 | 0/1 | 1 |
| `mmap` | 🔴 | 0/8 | 1 |
| `modulefinder` | 🔴 | 0/8 | 1 |
| `monitoring` | 🔴 | 0/8 | 1 |
| `msilib` | 🔴 | 0/1 | 1 |
| `multibytecodec` | 🔴 | 0/8 | 1 |
| `multiprocessing_fork` | 🔴 | 0/1 | 1 |
| `multiprocessing_forkserver` | 🔴 | 0/1 | 1 |
| `multiprocessing_main_handling` | 🔴 | 0/3 | 1 |
| `multiprocessing_spawn` | 🔴 | 0/1 | 1 |
| `netrc` | 🔴 | 0/8 | 1 |
| `nis` | 🔴 | 0/1 | 1 |
| `nntplib` | 🔴 | 0/8 | 1 |
| `ntpath` | 🔴 | 0/8 | 1 |
| `openpty` | 🔴 | 0/1 | 1 |
| `optparse` | 🔴 | 0/8 | 1 |
| `ordered_dict` | 🔴 | 0/8 | 1 |
| `os` | 🔴 | 0/8 | 1 |
| `osx_env` | 🔴 | 0/1 | 1 |
| `patma` | 🔴 | 0/6 | 1 |
| `pdb` | 🔴 | 0/8 | 1 |
| `peg_generator` | 🔴 | 0/1 | 1 |
| `pep646_syntax` | 🔴 | 0/1 | 1 |
| `picklebuffer` | 🔴 | 0/8 | 1 |
| `pickletools` | 🔴 | 0/8 | 1 |
| `pipes` | 🔴 | 0/8 | 1 |
| `pkgutil` | 🔴 | 0/8 | 1 |
| `poll` | 🔴 | 0/6 | 1 |
| `popen` | 🔴 | 0/5 | 1 |
| `poplib` | 🔴 | 0/8 | 1 |
| `positional_only_arg` | 🔴 | 0/8 | 1 |
| `posix` | 🔴 | 0/8 | 1 |
| `print` | 🔴 | 0/8 | 1 |
| `profile` | 🔴 | 0/6 | 1 |
| `pstats` | 🔴 | 0/8 | 1 |
| `pty` | 🔴 | 0/6 | 1 |
| `pulldom` | 🔴 | 0/8 | 1 |
| `pwd` | 🔴 | 0/3 | 1 |
| `py_compile` | 🔴 | 0/8 | 1 |
| `pyclbr` | 🔴 | 0/6 | 1 |
| `pydoc` | 🔴 | 0/8 | 1 |
| `pyexpat` | 🔴 | 0/8 | 1 |
| `readline` | 🔴 | 0/8 | 1 |
| `regrtest` | 🔴 | 0/8 | 1 |
| `repl` | 🔴 | 0/6 | 1 |
| `reprlib` | 🔴 | 0/8 | 1 |
| `resource` | 🔴 | 0/8 | 1 |
| `rlcompleter` | 🔴 | 0/8 | 1 |
| `robotparser` | 🔴 | 0/8 | 2 |
| `runpy` | 🔴 | 0/8 | 1 |
| `sax` | 🔴 | 0/8 | 2 |
| `sched` | 🔴 | 0/8 | 1 |
| `scope` | 🔴 | 0/8 | 1 |
| `script_helper` | 🔴 | 0/8 | 1 |
| `select` | 🔴 | 0/6 | 1 |
| `selectors` | 🔴 | 0/8 | 1 |
| `set` | 🔴 | 0/2 | 1 |
| `shelve` | 🔴 | 0/8 | 1 |
| `site` | 🔴 | 0/8 | 1 |
| `smtplib` | 🔴 | 0/8 | 1 |
| `smtpnet` | 🔴 | 0/5 | 1 |
| `sndhdr` | 🔴 | 0/2 | 1 |
| `socket` | 🔴 | 0/8 | 1 |
| `socketserver` | 🔴 | 0/8 | 1 |
| `sqlite3` | 🔴 | 0/8 | 11 |
| `ssl` | 🔴 | 0/8 | 1 |
| `stable_abi_ctypes` | 🔴 | 0/3 | 1 |
| `stringprep` | 🔴 | 0/1 | 1 |
| `strptime` | 🔴 | 0/8 | 1 |
| `strtod` | 🔴 | 0/8 | 1 |
| `sunau` | 🔴 | 0/5 | 1 |
| `sundry` | 🔴 | 0/1 | 1 |
| `support` | 🔴 | 0/8 | 1 |
| `symtable` | 🔴 | 0/8 | 1 |
| `sysconfig` | 🔴 | 0/8 | 1 |
| `syslog` | 🔴 | 0/8 | 1 |
| `tarfile` | 🔴 | 0/8 | 1 |
| `tcl` | 🔴 | 0/1 | 1 |
| `telnetlib` | 🔴 | 0/8 | 1 |
| `tempfile` | 🔴 | 0/8 | 1 |
| `termios` | 🔴 | 0/8 | 1 |
| `thread` | 🔴 | 0/7 | 1 |
| `threadedtempfile` | 🔴 | 0/1 | 1 |
| `threadsignals` | 🔴 | 0/6 | 1 |
| `timeout` | 🔴 | 0/8 | 1 |
| `tix` | 🔴 | 0/2 | 1 |
| `tkinter` | 🔴 | 0/1 | 1 |
| `tokenize` | 🔴 | 0/8 | 1 |
| `tools` | 🔴 | 0/2 | 2 |
| `trace` | 🔴 | 0/8 | 1 |
| `traceback` | 🔴 | 0/8 | 2 |
| `ttk` | 🔴 | 0/2 | 2 |
| `tty` | 🔴 | 0/4 | 1 |
| `tuple` | 🔴 | 0/6 | 1 |
| `turtle` | 🔴 | 0/1 | 1 |
| `type_aliases` | 🔴 | 0/8 | 1 |
| `type_annotations` | 🔴 | 0/8 | 1 |
| `type_cache` | 🔴 | 0/7 | 1 |
| `type_comments` | 🔴 | 0/8 | 1 |
| `type_params` | 🔴 | 0/8 | 1 |
| `types` | 🔴 | 0/8 | 1 |
| `typing` | 🔴 | 0/8 | 1 |
| `ucn` | 🔴 | 0/8 | 1 |
| `unittest` | 🔴 | 0/8 | 16 |
| `univnewlines` | 🔴 | 0/8 | 1 |
| `unpack` | 🔴 | 0/2 | 2 |
| `unparse` | 🔴 | 0/8 | 1 |
| `urllib2net` | 🔴 | 0/8 | 1 |
| `urllibnet` | 🔴 | 0/8 | 1 |
| `urlparse` | 🔴 | 0/8 | 2 |
| `userdict` | 🔴 | 0/4 | 1 |
| `userstring` | 🔴 | 0/3 | 1 |
| `utf8_mode` | 🔴 | 0/8 | 1 |
| `uu` | 🔴 | 0/8 | 1 |
| `wait3` | 🔴 | 0/1 | 1 |
| `wait4` | 🔴 | 0/1 | 1 |
| `warnings` | 🔴 | 0/8 | 1 |
| `wave` | 🔴 | 0/8 | 1 |
| `webbrowser` | 🔴 | 0/8 | 1 |
| `winconsoleio` | 🔴 | 0/1 | 1 |
| `winreg` | 🔴 | 0/1 | 1 |
| `wsgiref` | 🔴 | 0/8 | 1 |
| `xdrlib` | 🔴 | 0/6 | 1 |
| `xml_dom_minicompat` | 🔴 | 0/8 | 1 |
| `xml_dom_xmlbuilder` | 🔴 | 0/4 | 1 |
| `xml_etree` | 🔴 | 0/8 | 2 |
| `xml_etree_c` | 🔴 | 0/8 | 1 |
| `xmlrpc` | 🔴 | 0/8 | 2 |
| `xxlimited` | 🔴 | 0/8 | 1 |
| `xxtestfuzz` | 🔴 | 0/1 | 1 |
| `zipapp` | 🔴 | 0/8 | 1 |
| `zipfile` | 🔴 | 0/8 | 3 |
| `zipfile64` | 🔴 | 0/3 | 1 |
| `zipimport` | 🔴 | 0/8 | 2 |

<!-- COVERAGE:END -->

### Third-party libraries — 57 libs
✅ 0 · 🟢 6 · 🟡 0 · 🔴 51  (✅≥80% · 🟢 50–79% · 🟡 1–49% · 🔴 0%, sampled ≤8/lib)

| lib | | pass | lib | | pass |
|-----|--|------|-----|--|------|
| `anyio` | 🟢 | 2/3 | `click` | 🟢 | 2/3 |
| `markupsafe` | 🟢 | 2/3 | `numpy` | 🟢 | 2/3 |
| `pandas` | 🟢 | 2/3 | `rich` | 🟢 | 2/3 |
| `_baseline` | 🔴 | 0/4 | `aiofiles` | 🔴 | 0/4 |
| `aiohttp` | 🔴 | 0/4 | `alembic` | 🔴 | 0/4 |
| `attrs` | 🔴 | 0/4 | `azure_core` | 🔴 | 0/4 |
| `azure_identity` | 🔴 | 0/4 | `azure_keyvault_secrets` | 🔴 | 0/4 |
| `azure_storage_blob` | 🔴 | 0/4 | `boto3` | 🔴 | 0/4 |
| `botocore` | 🔴 | 0/4 | `celery` | 🔴 | 0/4 |
| `certifi` | 🔴 | 0/4 | `charset_normalizer` | 🔴 | 0/4 |
| `cryptography` | 🔴 | 0/4 | `fastapi` | 🔴 | 0/4 |
| `flask` | 🔴 | 0/4 | `google_api_core` | 🔴 | 0/4 |
| `google_cloud_pubsub` | 🔴 | 0/4 | `google_cloud_storage` | 🔴 | 0/4 |
| `googleapis_common_protos` | 🔴 | 0/4 | `grpcio` | 🔴 | 0/4 |
| `grpclib` | 🔴 | 0/4 | `gunicorn` | 🔴 | 0/4 |
| `httpx` | 🔴 | 0/4 | `hypothesis` | 🔴 | 0/4 |
| `idna` | 🔴 | 0/4 | `jinja2` | 🔴 | 0/4 |
| `jmespath` | 🔴 | 0/4 | `jsonschema` | 🔴 | 0/4 |
| `kombu` | 🔴 | 0/4 | `marshmallow` | 🔴 | 0/4 |
| `mock` | 🔴 | 0/4 | `msgpack` | 🔴 | 0/4 |
| `orjson` | 🔴 | 0/4 | `packaging` | 🔴 | 0/3 |
| `protobuf` | 🔴 | 0/4 | `psycopg` | 🔴 | 0/4 |
| `pydantic` | 🔴 | 0/4 | `pydantic_core` | 🔴 | 0/4 |
| `pyopenssl` | 🔴 | 0/4 | `pytest` | 🔴 | 0/4 |
| `redis` | 🔴 | 0/4 | `requests` | 🔴 | 0/4 |
| `s3transfer` | 🔴 | 0/4 | `sqlalchemy` | 🔴 | 0/4 |
| `starlette` | 🔴 | 0/4 | `typing_extensions` | 🔴 | 0/4 |
| `urllib3` | 🔴 | 0/4 | `uvicorn` | 🔴 | 0/4 |
| `werkzeug` | 🔴 | 0/4 |  | | |

## Capability status — the four axes, measured

A first real measurement pass (release binary vs CPython 3.12, `getrusage`
CPU-time + peak-RSS, best-of-N). The honest state of each axis:

| axis | where it stands | reading |
|------|-----------------|---------|
| **① Type** (enforce + sound) | **74.1%** enforced of the gradable denom (auto-measured, up from ~42%/13.5%) + **100%** value-soundness (104/104) | enforcement: the call-site scalar hook fires for **constructors + methods + scalar-prefix sigs** (rejected at compile time before the runtime setup crash; custom-type params `_NL`/protocols out of scope by design, false-positive-clean). soundness: every correctly-typed float op (return/comprehension/generator/container/dict-key/callback) computes the right value — no NaN-box leaks. Both auto-graded by `tools/type_enforce_matrix.py`. |
| **② Behavior** (run correctly) | **~18%** corpus-weighted does-it-run-correctly | leaf modules land in the 50–80% range; the `behavior` facet (79% of the corpus) is the runner-dominated structural ceiling (needs full unittest+import). |
| **③ Perf** (CPU + RSS vs CPython) | **bimodal** — see below | compute wins big; object/dynamic/float loses **and** blows up memory. The value model is the keystone. |
| **④ Safety** (no leak/crash on hostile input) | **~87%** security-matrix | the `re` ReDoS class is a structural win (Rust linear-time regex); residual gaps are per-module hardening (`struct`/`json`). |
| **core stability** | **99.1%** | the substrate is sound — only edge crashes (deep recursion, generator-nesting cap, MRO, async-gen hang). |

<!-- TYPE-CAPABILITY:BEGIN (generated by tests/harness/cpython/tools/type_enforce_matrix.py — do not edit by hand) -->

### Type capability — measured

Auto-graded by `tools/type_enforce_matrix.py` (mamba `mamba` vs the fixture contract). Two complementary walls:

| wall | what it proves | gradable | green | rate |
|------|----------------|----------|-------|------|
| **① Enforcement** | mamba rejects wrong-typed code CPython accepts | 6708 | 5902 | **88.0%** |
| **① Soundness** | correctly-typed ops compute the right value (no float leaks) | 104 | 104 | **100.0%** |

Enforcement: 5902 enforced · 806 leaked · 2228 ungradable (crash/timeout/malformed) of 8936 fixtures.

| enforcement bucket | enforced | leaked | rate |
|--------------------|----------|--------|------|
| builtin-libs | 235 | 28 | 89.4% |
| core | 54 | 6 | 90.0% |
| std-libs | 5613 | 772 | 87.9% |

<!-- TYPE-CAPABILITY:END -->

<!-- CONCURRENCY-CAPABILITY:BEGIN (generated by tests/harness/cpython/tools/concurrency_matrix.py — do not edit by hand) -->

### Concurrency capability — measured

Thread-safety matrix auto-graded by `tools/concurrency_matrix.py` (N=8 unsynchronized threads × K=5000 ops, 20 trials, expected=40000). Oracle is **free-threaded CPython (PEP 703)**, NOT GIL-CPython — the GIL masks races that the real contract exposes.

| shared-state pattern | CPython3.12 (GIL) | CPython3.13t (free-threaded) | mamba |
|---|---|---|---|
| int `+=` (shared, compound) | SAFE 40000 | RACY 8460..12494/40000 | SAFE 40000 |
| list.append (shared) | SAFE 40000 | SAFE 40000 | SAFE 40000 |
| dict[k] `+=` (shared key, compound) | SAFE 40000 | RACY 8884..12279/40000 | SAFE 40000 |
| set.add (distinct keys) | SAFE 40000 | SAFE 40000 | SAFE 40000 |
| check-then-act (per key) | SAFE 40000 | RACY 39388..39814/40000 | SAFE 40000 |

**Contract** (= free-threaded CPython 3.13t): a *single* built-in container mutation (`list.append`, `set.add`, `dict[k]=v`) is **atomic** (per-object critical section, uncontended-fast); a *compound* op (`+=`, check-then-act) is **not** atomic and the caller must lock; and **no** pattern may ever CORRUPT (crash / impossible value / wrong-deterministic result). GIL-CPython showing SAFE everywhere is the artifact the contract rejects.

An `ERR timeout` cell is a runtime too slow to finish this scale — a *performance* issue, not a thread-safety signal; thread-safety correctness at reduced scale is covered by the `concurrency/` fixtures.

<!-- CONCURRENCY-CAPABILITY:END -->

## Performance data — measured (mamba vs CPython 3.12)

First real comparison via `benches/3p/cross_runtime.rs` (best-of-N wall + peak
RSS). The shape is exactly what force-typing predicts — and it exposes the one
keystone gap.

**Speed (CPU):** on typed/compute workloads mamba is **dramatically faster** —
**median ~13× (up to 22×)**, 36 / 41 sampled fixtures win:

| workload class | mamba vs CPython | examples |
|----------------|------------------|----------|
| typed compute / hot loops | **8–22× faster** | `configparser` 22× · `Counter` 20× · `binascii` 19× · `fib` 12× · `closures` 7× |
| object / dynamic dispatch | **2.7–7× SLOWER** | `any/all` 7× · `listcomp` 3.6× · `abc_dispatch` 2.7× · `class_method` 2.9× |
| generator / dataclass construction | **10–25× SLOWER** | `gen_iter` 25× · `dataclass_create` 10× |

**Memory (peak RSS):** mamba currently uses **more** RSS almost everywhere
(median ~1.4×), and **catastrophically** on object/float-heavy code:
`abc_dispatch` 20× (179 MB vs 9 MB), spectral-norm **5.5 GB** vs 10 MB. This is
the boxed-value model (104 B `MbObject` header + boxed floats) — the **#1
keystone fix** (unboxed typed data), and the reason the compute wins don't yet
generalize.

## Performance ceiling — match Golang (`mamba/go`, in progress)

CPython is the daily floor anchor (clean same-source A/B); **Go is the ceiling**,
compared cross-language. `benches/lang/lang_bench.py` runs the **same algorithm
+ same N + same measure method** across **python / go / mamba**, output-equality
gated. From the Computer Language Benchmarks Game compute kernels:

- Go is **~37–63× faster than CPython** on pure compute (spectral-norm 63×,
  n-body 58×); the library/IO kernels (regex-redux 1.0×, pidigits 1.6×) are *not*
  language-speed signals (a C library does the work).
- mamba is **~13× faster than CPython** on compute today → **~4× behind Go**.
- "Match Golang" = lift mamba's CPython-relative multiple from ~13× to ~50×. The
  blockers are the **value model** (unboxed float) and **codegen depth**
  (Cranelift JIT → LLVM AOT for the ceiling) — both engineering, *not* a
  JIT-vs-AOT limit: mamba is force-typed, so it is AOT-capable like Go.

> Numbers are quoted from composite/hot-loop workloads measured both (all three)
> ways on one machine — never from stubs or startup-dominated micro-calls.

## py3.13 / py3.14 feature candidates (lib-level roadmap)

mamba targets **CPython 3.12 parity**. Features introduced in 3.13+/3.14+ are
tracked as *candidates* (not yet in scope) at lib granularity — listed here so the
gap is explicit, not silently missing. Curated/non-exhaustive; language-level PEPs
(t-strings, deferred annotations, free-threading) are separate from these stdlib deltas.

| lib | candidate (version) | notable addition |
|-----|---------------------|------------------|
| `os` | 3.13+ | `process_cpu_count()`; `os.path.isreserved` |
| `copy` | 3.13+ | `copy.replace()` (replace fields on immutables) |
| `pathlib` | 3.13+ / 3.14+ | `PurePath.full_match()`, `Path.from_uri()` (3.13); `Path.copy()`/`Path.move()` (3.14) |
| `glob` | 3.13+ | `glob.translate()` (glob→regex) |
| `dbm` | 3.13+ | `dbm.sqlite3` backend |
| `base64` | 3.13+ | `z85encode` / `z85decode` |
| `array` | 3.13+ | `'w'` typecode (UCS4 unicode) |
| `statistics` | 3.13+ | `kde()`, `kde_random()` |
| `random` | 3.13+ | CLI `python -m random`; `Random` pickling tweaks |
| `argparse` | 3.13+ | `deprecated=` for arguments/subcommands |
| `warnings` | 3.13+ | `warnings.deprecated()` decorator (PEP 702) |
| `typing` | 3.13+ | `ReadOnly`, `TypeIs`, PEP 696 TypeVar defaults |
| `ipaddress` | 3.13+ | `IPv4Address.ipv6_mapped`, `is_global` fixes |
| `asyncio` | 3.14+ | introspection / graph tooling, eager-task defaults |
| `concurrent` | 3.14+ | `concurrent.interpreters` (PEP 734 stdlib interpreters) |
| `unicodedata` | 3.14+ | Unicode 16.0 |
| `zlib`/`bz2`/`lzma` | 3.14+ | sibling `compression.zstd` (PEP 784) |
| `annotationlib` | 3.14+ | new module (deferred-annotation introspection, PEP 749) |

> These are **not** counted in the pass matrix below (which is 3.12-scoped). They
> are the forward backlog; a candidate ships only after its 3.12 baseline lib is green.

## C-extension packages — hack, bridge, or native

Pure-Python packages mamba compiles to native directly. **C-extension packages
cannot load** — mamba has no FFI (no `dlopen` / `ctypes` / libffi), so any package
whose core is a C/C++ extension needs one of three treatments:

1. **Hack** — a mamba-native **shim** that reimplements the package's *API* on top of
   a mature Rust crate (e.g. `protobuf` → `prost`). API-compatible by intent,
   MVP coverage in practice — **not the upstream package**: common paths match
   CPython, exotic API may diverge.
2. **Bridge** — run the C-extension under an embedded / subprocess CPython. Any
   package works, but bridged code loses the native / no-GIL / low-RSS wins. This is
   acceptable precisely because the C-ext mamba can't do natively (cloud/DB clients,
   gRPC) is mostly **I/O-bound** — where mamba wouldn't beat CPython anyway, so the
   app's CPU-bound code stays native-fast and the I/O parts borrow CPython cheaply.
3. **Native kit** — a full mambalib (see [C3](#c3-mambalibs-end-to-end--axis-3-3457)),
   the lib reimplemented natively end-to-end.

### Hacked C-extension foundations (chokepoints)

A few C-ext libs are *foundations*: hacking one unblocks a whole downstream tree.
**These are native shims, not the upstream package** — listed here so users know
exactly what they're getting, and as the forward roadmap. The best hack targets are
C-ext libs that already have a mature Rust crate (the shim wraps it as a mambalib
kit). Status today is **stub / planned** — none are real shims yet.

| package | native backing | unblocks | status |
|---------|----------------|----------|--------|
| `protobuf` | `prost` | grpc · GCP · all RPC (**the foundation**) | 🔴 stub |
| `grpcio` | `tonic` | GCP · microservice clients (needs protobuf first) | 🔴 stub |
| `cryptography` | rustls / ring / openssl (already Rust upstream) | auth · TLS · JWT · requests | 🔴 stub |
| `pydantic-core` | already Rust upstream | FastAPI · validation | 🟡 stub → partial |
| `orjson` / `msgpack` | serde (already Rust upstream) | JSON / msgpack serialization | 🔴 stub |
| `numpy` | `arraykit` | pandas · scipy · sklearn (whole data stack) | 🔴 none |
| `gevent` / `greenlet` | mamba native async / threads | gunicorn gevent workers | 🔴 none |

> **`gevent` is special — a free upgrade, not a port.** `greenlet` exists to work
> around CPython's GIL with cooperative coroutines. mamba has *no* GIL, so the shim
> maps gevent's API onto **real native concurrency** instead of porting greenlet's
> stack-switching C magic. Libs that exist to patch a CPython limitation mamba
> doesn't have (gevent, uvloop) become thin shims over native primitives.

> **What you actually get.** A hacked package is API-compatible by intent and
> MVP-coverage in practice. If you depend on `protobuf` under mamba you receive the
> `prost`-backed shim, **not** Google's C++ protobuf — verify edge cases. Per-package
> coverage is in the library matrix above; the engine (prost/tonic/rustls) is the
> easy part, the package's full Python API surface is the real work.

> **GCP** is the marquee case: it sits on `protobuf` + `grpcio` + `cryptography`
> (all C-ext). The path is *hack those three foundations first*, then the pure-Python
> `google.cloud.*` layer compiles on top — incremental, not one giant GCP port.

## Standardization obligations

Mamba is an existing-project takeover under Agentic Workflow. Beyond the four user-facing capabilities, three orthogonal layers must converge before mamba is "done" in the SDD sense. These are not promises to users — they are obligations to the SDD pipeline.

| Layer        | What it means                                                                   | Current  | Gate                                       | Epic |
|--------------|---------------------------------------------------------------------------------|----------|--------------------------------------------|------|
| **managed**     | Every in-scope file is marked `CODEGEN` or `HANDWRITE`. No unmarked files.       | **5.1%** (119 / 2343; 90 HANDWRITE + 29 CODEGEN + 2224 unmarked) | `aw standardize managed report mamba` | [#3882](https://github.com/chrischeng-c4/cclab/issues/3882) |
| **semantic**    | Source behavior is covered by semantic TD (claim TDs do not count).              | **0.7%** (17 semantic / 2343 units) | `aw standardize semantic report mamba` | [#3883](https://github.com/chrischeng-c4/cclab/issues/3883) |
| **regenerable** | Every in-scope file is fully `CODEGEN`-owned (no remaining `HANDWRITE` regions). | **0.1%** maturity; 27 audit/replay drift | `aw standardize regenerable report mamba` | [#3884](https://github.com/chrischeng-c4/cclab/issues/3884) |

Layer order is **managed → semantic → regenerable** — you cannot specify what to generate before you know which files are in scope, and you cannot regenerate before the spec is semantic. There is **no skip state for source ownership**: if codegen can't generate a region yet, mark it `HANDWRITE`, name the concrete generator gap, and feed the gap back into Agentic Workflow until it can become `CODEGEN`.

## Non-goals

- A second ABI for out-of-tree native modules. Native code ships as a kit inside mamba (C3).
- "Issues closed per week" or any proxy metric. The capability gates and standardization layers above are the only completion signal.

## Status

Measured numbers per axis are in **[Capability status — the four axes](#capability-status--the-four-axes-measured)** above; this is the coarse production-readiness roll-up.

| Track | Item | Production-ready |
|-------|------|------------------|
| Capability      | C1 Py3.12 parity            | No — ① type **74.1%** enforced (auto-measured) + **100%** sound · ② ~18% run-correct |
| Capability      | C2 Perf > CPython           | No — compute median ~13× faster, but object/float slower **and** memory regresses; the boxed value model is the keystone |
| Capability      | C3 mambalibs end-to-end     | No — most kits stub-only |
| Capability      | C4 Package manager (uv-like)| Yes — offline uv-like workflow gates cover init/auth/index/add/remove/lock/export/tree/version/pip/venv/python/workspace/shell/sync/run/install/tool/hash/cache |
| Runtime         | core substrate stability    | **99.1%** — sound; only edge crashes (deep recursion, gen-nesting cap, MRO, async-gen hang) |
| Ceiling         | match Golang (`mamba/go`)   | ~4× behind Go on compute today (Go ~50× vs CPython, mamba ~13×); gap = value model + codegen, not JIT-vs-AOT |
| Standardization | managed                     | No — 5.1%; epic [#3882](https://github.com/chrischeng-c4/cclab/issues/3882) |
| Standardization | semantic                    | No — 0.7%; epic [#3883](https://github.com/chrischeng-c4/cclab/issues/3883) |
| Standardization | regenerable                 | No — 0.1%; epic [#3884](https://github.com/chrischeng-c4/cclab/issues/3884) |
