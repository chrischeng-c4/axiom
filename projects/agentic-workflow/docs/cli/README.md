# Agentic Workflow CLI

Local-first SDD and work-item toolkit for agent workflows.

## What is this

Agentic Workflow is the local CLI companion to [Conductor](../conductor/), the cloud SDD platform. Both implement the same Spec-Driven Development (SDD) concept using the same `cclab-*` arsenal, differing only in delivery:

- **Agentic Workflow** (this project): Rust library plus `aw` binary, filesystem storage, single-user local
- **Conductor**: Web UI, PostgreSQL, K8s Jobs, multi-user cloud

See `PRODUCT.md` for product definition and `CLAUDE.md` for agent guidance.

## Status

Pre-release. Score and SDD have been merged into `projects/agentic-workflow`.

## Structure

```
projects/agentic-workflow/
├── Cargo.toml       ← package, library, and `aw` binary definition
├── src/bin/aw.rs    ← CLI entry point
├── src/cli/         ← CLI command modules
└── src/             ← shared workflow and SDD library surface
```

## Building

```bash
cargo build -p agentic-workflow --bin aw
```

`aw` is a standalone binary (not a subcommand of `cclab`). Install via the repo-root `install.sh` (which installs both `cclab` and `aw`) or directly with:

```bash
cargo install --path projects/agentic-workflow --bin aw
```
