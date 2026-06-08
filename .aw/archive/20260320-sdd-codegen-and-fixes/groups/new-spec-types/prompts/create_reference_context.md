# Task: Gather Reference Context for Group 'new-spec-types' (Change 'sdd-codegen-and-fixes')

Issues: #937_feat-sdd-frontend-codegen-wireframe-component-desi, #934_feat-sdd-deployment-spec-type-infra-as-code-integr

## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to create/modify artifact files directly.**
You MUST use the CLI command below to write the artifact. The system verifies
artifacts were written via CLI — direct file writes will be REJECTED and you
will have to redo the work.

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-and-fixes/groups/new-spec-types/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. **Write a JSON payload file** then run the CLI command below

## Suggested Sections (from requirements analysis)
Based on keyword analysis of requirements: [overview, db-model, schema, state-machine, logic, dependency, interaction, rest-api, cli, wireframe, component, design-token, config, test-plan, changes]
Use these as starting point for spec_plan.sections. Adjust based on your analysis.

## In-Scope Specs

### cclab-sdd
- `read_path:specs/cclab-sdd/README.md`
- `read_path:specs/cclab-sdd/sdd-cli.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-sdd/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Step 1: Write payload JSON file
Write file: cclab/changes/sdd-codegen-and-fixes/payloads/create-reference-context.json

# Step 2: Run artifact CLI (MUST use this — do NOT write reference_context.md directly)
cclab sdd artifact create-reference-context sdd-codegen-and-fixes cclab/changes/sdd-codegen-and-fixes/payloads/create-reference-context.json
```