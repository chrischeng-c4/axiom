# Knowledge Base

Welcome to the **cclab** Knowledge Base.

This directory serves as the **Single Source of Truth** for the system architecture, design principles, and implementation details.

## Navigation

### Core Crates
- **[cclab-core](./cclab-core/README.md)**: Project overview, roadmap, and core principles.
- **[cclab](./cclab/README.md)**: Shared internals, common Mamba binding patterns and bridge logic.
- **[cclab-server](./cclab-server/README.md)**: Core API server implementation.
- **[cclab-probe](./cclab-probe/README.md)**: Parallel test execution engine.

### Storage Crates
- **[cclab-titan](./cclab-titan/README.md)**: High-performance MongoDB ORM.
- **[cclab-orbit](./cclab-orbit/README.md)**: High-performance PostgreSQL ORM.
- **[cclab-quasar](./cclab-quasar/README.md)**: Cloud Native Simple KV Store.
- **[cclab-ctx-inf-db](./crates/cclab-ctx-inf-db/README.md)**: Temporal knowledge graph DB with GPU-accelerated inference.

### Client Crates
- **[cclab-ion](./cclab-ion/README.md)**: Async HTTP client solution.

### Agent Framework
- **[cclab-agent](./cclab-agent/README.md)**: LLM agent framework (CodingAgent, AnalystAgent, multi-provider LLM, tool system).

### Analysis & Tools
- **[cclab-lens](./cclab-lens/README.md)**: Code intelligence engine (linting, semantic search, refactoring).

### SDD (Spec-Driven Development)
- **[cclab-sdd](./cclab-sdd/README.md)**: Unified SDD orchestrator (workflow + code generation).
  - [run-change/](./cclab-sdd/run-change/): Top-level orchestration (OpenRPC, phase routing, DAG loop)
  - [generate/](./cclab-sdd/generate/): Diagram and code generation specs
- **[cclab-cli](./cclab-cli/)**: CLI documentation and guides.

### Projects
- **[cue](./projects/cue/README.md)**: Prompt-to-Governed-App product architecture, Jet frontend target, Mamba backend target, and governance lifecycle.

## Quick Links

- [Crate Map](./crate-map.md) - All 35 crates overview
- [Roadmap](./cclab-core/01-roadmap.md)
- [Architecture Principles](./cclab-core/02-architecture-principles.md)
- [PostgreSQL OpenTelemetry Guide](./cclab-orbit/operations/OPENTELEMETRY.md) - Distributed tracing
