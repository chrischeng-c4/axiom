---
id: mcp-refs-cleanup
main_spec_ref: "crates/cclab-sdd/interfaces/cli/sdd-cli.md"
merge_strategy: extend
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: cli-workflow-chain
    claim: cli-workflow-chain
    coverage: full
    rationale: "Command/root TDs support CLI workflow chain routing and root-runner dispatch."
---

# Sdd Cli Spec

## Overview
<!-- type: overview lang: markdown -->

Bulk refactor of ~28 spec files under `.aw/tech-design/crates/cclab-sdd/`. SDD has fully migrated from MCP server to CLI execution, but specs still contain stale MCP references — wrong `files:` frontmatter paths (`mcp/tools/*.rs`), outdated terminology ("MCP tool", "MCP server"), dead `mcp__cclab-mcp__*` disallowedTools patterns, and one potentially obsolete spec (`generate/template-mcp-configs.md`).

This change updates all references to reflect the current architecture: tools live at `tools/*.rs` and `services/*.rs`, execution is via `cclab sdd` CLI commands, and `sdd-cli.md` R5 no longer references MCP parity.

No code changes. Spec-only refactor.
## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: mcp-refs-cleanup-requirements
title: MCP References Cleanup Requirements
requirements:
  R1:
    text: Update files frontmatter paths from mcp/tools/*.rs to tools/*.rs or services/*.rs
    type: functional
    priority: high
    risk: low
    verification: inspection
    notes: |
      Affects ~18 spec files with stale mcp/tools/ paths. All paths are relative
      to crates/cclab-sdd/src/.
  R2:
    text: Replace MCP terminology across spec prose
    type: functional
    priority: high
    risk: low
    verification: inspection
    notes: |
      Patterns: "MCP tool" -> "tool" or "CLI command"; "MCP server" -> remove or "CLI";
      "MCP Artifact Tools" -> "Artifact Tools"; etc.
  R3:
    text: Remove mcp__cclab-mcp__* disallowedTools patterns
    type: functional
    priority: medium
    risk: low
    verification: inspection
    notes: |
      Affects tools/utils/delegate-agent.md and generate/template-claude-md.md.
  R4:
    text: Archive generate/template-mcp-configs.md as obsolete
    type: functional
    priority: medium
    risk: low
    verification: inspection
    notes: |
      Move to .aw/archive/ or add status: archived frontmatter.
  R5:
    text: Reword sdd-cli.md R5 from "Parity with MCP Execution" to "Unified Executor Logic"
    type: functional
    priority: medium
    risk: low
    verification: inspection
    notes: |
      CLI is the sole execution path — no parallel MCP path exists.
  R6:
    text: No remaining active-MCP implications in spec files
    type: constraint
    priority: high
    risk: low
    verification: inspection
    notes: |
      Validation - grep -r 'MCP' .aw/tech-design/crates/cclab-sdd/ returns zero
      matches except archived files or historical notes.
---
requirementDiagram
    requirement R1 {
      id: R1
      text: Update files frontmatter paths
      risk: low
      verifymethod: inspection
    }
    requirement R2 {
      id: R2
      text: Replace MCP terminology
      risk: low
      verifymethod: inspection
    }
    requirement R3 {
      id: R3
      text: Remove mcp__cclab-mcp__ disallowedTools patterns
      risk: low
      verifymethod: inspection
    }
    requirement R4 {
      id: R4
      text: Archive template-mcp-configs.md
      risk: low
      verifymethod: inspection
    }
    requirement R5 {
      id: R5
      text: Reword sdd-cli.md R5
      risk: low
      verifymethod: inspection
    }
    requirement R6 {
      id: R6
      text: No remaining MCP implications
      risk: low
      verifymethod: inspection
    }
```

### R1: Update `files:` Frontmatter Paths

All spec files with `files:` frontmatter referencing `mcp/tools/*.rs` must be updated to the actual source paths under `tools/*.rs` or `services/*.rs`.

| Spec | Stale Path | Corrected Path |
|------|-----------|----------------|
| `logic/state-machine.md` | `mcp/tools/phase_transition.rs` | `tools/phase_transition.rs` |
| `logic/executor-resolution.md` | `mcp/tools/workflow_common.rs` | `tools/workflow_common.rs` |
| `interfaces/workflow/implement.md` | legacy `mcp/tools/change_impl/{common,create,review,revise}.rs` | `src/workflow/implement.rs` |
| `logic/post-clarifications.md` | `mcp/tools/post_clarifications/{mod,create}.rs` | `tools/create_post_clarifications.rs` |
| `logic/pre-clarifications.md` | `mcp/tools/create_pre_clarifications.rs` | `tools/create_pre_clarifications.rs` |
| `tools/create_change_merge/workflow.md` | legacy `mcp/tools/change_merge/create.rs` | `src/tools/create_change_merge.rs` |
| `logic/change-spec.md` | `mcp/tools/change_spec/{common,create,review,revise}.rs` | `tools/{common_change_spec,create_change_spec,review_change_spec,revise_change_spec}.rs` |
| `logic/restructure-input.md` | `mcp/tools/restructure_input.rs` | `tools/restructure_input.rs` |
| `tools/utils/delegate-agent.md` | `mcp/tools/agent.rs` | `tools/agent.rs` |
| `tools/utils/write-artifact.md` | `mcp/tools/artifact_write.rs` | `tools/artifact_write.rs` |
| `tools/utils/read-artifact.md` | `mcp/tools/artifact_read.rs` | `tools/artifact_read.rs` |
| `tools/utils/fetch-issues.md` | `mcp/tools/fetch_issues.rs` | `tools/fetch_issues.rs` |
| `tools/utils/read-implementation-summary.md` | `mcp/tools/implementation.rs` | `tools/implementation.rs` |
| `tools/utils/list-changed-files.md` | `mcp/tools/implementation.rs` | `tools/implementation.rs` |
| `tools/utils/analyze-code-for-spec.md` | `mcp/tools/analyze/{mod,python,typescript,rust_lang,suggestions}.rs` | `tools/analyze/{mod,python,typescript,rust_lang,suggestions}.rs` |
| `tools/utils/platform-sync.md` | `mcp/tools/platform_sync.rs` | `tools/platform_sync.rs` |
| `tools/utils/validate-spec-completeness.md` | `mcp/tools/validate_spec.rs` | `tools/validate_spec.rs` |
| `tools/utils/validate-change.md` | `mcp/tools/validate.rs` | `tools/validate.rs` |

All paths are relative to `crates/cclab-sdd/src/`.

### R2: Replace MCP Terminology

Replace stale MCP terminology across spec prose:

| Pattern | Replacement |
|---------|-------------|
| "MCP tool" | "tool" or "CLI command" (context-dependent) |
| "MCP server" | remove or replace with "CLI" |
| "MCP Artifact Tools" (title) | "Artifact Tools" |
| "MCP Utility Tools" (title) | "Utility Tools" |
| "Unified MCP tool" | "Unified tool" |
| "direct MCP tool invocation" | "direct tool invocation" |
| "MCP response" | "tool response" |
| "via MCP tool" | "via CLI command" |
| "calling the `sdd_run_change` MCP tool" | "calling the `sdd_run_change` tool" |
| "Uses Lens MCP tools" | "Uses Lens tools" |
| "MCP tool pointers" | "tool pointers" |
| "LLM tools can read knowledge via MCP" | "LLM tools can read knowledge via CLI" |

### R3: Remove `mcp__cclab-mcp__*` Patterns

Remove or update `mcp__cclab-mcp__sdd_delegate_agent` disallowedTools patterns in:
- `tools/utils/delegate-agent.md` (L184-185)
- `generate/template-claude-md.md` (`mcp__cclab__sdd_*` references)

These patterns referenced the MCP server tool namespace which no longer exists.

### R4: Archive `generate/template-mcp-configs.md`

The spec `generate/template-mcp-configs.md` documents MCP config generation for `.mcp.json`, `.gemini/settings.json`, `.codex/config.toml`. If `cclab-mcp` server is no longer used, this spec is fully obsolete. Archive by moving to `.aw/archive/` or adding `status: archived` frontmatter.

### R5: Reword `sdd-cli.md` R5

R5 currently reads "Parity with MCP Execution" — implying MCP is a parallel execution path. Reword to reflect that CLI is the sole execution path:

| Before | After |
|--------|-------|
| "Parity with MCP Execution" | "Executor Parity" or "Unified Executor Logic" |
| "compared to the MCP tools (`sdd_write_artifact`, etc.)" | "compared to the underlying executor logic" |
| Overview paragraph referencing MCP dependency | Remove MCP dependency framing; state CLI as the native interface |

### R6: No Remaining Active-MCP Implications

After all changes, no spec file under `.aw/tech-design/crates/cclab-sdd/` should contain text implying the MCP server is in active use. Validation: `grep -r 'MCP' .aw/tech-design/crates/cclab-sdd/` returns zero matches except in archived files or historical notes explicitly marked as deprecated.
## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  S1:
    name: Frontmatter path update
    verifies: [R1]
    given: |
      logic/state-machine.md has `files: [mcp/tools/phase_transition.rs]`
    when: |
      Refactor is applied
    then: |
      - Frontmatter reads `files: [tools/phase_transition.rs]`
      - Path resolves to an existing file under crates/cclab-sdd/src/
  S2:
    name: Terminology replacement in tool spec
    verifies: [R2]
    given: |
      tools/utils/write-artifact.md contains "Unified MCP tool for all artifact lifecycle operations"
    when: |
      Refactor is applied
    then: |
      Description reads "Unified tool for all artifact lifecycle operations" with no MCP reference
  S3:
    name: Title update in interface spec
    verifies: [R2]
    given: |
      interfaces/tools/artifact-tools.md has title "MCP Artifact Tools — OpenRPC Definitions"
    when: |
      Refactor is applied
    then: |
      Title reads "Artifact Tools — OpenRPC Definitions"
  S4:
    name: disallowedTools pattern removal
    verifies: [R3]
    given: |
      tools/utils/delegate-agent.md contains `mcp__cclab-mcp__sdd_delegate_agent` in disallowedTools block
    when: |
      Refactor is applied
    then: |
      MCP-namespaced pattern is removed or replaced with CLI-era equivalent
  S5:
    name: R5 reword in sdd-cli.md
    verifies: [R5]
    given: |
      interfaces/cli/sdd-cli.md R5 reads "Parity with MCP Execution"
    when: |
      Refactor is applied
    then: |
      - R5 reads "Unified Executor Logic"
      - Body text no longer references MCP tools as a comparison baseline
  S6:
    name: Obsolete spec archival
    verifies: [R4]
    given: |
      generate/template-mcp-configs.md documents MCP config generation for a server no longer in use
    when: |
      Refactor is applied
    then: |
      - File is archived (moved to .aw/archive/ or marked status: archived)
      - No longer appears in active spec listings
  S7:
    name: Post-refactor validation
    verifies: [R6]
    given: |
      All changes have been applied
    when: |
      grep -r 'MCP' .aw/tech-design/crates/cclab-sdd/ is run
    then: |
      Zero matches are returned (excluding archived files)
```

<!-- Legacy scenario prose preserved for reference -->

### Scenario: Frontmatter path update
- **GIVEN** `logic/state-machine.md` has `files: [mcp/tools/phase_transition.rs]`
- **WHEN** the refactor is applied
- **THEN** frontmatter reads `files: [tools/phase_transition.rs]` and the path resolves to an existing file under `crates/cclab-sdd/src/`

### Scenario: Terminology replacement in tool spec
- **GIVEN** `tools/utils/write-artifact.md` contains "Unified MCP tool for all artifact lifecycle operations"
- **WHEN** the refactor is applied
- **THEN** the description reads "Unified tool for all artifact lifecycle operations" with no MCP reference

### Scenario: Title update in interface spec
- **GIVEN** `interfaces/tools/artifact-tools.md` has title "MCP Artifact Tools — OpenRPC Definitions"
- **WHEN** the refactor is applied
- **THEN** the title reads "Artifact Tools — OpenRPC Definitions"

### Scenario: disallowedTools pattern removal
- **GIVEN** `tools/utils/delegate-agent.md` contains `mcp__cclab-mcp__sdd_delegate_agent` in a disallowedTools block
- **WHEN** the refactor is applied
- **THEN** the MCP-namespaced pattern is removed or replaced with the CLI-era equivalent

### Scenario: R5 reword in sdd-cli.md
- **GIVEN** `interfaces/cli/sdd-cli.md` R5 reads "Parity with MCP Execution"
- **WHEN** the refactor is applied
- **THEN** R5 reads "Unified Executor Logic" and body text no longer references MCP tools as a comparison baseline

### Scenario: Obsolete spec archival
- **GIVEN** `generate/template-mcp-configs.md` documents MCP config generation for a server no longer in use
- **WHEN** the refactor is applied
- **THEN** the file is archived (moved to `.aw/archive/` or marked `status: archived`) and no longer appears in active spec listings

### Scenario: Post-refactor validation
- **GIVEN** all changes have been applied
- **WHEN** `grep -r 'MCP' .aw/tech-design/crates/cclab-sdd/` is run
- **THEN** zero matches are returned (excluding archived files)
## Diagrams
<!-- type: diagram lang: mermaid -->

## API Spec
<!-- type: api lang: yaml -->

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: sdd-cli-test-plan
kind: requirementDiagram
---
requirementDiagram

    requirement R1 {
        id: R1
        text: "CLI correctly invokes the restructuring workflow"
        risk: Medium
        verification: Test
    }

    requirement R2 {
        id: R2
        text: "CLI correctly handles JSON payloads via --json-file"
        risk: Medium
        verification: Test
    }

    requirement R3 {
        id: R3
        text: "CLI emits structured JSON output with --json"
        risk: Low
        verification: Test
    }

    element E1 {
        type: "test"
        id: test_restructure_cli
        text: "Test restructure CLI command"
        given: "A valid change ID and project path"
        when: "The cclab sdd restructure command is run"
        then: "The service logic is invoked and phase updates to restructure_completed"
        test_type: "integration"
    }

    element E2 {
        type: "test"
        id: test_json_file_payload
        text: "Test passing complex input via --json-file"
        given: "A JSON file containing complex nested arguments"
        when: "A CLI command is run with --json-file pointing to the file"
        then: "The CLI successfully parses the file and forwards it to the service"
        test_type: "unit"
    }

    element E3 {
        type: "test"
        id: test_json_output
        text: "Test JSON output formatting"
        given: "A valid CLI command"
        when: "The command is executed with the --json flag"
        then: "Stdout contains valid JSON describing the result"
        test_type: "unit"
    }

    E1 - satisfies -> R1
    E2 - satisfies -> R2
    E3 - satisfies -> R3
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - file: "various spec files under .aw/tech-design/crates/cclab-sdd/"
    action: modify
    section: scenarios
    impl_mode: hand-written
    description: |
      Category 1 — Frontmatter `files:` path updates: replace mcp/tools/ prefix with tools/.
      See legacy prose tables below for full mapping.

  - file: "various spec files"
    action: modify
    section: requirements
    impl_mode: hand-written
    description: |
      Category 2 — MCP terminology replacement across prose (titles, body text, tool names).

  - file: tools/utils/delegate-agent.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: |
      Category 3 — Remove mcp__cclab-mcp__sdd_delegate_agent disallowedTools pattern.

  - file: generate/template-claude-md.md
    action: modify
    section: scenarios
    impl_mode: hand-written
    description: |
      Category 3 — Remove/replace mcp__cclab__sdd_* tool reference patterns.

  - file: interfaces/cli/sdd-cli.md
    action: modify
    section: requirements
    impl_mode: hand-written
    description: |
      Category 4 — R5 reword from "Parity with MCP Execution" to "Unified Executor Logic".

  - file: generate/template-mcp-configs.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: |
      Category 5 — Archive (move to .aw/archive/ or add status: archived frontmatter).

  - file: generate/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: |
      Remove reference to template-mcp-configs.md if present.
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```

### Category 1: Frontmatter `files:` path updates

All paths: replace `mcp/tools/` prefix with `tools/`. Flatten subdirectory structure where source was reorganized.

| Spec File | Action |
|-----------|--------|
| `logic/state-machine.md` | `mcp/tools/phase_transition.rs` → `tools/phase_transition.rs` |
| `logic/executor-resolution.md` | `mcp/tools/workflow_common.rs` → `tools/workflow_common.rs` |
| `interfaces/workflow/implement.md` | legacy `mcp/tools/change_impl/{common,create,review,revise}.rs` → `src/workflow/implement.rs` |
| `logic/post-clarifications.md` | `mcp/tools/post_clarifications/{mod,create}.rs` → `tools/create_post_clarifications.rs` |
| `logic/pre-clarifications.md` | `mcp/tools/create_pre_clarifications.rs` → `tools/create_pre_clarifications.rs` |
| `tools/create_change_merge/workflow.md` | legacy `mcp/tools/change_merge/create.rs` → `src/tools/create_change_merge.rs` |
| `logic/change-spec.md` | `mcp/tools/change_spec/{common,create,review,revise}.rs` → `tools/{common_change_spec,create_change_spec,review_change_spec,revise_change_spec}.rs` |
| `logic/restructure-input.md` | `mcp/tools/restructure_input.rs` → `tools/restructure_input.rs` |
| `tools/utils/delegate-agent.md` | `mcp/tools/agent.rs` → `tools/agent.rs` |
| `tools/utils/write-artifact.md` | `mcp/tools/artifact_write.rs` → `tools/artifact_write.rs` |
| `tools/utils/read-artifact.md` | `mcp/tools/artifact_read.rs` → `tools/artifact_read.rs` |
| `tools/utils/fetch-issues.md` | `mcp/tools/fetch_issues.rs` → `tools/fetch_issues.rs` |
| `tools/utils/read-implementation-summary.md` | `mcp/tools/implementation.rs` → `tools/implementation.rs` |
| `tools/utils/list-changed-files.md` | `mcp/tools/implementation.rs` → `tools/implementation.rs` |
| `tools/utils/analyze-code-for-spec.md` | `mcp/tools/analyze/{mod,python,typescript,rust_lang,suggestions}.rs` → `tools/analyze/{mod,python,typescript,rust_lang,suggestions}.rs` |
| `tools/utils/platform-sync.md` | `mcp/tools/platform_sync.rs` → `tools/platform_sync.rs` |
| `tools/utils/validate-spec-completeness.md` | `mcp/tools/validate_spec.rs` → `tools/validate_spec.rs` |
| `tools/utils/validate-change.md` | `mcp/tools/validate.rs` → `tools/validate.rs` |

### Category 2: MCP terminology replacement

| Spec File | Change |
|-----------|--------|
| `interfaces/tools/artifact-tools.md` | Title: "MCP Artifact Tools" → "Artifact Tools" |
| `interfaces/tools/utility-tools.md` | Title: "MCP Utility Tools" → "Utility Tools" |
| `tools/utils/write-artifact.md` | "Unified MCP tool" → "Unified tool" |
| `tools/utils/read-implementation-summary.md` | "MCP tool that produces" → "Tool that produces" |
| `tools/utils/list-changed-files.md` | "MCP tool that lists" → "Tool that lists" |
| `tools/utils/validate-change.md` | "direct MCP tool invocation" → "direct tool invocation"; "MCP response" → "tool response" |
| `tools/utils/delegate-agent.md` | L12: "via MCP tool" → "via CLI command" |
| `skills/agent.md` | All "MCP tool" → "tool"; "unified MCP tool" → "unified tool" |
| `skills/run-change.md` | L18: "calling the `sdd_run_change` MCP tool" → "calling the `sdd_run_change` tool" |
| `skills/fillback.md` | L104: "Uses Lens MCP tools" → "Uses Lens tools" |
| `generate/template-claude-md.md` | "MCP tool pointers" → "tool pointers"; replace `mcp__cclab__sdd_*` references |
| `generate/template-knowledge-index.md` | L66: "via MCP" → "via CLI" |
| `generate/spec-model.md` | Replace any "MCP tool" terminology |
| `generate/requirement-plus-enhancement.md` | Replace any "MCP tool" terminology |
| `generate/spec-ir-evaluation.md` | Replace any "MCP tool" terminology |

### Category 3: `mcp__cclab-mcp__*` pattern removal

| Spec File | Change |
|-----------|--------|
| `tools/utils/delegate-agent.md` | Remove `mcp__cclab-mcp__sdd_delegate_agent` disallowedTools pattern (L184-185) |
| `generate/template-claude-md.md` | Remove/replace `mcp__cclab__sdd_*` tool reference patterns |

### Category 4: R5 reword in sdd-cli.md

| Section | Before | After |
|---------|--------|-------|
| Overview | "relies heavily on an MCP...server for execution" | "provides a native CLI interface for the full SDD workflow" |
| R5 title | "Parity with MCP Execution" | "Unified Executor Logic" |
| R5 body | "compared to the MCP tools" | "compared to the underlying executor functions" |

### Category 5: Obsolete spec archival

| Spec File | Action |
|-----------|--------|
| `generate/template-mcp-configs.md` | Archive — move to `.aw/archive/crates/cclab-sdd/generate/template-mcp-configs.md` or add `status: archived` to frontmatter |
| `generate/README.md` | Remove reference to `template-mcp-configs.md` if present |
