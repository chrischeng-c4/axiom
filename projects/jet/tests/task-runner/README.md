# Block: task runner — replace Nx/Turbo-style workspace task execution

**Claim.** Jet detects workspace task graphs and executes them natively, so
monorepos don't need Nx/Turbo as a separate layer.

## Gates

| Gate | Command | Covers |
|---|---|---|
| Nx support | `cargo test -p jet --test nx_support` | Nx workspace detection and task execution |
| Task runner lib | `cargo test -p jet --lib task_runner -- --nocapture` | config, graph, proxy routing |

## Open gaps

- Turbo-style pipeline config coverage, remote/local cache semantics, and
  benchmark comparison against Nx/Turbo are not yet gated.
