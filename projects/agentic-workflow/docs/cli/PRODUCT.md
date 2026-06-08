# Score — Product Definition

## One-liner

Local-first SDD toolkit for Claude Code users. **Write the score; let the conductor play.**

## What is Score

Score is a Rust CLI that implements Spec-Driven Development (SDD) locally, backed by Claude Code. It consumes the same `cclab-*` arsenal as Conductor (the cloud SDD platform) but delivers it as a single-binary tool that works offline, stores artifacts in the filesystem, and integrates directly with Claude Code via MCP.

## Who uses it

| User | Why Score over Conductor |
|------|--------------------------|
| Solo developers | No server to run, no DB to provision |
| OSS maintainers | Git-native artifacts, versioned alongside code |
| Teams on private / offline repos | Cloud SDD tools not allowed |
| Power users | CLI + editor preferred over web UI |

## Value proposition

- **No server, no setup**: single Rust binary, works offline
- **Git-native**: all artifacts live under `cclab/` in the repo, versioned like code
- **Claude Code first**: deep MCP integration, not a generic LLM wrapper
- **Same SDD concept as Conductor**: artifacts + workflows + state machine, same arsenal crates, different delivery target

## Relationship to Conductor

Conductor is the cloud / multi-user SDD platform (web UI, PostgreSQL, K8s Jobs). Score is the local / single-user CLI (filesystem, Rust binary, Claude Code MCP). Both consume the same arsenal:

| Capability | Arsenal crate | Score uses | Conductor uses |
|---|---|---|---|
| HTTP | `httpkit` | (no HTTP — CLI only) | `mambalibs.http` |
| Storage | `agent::ArtifactStore` | filesystem impl (in this project) | PG impl via Mamba bindings |
| Agents | `agent::agents` | direct Rust calls | `mambalibs.agent` |
| Workflow | `cclab-queue::workflow` | direct Rust (Chain / Group / Chord) | `mambalibs.queue` |
| Models | `projects/agentic-workflow/schemas/` + codegen | generated Rust structs | generated Rust + TS/Mamba surfaces |
| Prompts | `projects/agentic-workflow/prompts_agkit/` | runtime loader | shared with Score |
| Frontend | — | — | `packages/@cclab/*` |

Users can start with Score on day 1, then adopt Conductor when their team grows.

## Non-goals

- Multi-user collaboration — use Conductor
- Auth / RBAC — local single-user only
- Cloud deployment — local only
- Web UI — CLI + editor integration
- Non-SDD workflows — Score is SDD-specific; general-purpose agent frameworks live in `cclab-agent`

## Tech stack

- **Language**: Rust 2021
- **Async runtime**: Tokio / Mamba runtime bindings
- **CLI framework**: `clap` + `cclab-cli-registry` for distributed subcommand registration
- **Claude Code integration**: MCP server (stdio transport)
- **Storage**: filesystem under `{project_root}/cclab/artifacts/{kind}/`
- **Arsenal composition**: see table above

## State

Pre-release. Score is now a standalone project backed by the `sdd` arsenal crate (Epic #1157 complete).
