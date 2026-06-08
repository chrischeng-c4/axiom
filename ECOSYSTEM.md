# cclab Ecosystem

AI-assisted software development harness. 4 layers from runtime to domain.

## Layer 1: Runtime & Language

Core systems. No business logic. Deterministic, correct answers exist.

| Component | Purpose |
|-----------|---------|
| `mamba` | Force-typed Python compiler (JIT via Cranelift) |
| `cclab-mamba-registry` | Mamba module auto-registration |
| `cclab-jet` | Frontend toolchain (bundler, dev server, HMR, CSS pipeline) |
| `cclab-kv` | Embedded key-value store (WAL-backed) |
| `cclab-wal` | Write-ahead log |
| `cclab-core` | Shared primitives, error handling |
| `cclab-cli` | Unified CLI binary |
| `cclab-cli-registry` | CLI module auto-registration (linkme) |

## Layer 2: Libraries

Data, storage, networking, scientific computing. Tools with correct answers.

| Component | Purpose |
|-----------|---------|
| `cclab-pg` | PostgreSQL async ORM + migrations |
| `cclab-mongo` | MongoDB driver |
| `cclab-fetch` | HTTP client (Rust-backed) |
| `cclab-log` | Structured logging |
| `cclab-schema` | Validation (Pydantic compat) |
| `cclab-crypto` | Cryptographic primitives |
| `cclab-hive` | Hive/data lake connector |
| `cclab-array` | N-dimensional arrays |
| `cclab-frame` | DataFrames |
| `cclab-sci` | Scientific computing |
| `cclab-learn` | Machine learning |
| `cclab-plot` | Visualization |
| `cclab-media` | Image/audio processing |
| `cclab-text` | NLP / text processing |
| `cclab-grid-*` | Spreadsheet engine (6 crates) |

## Layer 3: Framework

Application frameworks. Build servers, agents, pipelines. Opinionated but general-purpose.

| Component | Purpose |
|-----------|---------|
| `cclab-api` | HTTP server framework (Rust server + ASGI compat) |
| `cclab-queue` | Background job engine |
| `cclab-agent` | LLM agent framework (providers, tools, agentic loop) |
| `qc` (projects/) | Profiling + security issue finder — embed as a library or run as a capture tool; delegates test execution |
| `cclab-server` | MCP server |
| `cclab-cmd` | Command execution |
| `cclab-typer` | CLI builder |
| `cclab-tqdm` | Progress bars |

## Layer 4: Agkit (Agentic Development Kit)

Business domain — no single correct answer, design decisions live here.

| Component | Type | Purpose |
|-----------|------|---------|
| `projects/agentic-workflow/schemas` | JSON Schema | Issue, Spec, Change, Pipeline, Project definitions (absorbed from former `cclab-agkit`) |
| `projects/agentic-workflow/prompts_agkit` | prompts | SDD prompt library (absorbed from former `cclab-agkit`) |
| `@cclab/ui` | package | UI design system — Card, Badge, InlineEdit, FileBrowser |
| `@cclab/spec-viewer` | package | Spec rendering — Markdown, Mermaid, code blocks |
| `@cclab/pipeline` | package | Pipeline DAG visualization |
| `cclab-razer` | crate | Code analysis / transformation |

## Projects

Applications built on Layers 1-4. Orchestrators, not libraries.

| Project | Description |
|---------|-------------|
| `cclab-sdd` | Spec-Driven Development engine — CLI workflow, agent dispatch |

## Dependency Flow

```
Projects (cclab-sdd)
    ↓ consumes
Layer 4: Agkit (agkit + ui + spec-viewer + pipeline)
    ↓ uses
Layer 3: Framework (api + queue + agent)
    ↓ uses
Layer 2: Libraries (pg + fetch + log + array + frame)
    ↓ uses
Layer 1: Runtime (mamba + runtime + jet + kv)
```

## Binding Layers

Each Rust crate may have companion crates:
- `*-mamba` — Mamba JIT bindings
- `*-cli` — CLI subcommands
