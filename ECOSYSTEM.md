# axiom Ecosystem

Rust-built developer infrastructure, split into four reusable layers plus
project-level tools and services. The root README is the project index;
CONTRIBUTING.md is the authoring and service-archetype contract.

## Layer 1: Runtime and Language

Core runtimes and command surfaces. These should stay deterministic and free of
business-domain policy.

| Component | Purpose |
|-----------|---------|
| `mamba` | Force-typed Python compiler: lex/parse/lower Python and emit native code through Cranelift JIT/AOT. |
| `jet` | Rust-native web toolchain: package management, dev server, production build, tests/e2e, WASM and multi-target execution. |
| `kv` | Embedded key-value/runtime storage primitive. |
| `core` | Shared runtime primitives, errors, and foundational utilities. |
| `cli` / tool binaries | Agent-facing CLIs with the standard `llm`, `upgrade`, and `issue` command set. |

## Layer 2: Libraries

Reusable libraries. Services compose these rather than reimplementing transport,
storage, auth, backup, or codegen patterns locally.

| Component | Purpose |
|-----------|---------|
| `pg` | PostgreSQL integration. |
| `fetch` | HTTP client utilities. |
| `log` | Structured logging and tracing conventions. |
| `schema` | Schema validation and typed contracts. |
| `array`, `frame`, `sci`, `learn`, `plot`, `media`, `text`, `grid` | Data, scientific, media, text, plotting, and spreadsheet primitives. |
| `raft-core` | Step-driven raft consensus core. |
| `raft-host` | Raft host: h2c peer transport, apply loop, snapshots, compaction, and k8s topology auto-mode. |
| `h2c` | HTTP/2 cleartext server/client transport. |
| `service-http` | Standard service shell: probes, readiness, metrics, tracing, graceful drain, and h2c composition. |
| `service-auth` | Shared bearer-token extraction/rejection/injection middleware and verifier trait. |
| `service-backup` | Snapshot/backup destination, sink, and runner contracts. |
| `operator` | Kubernetes operator/render toolkit: CRD/control-plane/instance artifacts, owner refs, labels, StatefulSet topology, Services, PDBs, and CronJobs. |
| `cli-std` | Shared CLI convention implementation for `llm`, `upgrade`, and `issue`. |
| `openapi-codegen` | Typed client generation from a service's own OpenAPI contract. |

## Layer 3: Framework

General-purpose frameworks and agent-facing infrastructure built on the shared
libraries.

| Component | Purpose |
|-----------|---------|
| `api` | HTTP API framework and service integration layer. |
| `queue` | Background job and dispatch abstractions. |
| `agent` | LLM agent framework: providers, tools, and agent loops. |
| `guard` | Security posture gate over static and runtime evidence. |
| `meter` | Runtime/resource measurement and profiling. |
| `server` | MCP/server process integration. |

## Layer 4: Agkit

Agentic development kit and product-facing agent artifacts.

| Component | Purpose |
|-----------|---------|
| `agkit` | Domain models, prompts, and agent workflows. |
| `@cclab/ui` | UI design system. |
| `spec-viewer` | Spec rendering for Markdown, Mermaid, and code blocks. |
| `pipeline` | Pipeline/DAG visualization. |
| `projects/agentic-workflow/schemas` | Agentic Workflow domain schemas. |

## Service Portfolio

Long-running network services follow the CONTRIBUTING service archetype:
HA-ready, HTTP/2 + OpenAPI, k8s-native, standard probes/metrics, standard CLI
commands, Dockerfile render, and layered `k8s crd/operator/instance` artifacts.

| Service | Owns | Does not own |
|---------|------|--------------|
| `lumen` | Exact, lexical, semantic, perceptual, and duplicate search in one derived index service. | OLAP aggregation, source-of-truth storage, vector-only GPU database ownership. |
| `keep` | KV/result storage, claim-check payloads, collections, durable values, and HA storage path. | Broker delivery, workflow orchestration, analytical scans. |
| `relay` | Online broker delivery, durable ordered log, broadcast fan-out, and work-queue leasing. | Workflow decisions, long-term replay/archive, delayed HTTP task dispatch. |
| `loom` | Workflow state, DAG scheduling, runner selection, timers, and fair dispatch. | Broker delivery, payload bytes, replay archive. |
| `tape` | Topic history, offset/time replay, consumer checkpoints, retention, and backfill. | Online broker delivery, workflow decisions. |
| `defer` | Delayed HTTP task lifecycle, retry/DLQ, rate limits, and dedupe keys. | Pub/sub fan-out, topic replay archive. |
| `cube` | Columnar facts, OLAP scan/filter/group-by/aggregate, rollups, and partitions. | Search ranking, vector ANN, KV payload storage. |
| `beam` | GPU vector indexes, vector ingest/rebuild, nearest-neighbor query. | Lexical/perceptual/duplicate search, OLAP aggregation. |

Reference service adopters today are `keep`, `relay`, `lumen`, and `loom`;
planned service placeholders are `tape`, `defer`, `cube`, and `beam`.

## Runtime and Agent Tools

These are project-level tools rather than service-portfolio data planes.

| Project | Purpose |
|---------|---------|
| `agentic-workflow` | Workflow protocol and `aw` CLI for capability-driven project takeover, WI planning, TD/CB lifecycle, and readiness rollup. |
| `vat` | Agent-native host-process runtime/dev-container harness with GPU access and JSON state for local service/test environments. |
| `cap` | Resource-protection wrapper for heavy local commands. |
| `meter` | Measurement/profiling CLI used by service EC gates. |
| `guard` | Security posture CLI and report surface. |
| `arena` | Benchmark result comparison across targets. |
| `rig` | Request/query/workload driver for runtime evidence. |

## Dependency Flow

```text
Projects and services
    -> Layer 4: Agkit artifacts where agent/domain workflows are needed
    -> Layer 3: Framework shells and agent infrastructure
    -> Layer 2: Shared service/library kit
    -> Layer 1: Runtime, language, and CLI foundations
```

## CLI Binding Convention

Each CLI surface exposes `llm`, `upgrade`, and `issue`. K8s-native service CLIs
also expose `dockerfile render` and layered `k8s crd/operator/instance`
commands. Add new subcommands through the relevant `*-cli` crate and linkme
registration path described in AGENTS.md and CONTRIBUTING.md.
