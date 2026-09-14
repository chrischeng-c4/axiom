---
project:
  name: axiom
  owner: chrischeng-c4
  url: https://github.com/chrischeng-c4/axiom
  ssh: git@github.com:chrischeng-c4/axiom.git
  default_branch: main
---

# AGENTS.md - Codex bootstrap

## Review prompts

When a review prompt supplies its standard, paths, and output contract, review
only those artifacts. Do not edit them, run AW lifecycle verbs, or write Git,
tracker, release, or cloud state. Run any needed Git read through
`git -c core.fsmonitor=false`.

## Default product route

The main Astra controller fixes scope, dispatches work, reads short worker
summaries, and owns Git, tracker mutation, and final acceptance. It does not
investigate product source, documents, or raw logs.

| Request | Route |
|---|---|
| unclear product outcome | `product-ideate` through `<p>-pm` |
| approved outcome needing scope | `product-plan` through `<p>-pm` and `<p>-tl` |
| scoped delivery, test-only work, or read-only review | `product-deliver` |
| cross-project boundary | `cto`; use `integration-qa` only for cross-project verification |
| explicitly requested legacy AW work | one renamed explicit-only legacy skill |
| bounded `apps/aw` change | `aw-dev` |

New behavior work is QA e2e evidence, Dev red unit test and implementation,
then fresh `<p>-qa` final verification. A single-project final review never
uses Integration QA. QA owns e2e cases and required registration. Dev owns
source, colocated unit tests, and test wiring. Dev cannot weaken QA tests.
PM owns authorized product documents. TL owns a proportionate technical task
draft, not source. Do not require a large up-front design, receipt, or extra
approval checkpoint.

The controller may choose e2e-only, impl-only, test-only, or read-only-review
for a partial scope. Final verification runs the owning project's declared
complete, unfiltered gate. Workers report paths, commands, exit codes, and
short failures only. Permit one writer in one worktree at a time.

The seven legacy skills are `prepare-goal`, `grill-release`, `e2e-for`,
`impl-for`, `test-for`, `review`, and `ask-user`. They are explicit-only and
are not aliases for the default route. AW phase state, commit, lifecycle, and
close actions remain controller-only even in legacy work.

The new product route is PENDING ACTIVATION. Do not start a product writer
task or pilot as part of this migration.

## Fleet and permission boundary

The 226-role fleet is a generated candidate target, not the active fleet. It
has PM/TL/QA/Dev for 25 apps and 30 libs, custom `aw-dev`, and six further
singleton roles. Templates under
`scripts/agents/templates/` render project Markdown and `.codex` projections
through `scripts/agents/render_fleet.py`; never hand-edit generated files.

The candidate maps PM/TL/CTO and Integration QA to Terra and QA/Dev to Luna.
Its efforts are PM high, TL xhigh, QA max, and Dev medium. Only one declared
model result was measured: `lumen-qa` used Luna at max effort.

The CLI canary allowed all six forbidden writes. The native read-only canary
also wrote its first `/private/tmp` sentinel. The desktop admission test then
ignored the installed hook and launched `lumen-dev`; it did no tool or write
action. The global configuration was restored. These results do not prove
permission isolation or hook enforcement. Do not claim either for Codex or
Claude.

## Git and external work

Run Git as `git -c core.fsmonitor=false …`. Preserve unrelated work. The
controller alone commits, pushes, changes tracker state, publishes, or closes
work. Use one fresh `agy-operator` only for a user-authorized frozen payload;
the controller owns all semantic verification and acceptance. Use
`gke-operator` only for an authorized paid GKE run.

Treat `main`, `app/*`, `lib/*`, `project-mamba`, `project-lumen`, and
`examples` as persistent refs. Never delete or force-overwrite one without
explicit human confirmation. Preserve dirty changes and use the Git skill
routes. Use `build-release` for an explicit release from an immutable verified
candidate. Candidate acceptance is not stable-release authority. Never tag,
promote, publish, or close work automatically. A worker report or partial gate
is not final acceptance. The controller accepts only after independent,
necessary verification.

`aw` is the Python Typer CLI at `apps/aw`, invoked as
`uv run --project apps/aw aw <group> …`. Its stdout and exit-code protocol is
preserved for explicit legacy use.
