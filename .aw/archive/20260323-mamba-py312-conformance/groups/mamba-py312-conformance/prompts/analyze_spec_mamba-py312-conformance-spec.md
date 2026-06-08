# Task: Analyze Spec 'mamba-py312-conformance-spec' for Change 'mamba-py312-conformance'

A skeleton has been generated. Find it via `cclab sdd workflow read-artifact mamba-py312-conformance`.

## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to modify spec files directly.**
You MUST use the artifact CLI command to write each section.
Direct file writes will be REJECTED.

## Instructions

1. Read context from `cclab/changes/mamba-py312-conformance/proposal.md` or `reference_context.md`
2. Read the skeleton via `cclab sdd workflow read-artifact mamba-py312-conformance` with scope="mamba-py312-conformance-spec"
3. Determine `main_spec_ref` (target path in `cclab/specs/`) and `merge_strategy`
4. Decide which sections to fill (overview, requirements, scenarios are mandatory)
5. Write the **overview** section first via artifact CLI with `fill_sections` param

Then call the artifact CLI for each remaining section in sequence.

## CLI Commands

```
# Write payload JSON to the EXACT path passed as argument (do NOT write to other locations), then run:
cclab sdd artifact create-change-spec mamba-py312-conformance <payload_path>
```
