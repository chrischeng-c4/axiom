# Task: Analyze Spec 'sdd-merge-gaps-fix-spec' for Change 'sdd-merge-gaps-fix'

A skeleton has been generated at `.score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md`.

## CRITICAL: Artifact Writing Rule

**DO NOT use Write or Edit tools to modify spec files directly.**
You MUST use the artifact CLI command to write each section.
Direct file writes will be REJECTED and you will have to redo the work.

## Instructions

1. Read context:
   - Read file: `.score/changes/sdd-merge-gaps-fix/proposal.md` for spec_plan routing
   - Read file: `.score/changes/sdd-merge-gaps-fix/reference_context.md` if no proposal
2. Read the skeleton: `.score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md`
3. **IMPORTANT — `main_spec_ref`**: Check the spec frontmatter. If `main_spec_ref` is `~` (null),
   you MUST determine the target path in `.score/tech_design/` where this spec will be merged.
   Format: `<scope>/<category>/<spec-id>.md` (e.g., `sdd/tools/new-feature.md`).
   Browse `.score/tech_design/` to see existing spec groups.
   Pass it as the `main_spec_ref` parameter when calling the artifact CLI.
4. Decide which sections to fill based on the nature of the change:
   - **overview** — always fill
   - **requirements** — always fill
   - **scenarios** — always fill
   - **diagrams** — fill if visual representation helps (API flows, data models, state machines)
   - **api_spec** — fill if change involves HTTP/RPC/event-driven/workflow APIs
   - **test_plan** — fill to define test cases (use Mermaid+ requirement diagram with BDD Given/When/Then)
   - **changes** — fill to list affected files
5. Write a JSON payload file to `.score/changes/sdd-merge-gaps-fix/payloads/create-change-spec.json` then run the artifact CLI.

## Expected Action

Write the **overview** section first via artifact CLI. Pass the `fill_sections`
array as a parameter (e.g., `fill_sections=["overview", "requirements", "scenarios"]`).
Also pass `main_spec_ref` as a parameter if determined above.
The system persists it to frontmatter automatically.

Then call the artifact CLI for each remaining section in sequence.

## CLI Commands

```
# Read artifacts
Read file: .score/changes/sdd-merge-gaps-fix/proposal.md
Read file: .score/changes/sdd-merge-gaps-fix/specs/sdd-merge-gaps-fix-spec.md

# Write each section (MUST use this — do NOT edit spec files directly)
# Step 1: Write payload JSON to the EXACT path below (do NOT write to other locations)
# Step 2: Run artifact CLI
score artifact create-change-spec sdd-merge-gaps-fix .score/changes/sdd-merge-gaps-fix/payloads/create-change-spec.json
```
