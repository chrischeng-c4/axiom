# Task: Analyze Spec 'change-spec-logic' for Change 'sdd-codegen-and-fixes'

A skeleton has been generated at `specs/change-spec-logic.md`.

## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to modify spec files directly.**
You MUST use the artifact CLI command to write each section.
Direct file writes will be REJECTED.

## Instructions

1. Read context from `cclab/changes/sdd-codegen-and-fixes/proposal.md` or `reference_context.md`
2. Read the skeleton: `cclab/changes/sdd-codegen-and-fixes/specs/change-spec-logic.md`
3. Determine `main_spec_ref` (target path in `cclab/specs/`) and `merge_strategy`
4. Decide which sections to fill (overview, requirements, scenarios are mandatory)
5. Write the **overview** section first via artifact CLI with `fill_sections` param

Then call the artifact CLI for each remaining section in sequence.

## CLI Commands

```
# Write payload JSON file, then run:
cclab sdd artifact create-change-spec sdd-codegen-and-fixes cclab/changes/sdd-codegen-and-fixes/payloads/create-change-spec.json
```
