# Score — Agent Guidance

## Project identity

Score is a **project** (show case), not a crate. It lives under `projects/agentic-workflow/` and composes arsenal crates from `crates/cclab-*` + schemas from `packages/cclab-agkit/`.

See `PRODUCT.md` for product definition and relationship to Conductor.

## Repository role

```
Arsenal                              Show case (this project)
─────────                            ─────────
crates/cclab-*         ──────────→   projects/agentic-workflow/      ← CLI binary
packages/cclab-agkit   ──────────→   projects/agentic-workflow/          ← schema consumer
                                     projects/agentic-workflow/mcp/      ← MCP server (when extracted)
```

Score should contain **only**:

1. CLI subcommand implementations (thin handlers calling arsenal APIs)
2. MCP server wiring (stdio transport, tool registration)
3. Filesystem storage impl of `cclab-agent::ArtifactStore` trait
4. Score-specific UX glue (progress bars, terminal output, interactive prompts)
5. `PRODUCT.md`, `CLAUDE.md`, `README.md`

Score should **not** contain:

- Business logic (belongs in `cclab-agent`, `sdd`, etc.)
- Domain models (belongs in `cclab-agkit/schemas/` + codegen to Rust)
- LLM provider code (belongs in `cclab-agent::llm`)
- Workflow orchestration primitives (belongs in `cclab-queue::workflow`)
- Prompt templates (belongs in `cclab-agkit/prompts/`)

If something in Score looks reusable by Conductor or other projects, move it up to the arsenal first.

## Surfaces

Agentic Workflow has two complementary surfaces, each with a distinct audience:

- **Skills (`/aw:*`)** — face-to-user. Multi-step orchestration; user invokes via slash command. Lives under `projects/agentic-workflow/templates/mainthread/skills/score-*/SKILL.md`.
- **CLI verbs (`aw <verb>`)** — face-to-agent. Single-question machine surface; emits JSON envelopes (`dispatch / done / error / batch`); agents invoke as one bounded step. Lives in `projects/agentic-workflow/src/cli/`.

Design rule: when adding a new capability, ask "is this one bounded question (CLI) or a multi-step lifecycle (skill that wraps several CLI calls)?" Skills wrap CLI; CLI never wraps skills.

## Development commands

```bash
# Build the aw binary
cargo build -p agentic-workflow --bin aw

# Run the CLI locally
cargo run -p agentic-workflow --bin aw -- <subcommand>

# Install aw to ~/.cargo/bin
cargo install --path projects/agentic-workflow --bin aw
# Or use the repo-root installer which installs both cclab and aw:
./install.sh

# Run tests
cargo test -p agentic-workflow
```

## Agentic Workflow

Agentic Workflow is itself a project that follows SDD. Changes go through:

1. Update `specs/` when applicable.
2. Update code in `projects/agentic-workflow/**`
3. Add tests
4. Use `aw td` / `aw cb` via Agentic Workflow itself or open a PR manually

## Standardization workflow

Agentic Workflow has two distinct workflows. Don't conflate them — they share
primitives but answer different questions.

| Workflow | Question | Loop shape | Termination |
|---|---|---|---|
| **正流程 (forward CRRR)** | "Land this one change." | issue → td → cb → merge, single-issue | Issue closes |
| **標準化 (regenerability)** | "Make the whole repo deletable + regeneratable." | tick → audit → 1 bounded action → tick | coverage = 100% |

The mission invariant for 標準化 lives in `projects/agentic-workflow/CLAUDE.md` and the
full contract in
`projects/agentic-workflow/tech-design/surface/specs/score-standardization.md`.

### CLI mapping for standardization actions

The driver picks one action per tick in this priority order:

| # | Action | CLI verb | Status |
|---|---|---|---|
| 0 | inventory | `aw standardize managed report` / `aw standardize managed next` | implemented |
| 1 | regen_drift | `aw cb gen <slug>` (driven by `cb check` drift report) | partial — driver missing |
| 2 | promote_handwrite | mainthread rewrite of HANDWRITE → CODEGEN | **missing** |
| 3 | issue_marker_gap | `aw wi create` (gap-blocker) + edit marker | manual |
| 4 | fix_spec_rule | `aw td check` → `aw td revise` (CRRR) | partial |
| 5 | fold_shadow | mainthread wraps shadow region in CODEGEN/HANDWRITE | **missing** |
| 6 | claim_code | `aw cb claim <path>` | implemented as HANDWRITE claim; CODEGEN promotion is follow-up |

The umbrella driver is `aw standardize managed next` — emits one
StandardizeAction envelope per tick. Cron / `/aw:standardize-cron`
runs the action and ticks again. After managed coverage is complete,
`aw standardize regenerable next <project>` reports the second layer:
remaining HANDWRITE blockers before full CODEGEN ownership.

### Loop pattern (all CLI verbs must support)

1. **Per-tick statelessness** — every tick rediscovers state from the
   repo (issue + TD frontmatter, CODEGEN/HANDWRITE markers, git log
   trailers, file scan). No in-memory loop state.
2. **One bounded action per tick** — `--next` returns at most one action.
3. **Idempotency** — re-running an action on already-fixed state is a
   no-op (or refresh). Never destructive.
4. **Resumability** — crash / sleep / session-restart between ticks
   leaves the repo coherent; next tick continues.

This mirrors the `aw wi idle` envelope contract, scaled to
cross-namespace + cross-time. State lives in the repo, not the loop.

## Cross-project conventions

- Rust: follow workspace-level `clippy.toml` and `rustfmt.toml`
- Never duplicate code that already exists in an arsenal crate — import it
- Storage format for artifacts: YAML frontmatter + markdown body (decided in #1169)
- CLI binary is `aw <action>` — standalone binary, not a subcommand of `cclab`. Entry point at `projects/agentic-workflow/src/bin/aw.rs`, library API at `projects/agentic-workflow/src/cli/mod.rs`.
- MCP transport has been removed — Claude Code invokes tool logic via CLI directly through its Bash tool
- Skill templates under `projects/agentic-workflow/templates/cli/mainthread/skills/score-*/SKILL.md` define the `/aw:*` slash commands
