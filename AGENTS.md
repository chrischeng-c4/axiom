---
project:
  name: axiom
  owner: chrischeng-c4
  url: https://github.com/chrischeng-c4/axiom
  ssh: git@github.com:chrischeng-c4/axiom.git
  default_branch: main
---

# AGENTS.md - Implementation Essentials

> **Start here.** Before changing anything, read the repo-root `README.md` (what
> each project is) and `CONTRIBUTING.md` (the authoritative, repo-wide contract
> for authoring files, services, and CLIs — file granularity, semantic paths,
> the service archetype, and the CLI convention). This file is the
> implementation quick-reference; when the two disagree on authoring,
> `CONTRIBUTING.md` wins.

## Ecosystem (4 layers)

```
Layer 1: Runtime    — mamba, jet, kv, core, cli
Layer 2: Libraries  — pg, fetch, log, schema, array, frame, sci, learn, plot, media, text, grid
Layer 3: Framework  — api, queue, agent, guard, meter, server
Layer 4: Agkit      — agkit (domain models + UI + prompts), @cclab/ui, spec-viewer, pipeline
Projects            — agentic-workflow
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

For Agentic Workflow itself (`agentic-workflow` / `aw`), do not turn
`aw health` or `aw standardize` into a full AW takeover gate. Self-health only
hard-gates capability contracts and EC claim closure. Managed/semantic/
traceability, TD lock, CB verify, cold rebuild, and workspace test gates are
advisory metrics for self-AW unless the README capability contract or EC
inventory makes them explicit production obligations.

Codex should translate Claude slash-command references such as `/aw:td` or
`/aw:wi` to the equivalent `aw ...` CLI command unless the user
explicitly asks for Claude-specific behavior.

### Workflow CLI

| CLI | Use it for |
|-----|------------|
| `aw run` | Root-driven workflow runner. Choose exactly one root with `--project <project>`, `--capability <project>:<capability-id>`, or `--wi <id>`; follow `invoke.command` and `agent_prompt` until `completion.workflow_complete=true` or `requires_hitl=true`. |
| `aw wi` | Work-item inventory and planning: `draft`, `list`, `show`, `create`, `update`, `close`, `find`, `epicize`, `atomize`, `prioritize`, `enrich`, `validate`, and `fill-section`. Planning commands write local artifacts under `/tmp/aw/{project}/...` and do not publish tracker changes. |
| `aw td` | Tech-design and code-artifact lifecycle. TD defines candidate implementation structure; capability and EC gates are the source of product truth. Primary verbs are `create`, `validate`, `merge`, `check`, `ast`, `migrate-mermaid`, and `claim`; code-artifact verbs inherited by TD are `gen`, `gen-source`, `fill`, `code-check`, and `code-claim`. |
| `aw standardize` | Existing-project takeover workflow and remediation guidance. Run `aw standardize --project <project>` first and follow its `next.command`; targeted layer commands are `audit`, `managed`, `semantic`, and `traceability`. Capability remediation routes through `aw capability`; readiness and regenerability metrics live in `aw health`. |
| `aw capability` | Product capability completion loop. Verbs are `report`, `next`, `draft`, `apply-draft`, `init`, `migrate`, `run`, `check`, and `sweep`. For multi-project README rollout, run `sweep --write-rollout --human --skip-issue-inventory` first, then use the rollout/draft/WI/action queue artifacts instead of freehand README edits. Treat `create_wi:issue_inventory_skipped` as tracker-sync work, not WI backlog. Use `migrate` only for YAML/legacy-to-canonical Markdown conversion, and use `check --verify` when capability proof should include configured test gates. README is the default `cap_path` and uses `## Brief`, `## Capabilities`, `### Capability Index`, field-style capability contracts, and work-root tables. YAML `## Capability:` sections and legacy capability tables are migration input only. |
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
- `lib-{name}` — persistent work-area branches for `libs/` internal libraries,
  such as `lib-compass` or `lib-raft-host`

One `project-{name}` (or `lib-{name}`) maps to one dedicated worktree and one
agent session. Do not delete or force-overwrite `main`, `project-*`, or
`lib-*` without explicit user confirmation. Prefer non-destructive convergence
for stale `project-*` / `lib-*` refs. `project-*` and `lib-*` branches are
deletion-protected on GitHub via the `protect-persistent-branches` repository
ruleset (force-push is intentionally left unprotected so rebase-based landing
still works).

WI never creates or switches git branches. TD lifecycle branches
(`td-<id>`) are short-lived and may be created only when launched
from `main`; off-main TD commands stay on the current branch. When the user
says "the mamba branch" or "the agentic-workflow branch" without a prefix, prefer
`project-<name>` if it exists.

## Work-Item Rules

Canonical verb: `aw wi`. Legacy work-item aliases are removed from the active
CLI surface.

- One issue-platform id is one workflow root; do not invent a second slug.
- Draft/planning intermediate state lives under `/tmp/aw/{project}/workitems`.
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
fall back to source code only when needed, then consider `aw td code-claim`.

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
to bypass it: `aw wi` -> `aw ec gen` -> `aw td create/gen/fill/code-check` -> `aw td merge`. The
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
structure is Markdown-first: `#` is the project root, `## Brief` is the
agent-readable project summary, `## Capabilities` owns the capability registry,
and `### Capability Index` is the compact scan surface. H3-Hn capability
headings use field-style contracts and work-root tables to map headings to
epic/subepic WI roots. Atomic `change` WIs usually come from `aw wi atomize`
rather than README rows. YAML `## Capability:` sections and legacy capability
tables are migration input only. Verified progress requires closed/non-deferred
work roots, passing declared verification gates or linked validation
inventories, and resolving WI/TD refs. Do not use the old capability shorthand.
Project-local `aw.toml` may declare `[capability.profile].traits`; agents must
let those traits derive required baseline capabilities before adding
domain-specific capability roots. Trait-derived baseline capabilities are a
mandatory minimum, not the complete capability set, and traits are not README
capabilities. `CapabilityType` classifies one capability's EC-dimension ceiling;
it is not the project archetype. `http2_api` means the project owes a public API
list baseline, not OpenAPI completeness. `kubernetes_native` derives a
Kubernetes-native deployment baseline. `primary_replicas` derives a primary /
replica topology baseline and should only be selected for projects that actually
support that topology.

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

## CLI Convention: every CLI ships `llm`, `upgrade`, `issue`

Every CLI surface (`mamba`, `jet`, `lumen`, `vat`, `aw`/`cclab`, and any new
tool) MUST expose three agent-facing subcommands; a CLI is not done until all
three appear in `--help`. Positionals name **subcommands** or a verb's one
primary object (id/query/prose, e.g. `issue view <n>`, `issue create [msg…]`);
structured parameters (topic/title/version/tag/state) are flags.

- `llm [--topic <t>] [--format md|json]` — offline self-documentation that
  teaches an agent to drive the tool (topic via the `--topic` flag, default
  `outline`). Logic is the shared `libs/cli-std` crate (`cli_std::llm::render`);
  each tool supplies its `&[cli_std::llm::Topic]` list (the in-code source of
  truth) + a `ToolInfo`.
- `upgrade [--version <tag>] [--check]` — self-update to the latest
  `<project>@*` GitHub release; the in-binary form of
  `projects/<project>/install.sh` (detect target → download tarball → verify
  sha256 → atomic replace).
- `issue search [query]` · `view <n>` · `create [--title <t>] [message...]` —
  read **and** write the tool's issues via `cli_std::issue::{search,view,create}`.
  `search`/`view` are read-only (tokenless on public repos), filtered to
  `project:<name>`; `create` files a structured issue (diagnostics + the
  `project:<name>` label). Named `issue` (not `report`), leaving domain `report`
  verbs (`jet report` = HTML test reports) untouched.

Full spec: **`CONTRIBUTING.md` → "CLI convention: every CLI ships `llm`,
`upgrade`, `issue`"**.

## Service CLI Convention: `dockerfile` and layered `k8s`

K8s-native service CLIs also expose deployment artifact commands:

- `<cli> dockerfile render --variant source|release [--version <tag>] [--out <path-or-dir>]`
  renders image artifacts independently of Kubernetes because the same image is
  used by compose, kind, and registries.
- `<cli> k8s crd render [--out <path>]` renders the cluster-scoped API layer.
- `<cli> k8s operator render [--namespace <ns>] [--out <path-or-dir>]` renders
  the control-plane namespace/RBAC/deployment layer; `<cli> k8s operator run`
  is the controller process/container entrypoint.
- `<cli> k8s instance render --profile dev|staging|prod|template [--out <path-or-dir>]`
  renders the app-namespace custom resource consumed by the operator.

Do not put Dockerfile generation under `k8s`, and do not collapse the CRD,
operator, and instance layers into one command.

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
