# Task: Analyze Spec 'enhancement-auto-inject-page-fixture-for-playwright-compatible-spec' for Change 'enhancement-auto-inject-page-fixture-for-playwright-compatible'

A skeleton has been generated at `.score/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md`.

## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to modify spec files directly.**
You MUST use the artifact CLI command to write each section.
Direct file writes will be REJECTED and you will have to redo the work.

## Instructions

1. Read context:
   - Read the issue file in `.score/issues/open/` that initiated this change (see user_input.md for the slug)
   - The issue's ## Problem, ## Requirements, ## Scope, and ## Reference Context sections are your primary context
2. Read the skeleton: `.score/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md`
3. **IMPORTANT — `main_spec_ref`**: Check the spec frontmatter. If `main_spec_ref` is `~` (null),
   you MUST determine the target path in `.score/tech_design/` where this spec will be merged.
   Format: `<scope>/<category>/<spec-id>.md` (e.g., `sdd/tools/new-feature.md`).
   Browse `.score/tech_design/` to see existing spec groups.
   Pass it as the `main_spec_ref` parameter when calling the artifact CLI.
4. Decide which sections to fill based on the nature of the change. Pick ONLY leaf section names from this list — NEVER pass umbrella words like `diagrams`, `api_spec`, or `test_plan`:
   Always fill: `overview`, `requirements`, `scenarios`, `changes`
   Diagrams (pick those that apply): `interaction`, `logic`, `state-machine`, `mindmap`, `dependency`, `db-model`
   API shape (pick those that apply): `rest-api`, `rpc-api`, `async-api`, `cli`, `schema`, `config`
   UI (pick those that apply): `wireframe`, `component`, `design-token`
   Testing: `test-plan` (Mermaid+ requirement diagram with BDD Given/When/Then)
   Docs: `doc`
5. Write a JSON payload file to `.score/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/payloads/create-change-spec.json` then run the artifact CLI.

## Expected Action

Write the **overview** section first via artifact CLI. Pass the `fill_sections`
array as a parameter — USE LEAF NAMES ONLY from the allowed list above.
Example (adapt to this change): `fill_sections=["overview", "requirements", "scenarios", "interaction", "logic", "changes"]`.
Never pass `diagrams`, `api_spec`, or `test_plan` (umbrella names).
Also pass `main_spec_ref` as a parameter if determined above.
The system persists it to frontmatter automatically.

Then call the artifact CLI for each remaining section in sequence.

## CLI Commands

```
# Read artifacts
Read file: .score/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/proposal.md
Read file: .score/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/specs/enhancement-auto-inject-page-fixture-for-playwright-compatible-spec.md

# Write each section (MUST use this — do NOT edit spec files directly)
# Step 1: Write payload JSON to the EXACT path below (do NOT write to other locations)
# Step 2: Run artifact CLI
score artifact create-change-spec enhancement-auto-inject-page-fixture-for-playwright-compatible .score/changes/enhancement-auto-inject-page-fixture-for-playwright-compatible/payloads/create-change-spec.json
```
