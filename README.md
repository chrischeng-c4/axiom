# axiom

A monorepo of high-performance, Rust-built developer infrastructure. Each
project below is self-contained and ships its own README — follow the links for
details.

## Projects

<!-- aw:projects-table:start -->
| Project | What it is |
|---------|------------|
| [agentic-workflow](apps/agentic-workflow/README.md) | Workflow protocol and CLI chain for capability-driven project takeover, work-item planning, TD/CB lifecycle execution, and production-readiness rollup. |
| [cap](apps/cap/README.md) | `cap` keeps heavy local commands (`cargo test`, `uv run`, `pnpm build`, …) from eating the whole machine. |
| [vat](apps/vat/README.md) | `vat` is a local development test runner for the one operator Docker was never designed for: a coding/ML agent. |
| [loom](apps/loom/README.md) | Loom is the workflow scheduler in the Axiom service stack. |
| [preview](apps/preview/README.md) | `preview` manages MR-scoped UAT preview environments for GKE. |
| [tape](apps/tape/README.md) | Tape is the topic replay journal in the Axiom service stack. |
| [defer](apps/defer/README.md) | Defer is the Cloud Tasks-like delayed task dispatch service in the Axiom stack. |
| [cube](apps/cube/README.md) | Cube is the OLAP service in the Axiom service stack. |
| [beam](apps/beam/README.md) | Beam is the GPU vector database in the Axiom service stack. |
| [arena](apps/arena/README.md) | N-target competitive comparison runner — fan one workload across targets, ratio + ratchet-gate, one agent-readable JSON report. |
| [guard](apps/guard/README.md) | Security posture gate for the cclab ecosystem. |
| [rig](apps/rig/README.md) | Declarative test-scenario harness engine for the cclab ecosystem. |
| [mamba](projects/mamba/README.md) | Force-typed Python compiler. |
| [lumen](projects/lumen/README.md) | A K8s-native, log-replicated search specialist. |
| [jet](apps/jet/README.md) | Jet is a Rust-native frontend toolchain. |
| [relay](apps/relay/README.md) | `relay` is the durable ordered-log and queue broker in the Axiom stack. |
| [keep](apps/keep/README.md) | Cloud-native, multi-core key-value / claim-check store — the loom/relay data plane and a Redis / Dragonfly replacement. |
| [cgdb](apps/cgdb/README.md) | Cgdb is a local graph database for agentic codebase understanding. |
| [meter](apps/meter/README.md) | Local resource measurement for agent-driven Rust development. |
<!-- aw:projects-table:end -->

## Shared Libraries

Services and tools compose the internal libraries below instead of
reimplementing transport, auth, metrics, codegen, replication, durable local
storage, backup, or operator plumbing locally. Shared service capabilities
belong in `libs/*`; apps supply domain behavior and wiring. Libraries have no
user-facing CLI or release pipeline; those surfaces belong under `apps/`.

| Library | What it is |
|---------|------------|
| [build-stamp](libs/build-stamp/Cargo.toml) | Shared `build.rs` stamping for service CLIs: git short SHA, build epoch, and target triple. |
| [claimtoken](libs/claimtoken/Cargo.toml) | Scoped claim-check access tokens; issuers sign bounded payload/key access and services verify the scope. |
| [cli-std](libs/cli-std/Cargo.toml) | Shared implementation for the required `llm`, `upgrade`, and `issue` CLI convention. |
| [compass](libs/compass/README.md) | Code-intelligence engine for navigation, analysis, refactoring, and watch workflows. |
| [h2c](libs/h2c/Cargo.toml) | Shared HTTP/2 cleartext transport: single client, round-robin pool, frame-level manager, and connection-count heuristic. |
| [openapi-codegen](libs/openapi-codegen/Cargo.toml) | Typed TypeScript, Python, and Rust API client generation from OpenAPI 3.0/3.1 documents. |
| [server-core](libs/server-core/Cargo.toml) | Shared server substrate: bind config, shutdown/drain, readiness signals, connection budgets, and metrics hooks. |
| [tcp-server](libs/tcp-server/Cargo.toml) | Shared TCP accept/runtime layer for raw protocols, proxies, and poolers, built on server-core. |
| [http-server](libs/http-server/Cargo.toml) | Shared HTTP runtime for tool/dev servers and service shells: HTTP/1.1 + h2c serve and request tracing. |
| [operator](libs/operator/Cargo.toml) | Shared Kubernetes operator scaffold: reconcile controller, leader election, and HA render toolkit. |
| [raft-core](libs/raft-core/Cargo.toml) | Transport- and storage-agnostic, step-driven Raft consensus core. |
| [raft-host](libs/raft-host/Cargo.toml) | Shared Raft host driver over h2c peer transport with snapshots, compaction, and read-your-write propose. |
| [service-auth](libs/service-auth/Cargo.toml) | Shared request-auth middleware: extract, verify, reject, and inject verified identity into service handlers. |
| [service-backup](libs/service-backup/Cargo.toml) | Shared backup contract: destination and policy schema, sink trait, local and S3-compatible sinks, and runner primitive. |
| [service-durability](libs/service-durability/Cargo.toml) | Shared durable local storage primitives: fsync policy, atomic replace, CRC-framed append logs, and sequence-named snapshot stores. |
| [service-http](libs/service-http/Cargo.toml) | Standard HTTP service shell: probes, readiness, metrics, OpenAPI/docs routes, tracing, graceful drain, and h2c serve. |
| [service-metrics](libs/service-metrics/Cargo.toml) | Lock-free Prometheus primitives and text encoder for service metrics. |
| [service-tls](libs/service-tls/Cargo.toml) | Peer mTLS material loading and rustls server/client config builders. |
| [surface](libs/surface/Cargo.toml) | Renderer-neutral UI element model shared by Jet WASM, native readers, renderers, and parity tools. |
| [ui-runtime](libs/ui-runtime/Cargo.toml) | Renderer-neutral component runtime: hooks, fiber storage, mount, flush, and update scheduling. |

## Install

Each binary ships a `curl | sh` installer that downloads the right prebuilt
binary from GitHub Releases and drops it on your `PATH` (default
`$HOME/.local/bin`). Self-update later with `<binary> upgrade`. Projects without
an installer yet are marked _coming soon_.

| Project | Binary | Install |
|---------|--------|---------|
| [agentic-workflow](apps/agentic-workflow/README.md) | `aw` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/apps/agentic-workflow/install.sh \| sh` |
| [arena](apps/arena/README.md) | `arena` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/apps/arena/install.sh \| sh` |
| [cap](apps/cap/README.md) | `cap` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/apps/cap/install.sh \| sh` |
| [guard](apps/guard/README.md) | `guard` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/apps/guard/install.sh \| sh` |
| [jet](apps/jet/README.md) | `jet` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/apps/jet/install.sh \| sh` |
| [beam](apps/beam/README.md) | `beam` | _coming soon_ |
| [cube](apps/cube/README.md) | `cube` | _coming soon_ |
| [defer](apps/defer/README.md) | `defer` | _coming soon_ |
| [keep](apps/keep/README.md) | `keep` | _coming soon_ |
| [loom](apps/loom/README.md) | `loom` | _coming soon_ |
| [lumen](projects/lumen/README.md) | `lumen` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/lumen/install.sh \| sh` |
| [mamba](projects/mamba/README.md) | `mamba` | _coming soon_ |
| [meter](apps/meter/README.md) | `meter` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/apps/meter/install.sh \| sh` |
| [preview](apps/preview/README.md) | `preview` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/apps/preview/install.sh \| sh` |
| [relay](apps/relay/README.md) | `relay` | _coming soon_ |
| [rig](apps/rig/README.md) | `rig` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/apps/rig/install.sh \| sh` |
| [tape](apps/tape/README.md) | `tape` | _coming soon_ |
| [vat](apps/vat/README.md) | `vat` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/apps/vat/install.sh \| sh` |

## Runtime Evidence Loop

The runtime tools are intentionally split by responsibility:

- `vat` prepares and runs the local environment.
- `rig` drives requests, queries, and workload traffic.
- `meter measure` observes a running executable or service from the outside and
  records cpu time, wall time, peak RSS, and optional stack samples under
  `.meter/`.
- `meter profile` folds embedded/source-aware profiling data, such as phase
  breakdowns emitted by code that uses meter APIs.
- `arena` compares collected benchmark results across targets.
- `guard` turns static and runtime security evidence into one posture report.

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the repo-wide authoring contract:
how to shape files, paths, and names so the tree stays legible to agents and
tooling, plus the shared **service archetype** (durable-only, scheduled object
snapshots, HA, HTTP/2 + OpenAPI, k8s-native) and the **CLI convention** every
binary follows (`llm` / `upgrade` / `issue`).

## License

MIT
