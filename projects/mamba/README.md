# Mamba

Force-typed Python compiler. Lexes Python source with `logos`, lowers through HIR/MIR, and emits native machine code via Cranelift JIT/AOT. Not a transpiler, not an interpreter — produces real binaries.

For implementation map, see [llms.txt](llms.txt).

## Capabilities

Mamba promises four capabilities. Completion of all four = mamba is done. Proxy metrics (issue close count, fixture count, etc.) do not count.

### C1. Py3.12 functional parity — Axis 1 ([#3331](https://github.com/chrischeng-c4/cclab/issues/3331))

Run real Python 3.12 programs without semantic divergence. Covers four layers:

- **Language core**: control flow, scopes, closures, comprehensions, descriptor protocol, full dunder set, decorators, async/await, generators.
- **PEPs**: 654 (exception groups), 695 (type aliases + generic syntax), 701 (f-string improvements), match statement.
- **Builtins**: numeric / comparison / container / reflected dunders, `print`/`len`/`type`/`isinstance`, etc.
- **Stdlib**: `os`, `json`, `re`, `itertools`, `math`, … per typeshed surface coverage.
- **3rd-party libs**: attrs, click, jinja2, pydantic, flask, fastapi, httpx, requests, pytest, sqlalchemy, and friends.

**Gate**: `cargo test -p mamba --test cpython_lib_test_runner --release`. Red = Axis 1 not met. Tracked under epic [#3331](https://github.com/chrischeng-c4/cclab/issues/3331); CPython `Lib/test/test_*.py` is the canonical denominator ([#1396](https://github.com/chrischeng-c4/cclab/issues/1396)), supplemented by typeshed surface coverage ([#1397](https://github.com/chrischeng-c4/cclab/issues/1397)).

### C2. Performance > CPython — Axis 2 ([#3880](https://github.com/chrischeng-c4/cclab/issues/3880))

Beat CPython 3.12 on real workloads, not micro-benches alone.

Tier targets:

| Tier    | Target          | Status               |
|---------|-----------------|----------------------|
| compute | ≥ 10× CPython   | `fib_recursive ≈ 17×` |
| app     | ≥  3× CPython   | not yet measured     |
| dynamic | ≥ 1.5× CPython  | not yet measured     |
| floor   | ≥ 1.0× CPython  | not yet measured     |

Harness: `criterion` micro-benches (`benches/mamba_bench.rs`) + cross-runtime ratio runner (`benches/3p/cross_runtime.rs`).

### C3. mambalibs end-to-end — Axis 3 ([#3457](https://github.com/chrischeng-c4/cclab/issues/3457))

A statically linked set of Rust-native libraries exposed as importable Python modules inside mamba. Each kit registers via `MambaModule` + `linkme` distributed slice and is force-linked into the final mamba binary — no separate ABI, no dynamic plugin layer.

Current kits in [mambalibs/](mambalibs/): `agentkit`, `httpkit`, `arraykit`, `cryptokit`, `mediakit`, `mongokit`, `pgkit`, `plotkit`, `queuekit`, `scikit`.

**Gate (per #3331 spot-check)**: every kit's top-level imports must resolve to a real callable, not `NoneType`. FFI binding readiness for numpy / pandas / scipy / pillow / cryptography is tracked under epic [#3457](https://github.com/chrischeng-c4/cclab/issues/3457).

### C4. Package manager — uv-like ([#3881](https://github.com/chrischeng-c4/cclab/issues/3881))

A built-in package manager: resolver, lockfile, PyPI client, project manifest (`mamba.toml`), all in one binary. Lives under [src/pkgmanage/](src/pkgmanage/). Goal is parity with `uv`'s ergonomics over the mamba runtime.

Current state: skeleton (`builder/`, `lockfile/`, `manifest/`, `pkgmgr/`, `source/`). Resolver, installer, and PyPI fetch are not yet wired.

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

| Track | Item | Production-ready |
|-------|------|------------------|
| Capability      | C1 Py3.12 parity            | No — Axis 1 gate red |
| Capability      | C2 Perf > CPython           | No — only compute tier validated |
| Capability      | C3 mambalibs end-to-end     | No — most kits stub-only |
| Capability      | C4 Package manager (uv-like)| No — skeleton only |
| Standardization | managed                     | No — 5.1%; epic [#3882](https://github.com/chrischeng-c4/cclab/issues/3882) |
| Standardization | semantic                    | No — 0.7%; epic [#3883](https://github.com/chrischeng-c4/cclab/issues/3883) |
| Standardization | regenerable                 | No — 0.1%; epic [#3884](https://github.com/chrischeng-c4/cclab/issues/3884) |
