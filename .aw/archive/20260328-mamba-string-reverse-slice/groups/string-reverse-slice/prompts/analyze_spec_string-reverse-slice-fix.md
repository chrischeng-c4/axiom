# Task: Analyze Spec 'string-reverse-slice-fix' for Change 'mamba-string-reverse-slice'

A skeleton has been generated. Find it via `cclab sdd workflow read-artifact mamba-string-reverse-slice`.

## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to modify spec files directly.**
You MUST use the artifact CLI command to write each section.
Direct file writes will be REJECTED.

## Instructions

1. Read context from `cclab/changes/mamba-string-reverse-slice/proposal.md` or `reference_context.md`
2. Read the skeleton via `cclab sdd workflow read-artifact mamba-string-reverse-slice` with scope="string-reverse-slice-fix"
3. Determine `main_spec_ref` (target path in `cclab/specs/`)
4. Decide which sections to fill (overview, requirements, scenarios are mandatory)
5. Write the **overview** section first via artifact CLI with `fill_sections` param

Then call the artifact CLI for each remaining section in sequence.

## CLI Commands

```
# Write payload JSON to the EXACT path passed as argument (do NOT write to other locations), then run:
cclab sdd artifact create-change-spec mamba-string-reverse-slice <payload_path>
```
