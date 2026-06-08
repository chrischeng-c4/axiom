# cgdb — Session Handoff (v0 kickoff)

## 1. Problem & Current State

Build **cgdb** — an edge graph database in Rust, macOS first, designed to
assist agentic development (Claude Code / Codex / Cursor-style agents).
Surfaces: **CLI + MCP server**.

- Status: 0% — empty crate dir on branch `project-cgdb` (off `main`).
- Not blocking; greenfield.
- CLAUDE.md rule: one worktree ↔ one Claude session. This worktree
  (`/Users/chris.cheng/cclab/project-cgdb`) is the dedicated home of cgdb.
- Already done in the kickoff session (on main worktree): registered
  the project in `.score/config.toml` (`[[projects]] name = "cgdb"`,
  path `crates/cgdb`). Hook now permits edits under `crates/cgdb/`.

## 2. Findings (design decisions from kickoff chat)

**Positioning** — *not* a general-purpose graph DB. The workload is
*code / spec / conversation as a semantic graph*. Real constraints:

1. Incremental indexing (file change → ms-level partial reindex).
2. Hybrid retrieval (graph traversal + vector similarity + keyword
   composable in one query).
3. Agent-friendly API (returns structured context, not raw rows).

**Tech stack picks**

| Layer        | Choice                                       | Why |
|--------------|----------------------------------------------|-----|
| Storage      | Single-file mmap + WAL (SQLite-style)         | Edge = single-user single-host; no server. |
| Graph engine | Hand-written CSR + adjacency (Rust)           | Neo4j/Kuzu too heavy for this scope. |
| Vector       | HNSW (hand-roll or `hnswlib-rs`)              | Million-node scale, no faiss dep. |
| GPU          | **Metal Performance Shaders** via `metal-rs`  | macOS-first differentiator; Apple Silicon unified memory fits graph workload. |
| CPU parallel | `rayon` + SIMD (NEON on ARM)                  | Traversal + vector distance are embarrassingly parallel. |
| Bindings     | `PyO3` + `napi-rs` (later)                    | Agent runtimes are Py/Node. |

**Three key design decisions to commit to**

1. **Schema** — typed property graph (not RDF, not freeform). Agent
   needs predictable structure for prompting.
2. **Vector index granularity** — one HNSW per node type (not one
   global). Agent queries usually know type scope ("similar function"
   vs "similar spec").
3. **GPU scope** — *not* every query (kernel-launch overhead beats
   small traversals). Only:
   - Batch vector search (>1000 queries)
   - Graph embedding (node2vec / GraphSAGE)
   - Bulk reindex (initial build)
   Normal queries stay on CPU + SIMD.

**Differentiator** — **temporal graph**: every node/edge carries
`commit_hash + timestamp`. Enables agent queries like "who called this
fn 3 days ago?" / "which commit first introduced this dependency?".
Neither Neo4j nor Kuzu does this well, and it's the biggest leverage
point for agentic codebase understanding.

## 3. What Was Done

- `git worktree add -b project-cgdb /Users/chris.cheng/cclab/project-cgdb main`
- Added `[[projects]] name = "cgdb"` entry to `.score/config.toml`
  on the `project-cgdb` branch (path `crates/cgdb`, label
  `project:cgdb`, workspace target `rust`, test_cmd `cargo test -p cgdb`).
- Created empty `crates/cgdb/` directory.
- This handoff doc.

Nothing committed yet on `project-cgdb` — both the config.toml change
and this file are untracked / unstaged.

## 4. Next Steps (v0 scope — target: runnable today)

Resume in a **new Claude Code session** rooted at
`/Users/chris.cheng/cclab/project-cgdb`.

**v0 scope (deliberately tiny):**

1. First commit: stage the `.score/config.toml` registration + this
   HANDOFF.md as the seed commit on `project-cgdb`.
2. `cargo init --lib crates/cgdb --name cgdb` — single crate, two
   binaries (`cgdb` CLI, `cgdb-mcp` server). Update workspace
   `Cargo.toml` to include `crates/cgdb`.
3. Data model:
   ```rust
   struct Node { id: String, ty: String, props: serde_json::Value }
   struct Edge { from: String, to: String, ty: String, props: serde_json::Value }
   ```
4. Storage: in-memory `HashMap`s + JSONL append-only log
   (NOT mmap/WAL yet — keep it dead simple).
5. CLI commands: `add-node`, `add-edge`, `get`, `neighbors`, `dump`.
6. MCP server: same 5 ops exposed as MCP tools. Crate to check first:
   `rmcp` (official Rust SDK) — verify it's published and recent
   before committing.
7. Smoke test: insert 100 nodes + 200 edges, query neighbors, restart
   the process, verify reload from JSONL.

**Explicitly NOT in v0** — defer until core works:
- mmap / WAL.
- HNSW / vector index.
- Metal / GPU.
- temporal columns (but see Note below on forward-compat).
- typed schema (props are just `serde_json::Value`).
- PyO3 / napi-rs bindings.

## 5. Success Criteria

v0 is done when:

- `cargo run --bin cgdb -- add-node --type fn --id foo --props '{"file":"a.rs"}'`
  works.
- `cargo run --bin cgdb -- neighbors foo` returns adjacent node IDs.
- Process restart preserves data (JSONL replay).
- `cgdb-mcp` server starts and Claude Code (or `mcp inspector`) can
  call the 5 tools.
- One end-to-end demo script under `crates/cgdb/examples/` showing
  agent-ish usage (insert code-graph, query neighbors).

## 6. Notes

- **Workspace integration**: the new crate should join the cclab
  `Cargo.toml` workspace `members = [...]` list when added. Don't
  forget; otherwise `cargo test -p cgdb` (per config.toml test_cmd)
  won't resolve.
- **MCP crate choice** is open. `rmcp` is the safe default; if it's
  not mature enough, second pick is hand-rolling MCP over stdio with
  `tokio` + `serde_json` (the protocol is JSON-RPC 2.0; small).
- **Don't optimise prematurely** — the whole point of v0 is to get
  the API shape right. Storage rewrite (mmap+WAL) and vector layer
  are separate later projects with their own TDs.
- **Differentiator (temporal) is deferred but don't design v0 in a
  way that blocks it**. Prefix every JSONL line with a version tag
  (`{"v":1,...}`) so future schema migrations have a hook. Adding
  `commit_hash` / `ts` later then becomes a v2 line variant, not a
  breaking change.
- **SDD pipeline (`score wi` / `score td`) is not yet wired for
  this project** — for v0 you can skip the full CRRR ceremony. Once
  v0 lands, the next change should go through proper SDD
  (`score wi create --type enhancement --project cgdb ...`).
- **Hook gotcha encountered**: `.score/` hooks reject edits on a
  `project-*` branch until `[[projects]]` is registered. The
  bootstrap is chicken-and-egg if you try to register from inside
  the new worktree. Solution used: edit `.score/config.toml` on the
  `main` worktree, capture as patch (`git diff > /tmp/...patch`),
  revert main, apply to `project-cgdb` worktree. Remember this for
  future `project-*` kickoffs.
