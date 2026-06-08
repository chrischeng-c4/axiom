# Task: Analyze Spec 'scheduler-delay' for Change 'queue-test-coverage'

A skeleton has been generated. Find it via `cclab sdd workflow read-artifact queue-test-coverage`.

## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to modify spec files directly.**
You MUST use the artifact CLI command to write each section.
Direct file writes will be REJECTED.

## Instructions

1. Read context from `cclab/changes/queue-test-coverage/proposal.md` or `reference_context.md`
2. Read the skeleton via `cclab sdd workflow read-artifact queue-test-coverage` with scope="scheduler-delay"
3. Determine `main_spec_ref` (target path in `cclab/specs/`)
4. Decide which sections to fill (overview, requirements, scenarios are mandatory)
5. Write the **overview** section first via artifact CLI with `fill_sections` param

Then call the artifact CLI for each remaining section in sequence.

## CLI Commands

```
# Write payload JSON to the EXACT path passed as argument (do NOT write to other locations), then run:
cclab sdd artifact create-change-spec queue-test-coverage <payload_path>
```
