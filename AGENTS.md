---
project:
  name: axiom
  owner: chrischeng-c4
  url: https://github.com/chrischeng-c4/axiom
  ssh: git@github.com:chrischeng-c4/axiom.git
  default_branch: main
---

# AGENTS.md - Implementation Essentials

## Ecosystem (4 layers)

```
Layer 1: Runtime    — mamba, jet, kv, core, cli
Layer 2: Libraries  — pg, fetch, log, schema, array, frame, sci, learn, plot, media, text, grid
Layer 3: Framework  — api, queue, agent, guard, meter, server
Layer 4: Agkit      — agkit (domain models + UI + prompts), @cclab/ui, spec-viewer, pipeline
Projects            — agentic-workflow, cue
```

Full details: `ECOSYSTEM.md`. Domain model schemas: `projects/agentic-workflow/schemas/`.

## Codex Operational Rules

Use `rg` for search, use `apply_patch` for manual edits, respect the workspace
sandbox, and request escalation when GitHub/network-backed commands need it.

<!-- aw:start -->
## Agentic Workflow CLI Surface

Agentic Workflow is the workflow protocol. Agents should use the CLI verbs
below, run `aw <verb> --help` when an argument shape matters, and treat stdout
as the live prompt for the current binary and repository state. Prefer the
shortest agent-facing invocation; do not add compatibility/no-op flags such as
`--json` when stdout already is the protocol. If stdout contains a JSON
envelope, payload path, `invoke.command`, validation error, or next command,
follow it exactly. For `aw run` output, do not declare the workflow complete
unless `completion.workflow_complete=true`; `action=done` can mean only the
current child root is complete and the envelope is asking you to inspect the
parent.

Codex should translate Claude slash-command references such as `/aw:td` or
`/aw:wi` to the equivalent `aw ...` CLI command unless the user
explicitly asks for Claude-specific behavior.

### Workflow CLI

| CLI | Use it for |
|-----|------------|
| `aw run` | Root-driven workflow runner. Choose exactly one root with `--project <project>`, `--capability <project>:<capability-id>`, or `--wi <id>`; follow `invoke.command` and `agent_prompt` until `completion.workflow_complete=true` or `requires_hitl=true`. |
| `aw wi` | Work-item inventory, planning, and CRRR: `draft`, `list`, `show`, `create`, `update`, `close`, `find`, `epicize`, `atomize`, `prioritize`, `enrich`, `validate`, `fill-section`, `review`, `arbitrate`. Planning commands write local artifacts under `/tmp/aw/{project}/...` and do not publish tracker changes. |
| `aw td` | Tech-design lifecycle and checks: `create`, `validate`, `review`, `revise`, `merge`, `arbitrate`, plus read-only/utility verbs `check`, `ast`, `migrate-mermaid`, `claim`. |
| `aw cb` | Code-artifact lifecycle: `gen`, `check`, `claim`, `fill`, `review`, `revise`, `arbitrate`. |
| `aw standardize` | Existing-project takeover workflow and remediation guidance. Run `aw standardize <project>` first and follow its `next.command`; targeted layer commands are `audit`, `managed`, `semantic`, and `traceability`. Capability remediation routes through `aw capability`; readiness and regenerability metrics live in `aw health`. |
| `aw capability` | Product capability completion loop. Verbs are `report`, `next`, `run`, and `check`; use `check --verify` when capability proof should include configured test gates. README is the default `cap_path` and uses Markdown H1-Hn capability headings plus contract/work-root tables. YAML `## Capability:` sections and legacy capability tables are migration input only. |
| `aw health` | Aggregate project readiness metrics: capability readiness, managed/semantic/traceability coverage, command traceability, regenerable maturity, cb verify, cold verify, configured test gates, and HITL status. Run `aw health --project <project>` for production readiness; add focused sections such as `regenerable`, `gates`, or `blockers` only when detail is needed. Add `-v/--verbose` only when progress events are useful. Add targeted `--verify-*` flags only when intentionally debugging a subset. |

### Support CLI

| CLI | Use it for |
|-----|------------|
| `aw init` | Bootstrap or refresh `.aw/` config, skills, and settings. |
| `aw chat post/list/read/members/listen` | Cross-checkout coordination through the shared Agentic Workflow chat channel. |
| `aw td check` | Check TD/spec files for structure, section-format rules, and logical consistency. |

When the user asks for `aw wi`, `sdd issues`, `sdd gh issue`, or similar
wording after the merge, inspect Agentic Workflow-managed GitHub issues for the
merged project:

```bash
aw wi list --project agentic-workflow
aw wi show <number>
```

Do not run the literal command `gh issues`; GitHub CLI uses singular
`gh issue`, and Agentic Workflow work-item state is routed through the
configured backend.

## Project and Branch Allocation

Agentic Workflow owns the project/worktree allocation strategy. Primary working-area
branches are:

- `main`
- `project-{name}` — persistent work-area branches such as `project-mamba` or
  `project-agentic-workflow`

One `project-{name}` maps to one dedicated worktree and one agent session. Do
not delete or force-overwrite `main` or `project-*` without explicit user
confirmation. Prefer non-destructive convergence for stale `project-*` refs.

WI never creates or switches git branches. TD/CB lifecycle branches
(`td-<id>` / `cb-<id>`) are short-lived and may be created only when launched
from `main`; off-main TD/CB commands stay on the current branch. When the user
says "the mamba branch" or "the agentic-workflow branch" without a prefix, prefer
`project-<name>` if it exists.

## Work-Item Rules

Canonical verb: `aw wi`. Legacy work-item aliases are removed from the active
CLI surface.

- One issue-platform id is one workflow root; do not invent a second slug.
- Draft/CRRR intermediate state lives under `/tmp/aw/{project}/workitems`.
- Published state is projected to the issue platform configured in
  `.aw/config.toml`.
- `.aw/issues/{open,closed}` is retired from the AW ecosystem. Do not create,
  read, or commit issue lifecycle/cache files there; ephemeral issue working
  copies live under `/tmp/aw`.
- Backend selection comes from `.aw/config.toml`; do not add ad-hoc backend
  flags to `aw wi`.
- `--label` is not the public create path. Labels are derived from typed flags:
  `--type`, `--project`, `--priority`, and `--agent`.
- Non-epic work-items must be bounded before TD: include `## Capability
  Alignment`, `## Scope`, `## Acceptance Criteria`, and `## Reference
  Context`. Roadmap-sized or decision-blocked work must go through `aw wi
  atomize` or HITL review before `aw td`.

## SDD and Codegen Rules

Specs are the source of truth. Consult `projects/agentic-workflow/tech-design/` first;
fall back to source code only when needed, then consider `aw cb claim`.

New TD test taxonomy is artifact-oriented: use `unit-test` for generated unit
test design and `e2e-test` for product journey / side-effect verification.
Legacy `test-plan` and `tests` sections may parse with warnings, but new TDs,
templates, and skills should not create them. Product explanation belongs in
README capabilities or external docs; TD sections should exist only when they
drive codegen, handwrite, or verification artifacts.

`aw run` output may include `artifact_quality_profile` and the stdout prompt may
include an `Artifact Quality Gate`. Treat that gate as part of the lifecycle
contract, not optional advice. Frontend/UI artifacts require machine-verifiable
desktop and mobile viewport evidence, interaction smoke proof,
accessibility/readability smoke proof, and placeholder-free primary-state
evidence before production readiness can be claimed.

Every implementation change goes through Agentic Workflow unless the user explicitly asks
to bypass it: `aw wi` -> `aw td` -> `aw cb` -> `aw td merge`. The
CLI owns the concrete phase queue, prompt text, validation gates, commits, git
trailers, and next command.

Existing-project takeover uses `aw standardize` for bounded workflow guidance
and `aw health` for the project-readiness metric surface.

Standardize readiness layers and maturity signal:

- `capability`: README capability roots are Markdown-table runnable.
- `managed`: every in-scope file is marked `CODEGEN` or `HANDWRITE`.
- `semantic`: source behavior is covered by semantic TD and generator primitive
  gaps.
- `traceability`: active commands, TDs, source refs, and CB blocks close back to
  README capabilities.
- `regenerable`: automation maturity signal; convert as much `HANDWRITE`
  to `CODEGEN` as deterministic generator primitives allow. Regenerability
  gaps block production only when the project is generator-authoritative
  (for example Agentic Workflow itself) or when a capability explicitly
  requires full regenerability. External/advisory projects keep remaining
  generator gaps in `optional_regenerability_gaps`.

Project health gates/metrics:

- `capability`: capability readiness, release-scope roots, and production blockers.
- `managed`, `semantic`, `traceability`, and `command_traceability` coverage.
- `cb/cold verify`: deterministic generation and cold rebuild gates are clean.
- Configured test gates, workflow locks, HITL state, and artifact quality gates.
- `regenerable`: maturity signal, not a required 100% production gate by default.

There is no skip state for source ownership. If codegen cannot generate a
region yet, mark it as `HANDWRITE`, name the concrete generator gap/tracker,
add `@spec` annotations where appropriate, and feed the gap back into
Agentic Workflow until it can become `CODEGEN`.

Product capability completion is separate from source ownership. `aw
capability` reads the project README or configured `cap_path`; README capability
structure is Markdown-first: `#` is the project root, H2-Hn headings define
capability/sub-capability roots, and contract/work-root tables map headings to
epic/subepic WI roots. Atomic `change` WIs usually come from `aw wi atomize`
rather than README rows. YAML `## Capability:` sections and legacy capability
tables are migration input only. Verified progress requires closed/non-deferred
work roots, passing declared verification gates or linked validation
inventories, and resolving WI/TD refs. Do not use the old capability shorthand.

Fix Agentic Workflow first when the pipeline breaks; do not work around a
broken lifecycle.
<!-- aw:end -->

## CLI Auto-Registration

Each crate registers CLI subcommands via a separate `{crate-name}-cli` crate + `linkme` distributed slice.

To add a new subcommand:
1. Create `crates/{name}-cli/` implementing `CliModule` trait
2. Register with `#[distributed_slice(CLI_MODULES)]`
3. Add dependency in `cclab-cli/Cargo.toml` and force-link in `main.rs`: `use {name}_cli as _;`

Both steps in (3) are required — missing either will silently fail to register.

## Constraints

Use rustup toolchain, not Homebrew rustc:

```bash
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" wasm-pack build crates/cclab-grid-wasm --target web --out-dir ../../pkg
```

## Authoring: right-sized files, semantic paths, explicit names

Optimize every artifact tree so an agent learns *what exists* and *where to act* from `ls`, paths, and filenames alone — without opening files. Fewer reads = fewer tool calls = faster, cheaper, more reliable agents.

- **Right-sized files** — one coherent concern per file (a single reason to open it). Grain follows access pattern + cohesion, NOT minimizing size: split when readers/reviews/runs are independent; keep together when the parts are one concept or share setup.
- **Semantic paths** — the directory *is* the taxonomy; the path tells you a file's role before you open it.
- **Explicit names** — the leaf filename briefs the case, so `ls <dir>/` is a table of contents. A vague name is a defect — rename it.

Push granularity as fine as your tooling keeps consistent: a generator+linter (`fixture_gen`/`fixture_lint`) makes maximal one-case-per-file cheap; hand-authored trees lean toward cohesion. Strongest for naturally decomposable trees (fixtures/configs/generated/docs); applied with judgment to cohesive code (Rust `#[test]` fns stay in a `mod tests`).

Full principle: **`CONTRIBUTING.md`**. mamba CPython-test mechanics (PEP 723 `[tool.mamba]`, the six dimensions, `tools/fixture_gen.py` → fill → `tools/fixture_lint.py`, manifest): **`projects/mamba/tests/cpython/conventions/FIXTURE-LAYOUT.md`** — read before authoring/decomposing fixtures.

## Testing

- **Real services over mocks**: Use real Docker/Homebrew services for integration tests. Only mock SaaS (GCP Cloud Tasks, Cloud Scheduler, etc.).
- **Local services**: `brew services start redis` (Redis), `brew services start nats-server` (NATS). Tests skip gracefully if unavailable.
- **Skip pattern**: `let Some(x) = connect().await.ok() else { return };`
- **Feature gates**: Redis tests behind `#[cfg(feature = "redis")]`, NATS behind `#[cfg(feature = "nats")]`, Ion behind `#[cfg(feature = "ion")]`

## Debugging

- **Server log**: `~/.cclab/server.log` — MCP server stdout/stderr
- **Server status**: `cclab server list` — show running server PID, port, registered projects
- **Stack overflow**: Server crashes silently (no log) → check tokio worker thread stack size (#182)
