# Mamba product requirements

Mamba is a force-typed Python compiler that also ships a `uv`-shaped package
manager. This directory is the product requirements document: what mamba
promises to Python developers, written down before the work items that
deliver it. Release Milestones are carved from these sections, not the other
way round.

## How this directory is organised

- One file per capability area, named for the area and never for a work item.
  Each `## <title>` section is one promise.
- A shipped promise names the [STATUS](../../STATUS.md) rows that measure it.
  A future promise names the [ROADMAP](../../ROADMAP.md) outcome that owns it
  and ends with `Tracking: Not assigned.` until its release Milestone exists.
- A future section is written before its release Milestone. When the
  Milestone is opened with `/aw-grill-meta-to-milestone`, the Milestone title
  carries the version and the section heading gains the Milestone binding.
  Its issue set is carved from the section's Promise by
  `/aw-grill-milestone-to-issue`, so nothing is promised here that the
  tracker cannot measure.
- Every section carries the parts `/aw-grill-me-to-meta` interviews for —
  Problem, Who, Promise, Non-goals, Neighbours — plus `Open:` lines for
  decisions the downstream grills still have to settle. An `Open:` line is a
  question, not a default; the issue body answers it or the human does.
- A new capability area is a change to this index first and to the README
  `### Capability index` when the area ships.

## Positioning

Mamba's first deliverable to a Python developer is the package manager: a
drop-in for the common `uv` workflow that needs no mamba runtime at all. The
compiler and the CPython runtime replacement it enables come second, in the
tier order the README records, and they never change the package manager's
contract. This order was decided on 2026-09-03; until then the runtime tiers
came first.

## Who mamba is for

| Reader | What they hold mamba to |
|---|---|
| Python developer | `mamba` behaves like `uv` for the project workflow, on the system or managed CPython they already have. |
| Compiler adopter | A compiled program gives the CPython 3.12 result with less CPU time and less memory, tier by tier. |

## Horizons

| Horizon | Outcome | Section |
|---|---|---|
| H1 | `uv-workflow-parity` | [package-manager.md](package-manager.md) § uv workflow parity |
| H2 | `cpython-runtime-replacement` | [runtime.md](runtime.md) § CPython runtime replacement |

## Section index

| Section | File | Kind | Owner |
|---|---|---|---|
| Offline project workflow | package-manager.md | shipped, limited | STATUS `project-dependencies`, `environment-and-run`, `interpreter-management`, `build-and-version`, `tooling-and-cache`, `sources-and-credentials` |
| uv workflow parity | package-manager.md | outcome | ROADMAP `uv-workflow-parity` |
| CPython runtime replacement | runtime.md | outcome | ROADMAP `cpython-runtime-replacement` |

Non-goals are not sections. Each file ends with the non-goals a reader of that
area would otherwise assume, pointing at the ROADMAP entry that gives the
reason: `sdist-c-extension-builds`, `resolver-speed-parity-with-uv`,
`full-pip-option-surface`.
