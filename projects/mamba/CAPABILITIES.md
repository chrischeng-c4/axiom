# Mamba

## Brief

Machine-readable capability contract for Mamba.

## Capabilities

Canonical field-style capability contracts below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

Roadmap execution order (policy-only; enforced operationally by the GitHub tier
labels and #1996 Delivery Queue): Mamba Core Semantics → Language Core →
Built-ins → C-based stdlibs → Hot stdlibs → Third-party → Other stdlibs
(7a vendor, then 7b native rewrite). A later tier is not dependency-ready while
a required earlier-tier EC is red.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| T1. Mamba Core Semantics | #1996 | partial | planned | conformance | not_ready | Force Typed + Always Free-Threaded intentional divergence contract; later tiers wait for required ECs |
| T2. CPython-compatible Language Core | #2002 | partial | planned | conformance | not_ready | parser through native codegen under Tier 1 invariants |
| T3. Built-ins and Core Value Model | #2001 | partial | planned | conformance | not_ready | numeric, container, text/binary, iterator and reflection parity |
| T4. C-backed Stdlib Compatibility | #2003 | partial | planned | conformance | not_ready | OS/native-backed stdlib with platform and resource evidence |
| T5. Hot Stdlib Native/Hybrid Paths | #1103 | partial | planned | conformance | not_ready | native paths retained only with parity and measured value |
| T6. Third-party Ecosystem | #1119 | partial | planned | conformance | not_ready | real-package install, build and user-journey readiness |
| T7. Other Stdlib: Vendor then Rewrite | #2004 | partial | planned | conformance | not_ready | 7a vendor compatibility precedes selective 7b native rewrites |
| C1. Py3.12 functional parity — Axis 1 | #702 | partial | planned | conformance | not_ready | confirmed README promise; CPython oracle gate remains open |
| C2. Less CPU time AND less memory than CPython — Axis 2 | #707 | planned | planned | conformance | not_ready | confirmed README promise; CPU/RSS ratio gates remain open |
| C3. mambalibs end-to-end — Axis 3 | #714 | partial | planned | conformance | not_ready | confirmed README promise; native module coverage remains open |
| C4. Package manager — uv-like | #459 | implemented | verified | conformance | ready | uv-like offline workflow coverage is green across init/auth/index/add/remove/lock/export/tree/version/pip/venv/python/workspace/shell/sync/run/install/tool/hash/cache |

### T1. Mamba Core Semantics

ID: mamba-core-semantics
Root WI: #1996
Status: confirmed
Type: RuntimeTool
Required Verification: conformance, negative, stability, efficiency, multicore
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
- projects/mamba/external-contracts/type-system.md
- projects/mamba/external-contracts/concurrency.md
- projects/mamba/validation
- #1996 Delivery Queue
Surfaces:
- Compiler: `mamba check` + `mamba build` + `mamba run` - inference and wall enforcement
- Python: `threading` + `concurrent.futures` + `asyncio` + built-in container mutation - always-free-threaded runtime behavior
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
| Force Typed contract completion | subepic | #2011 | planned | planned | negative | explicit `Any`, inference failure, subtype, union and wall children |
| Always-free-threaded runtime state and ownership | subepic | #2024 | planned | planned | conformance | state topology, object lifetime, synchronization and executor children |
| Tier 1 race/deadlock/leak matrix | change | #2019 | planned | planned | conformance | multi-thread stress, reproducible seeds and bounded-resource evidence |
| Tier 1 multicore CPU/RSS gate | change | #2022 | planned | planned | conformance | 1/2/4/8-worker scaling and regression thresholds |
| Tier 1 exit gate | change | #2028 | planned | planned | conformance | fail-closed rollup that releases Tier 2 |

### T2. CPython-compatible Language Core

ID: mamba-language-core
Root WI: #2002
Status: confirmed
Type: RuntimeTool
Required Verification: conformance, corpus, negative, efficiency
Promise:
After Tier 1 establishes Mamba's intentional divergences, parsing, ASTs,
scope, calls, objects, exceptions, generators, coroutines, dynamic code,
imports, introspection, operator dispatch, compiler lowering, native codegen,
and optimizer passes preserve CPython 3.12-visible behavior. Tier 2 never
restores implicit `Any` or GIL-dependent execution.
Gate Inventory:
- projects/mamba/external-contracts/frontend.md
- projects/mamba/external-contracts/codegen.md
- projects/mamba/external-contracts/name-resolution.md
- projects/mamba/external-contracts/calling-convention.md
- projects/mamba/external-contracts/object-model.md
- projects/mamba/external-contracts/exceptions.md
- projects/mamba/external-contracts/iterators.md
- projects/mamba/external-contracts/import-system.md
- projects/mamba/tests/harness/cpython
Surfaces:
- Compiler: `mamba check` + `mamba build` + `mamba run` - parse, lower, compile and execute Python 3.12 programs
- Python: language syntax, data model, frames and import machinery - CPython-compatible observable behavior
EC Dimensions:
- behavior: CPython 3.12 oracle across the full language-core denominator
- stability: deterministic compiler/runtime errors and thread-safe execution
- efficiency: compile, startup, CPU and peak-RSS pins for language hot paths

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Parser and SyntaxError fidelity | subepic | #2036 | planned | planned | conformance | grammar and invalid-syntax denominator |
| AST fields, contexts and locations | subepic | #2033 | planned | planned | conformance | parse-transform-compile oracle |
| Scope, closures and cells | subepic | #2042 | planned | planned | conformance | LEGB/global/nonlocal/closure matrix |
| Calls and argument binding | subepic | #2037 | planned | planned | conformance | direct/dynamic binder matrix |
| Object model and metaclasses | subepic | #2030 | planned | planned | conformance | descriptor/MRO/super/metaclass/slots matrix |
| Exceptions and tracebacks | subepic | #2032 | planned | planned | conformance | propagation/chaining/ExceptionGroup matrix |
| Generators and async language semantics | subepic | #2039 | planned | planned | conformance | generator plus #2038 coroutine/async-generator children |
| Dynamic code and imports | subepic | #2041 | planned | planned | conformance | compile/eval/exec plus #2045 import lifecycle |
| Frames, tracing and operator dispatch | subepic | #2043 | planned | planned | conformance | introspection plus #2034 data-model dispatch |
| Compiler lowering and native codegen | subepic | #2129 | planned | planned | conformance | compiler_assemble/compiler_codegen denominator |
| Opcode and peephole optimizer parity | subepic | #2130 | planned | planned | conformance | optimized-vs-disabled equivalence |
| Tier 2 exit gate | change | #2035 | planned | planned | corpus | fail-closed language denominator that releases Tier 3 |

### T3. Built-ins and Core Value Model

ID: mamba-builtins
Root WI: #2001
Status: confirmed
Type: RuntimeTool
Required Verification: conformance, corpus, negative, stability, efficiency
Promise:
Mamba's numeric values, mappings, sequences, sets, Unicode and binary values,
iterators, reductions, reflection, codecs, and built-in errors match CPython
3.12 except for declared Tier 1 type divergences. Built-in operations retain
Tier 1 single-mutation memory safety and free-threaded execution.
Gate Inventory:
- projects/mamba/external-contracts/numbers.md
- projects/mamba/external-contracts/collections.md
- projects/mamba/external-contracts/strings.md
- projects/mamba/external-contracts/iterators.md
- projects/mamba/tests/harness/cpython
Surfaces:
- Python: built-in types and functions - core value construction, mutation, iteration and reflection
EC Dimensions:
- behavior: CPython built-in denominator and negative argument/error matrix
- stability: container contention, alias and lifetime safety
- efficiency: numeric, container, text/binary and iterator CPU/RSS pins

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Numeric value model | subepic | #2057 | planned | planned | conformance | bool/int/float/complex equality, hash and conversion |
| Mapping and set model | subepic | #2046 | planned | planned | conformance | dict/views plus #2054 set/frozenset children |
| Sequence model | subepic | #2052 | planned | planned | conformance | list/tuple/range/slice matrix |
| Text, binary and codecs | subepic | #2047 | planned | planned | conformance | str plus #2055 binary values and #2049 codecs |
| Iteration, reduction and ordering | subepic | #2048 | planned | planned | conformance | iterator protocol plus #2051 reductions |
| Reflection and built-in errors | subepic | #2053 | planned | planned | negative | protocol built-ins plus #2050 error matrix |
| Tier 3 exit gate | change | #2056 | planned | planned | corpus | built-in denominator, contention and perf rollup |

### T4. C-backed Stdlib Compatibility

ID: mamba-c-stdlib
Root WI: #2003
Status: confirmed
Type: RuntimeTool
Required Verification: conformance, corpus, negative, stability, efficiency
Promise:
Required CPython modules backed by C or operating-system facilities work on
Mamba through real macOS/Linux backends with explicit platform classification,
correct resource lifetime, blocking/cancellation behavior, and no stub or mock
success. Unsupported platform rows remain visible and issue-owned.
Gate Inventory:
- projects/mamba/external-contracts/stdlib.md
- projects/mamba/tests/harness/cpython/tools/platform_readiness.py
- projects/mamba/src/runtime/stdlib
Surfaces:
- Python: OS, process, network, TLS, compression, crypto, database, buffer, time, XML and FFI stdlib APIs
EC Dimensions:
- behavior: CPython module oracles and real OS/service journeys
- stability: fd/handle/resource lifetime, timeout and cancellation stress
- efficiency: native-call CPU/RSS and blocking-progress pins

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| OS, filesystem and processes | subepic | #2066 | planned | planned | conformance | OS/fd/environment plus #2058 subprocess/multiprocessing |
| Network, TLS and signals | subepic | #2067 | planned | planned | conformance | sockets/selectors plus #2062 TLS and #2069 signals |
| Compression and crypto primitives | subepic | #2059 | planned | planned | conformance | compression plus #2064 hash/HMAC/random |
| Databases and binary buffers | subepic | #2068 | planned | planned | conformance | sqlite/DBM plus #2060 array/struct/binascii/mmap |
| Time, XML and platform FFI | subepic | #2063 | planned | planned | conformance | datetime/zoneinfo plus #2065 XML and #2070 FFI/native modules |
| Tier 4 exit gate | change | #2061 | planned | planned | corpus | platform/resource/performance rollup that releases Tier 5 |

### T5. Hot Stdlib Native/Hybrid Paths

ID: mamba-hot-stdlib
Root WI: #1103
Status: confirmed
Type: RuntimeTool
Required Verification: conformance, corpus, stability, efficiency
Promise:
The selected import- and request-hot pure-Python stdlib paths run natively only
when they preserve complete CPython behavior and demonstrate module-specific
CPU, RSS, or startup value. Unimplemented tails resolve through one proven
vendored module identity rather than native shells or sentinel stubs.
Gate Inventory:
- projects/mamba/external-contracts/stdlib.md
- projects/mamba/tests/harness/cpython/config/perf
- projects/mamba/src/runtime/stdlib
- projects/mamba/src/runtime/stdlib/vendor_lib.rs
Surfaces:
- Python: re, posixpath, functools, urllib.parse, json, typing, enum, pathlib, logging, dataclasses, heapq, bisect and copy
EC Dimensions:
- behavior: complete public module denominator and native/vendor equivalence
- stability: thread-safe caches, callbacks and resource ownership
- efficiency: module-specific CPU/RSS/startup thresholds

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Tier 5 module queue | epic | #1103 | partial | planned | conformance | #2071-#2083 module children plus #235 advanced re |
| Native/vendor hybrid contract | subepic | #2075 | planned | planned | conformance | symbol routing, identity and rollback |
| Tier 5 exit gate | change | #2080 | planned | planned | corpus | parity and native-value rollup that releases Tier 6 |

### T6. Third-party Ecosystem

ID: mamba-third-party
Root WI: #1119
Status: confirmed
Type: RuntimeTool
Required Verification: conformance, corpus, negative, stability, efficiency
Promise:
Selected real third-party packages install or build on Mamba and complete
nontrivial user journeys through an explicit pure-Python, native-SDK,
replacement, bounded-emulation, or unsupported route. Import-only probes,
sentinel shims, and fake modules never count as readiness.
Gate Inventory:
- projects/mamba/external-contracts/third-party.md
- projects/mamba/mambalibs
- projects/mamba/tests/harness
Surfaces:
- Package: wheels, sdists and dependency metadata - deterministic installation and routing
- Python: selected package APIs - real validation, serialization, network, database, scientific and cloud journeys
EC Dimensions:
- behavior: pinned real-package upstream slices and end-to-end journeys
- stability: ABI ownership, thread safety, cancellation and resource cleanup
- efficiency: package/workload CPU and peak-RSS thresholds

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Package route and install baseline | subepic | #2089 | planned | planned | conformance | routing matrix plus #2097 packaging/wheels |
| Mamba native-extension SDK | subepic | #2092 | planned | planned | conformance | PyO3 backend without CPython GIL-state assumptions |
| Protocol and serialization foundations | subepic | #2085 | planned | planned | conformance | protobuf plus #2086 grpc and #2091 orjson/msgpack |
| Validation, crypto and scientific foundations | subepic | #2099 | planned | planned | conformance | pydantic plus #2087 crypto and #2093 NumPy foundation |
| Ecosystem journeys | subepic | #2094 | planned | planned | corpus | web, database, cloud, scientific downstream and migration children |
| Tier 6 exit gate | change | #2096 | planned | planned | corpus | real-package readiness rollup that releases Tier 7 |

### T7. Other Stdlib: Vendor then Rewrite

ID: mamba-other-stdlib
Root WI: #2004
Status: confirmed
Type: RuntimeTool
Required Verification: conformance, corpus, stability, efficiency
Promise:
Mamba first ships every remaining supported pure-Python stdlib module from a
governed, version-pinned CPython Lib tree. Only after a module's vendored row is
green may a measured candidate move to a native or hybrid implementation, and
the vendored path remains the oracle and rollback until promotion is proven.
Gate Inventory:
- projects/mamba/external-contracts/stdlib.md
- projects/mamba/src/runtime/stdlib/vendor_lib.rs
- projects/mamba/vendor
- projects/mamba/tests/harness/cpython
Surfaces:
- Distribution: vendored CPython Lib tree, data files, notices and provenance
- Python: all remaining supported stdlib modules - vendor-first compatibility and selective native acceleration
EC Dimensions:
- behavior: complete module/source-mode denominator and CPython oracle
- stability: loader/import/resource and rollback safety
- efficiency: startup/package-size plus approved native-candidate CPU/RSS thresholds

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Tier 7a vendored stdlib | epic | #862 | partial | planned | corpus | governed source, loader, domain batches and provenance |
| Tier 7a exit gate | change | #2104 | planned | planned | corpus | complete vendored denominator before native candidates |
| Tier 7b selective native rewrites | epic | #2000 | planned | planned | conformance | scorecard-approved candidates only |
| Tier 7b exit gate | change | #2116 | planned | planned | corpus | parity, value and rollback rollup |

### C1. Py3.12 functional parity — Axis 1

ID: c1-py3-12-functional-parity-axis-1
Root WI: #702
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
| Python 3.12 replacement-readiness gate | epic | #702 | partial | planned | conformance | full parity taxonomy and issue ownership |
| Language core | epic | #2002 | partial | planned | corpus | Tier 2 exit #2035 |
| Built-ins | epic | #2001 | partial | planned | corpus | Tier 3 exit #2056 |
| C-backed stdlib | epic | #2003 | partial | planned | corpus | Tier 4 exit #2061 |
| Hot and other stdlib | epic | #1103 | partial | planned | corpus | Tier 5 exit #2080; Tier 7 root #2004 |
| Third-party compatibility | epic | #1119 | partial | planned | corpus | Tier 6 exit #2096 |

### C2. Less CPU time AND less memory than CPython — Axis 2

ID: c2-less-cpu-time-and-less-memory-than-cpython-axis-2
Root WI: #707
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
| CPython CPU/RSS ratio gate | epic | #707 | partial | planned | conformance | `cargo test -p mamba --release --test perf_pin -- perf_pin`; `cargo bench -p mamba --bench mamba_bench`; projects/mamba/benches/3p/cross_runtime.rs; projects/mamba/tests/harness/cpython/config/perf/pins |
| Go competitiveness and single binary | epic | #1071 | partial | planned | conformance | call convention, escape/refcount, typed layouts and AOT children |
| Typed specialization and unboxed layout | subepic | #2008 | planned | planned | conformance | correctness plus CPU/RSS gate |
| Free-threaded multicore efficiency | change | #2022 | planned | planned | conformance | 1/2/4/8-worker scaling and resource thresholds |
| Hot-stdlib native value | change | #2080 | planned | planned | conformance | module-specific native/vendor value decision |

### C3. mambalibs end-to-end — Axis 3

ID: c3-mambalibs-end-to-end-axis-3
Root WI: #714
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
| Native mambalibs import/callable surface | epic | #714 | partial | planned | conformance | `cargo test -p mamba --test mambalibs`; projects/mamba/mambalibs; projects/mamba/src/pkgmanage/builder/force_link.rs |
| httpkit HTTP/2 client contract | change | #526 | implemented | verified | conformance | `cargo test -p mambalibs-http --test client_http2_test`; projects/mamba/mambalibs/httpkit/src/client |
| Native extension SDK and PyO3 backend | subepic | #2092 | planned | planned | conformance | shared registration/value/error/buffer/async/thread contract |
| Protobuf and gRPC native foundations | subepic | #2085 | planned | planned | conformance | prost plus #2086 tonic journey |
| Pydantic, cryptography and NumPy foundations | subepic | #2099 | planned | planned | conformance | real package rebuild and journey evidence |

### C4. Package manager — uv-like

ID: c4-package-manager-uv-like
Root WI: #459
Status: verified
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
| Full uv package-manager parity and beyond | epic | #519 | implemented | verified | conformance | `cargo test -p mamba --test pkgmgr`; `./target/debug/mamba pkgmgr-validate --json`; projects/mamba/src/pkgmanage/pkgmgr; projects/mamba/tests/pkgmgr |
| `mamba run` command mode | change | #525 | implemented | verified | conformance | `cargo test -p mamba --test pkgmgr run_preflight::run_command_mode`; projects/mamba/src/main.rs; projects/mamba/src/pkgmanage/run.rs |

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
