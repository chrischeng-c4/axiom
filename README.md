# axiom

A monorepo of high-performance, Rust-built developer infrastructure. Each
project below is self-contained and ships its own README — follow the links for
details.

## Projects

<!-- aw:projects-table:start -->
| Project | What it is |
|---------|------------|
| [agentic-workflow](projects/agentic-workflow/README.md) | Workflow protocol and CLI chain for capability-driven project takeover, work-item planning, TD/CB lifecycle execution, and production-readiness rollup. |
| [cap](projects/cap/README.md) | `cap` keeps heavy local commands (`cargo test`, `uv run`, `pnpm build`, …) from eating the whole machine. |
| [vat](projects/vat/README.md) | `vat` is a local development test runner for the one operator Docker was never designed for: a coding/ML agent. |
| [loom](projects/loom/README.md) | Loom is the workflow scheduler in the Axiom service stack. |
| [preview](projects/preview/README.md) | `preview` manages MR-scoped UAT preview environments for GKE. |
| [tape](projects/tape/README.md) | Tape is the topic replay journal in the Axiom service stack. |
| [defer](projects/defer/README.md) | Defer is the Cloud Tasks-like delayed task dispatch service in the Axiom stack. |
| [cube](projects/cube/README.md) | Cube is the OLAP service in the Axiom service stack. |
| [beam](projects/beam/README.md) | Beam is the GPU vector database in the Axiom service stack. |
| [arena](projects/arena/README.md) | N-target competitive comparison runner — fan one workload across targets, ratio + ratchet-gate, one agent-readable JSON report. |
| [guard](projects/guard/README.md) | Security posture gate for the cclab ecosystem. |
| [rig](projects/rig/README.md) | Declarative test-scenario harness engine for the cclab ecosystem. |
| [mamba](projects/mamba/README.md) | Force-typed Python compiler. |
| [lumen](projects/lumen/README.md) | A K8s-native, log-replicated search specialist. |
| [jet](projects/jet/README.md) | Jet is a Rust-native frontend toolchain. |
| [relay](projects/relay/README.md) | `relay` is the durable ordered-log and queue broker in the Axiom stack. |
| [keep](projects/keep/README.md) | Cloud-native, multi-core key-value / claim-check store — the loom/relay data plane and a Redis / Dragonfly replacement. |
| [cgdb](projects/cgdb/README.md) | Cgdb is a local graph database for agentic codebase understanding. |
| [httpkit-demo](projects/httpkit-demo/README.md) | httpkit-demo is a generated demo consumer of the `mambalibs.http` framework. |
| [meter](projects/meter/README.md) | Local resource measurement for agent-driven Rust development. |
| [qc](projects/qc/README.md) | Qc is the planned agent-facing quality-control CLI surface for structured reports, security findings, and performance boundary-cost findings. |
| [queue](projects/queue/README.md) | Queue is the Rust distributed-task-queue library surface for cclab. |
<!-- aw:projects-table:end -->

## Install

Each binary ships a `curl | sh` installer that downloads the right prebuilt
binary from GitHub Releases and drops it on your `PATH` (default
`$HOME/.local/bin`). Self-update later with `<binary> upgrade`. Projects without
an installer yet are marked _coming soon_.

| Project | Binary | Install |
|---------|--------|---------|
| [agentic-workflow](projects/agentic-workflow/README.md) | `aw` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/agentic-workflow/install.sh \| sh` |
| [arena](projects/arena/README.md) | `arena` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/arena/install.sh \| sh` |
| [cap](projects/cap/README.md) | `cap` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/cap/install.sh \| sh` |
| [guard](projects/guard/README.md) | `guard` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/guard/install.sh \| sh` |
| [jet](projects/jet/README.md) | `jet` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/jet/install.sh \| sh` |
| [beam](projects/beam/README.md) | `beam` | _coming soon_ |
| [cube](projects/cube/README.md) | `cube` | _coming soon_ |
| [defer](projects/defer/README.md) | `defer` | _coming soon_ |
| [keep](projects/keep/README.md) | `keep` | _coming soon_ |
| [loom](projects/loom/README.md) | `loom` | _coming soon_ |
| [lumen](projects/lumen/README.md) | `lumen` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/lumen/install.sh \| sh` |
| [mamba](projects/mamba/README.md) | `mamba` | _coming soon_ |
| [meter](projects/meter/README.md) | `meter` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/meter/install.sh \| sh` |
| [preview](projects/preview/README.md) | `preview` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/preview/install.sh \| sh` |
| [relay](projects/relay/README.md) | `relay-server` | _coming soon_ |
| [rig](projects/rig/README.md) | `rig` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/rig/install.sh \| sh` |
| [tape](projects/tape/README.md) | `tape` | _coming soon_ |
| [vat](projects/vat/README.md) | `vat` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/vat/install.sh \| sh` |

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
tooling, plus the shared **service archetype** (HA, HTTP/2 + OpenAPI,
k8s-native) and the **CLI convention** every binary follows (`llm` / `upgrade` /
`issue`).

## License

MIT
