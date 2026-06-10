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
| [meter](projects/meter/README.md) | Local resource measurement for agent-driven Rust development — CPU hot spots, phase/boundary cost, benchmark regression folding, and delegated test-failure packaging as deterministic JSON. |

## License

MIT
