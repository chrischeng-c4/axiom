---
id: score-refactor-remove-legacy-standardize-aliases
fill_sections: [cli, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Remove legacy `aw standardize {report|next|run|codegen}` aliases

## CLI
<!-- type: cli lang: yaml -->

```yaml
command: aw standardize
description: Existing-project standardization — managed adoption and regenerability
args: []
options: []
subcommands:
  - name: managed
    description: "Adoption layer: every in-scope file is CODEGEN or HANDWRITE"
    subcommands:
      - name: report
        description: Emit coverage for an in-scope project or source scope
        args:
          - name: project
            type: Option<String>
            description: Project name from .aw/config.toml, e.g. `sdd`
        options:
          - name: --all
            type: bool
            description: Run every configured project
          - name: --scope
            type: Vec<String>
            description: Override workspace scopes (repeatable)
          - name: --json
            type: bool
            description: Emit machine-readable JSON
      - name: next
        description: Emit the next deterministic action without mutating files
        args:
          - name: project
            type: Option<String>
            description: Project name from .aw/config.toml
        options:
          - name: --all
            type: bool
            description: Run every configured project
          - name: --scope
            type: Vec<String>
            description: Override workspace scopes (repeatable)
          - name: --json
            type: bool
            description: Emit machine-readable JSON
      - name: run
        description: Run actions for a project until complete, blocked, or max ticks
        args:
          - name: project
            type: Option<String>
            description: Project name from .aw/config.toml
        options:
          - name: --all
            type: bool
            description: Run every configured project
          - name: --scope
            type: Vec<String>
            description: Override workspace scopes (repeatable)
          - name: --non-interactive
            type: bool
            description: Emit a blocked envelope and exit non-zero when HITL/mainthread work is required
          - name: --max-ticks
            type: Option<usize>
            description: Stop after N successful ticks
          - name: --json
            type: bool
            description: Emit machine-readable JSON envelopes
          - name: --push
            type: bool
            description: Push after each successful per-action commit

  - name: regenerable
    description: "Regenerability layer: every in-scope file is fully CODEGEN-owned"
    subcommands:
      - name: report
        description: Emit regenerability coverage for an in-scope project or source scope
        args:
          - name: project
            type: Option<String>
            description: Project name from .aw/config.toml
        options:
          - name: --all
            type: bool
            description: Run every configured project
          - name: --scope
            type: Vec<String>
            description: Override workspace scopes (repeatable)
          - name: --json
            type: bool
            description: Emit machine-readable JSON
      - name: next
        description: Emit the next deterministic regenerability action without mutating files
        args:
          - name: project
            type: Option<String>
            description: Project name from .aw/config.toml
        options:
          - name: --all
            type: bool
            description: Run every configured project
          - name: --scope
            type: Vec<String>
            description: Override workspace scopes (repeatable)
          - name: --json
            type: bool
            description: Emit machine-readable JSON
      - name: run
        description: Run regenerability actions for a project until complete or blocked
        args:
          - name: project
            type: Option<String>
            description: Project name from .aw/config.toml
        options:
          - name: --all
            type: bool
            description: Run every configured project
          - name: --scope
            type: Vec<String>
            description: Override workspace scopes (repeatable)
          - name: --non-interactive
            type: bool
            description: Emit a blocked envelope and exit non-zero when HITL/mainthread work is required
          - name: --max-ticks
            type: Option<usize>
            description: Stop after N successful ticks
          - name: --json
            type: bool
            description: Emit machine-readable JSON envelopes
          - name: --push
            type: bool
            description: Push after each successful per-action commit

removed_subcommands:
  - name: report
    reason: legacy top-level alias for `managed report`; removed in this refactor
  - name: next
    reason: legacy top-level alias for `managed next`; removed in this refactor
  - name: run
    reason: legacy top-level alias for `managed run`; removed in this refactor
  - name: codegen
    reason: legacy full-codegen coverage alias; removed in this refactor — use `regenerable report` instead

examples:
  - cmd: aw standardize managed report --scope examples/fixture_platform/backend
    desc: Coverage report for the fixture_platform backend (managed/adoption layer)
  - cmd: aw standardize managed next fixture_platform
    desc: Show the next deterministic managed action for the fixture_platform project
  - cmd: aw standardize regenerable report sdd
    desc: Regenerability coverage for the sdd project
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: projects/agentic-workflow/src/cli/standardize.rs
    action: modify
    section: cli
    impl_mode: hand-written
    changes:
      - Remove Report, Codegen, Next, Run variants from StandardizeCommand enum (lines ~44-51 in pre-refactor source)
      - Remove the StandardizeCodegenArgs struct definition (lines ~88-102)
      - Remove the four legacy dispatch arms in run(StandardizeArgs) so only Managed and Regenerable remain (lines ~302-305)
      - Keep StandardizeReportArgs, StandardizeNextArgs, StandardizeRunArgs (still referenced by StandardizeStageCommand)

  - path: projects/agentic-workflow/tests/standardize_test.rs
    action: modify
    section: cli
    impl_mode: hand-written
    changes:
      - Replace `aw standardize codegen` invocation at line 76 with `aw standardize managed report`
      - Update any related assertion text that references the legacy verb

  - path: projects/agentic-workflow/CLAUDE.md
    action: modify
    section: cli
    impl_mode: hand-written
    changes:
      - Update the Standardization workflow CLI mapping table to reference `aw standardize managed report` and `aw standardize managed next` instead of the bare forms
      - Update the umbrella-driver sentence to use the managed form

  - path: projects/agentic-workflow/CLAUDE.md
    action: modify
    section: cli
    impl_mode: hand-written
    changes:
      - Update the Standardization layers table and adjacent prose to use the two-layer form
      - Replace `aw standardize codegen sdd` with `aw standardize regenerable run sdd`

  - path: projects/agentic-workflow/tech-design/surface/specs/score-standardization.md
    action: modify
    section: cli
    impl_mode: hand-written
    changes:
      - Drop the three "legacy alias" sentences (Standardization Layers paragraph, Scope Resolution paragraph, CLI section)
      - Tighten the surrounding prose so the two-layer form is described without a "legacy" carve-out

  - path: projects/agentic-workflow/tech-design/surface/src/standardize.md
    action: regen
    section: cli
    impl_mode: codegen
    changes:
      - Regenerate Symbols table from the post-change source AST (drops the StandardizeCodegenArgs row)
```

# Reviews

## Review 1
**Verdict:** approved

- [cli] Surface change is correct and narrow: top-level `aw standardize {report|next|run|codegen}` removed; canonical `aw standardize {managed|regenerable} {report|next|run}` retained. The four legacy variants in `StandardizeCommand` go away cleanly and the dispatcher loses its four arms.
- [changes] Single-file refactor in `projects/agentic-workflow/src/cli/standardize.rs` plus one test fixture update is the right surface. CLAUDE.md doc text update keeps user-facing docs aligned. Risk is low because the canonical forms have been the documented entrypoint since 2026-05.

## Review 2
**Verdict:** approved

- [cli] Revised surface remains correct: only canonical forms persist; all four legacy aliases are removed.
- [changes] Implementation surface unchanged from review #1 — single Rust file, single test edit, doc update.
