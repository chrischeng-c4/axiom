---
project: lumen
branch: project-lumen
label: "project:lumen"
repo: chrischeng-c4/axiom
pick_order: priority
build: skip
verify:
  test: cargo test -p lumen
  perf: conditional
done_gates:
  - test_passes
  - perf_verified
pr:
  base: main
  merge_strategy: squash
  rebase_after_merge: true
---

# Lumen issue loop — per-issue rules

Lumen is a log-replicated **search index** (not a document store, not an
analytics engine). Strategic frame (2026-06-02): position lumen as the search
layer that fixes what OLTP stores (Postgres / AlloyDB / MongoDB) are bad at —
real BM25, filter-correct vector kNN, hybrid retrieval, CJK — while the OLTP
store stays the source of truth. CDC/ingestion is the caller's own pub/sub
(lumen bundles no connector). Output is `external_id` + score for an agent to
hydrate against the caller's store. Do **not** add ecosystem weight (Kibana-like
UIs, ingestion pipelines, bundled connectors).

## Branch policy
- All work happens on `project-lumen`. If the current branch is anything else,
  merge it into `project-lumen` first — don't escape to a feature branch;
  `project-lumen` IS the working branch.
- One issue → one PR → squash-merge to `main` → `git pull --rebase origin main`
  back onto `project-lumen`. Push.

## Build policy: skip
- **Do not run `cargo build` per issue.** `cargo test -p lumen` already compiles
  the crate + test binaries; a separate build is wasted time.
- Only build if a test failure smells like a stale artifact AND
  `cargo clean -p lumen && cargo test -p lumen` doesn't reproduce.
- Use the rustup toolchain, not Homebrew rustc (see root CLAUDE.md).

## Definition of done
1. **Test gate** — `cargo test -p lumen` passes from a clean state. Integration
   tests that need a live service (NATS / Redis) **skip gracefully** when it's
   absent (`let Some(x) = connect().await.ok() else { return };`); that is not a
   failure. Feature-gated tests: NATS behind `--features nats`, Redis behind
   `--features redis`, Ion behind `--features ion`. If an issue touches one of
   those paths, run that feature's tests too (and start the service via
   `brew services start nats-server` / `redis` if available).
2. **Perf gate (conditional)** — required only when the issue touches a search
   hot path (query planner / boolean eval in `src/storage.rs`, `src/vector_index.rs`,
   tokenizer, postings). Run `cargo test -p lumen --test perf_gate` and, where
   relevant, `scripts/bench_vs_db.py`; confirm no regression vs `project-lumen`
   HEAD before the change. For docs / examples / k8s-manifest / spec-only
   changes, write "perf N/A because …" in the PR body — never silently skip.
   - **Correctness-of-ranking changes** (filtered-HNSW recall, RRF fusion) must
     ship a conformance/recall test that demonstrates the new behaviour, not
     just "no regression."

## SDD note
- Root CLAUDE.md routes implementation through `aw wi → aw td → aw cb`. For these
  bounded roadmap issues, inline implementation on `project-lumen` is acceptable;
  if an issue genuinely needs a tech-design (new wire shape, e.g. the `rrf`
  QueryNode), run `aw td`/`aw cb` **on mainthread only** (the PostToolUse hook
  lockfile chain doesn't reach subagent Bash).

## Known quirks
- `aw wi close <github-number> --push` returns NOT_FOUND for remote-only issues
  (close looks up a local store). For backlog reconcile of GitHub-only issues use
  `gh issue close` (singular) directly — it is the configured backend.
- The LSM disk backend (`storage_lsm`) is behind the `experimental` feature and
  unwired / out-of-scope. Don't pull work into it unless an issue explicitly
  promotes the tier.
- openraft was retired (commit 0473a5bb); NATS JetStream write-log is the
  replication substrate. There is no write leader — writes publish to the log and
  any replica accepts them. Reject any "Raft leader / consensus" framing.

## PR body must include
- `Closes #<n>`.
- Test gate output (the passing `cargo test -p lumen` summary line).
- Perf gate output OR an explicit "perf N/A because …" line.

## Loop exit
- Stop when `gh issue list --label project:lumen --state open` returns 0.
- If an issue is blocked (depends on another, needs a human decision, fails a
  gate that isn't a quick fix), comment explaining the block, label it, and move
  to the next. Don't loop on a stuck issue.
