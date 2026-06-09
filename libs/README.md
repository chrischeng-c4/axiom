# libs

Internal libraries with **no external interface** — pure code linkage consumed
by other crates/projects. Nothing here ships a user-facing CLI, server, or
release pipeline; deliverables live in `../projects`.

## Conventions

- **Polyglot.** Unlike `../crates` (Rust/Cargo-specific by name), `libs/` is
  language-neutral. A library here may be Rust, TypeScript, Python, etc. The
  language is self-described by each library's own manifest — `Cargo.toml`,
  `package.json`, `pyproject.toml`, …
- **Flat layout.** One directory per library: `libs/<name>`. No per-language
  subdirectories; `ls libs/` is the table of contents.
- **Workspace wiring.** Rust libraries are Cargo workspace members — add their
  path to the root `Cargo.toml` `members` list. Non-Rust libraries are not
  Cargo members and carry their own build config.

## Boundary

A directory belongs in `libs/` (not `projects/`) when it has **no external
interface**: no `[[bin]]`/CLI entrypoint as its purpose, no served surface, no
release/install pipeline. If it grows one of those, it graduates to
`../projects`.

## Inventory

| Library | Language | Manifest | Notes |
|---|---|---|---|
| `compass` | Rust | `Cargo.toml` (`cclab-compass`) | Code-intelligence engine — parse, type inference, mutable AST, refactor, watch. Consumed by `agentic-workflow`. |
