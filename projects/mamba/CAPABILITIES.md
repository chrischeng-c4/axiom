# Mamba

## Brief

Machine-readable capability contract for Mamba.

## Capabilities

Canonical field-style capability contracts below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| T1. Mamba Core Semantics | #1505 | partial | planned | conformance | not_ready | Force Typed + Always Free-Threaded intentional divergence contract; later tiers wait for required ECs |
| C1. Py3.12 functional parity — Axis 1 | #3331 | partial | planned | conformance | not_ready | confirmed README promise; CPython oracle gate remains open |
| C2. Less CPU time AND less memory than CPython — Axis 2 | #3880 | planned | planned | conformance | not_ready | confirmed README promise; CPU/RSS ratio gates remain open |
| C3. mambalibs end-to-end — Axis 3 | #3457 | partial | planned | conformance | not_ready | confirmed README promise; native module coverage remains open |
| C4. Package manager — uv-like | #459 | implemented | verified | conformance | ready | uv-like offline workflow coverage is green across init/auth/index/add/remove/lock/export/tree/version/pip/venv/python/workspace/shell/sync/run/install/tool/hash/cache |

### T1. Mamba Core Semantics

ID: mamba-core-semantics
Root WI: #1505
Status: confirmed
Type: RuntimeTool
Required Verification: conformance, negative
Promise:
Mamba is force typed and always free-threaded. Type inference failure is a
compile error and `Any` is available only when explicitly requested; ingress,
egress, generic, subtype, union, and widening walls must reject invalid values
without rejecting valid subtypes. The runtime has no GIL: threads, thread
pools, and executors provide real CPU multicore execution, while ordinary tasks
on one event loop remain cooperatively serial. One built-in container mutation
is memory-safe and atomic; compound operations are not transactionally atomic
and require caller locking when a multi-step invariant matters. Readiness needs
correctness plus race, deadlock, leak, CPU, peak-RSS, and multicore evidence.
Gate Inventory:
- projects/mamba/external-contracts/type-system.md; projects/mamba/external-contracts/concurrency.md; projects/mamba/validation; #1996 Delivery Queue
Surfaces:
- Compiler: `mamba check` + `mamba build` + `mamba run` + `threading` + `concurrent.futures` + `asyncio` - inference and wall enforcement; Python: `threading` + `concurrent.futures` + `asyncio` + built-in container mutation - always-free-threaded runtime behavior
EC Dimensions:
- behavior: Force Typed rejection/acceptance and cooperative event-loop results
- efficiency: CPU/RSS plus multicore scaling
- stability: race/deadlock/leak and memory-safe container mutation

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Force Typed generic binding lifecycle | epic | #1505 | partial | planned | conformance | atomize before implementation; `external-contracts/type-system.md` |
| Force Typed inference failure, explicit Any, and wall completeness | epic | #1769 | partial | planned | conformance | product decision resolved by #1996; atomize before implementation |
| Deterministic type-wall outcomes | change | #1942 | partial | planned | conformance | repeated identical failing-set gate |
| Valid subtype acceptance at annotated ingress | change | #1953 | partial | planned | conformance | positive/negative widening pair |
| Always-free-threaded readiness denominator | epic | #713 | partial | planned | conformance | atomize; `external-contracts/concurrency.md` |
| Parallel `to_thread` gather preserves every result | change | #1841 | partial | planned | conformance | selected EC authoring WI; dependency #1845 closed |
| List mutating-return path remains memory-safe | change | #1857 | partial | planned | conformance | focused race/UAF/repeat evidence |

### C1. Py3.12 functional parity — Axis 1

ID: c1-py3-12-functional-parity-axis-1
Root WI: #3331
Status: confirmed
Type: RuntimeTool
Required Verification: conformance, corpus, negative
Promise:
Run real Python 3.12 programs without semantic divergence across language core, PEP syntax/semantics, builtins and stdlib, plus selected 3rd-party libraries. CPython `Lib/test` and typeshed are the authoritative denominators; declared force-typing divergences must be explicit rather than hidden as ordinary behavior failures.
Gate Inventory:
- `cargo test -p mamba --test conformance_cpython_lib_test`; `cargo test -p mamba --test conformance_contract`; `cargo test -p mamba --test conformance_real_world`; `cargo test -p mamba --test conformance_runtime_shutdown`; projects/mamba/tests/PRODUCTION-GATE.md
Surfaces:
- CLI: `mamba build` + `mamba check` + `mamba run` + `mamba test` + `mamba test-batch` + `mamba pytest` + `mamba surface-report` - compile, type-check, run, batch, pytest, and surface-conformance entrypoints
EC Dimensions:
- behavior: `cargo test -p mamba --test conformance_cpython_lib_test` - debug-build CPython 3.12 Lib/test oracle
- stability: `cargo test -p mamba --test conformance_runtime_shutdown` - runtime shutdown and crash-boundary checks

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Python 3.12 parity gate | epic | #3331 | partial | planned | conformance | `cargo test -p mamba --test conformance_cpython_lib_test`; `cargo test -p mamba --test conformance_contract`; `cargo test -p mamba --test conformance_real_world`; `cargo test -p mamba --test conformance_runtime_shutdown`; projects/mamba/tests/PRODUCTION-GATE.md |

### C2. Less CPU time AND less memory than CPython — Axis 2

ID: c2-less-cpu-time-and-less-memory-than-cpython-axis-2
Root WI: #3880
Status: confirmed
Type: RuntimeTool
Required Verification: conformance
Promise:
Performance is a committed capability: for the same program, mamba targets strictly less CPU time and strictly less peak RSS than CPython 3.12. The v1 bar is staged, not one-shot: at least 1.5x where force typing pays, no worse than roughly 0.8x on CPython-tuned C hot paths, and both CPU/RSS measured externally before claiming progress.
Gate Inventory:
- `cargo test -p mamba --release --test perf_pin -- perf_pin`; `cargo bench -p mamba --bench mamba_bench`; projects/mamba/benches/3p/cross_runtime.rs; projects/mamba/tests/harness/cpython/config/perf/pins
Surfaces:
- CLI: `mamba bench --compare cpython` + `mamba bench --fixtures` + `mamba bench --check` - benchmark and regression gate entrypoints
EC Dimensions:
- behavior: `mamba bench` - benchmark report generation contract
- efficiency: `cargo test -p mamba --release --test perf_pin -- perf_pin` - CPU/RSS ratio pins against CPython

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| CPython CPU/RSS ratio gate | epic | #3880 | planned | planned | conformance | `cargo test -p mamba --release --test perf_pin -- perf_pin`; `cargo bench -p mamba --bench mamba_bench`; projects/mamba/benches/3p/cross_runtime.rs; projects/mamba/tests/harness/cpython/config/perf/pins |

### C3. mambalibs end-to-end — Axis 3

ID: c3-mambalibs-end-to-end-axis-3
Root WI: #3457
Status: confirmed
Type: RuntimeTool
Required Verification: conformance
Promise:
A statically linked set of Rust-native libraries exposed as importable Python modules inside mamba. Each kit registers via `MambaModule` plus the `linkme` distributed slice and is force-linked into the final mamba binary, with import/callable coverage for native kits instead of a separate ABI or dynamic plugin layer.
Gate Inventory:
- `cargo test -p mamba --test mambalibs`; projects/mamba/mambalibs; projects/mamba/src/pkgmanage/builder/force_link.rs
Surfaces:
- Python: `mambalibs.*` + `mamba run` - import `mambalibs.*` through `mamba run` - Rust-native modules exposed inside the mamba runtime
- CLI: `mamba run` + `mamba <file>.py` - execute programs that import native kits
EC Dimensions:
- behavior: `cargo test -p mamba --test mambalibs` - native module registration, import, and callable coverage

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Native mambalibs import/callable surface | epic | #3457 | partial | planned | conformance | `cargo test -p mamba --test mambalibs`; projects/mamba/mambalibs; projects/mamba/src/pkgmanage/builder/force_link.rs |
| httpkit HTTP/2 client contract | change | #526 | implemented | verified | conformance | `cargo test -p mambalibs-http --test client_http2_test`; projects/mamba/mambalibs/httpkit/src/client |

### C4. Package manager — uv-like

ID: c4-package-manager-uv-like
Root WI: #459
Status: candidate
Type: DeveloperTool
Required Verification: conformance, negative
Promise:
A built-in package manager surface for project scaffold, dependency add/remove, lockfile generation, sync/install, cache, and validation workflows. The product promise is `uv`-style ergonomics over the mamba runtime with `mamba.toml` and `mamba.lock` as the agent-readable project contract.
Gate Inventory:
- `cargo test -p mamba --test pkgmgr`; `cargo test -p mamba --test schema_gates pkgmgr`; projects/mamba/tests/pkgmgr; projects/mamba/src/pkgmanage
Surfaces:
- CLI: `mamba init` + `mamba auth` + `mamba index` + `mamba add` + `mamba remove` + `mamba lock` + `mamba audit` + `mamba export` + `mamba tree` + `mamba version` + `mamba package` + `mamba publish` + `mamba pip` + `mamba venv` + `mamba python` + `mamba workspace` + `mamba shell` + `mamba sync` + `mamba install` + `mamba tool` + `mamba cache` + `mamba hash` + `mamba generate-shell-completion` + `mamba pkgmgr-validate` - project scaffold, credentials, frozen index, dependency, lockfile, audit, export, tree, version, package artifact build/publish upload, pip inventory, venv, local and standalone Python discovery/pinning/install management, workspace inspection, shell integration, install, uv-style tool administration, cache, completion, and validation workflows
- Config: `mamba.toml` + `mamba.lock` - manifest and resolved lockfile artifacts
EC Dimensions:
- behavior: `cargo test -p mamba --test pkgmgr` - uv-like workflow fixtures
- stability: `cargo test -p mamba --test schema_gates pkgmgr` - schema, pin, and idempotence contracts

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Local-first package manager baseline | epic | #459 | implemented | verified | conformance | `cargo test -p mamba --test pkgmgr`; `cargo test -p mamba --test schema_gates pkgmgr`; `./target/debug/mamba pkgmgr-validate --json`; projects/mamba/tests/pkgmgr; projects/mamba/src/pkgmanage |
| Full uv package-manager parity and beyond | epic | #519 | implemented | verified | uv-parity | `cargo test -p mamba --test pkgmgr`; `./target/debug/mamba pkgmgr-validate --json`; projects/mamba/src/pkgmanage/pkgmgr; projects/mamba/tests/pkgmgr |
| `mamba run` command mode | change | #525 | implemented | verified | uv-parity | `cargo test -p mamba --test pkgmgr run_preflight::run_command_mode`; projects/mamba/src/main.rs; projects/mamba/src/pkgmanage/run.rs |

Current state: `mamba init/auth/index/add/remove/lock/export/tree/version/package/publish/pip/venv/python/workspace/shell/sync/run/install/tool/hash/cache`
plus `pkgmgr-validate` are wired through offline frozen-index gates, direct
local wheel paths, explicit registry URL tests, lockfile export to
requirements.txt / pylock.toml, dependency-tree rendering, PEP 621 version
bumping, and pip-compatible requirements compile plus installed-environment
install/sync/uninstall/list/freeze/show/tree/check inspection and
dependency-tree rendering against frozen indexes and explicit registry URLs.
`mamba audit` checks `mamba.lock` against an offline
advisory database, and `mamba lock --check` / `mamba sync --check` provide
CI-friendly drift gates without mutating lockfiles or environments. `mamba package build`
now emits deterministic pure-Python wheel and sdist artifacts from PEP 621
`pyproject.toml` projects, and `mamba publish` / `mamba package publish`
upload PyPI legacy multipart payloads with `.pypirc`/CLI credential precedence,
CA-bundle support, JSON summaries, and `--dry-run` validation without leaking
tokens. `mamba venv` exposes create/remove safety around PEP 405
environments, and `mamba cache` now reports exact size/category info plus
dry-run, age, size, and package-targeted pruning. `mamba python` exposes local
interpreter list/find, `.python-version` pinning, managed Python directory
resolution, local-source registration, standalone archive download/install via
explicit URL or python-build-standalone release-tag composition, sha256
verification, uninstall, and shell PATH setup for managed Python launchers. `mamba workspace list/dir/metadata` inspects uv-compatible
`[tool.uv.workspace]` membership, member paths, root paths, and exclusion
patterns. `mamba index build` can
materialize a frozen local index from wheel files or directories for
`mamba add --index` / `mamba lock --index`. `mamba shell path/init` emits
managed PATH snippets for mamba tool bin directories, and
`mamba generate-shell-completion` emits clap-derived bash/zsh/fish/powershell/elvish
completion scripts from the live command tree. `mamba auth dir/login/token/logout`
manages plaintext package-index credentials under an overrideable credentials
directory, and stored credentials now feed explicit-index metadata requests,
resolver requests, and locked artifact downloads. `mamba tool run/install/upgrade/list/uninstall/dir/update-shell` wraps the
tool-install workflow behind a uv-style `tool` command family. The package-manager validation
profile requires twenty-one offline workflow families and keeps live network
coverage opt-in/report-only. `mamba add` / `mamba lock` do not treat public
PyPI as an implicit default source; callers must provide a frozen local index,
direct local wheel file, or explicit registry URL when resolving dependencies.
First-party pure-Python replacement packages use an explicit provider path:
`mamba add --provider mamba mamba-httpx-compat` records the mamba-owned
distribution name, preserves `provides` / compatibility metadata in
`mamba.lock`, and `mamba sync` installs real pure-Python files into `.venv`
so the provided import alias (for example `import httpx`) resolves without
confusing the package with the upstream PyPI distribution. This provider path
is separate from C3 `mambalibs`, which are Rust/native runtime modules.
`mamba run <file.py|file.tp>` remains the mamba runtime/compiler path, while
`mamba run -- <cmd> [args...]` runs arbitrary commands inside the synced project
environment with `.venv` executables and site-packages preferred before host
fallbacks.
No known release-blocking command-family gaps remain under #519; follow-up
parity work should be tracked as focused hardening or live-network fixtures.

