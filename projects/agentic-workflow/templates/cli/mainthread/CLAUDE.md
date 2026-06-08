# CLAUDE.md

<!-- aw:start -->
## Agentic Workflow CLI Surface

Agentic Workflow is the workflow protocol. Agents should use the CLI verbs
below, run `aw <verb> --help` when an argument shape matters, and treat stdout
as the live prompt for the current binary and repository state. If stdout
contains a JSON envelope, payload path, `invoke.command`, validation error, or
next command, follow it exactly. For `aw run` JSON, do not declare the workflow
complete unless `completion.workflow_complete=true`; `action=done` can mean only
the current child root is complete and the envelope is asking you to inspect the
parent.

### Workflow CLI

| CLI | Use it for |
|-----|------------|
| `aw run` | Root-driven workflow runner. Choose exactly one root with `--project <project>`, `--capability <project>:<capability-id>`, or `--wi <id>`; follow `invoke.command` and `agent_prompt` until `completion.workflow_complete=true` or `requires_hitl=true`. |
| `aw capability` | Product capability completion loop: `report`, `next`, `run`, and `check`; use `check --verify` when capability proof should include configured test gates. README is the default `cap_path` and uses Markdown H1-Hn capability headings plus contract/work-root tables. YAML `## Capability:` sections and legacy capability tables are migration input only. |
| `aw wi` | Work-item inventory, planning, and CRRR: `draft`, `list`, `show`, `create`, `update`, `close`, `find`, `epicize`, `atomize`, `prioritize`, `enrich`, `validate`, `fill-section`, `review`, `arbitrate`. Planning commands write local artifacts under `/tmp/aw/{project}/...` and do not publish tracker changes. |
| `aw td` | Tech-design lifecycle and checks: `create`, `validate`, `review`, `revise`, `merge`, `arbitrate`, plus read-only/utility verbs `check`, `ast`, `migrate-mermaid`, `claim`. |
| `aw cb` | Code-artifact lifecycle: `gen`, `check`, `claim`, `fill`, `review`, `revise`, `arbitrate`. |
| `aw standardize` | Existing-project takeover workflow and remediation guidance. `capability`, `managed`, `semantic`, `traceability`, and `regenerable` expose `report`, `next`, and `run` to drive bounded repair work; readiness metrics live in `aw health`. |
| `aw health` | Aggregate project readiness metrics: capability readiness, managed/semantic/traceability coverage, command traceability, regenerable maturity, cb verify, cold verify, configured test gates, and HITL status. Use `--verify-traceability --verify-cb --verify-cold --verify-tests` when production readiness must be evaluated. |

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

Every implementation change goes through Agentic Workflow unless the user explicitly asks
to bypass it: `aw wi` -> `aw td` -> `aw cb` -> `aw td merge`. The
CLI owns the concrete phase queue, prompt text, validation gates, commits, git
trailers, and next command.

New TD test taxonomy is artifact-oriented: use `unit-test` for generated unit
test design and `e2e-test` for product journey / side-effect verification.
Legacy `test-plan` and `tests` sections may parse with warnings, but new TDs,
templates, and skills should not create them. Product explanation belongs in
README capabilities or external docs; TD sections should exist only when they
drive codegen, handwrite, or verification artifacts.

Existing-project takeover uses `aw standardize` for bounded workflow guidance
and `aw health` for the project-readiness metric surface.

Standardize workflow layers:

- `capability`: README capability roots are Markdown-table runnable.
- `managed`: every in-scope file is marked `CODEGEN` or `HANDWRITE`.
- `semantic`: source behavior is covered by semantic TD and generator primitive
  gaps.
- `traceability`: active commands, TDs, source refs, and CB blocks close back to
  README capabilities.
- `regenerable`: optional automation maturity; convert as much `HANDWRITE` to
  `CODEGEN` as deterministic generator primitives allow, but do not block
  production readiness on 100% regenerability unless a capability explicitly
  declares that bar.

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

Fix Agentic Workflow first when the pipeline breaks; do not work around a
broken lifecycle.
<!-- aw:end -->
