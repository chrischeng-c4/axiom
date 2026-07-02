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
follow it exactly. For runner output (`aw wi run` / `aw capability run`), do
not declare the workflow complete
unless `completion.workflow_complete=true`; `action=done` can mean only the
current child root is complete and the envelope is asking you to inspect the
parent.

Do not use removed top-level helpers such as `aw check`, `aw hover`,
`aw daemon`, `aw serve`, or `aw context`.

For Agentic Workflow itself (`agentic-workflow` / `aw`), do not run the full
AW loop against its own repo, and do not turn `aw health` or `aw standardize`
into a self-takeover gate: a broken lifecycle cannot be required to fix
itself (self-deadlock). Self-AW hard-gates only the capability contract —
CAPABILITIES.md work-roots with resolvable gap/claim ids and closing WI/TD
refs. EC claim verification becomes a hard gate only once an EC inventory is
actually configured for aw; until then it is advisory, like managed/semantic/
traceability, TD lock, CB verify, cold rebuild, and workspace test gates.
Changes to aw itself land as direct commits with `Refs #<issue>` trailers
plus capability work-root registration — the sanctioned self-hosting mode,
not a lifecycle bypass.

Codex should translate Claude slash-command references such as `/aw:td` or
`/aw:wi` to the equivalent `aw ...` CLI command unless the user
explicitly asks for Claude-specific behavior.

### Workflow CLI

| CLI | Use it for |
|-----|------------|
| `aw wi run <id>` / `aw capability run [<cap-id>] --project <p>` | Root-driven workflow runners on the delivery nouns: `aw wi run <id>` drives one WI to terminal; `aw capability run <cap-id>` drives one capability's work-root WIs; `aw capability run --project <p>` is the project-wide run-to-end driver. Follow `invoke.command` and `agent_prompt` until `completion.workflow_complete=true` or `requires_hitl=true`. (The old top-level runner verb is deprecated and slated for removal.) |
| `aw wi` | Work-item inventory, planning, and CRRR: `draft`, `list`, `show`, `create`, `update`, `close`, `find`, `epicize`, `atomize`, `prioritize`, `enrich`, `validate`, `fill-section`, `review`, `arbitrate`. Planning commands write local artifacts under `/tmp/aw/{project}/...` and do not publish tracker changes. There is no `estimate`/`sprintize`; use `aw capability run --project <name>` as the run-to-end driver instead of cron-style sprint batches. |
| `aw td` | Tech-design + generated-code lifecycle (LINEAR — no review/revise; the gate is EC via `code-check`): `create`, `validate`, `gen`, `fill`, plus read-only/utility verbs `check`, `ast`, `migrate-mermaid`, `lock`, `claim`, `gen-source`, `code-check`, `code-claim`. (Code-artifact verbs are folded in here; the former standalone code-artifact namespace and the merge verb were removed — `code-check` is the terminal step.) TD defines candidate implementation structure; capability and EC gates remain the source of product truth. |
| `aw standardize` | Existing-project takeover audit-first preservation protocol: `audit check` / `audit record` is the ONLY surviving surface. Readiness layer metrics (`capability`, `managed`, `semantic`, `traceability`, `regenerable`) live entirely in `aw health`, whose `next.command` names the worker verb for the top gap (`aw td promote`, `aw td code-claim`, `aw td gen`, `aw wi create`, ...). Capability remediation is `aw capability`; HANDWRITE→CODEGEN promotion is `aw td promote`. |
| `aw capability` | Product capability completion loop: `report`, `next`, `draft`, `apply-draft`, `init`, `migrate`, `run`, `check`, and `sweep`. For multi-project README rollout, run `sweep --write-rollout --human --skip-issue-inventory` first, then use the rollout/draft/WI/action queue artifacts instead of freehand README edits. Treat `create_wi:issue_inventory_skipped` as tracker-sync work, not WI backlog. Use `migrate` only for YAML/legacy-to-canonical Markdown conversion, and use `check --verify` when capability proof should include configured test gates. README is the default `cap_path` and uses `## Brief`, `## Capabilities`, `### Capability Index`, field-style capability contracts, and work-root tables. YAML `## Capability:` sections and legacy capability tables are migration input only. |
| `aw health` | Read-only aggregate project readiness metrics: capability readiness, managed/semantic/traceability coverage, command traceability, regenerable maturity, cb verify, cold verify, configured test gates, and HITL status. Run `aw health --project <project>` for the full picture, or pass a focused `[SECTION]` (e.g. `regenerable`, `gates`, `blockers`) plus `-v/--verbose` when only one area needs detail. Use `--verify-traceability --verify-cb --verify-cold --verify-tests` when production readiness must be evaluated. `aw health` never mutates; its `next.command` field already names the exact remediation command to run next (`aw capability run`, `aw td promote <path>`, `aw ec gen --verify`, ...), so there is no `aw health fix` — diagnosis and remediation are deliberately separate commands. |

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
fall back to source code only when needed, then consider `aw td code-claim`.

New TD test taxonomy is artifact-oriented: use `unit-test` for generated unit
test design and `e2e-test` for product journey / side-effect verification.
Legacy `test-plan` and `tests` sections may parse with warnings, but new TDs,
templates, and skills should not create them. Product explanation belongs in
README capabilities or external docs; TD sections should exist only when they
drive codegen, handwrite, or verification artifacts.

Runner output may include `artifact_quality_profile` and the stdout prompt may
include an `Artifact Quality Gate`. Treat that gate as part of the lifecycle
contract, not optional advice. Frontend/UI artifacts require machine-verifiable
desktop and mobile viewport evidence, interaction smoke proof,
accessibility/readability smoke proof, and placeholder-free primary-state
evidence before production readiness can be claimed.

Every implementation change goes through Agentic Workflow unless the user explicitly asks
to bypass it. The lifecycle is LINEAR (no review/revise; the gate is EC):
`aw wi` -> `aw td` (author -> `gen` -> `fill`) -> `aw td code-check`. The
CLI owns the concrete phase queue, prompt text, validation gates, commits, git
trailers, and next command. Run `aw llm` for the binary-owned orientation (the
loop model: aw=loop, wi=state, caps=goal, ec=verifier, td=artifact).

Existing-project takeover uses `aw standardize audit` (`check`/`record`) for
the bounded preservation protocol and `aw health` for the project-readiness
metric surface and remediation routing; health's `next.command` names the
worker verb for the top gap, and batch remediation runs as the outer loop
(one worker-verb tick at a time, under a bootstrap WI).

Readiness layers (health axes — the standardize layer verbs are retired):

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

## CLI Convention: stdout tells the agent the next step

Every CLI's machine-readable output MUST carry either `next` — a runnable
command string an agent can execute verbatim — or an explicit terminal marker
meaning "done — report completion to the user"; errors carry a remediation
next step. Emitted commands must actually be executable (multi-level verbs
exist, chain-required args present). aw's aw.cli.v1 envelope is the reference
implementation, enforced by `projects/agentic-workflow/src/cli/chain.rs`
(`validate_aw_command_string` + `EMIT_REGISTRY`). `aw health` measures
completeness; this convention guarantees handoff executability — the two are
complementary, not overlapping. Full spec: **`CONTRIBUTING.md` → "CLI
convention: stdout tells the agent the next step"**.

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

Full principle: **`CONTRIBUTING.md`**. mamba CPython-test mechanics (PEP 723 `[tool.mamba]`, the six dimensions, `tools/fixture_gen.py` → fill → `tools/fixture_lint.py`, manifest): **`projects/mamba/tests/harness/cpython/conventions/FIXTURE-LAYOUT.md`** — read before authoring/decomposing fixtures.

## Testing

- **Real services over mocks**: Use real Docker/Homebrew services for integration tests. vat ships built-in Rust emulators for the GCP/Firebase surface (Pub/Sub, Firebase Auth, Cloud Tasks, Cloud Scheduler, Cloud Workflows, Cloud Storage) plus a transparent HTTP stub + record/replay proxy (`preset = "http-mock"`) for arbitrary third-party APIs — prefer those over hand-rolled mocks. Reach for a mock only when no real service or emulator exists.
- **Local services**: `brew services start redis` (Redis), `brew services start nats-server` (NATS). Tests skip gracefully if unavailable.
- **Skip pattern**: `let Some(x) = connect().await.ok() else { return };`
- **Feature gates**: Redis tests behind `#[cfg(feature = "redis")]`, NATS behind `#[cfg(feature = "nats")]`, Ion behind `#[cfg(feature = "ion")]`

## Debugging

- **Server log**: `~/.cclab/server.log` — MCP server stdout/stderr
- **Server status**: `cclab server list` — show running server PID, port, registered projects
- **Stack overflow**: Server crashes silently (no log) → check tokio worker thread stack size (#182)
