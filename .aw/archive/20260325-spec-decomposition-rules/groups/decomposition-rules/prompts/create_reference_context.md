# Task: Gather Reference Context for Group 'decomposition-rules' (Change 'spec-decomposition-rules')


## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to create/modify artifact files directly.**
You MUST use the CLI command below to write the artifact. The system verifies
artifacts were written via CLI — direct file writes will be REJECTED and you
will have to redo the work.


## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/spec-decomposition-rules/groups/decomposition-rules/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. **Write a JSON payload file** then run the CLI command below

## Suggested Sections (from requirements analysis)
Based on keyword analysis of requirements: [overview, schema, prompt, logic, dependency, interaction, rest-api, wireframe, component, test-plan, changes]
Use these as starting point for spec_plan.sections. Adjust based on your analysis.

## Output: spec_plan array

For each change spec that will be created:
- spec_id: identifier for the new change spec
- action: "modify" (copy existing) or "create" (new skeleton)
- main_spec_ref: target path in cclab/specs/ (REQUIRED — must include a named subfolder,
e.g. `crates/cclab-sdd/logic/foo.md`, not `crates/cclab-sdd/foo.md`)
- source: path of existing spec to copy (only for "modify")
- sections: array of section types this spec needs (see change-spec.md § Section Selection)

**Action preference**: Use `action: modify` for any file visible in the spec directory tree
above. Reserve `action: create` for genuinely new subsystems with no existing spec file.

## Specs

- List specs under `/Users/chris.cheng/cclab/cclab-sdd/cclab/specs/` using Glob
- Read at most 5 specs. Focus on the most relevant ones.

## CLI Commands

```
# Step 1: Write payload JSON file
Write file: cclab/changes/spec-decomposition-rules/payloads/create-reference-context.json

# Step 2: Run artifact CLI (MUST use this — do NOT write reference_context.md directly)
cclab sdd artifact create-reference-context spec-decomposition-rules cclab/changes/spec-decomposition-rules/payloads/create-reference-context.json
```