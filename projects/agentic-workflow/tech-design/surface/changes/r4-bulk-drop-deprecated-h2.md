---
id: score-r4-bulk-drop-deprecated-h2
fill_sections: [changes]
summary: |
  R4 of #1212 — bulk drop of deprecated `## Overview` and `## Requirements`
  H2 sections from 23 score-area TD specs (Subsets A and B from the R4
  conformance baseline). Pure body-section deletion + frontmatter
  `fill_sections:` cleanup; no source-code change. Subset C (8 specs with
  `missing-type-annotation` findings) tracked separately.
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# R4 Bulk: Drop Deprecated H2 Sections

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tech-design/surface/changes.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: |
      Drop deprecated `## Overview` H2 + annotation; remove `overview` from
      frontmatter `fill_sections:` array. Narrative content relocates to
      frontmatter `summary:` when load-bearing.
  - path: projects/agentic-workflow/tech-design/surface/handoff_data.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/issues_top.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/list.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/platform.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/proposal.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/sync.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/tasks.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/validate_proposal.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/specs/claude-md-codegen-era-file-size-rule.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/specs/mermaid-plus-primitive-vocabulary.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/specs/score-chat-cli-contract.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/specs/score-chat-jsonl-migration.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/specs/score-chat-listen-filter.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/specs/score-chat-msg-members-schema.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/specs/score-chat.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/specs/score-td-validate-lifecycle-extension.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview` and `## Requirements`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: |
      Drop deprecated `## Overview` and `## Requirements` H2 headings; also
      strip three deprecated `<!-- type: overview/requirements -->` annotations
      from non-deprecated H2 headings ("Storage Model", "Lifecycle-Stage
      Trailers", "Arbitration Decision Table") — those H2s reclassify as
      `missing-type-annotation` (Subset C territory).
  - path: projects/agentic-workflow/tech-design/surface/specs/sync-command.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview` and `## Requirements`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/specs/td-init-command.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview` and `## Requirements`; clean fill_sections.
  - path: projects/agentic-workflow/tech-design/surface/specs/three-role-contract.md
    action: modify
    section: changes
    impl_mode: hand-written
    description: Drop deprecated `## Overview` and `## Requirements`; clean fill_sections.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [changes] All 23 specs explicitly enumerated with action=modify, impl_mode=hand-written; description per-entry calls out the deletion + the issue-crrr-state-machine.md annotation-strip nuance.
- [scope] Subset C cleanly out-of-scope (named explicitly in summary); pilot reference (#1216 commit `465bbe2d4`) lineage clear.
- [verification] `aw td check --section-type-conformance` confirms 0 deprecated findings on our 23 specs (down from 38 across them); residual 44 findings are scoped to Subset C + 5 unrelated.
