# Task: Gather Reference Context for Group 'mamba-core-test-coverage' (Change 'mamba-core-test-coverage')

Issues: #1035_test-mamba-per-module-test-coverage-gaps-lower-res

## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to create/modify artifact files directly.**
You MUST use the CLI command below to write the artifact. The system verifies
artifacts were written via CLI — direct file writes will be REJECTED and you
will have to redo the work.

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chrischeng/projects/cclab-sdd/cclab/changes/mamba-core-test-coverage/groups/mamba-core-test-coverage/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. **Write a JSON payload file** then run the CLI command below

## Suggested Sections (from requirements analysis)
Based on keyword analysis of requirements: [overview, async-api, changes]
Use these as starting point for spec_plan.sections. Adjust based on your analysis.

## Specs

- List specs under `/Users/chrischeng/projects/cclab-sdd/cclab/specs/` using Glob
- Read at most 5 specs. Focus on the most relevant ones.

## CLI Commands

```
# Step 1: Write payload JSON file
Write file: cclab/changes/mamba-core-test-coverage/payloads/create-reference-context.json

# Step 2: Run artifact CLI (MUST use this — do NOT write reference_context.md directly)
cclab sdd artifact create-reference-context mamba-core-test-coverage cclab/changes/mamba-core-test-coverage/payloads/create-reference-context.json
```