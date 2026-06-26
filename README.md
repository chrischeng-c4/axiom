# axiom

A monorepo of high-performance, Rust-built developer infrastructure. Each
project below is self-contained and ships its own README — follow the links for
details.

## Projects

| Project | What it is |
|---------|------------|
| [agentic-workflow](projects/agentic-workflow/README.md) | Workflow protocol and `aw` CLI chain for capability-driven project takeover, work-item planning, TD/CB lifecycle, and production-readiness rollup. |
| [mamba](projects/mamba/README.md) | Force-typed Python compiler — lexes Python, lowers through HIR/MIR, and emits native machine code via Cranelift JIT/AOT. Real binaries, not a transpiler or interpreter. |
| [jet](projects/jet/README.md) | Rust-native web toolchain — package management, dev server, production builds, test/e2e, and WASM/multi-target execution. Replaces the Vite/pnpm/Playwright stack. |
| [lumen](projects/lumen/README.md) | K8s-native, log-replicated search specialist — exact, lexical (BM25), semantic (HNSW/GPU kNN), perceptual, and duplicate search in one engine. |
| [vat](projects/vat/README.md) | Agent-native, GPU-native dev containers — a sandboxed host-process runtime (no VM) where the GPU just works on Apple Silicon, with a single JSON state surface for agents. |
| [cap](projects/cap/README.md) | Resource-protection wrapper — throttles heavy local commands (and the Bash an agent fires) by watching free memory and pausing/resuming/killing, so nothing OOMs the box. |
| [meter](projects/meter/README.md) | Local resource measurement for agents — `measure` observes external executables for cpu/wall/RSS and sampled hot spots; `profile` folds embedded/source-aware phase data. |
| [guard](projects/guard/README.md) | Security posture gate — turns compass static findings plus future vat/rig/meter/arena evidence into one agent-readable security report. |

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
| [keep](projects/keep/README.md) | `keep` | _coming soon_ |
| [loom](projects/loom/README.md) | `loom` | _coming soon_ |
| [lumen](projects/lumen/README.md) | `lumen` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/lumen/install.sh \| sh` |
| [mamba](projects/mamba/README.md) | `mamba` | _coming soon_ |
| [meter](projects/meter/README.md) | `meter` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/meter/install.sh \| sh` |
| [relay](projects/relay/README.md) | `relay-server` | _coming soon_ |
| [rig](projects/rig/README.md) | `rig` | `curl -fsSL https://raw.githubusercontent.com/chrischeng-c4/axiom/main/projects/rig/install.sh \| sh` |
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
