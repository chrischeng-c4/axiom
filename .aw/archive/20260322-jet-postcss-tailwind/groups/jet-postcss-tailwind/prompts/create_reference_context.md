# Task: Gather Reference Context for Group 'jet-postcss-tailwind' (Change 'jet-postcss-tailwind')

Issues: #1029_feat-jet-postcss-pipeline-tailwind-css-jit-support

## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to create/modify artifact files directly.**
You MUST use the CLI command below to write the artifact. The system verifies
artifacts were written via CLI — direct file writes will be REJECTED and you
will have to redo the work.

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chrischeng/projects/cclab-sdd/cclab/changes/jet-postcss-tailwind/groups/jet-postcss-tailwind/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. **Write a JSON payload file** then run the CLI command below

## Suggested Sections (from requirements analysis)
Based on keyword analysis of requirements: [overview, state-machine, logic, dependency, interaction, wireframe, component, design-token, config, test-plan, changes]
Use these as starting point for spec_plan.sections. Adjust based on your analysis.

## In-Scope Specs

### cclab-jet
- `read_path:specs/cclab-jet/aot-build.md`
- `read_path:specs/cclab-jet/bundle-optimization-hoisting.md`
- `read_path:specs/cclab-jet/jet-remaining-spec.md`
- `read_path:specs/cclab-jet/jit-runner.md`
- `read_path:specs/cclab-jet/nx-support.md`
- `read_path:specs/cclab-jet/pkg-manager.md`
- `read_path:specs/cclab-jet/pkg-manager-pnpm-parity.md`
- `read_path:specs/cclab-jet/scope-hoisting.md`
- `read_path:specs/cclab-jet/tree-shaking.md`
- `read_path:specs/cclab-jet/variable-mangling.md`


Read these specs using the Read tool (file paths under `/Users/chrischeng/projects/cclab-sdd/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Step 1: Write payload JSON file
Write file: cclab/changes/jet-postcss-tailwind/payloads/create-reference-context.json

# Step 2: Run artifact CLI (MUST use this — do NOT write reference_context.md directly)
cclab sdd artifact create-reference-context jet-postcss-tailwind cclab/changes/jet-postcss-tailwind/payloads/create-reference-context.json
```