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
Apps                — agentic-workflow
```

Full project and shared-library inventory: `README.md`. Domain model schemas:
`apps/agentic-workflow/schemas/`.

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
follow it exactly. For goal lifecycle root output (`aw goal wi` / `aw goal
capability` / `aw goal backlog`), do not declare the workflow complete
unless `completion.workflow_complete=true`; `action=done` can mean only the
current child root is complete and the envelope is asking you to inspect the
parent.

Do not use removed top-level helpers such as `aw check`, `aw hover`,
`aw daemon`, `aw serve`, or `aw context`.

For Agentic Workflow itself (`agentic-workflow` / `aw`), do not run the full
AW loop against its own repo, and do not turn `aw health` (including its
`takeover-audit` axis, the retired `aw standardize` namespace's successor)
into a self-takeover gate: a broken lifecycle cannot be required to fix
itself (self-deadlock). Self-AW hard-gates only the capability contract —
CAPABILITIES.md work-roots with resolvable gap/claim ids; closing WI/TD
evidence is discovered from the WI/commit side, and doc-stored WI refs are
optional derived provenance (#1847). EC claim verification becomes a hard gate only once an EC inventory is
actually configured for aw; until then it is advisory, like managed/semantic/
traceability, TD lock, CB verify, cold rebuild, and workspace test gates.
Changes to aw itself land as direct commits with `Refs #<issue>` trailers
plus capability work-root registration — the sanctioned self-hosting mode,
not a lifecycle bypass.

Codex should translate Claude slash-command references such as `/aw:td` or
`/aw:wi` to the equivalent `aw ...` CLI command unless the user
explicitly asks for Claude-specific behavior.

### Workflow CLI

<!-- aw:cli-table:workflow:start -->
| Verb | About |
|------|-------|
| `aw meta` | Initialize, synchronize, and check repository/project META-docs |
| `aw wi` | Manage work-items — list/show/create/validate across local + GitHub backends |
| `aw capability` | Product capability completion loop: report/next/run/check |
| `aw td` | Tech-design and generated-code lifecycle |
| `aw ec` | External-contract lifecycle: draft/fill, independent semantic review, generate, and verify |
| `aw health` | Aggregate project readiness, production gates, and blocker status |
| `aw conf` | Manage `aw.toml` and Agentic Workflow configuration producers |
<!-- aw:cli-table:workflow:end -->

`aw goal` is aw's single loop verb: every invocation names a root and a
verifier. `aw goal wi <id>` drives one work item to terminal, `aw goal
capability [<cap-id>] --project <project>` drives one capability's work-root
WIs (omit `<cap-id>` to run the whole project end to end), and `aw goal
backlog --project <project>` drains every open work item for a project one
at a time, parking (not blocking) on HITL/hard blockers so the drain
continues. Follow `invoke.command` and `agent_prompt` from any of the three
until `completion.workflow_complete=true` or `requires_hitl=true`. If a HITL
envelope includes `hitl_question.interaction.kind=user_question`, invoke the
host's native user-question tool immediately with its question, choices, and
freeform prompt: Claude Code uses `AskUserQuestion`, Codex uses
`request_user_input` when that tool is exposed, and AGY uses `ask_user`. Do
not treat the envelope as terminal output or fabricate human approval. If
the host has no such tool, present every field as a blocking human question.
(The old top-level `aw run` verb and the retired `aw wi run`/`aw capability
run` verbs are gone — `wi`/`capability`/`backlog` are `aw goal` root types
now.)

`aw wi` is work-item inventory, planning, and bounded linear authoring: `draft`, `list`, `show`,
`create`, `update`, `close`, `find`, `epicize`, `atomize`, `prioritize`,
`enrich`, `validate`, `fill-section`; drive one WI to terminal with `aw goal
wi <id>` (see above). Planning commands write local artifacts under
`/tmp/aw/workspaces/<workspace>/workitems/{project}/...` and do not publish tracker changes. There is no
`estimate`/`sprintize`; use `aw goal backlog --project <name>` to drain every
open issue for a project, or `aw goal capability --project <name>` as the
run-to-end driver for one capability/the whole project, instead of
cron-style sprint batches.

`aw td` is the tech-design + generated-code lifecycle (LINEAR — no
review/revise; the gate is EC via `code-check`): `create`, `gen`, `fill`,
plus read-only/utility verbs `check`, `ast`, `migrate-mermaid`, `lock`,
`claim`, `gen-source`, `code-check`, `promote`, `audit-record`.
(Code-artifact verbs are folded in here; the former standalone
code-artifact namespace and the merge verb were removed — `code-check` is
the terminal step. `aw td`'s retired `validate` subcommand folded into
`check` (#1277) and its retired `code-claim` subcommand folded into
`create --from-source` (#1273).) `aw td check`
specifically checks TD/spec files for structure, section-format rules, and
logical consistency. TD defines candidate implementation structure;
capability and EC gates remain the source of product truth.

`aw ec` is the only semantic approval loop: `draft` -> `fill` -> structural
`check` -> independent `review`; `needs_revision` returns to bounded `fill`,
while `accepted` advances to `gen` and `verify`. Production-required EC review
evidence is digest-bound and independent: project `aw.toml`
`ec_review_backing` selects `either` (default, agent-first) | `agent` |
`human` (opt-in blocking human-only review). An unconfigured project's
`aw ec review` emits the non-blocking `pending_agent_review` envelope, and
agent verdicts come from a host-dispatched independent `aw-ec-reviewer`
subagent, with same-agent self-review never accepted (#1829). A human
`--evidence-file` submission always remains valid evidence regardless of
policy, so the post-completion human batch audit can reopen an
agent-accepted EC (#1859). `ec_review_mode = "deferred"` records a pending
human review as `deferred_pending_human` without blocking the runner and
queues it in `aw ec`/`aw health` for post-completion batch review (#1828).
`aw health` routes missing approval to `aw ec review` and accepted EC
generation gaps to `aw ec gen --verify`.

The former `aw standardize` namespace (including `audit check`/`audit
record`) is retired (#1278, epic #1270). Existing-project takeover uses
`aw td audit-record` to record a bounded preservation audit fixture and
`aw health`'s `takeover-audit` axis to check it. Readiness layer metrics
(`capability`, `managed`, `semantic`, `traceability`, `regenerable`) live
entirely in `aw health`, whose `next.command` names the worker verb for the
top gap (`aw td promote`, `aw td create --from-source`, `aw td gen`, `aw wi
create`, ...). Capability remediation is `aw capability`; HANDWRITE→CODEGEN
promotion is `aw td promote`.

`aw capability` is the product capability completion loop: `report`,
`next`, `draft`, `apply-draft`, `init`, `migrate`, `run`, `check`, and
`sweep`. For multi-project README rollout, run `sweep --write-rollout
--human --skip-issue-inventory` first, then use the rollout/draft/WI/action
queue artifacts instead of freehand README edits. Treat
`create_wi:issue_inventory_skipped` as tracker-sync work, not WI backlog.
Use `migrate` for YAML/legacy-to-canonical Markdown conversion and for
relocating README-resident capability structure to CAPABILITIES.md, and
use `check --verify` when capability proof should include configured test
gates. CAPABILITIES.md is the default `cap_path` and uses `## Brief`,
`## Capabilities`, `### Capability Index`, field-style capability
contracts, and work-root tables; README keeps a human-readable summary
that links to the capability contract. YAML `## Capability:` sections,
legacy capability tables, and README-resident capability structure are
migration input only.

`aw health` is a read-only aggregate of project readiness metrics:
capability readiness, managed/semantic/traceability coverage, command
traceability, regenerable maturity, cb verify, cold verify, configured test
gates, and HITL status. Run `aw health --project <project>` for the full
picture, or pass a focused `[SECTION]` (e.g. `regenerable`, `gates`,
`blockers`) plus `-v/--verbose` when only one area needs detail. Use
`--verify-traceability --verify-cb --verify-cold --verify-tests` when
production readiness must be evaluated. `aw health` never mutates; its
`next.command` field already names the exact remediation command to run
next (`aw goal capability`, `aw td promote <path>`, `aw ec gen --verify`,
...), so there is no `aw health fix` — diagnosis and remediation are
deliberately separate commands.

### Support CLI

<!-- aw:cli-table:support:start -->
| Verb | About |
|------|-------|
| `aw guard` | Agent-runtime direct edit/create guard for Codex, Claude Code, and AGY |
| `aw llm` | Offline agent orientation: outline + capability/td/ec pillars + loop |
| `aw upgrade` | Self-update this binary from a published GitHub release |
| `aw issue` | Search, view, or create Agentic Workflow issues |
| `aw goal` | Unified loop verb: lifecycle root types (`wi`, `capability`, `backlog`) plus the ad-hoc CLI-owned verifiable-condition loop for bounded work outside the WI lifecycle (`set`/`check`/`show`/`list`/`clear`) |
<!-- aw:cli-table:support:end -->

`aw conf check` verifies `aw.toml`'s generated project registry block
without writing; `aw conf sync` auto-discovers projects and refreshes that
block. Other projected artifacts are owned by their own producer commands and
should be routed through `aw health` once those health checks are wired.
`aw meta init|sync|check` is the sole producer/checker for repo/project
META-doc skeletons and AW-owned marker blocks; `check` is read-only and names
the exact `sync` remediation when drift exists.
`aw llm`, `aw upgrade`, and `aw issue` are the CLI-convention trio every
ecosystem binary ships — see "CLI Convention: every CLI ships `llm`,
`upgrade`, `issue`" below for the full contract.

`aw guard` is the agent-runtime direct edit/create guard for Codex and
Claude Code (live-denies out-of-lifecycle writes).

`aw goal` has a closed four-leaf root-type enum: `wi` and `capability`
(the lifecycle roots described above), `backlog` (drain every open WI for a
project — `aw goal backlog --project <project>`), and `adhoc` for bounded
work OUTSIDE the WI/TD/EC lifecycle — test-pass gates, migration sweeps, and
other ad-hoc tasks a human hands an agent directly. `aw goal set --gate
"<command>" <intent>` records the prose intent plus one or more required
machine-runnable gate commands as workspace-scoped state (never a repo-root
file); `aw goal check [<id>]` runs the gates and reports deterministically
(`done`, `blocked` with `next.command`, or `gave_up` on budget/24h expiry).
aw-managed roots use the `wi`/`capability`/`backlog` goal leaves; `adhoc` is
for everything outside that lifecycle — never a substitute for the
lifecycle root types.

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
- `app/{name}` — persistent work-area branches for `apps/` applications, such
  as `app/jet` or `app/aw`; their local worktree directories use underscores,
  such as `app_jet`.
- `lib/{name}` — persistent work-area branches for `libs/` internal libraries,
  such as `lib/openapi-codegen` or `lib/raft-host`; their local worktree
  directories use underscores, such as `lib_openapi-codegen`.
- `project-mamba` and `project-lumen` — retained legacy project work-area
  branches while those two roots remain under `projects/`.

One persistent app/lib/project branch maps to one dedicated worktree and one
agent session. Do not delete or force-overwrite `main`, `app/*`, `lib/*`,
`project-mamba`, or `project-lumen` without explicit user confirmation. Prefer
non-destructive convergence for stale persistent refs. These branches are
deletion-protected on GitHub via the `protect-persistent-branches` repository
ruleset (force-push is intentionally left unprotected so rebase-based landing
still works).

WI never creates or switches git branches. TD lifecycle branches
(`td-<id>`) are short-lived and may be created only when launched
from `main`; off-main TD commands stay on the current branch. When the user
says "the jet branch" or "the agentic-workflow branch" without a prefix, prefer
`app/<name>` for apps. For mamba and lumen, keep using `project-mamba` and
`project-lumen`.

## Work-Item Rules

Canonical verb: `aw wi`. Legacy work-item aliases are removed from the active
CLI surface.

- One issue-platform id is one workflow root; do not invent a second slug.
- Draft/planning intermediate state lives under `/tmp/aw/workspaces/<workspace>/workitems/{project}`.
- Published state is projected to the issue platform configured in
  `aw.toml`.
- Repo-root `.aw/` is retired from the AW ecosystem. Do not create, read, or
  commit repo-root `.aw/*` lifecycle/cache files; ephemeral working copies live
  under `/tmp/aw`, root configuration lives in `aw.toml`, and durable project
  artifacts live under their project directories.
- Backend selection comes from `aw.toml`; do not add ad-hoc backend
  flags to `aw wi`.
- `--label` is not the public create path. Labels are derived from typed flags:
  `--type`, `--project`, `--priority`, and `--agent`.
- Non-epic work-items must be bounded before TD: include `## Capability
  Alignment`, `## Scope`, `## Acceptance Criteria`, and `## Reference
  Context`. Roadmap-sized or decision-blocked work must go through `aw wi
  atomize` or HITL review before `aw td`.

## SDD and Codegen Rules

Specs are the source of truth. Consult `apps/agentic-workflow/tech-design/` first;
fall back to source code only when needed, then consider `aw td create
--from-source`.

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

Existing-project takeover uses `aw td audit-record` (the retired `aw
standardize audit record`) for the bounded preservation protocol, `aw
health`'s `takeover-audit` axis (the retired `aw standardize audit check`)
to read it back, and `aw health` for the project-readiness metric surface
and remediation routing; health's `next.command` names the worker verb for
the top gap, and batch remediation runs as the outer loop (one worker-verb
tick at a time, under a bootstrap WI).

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
capability` reads the project's `CAPABILITIES.md` (the default `cap_path`)
or configured `cap_path`; capability document structure is Markdown-first:
`#` is the project root, `## Brief` is the agent-readable project summary,
`## Capabilities` owns the capability registry, and `### Capability Index`
is the compact scan surface. H3-Hn capability headings use field-style
contracts and work-root tables to map headings to epic/subepic WI roots.
Atomic `change` WIs usually come from `aw wi atomize` rather than
CAPABILITIES.md rows. README-resident capability structure is migration
input — `aw capability migrate` relocates it to CAPABILITIES.md and leaves
a human-readable summary in README. YAML `## Capability:` sections and
legacy capability tables are also migration input only. Verified progress requires passing declared
verification gates or linked validation inventories plus claim closure; WI
linkage is derived provenance resolved from the WI side, and doc-stored WI
refs are optional (stale ones degrade to advisory findings, #1847). Do not use the old capability shorthand.
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
  `app:<name>`; `create` files a structured issue (diagnostics + the
  `app:<name>` label). Named `issue` (not `report`), leaving domain `report`
  verbs (`jet report` = HTML test reports) untouched.

Full spec: **`CONTRIBUTING.md` → "CLI convention: every CLI ships `llm`,
`upgrade`, `issue`"**.

## CLI Convention: stdout tells the agent the next step

Every CLI's machine-readable output MUST carry either `next` — a runnable
command string an agent can execute verbatim — or an explicit terminal marker
meaning "done — report completion to the user"; errors carry a remediation
next step. Emitted commands must actually be executable (multi-level verbs
exist, chain-required args present). aw's aw.cli.v1 envelope is the reference
implementation, enforced by `apps/agentic-workflow/src/cli/chain.rs`
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
