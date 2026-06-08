# Task: Post-Clarification for Group 'new-section-types' (Change 'section-type-coverage')

Issues: #1053_sdd-add-e2e-scenario-section-type-for-qa, #1055_sdd-add-qa-section-types-test-fixture-perf-test, #1051_epic-sdd-section-type-coverage-all-roles-fe-be-sre, #1057_sdd-add-backend-mle-agent-section-types-grpc-graph, #1056_sdd-add-sre-section-types-container-deploy-cloud-r, #1054_sdd-add-security-section-types-threat-model-auth-m

## Context Sources

Read these files before analysis:
1. `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/section-type-coverage/groups/new-section-types/requirements.md`
2. `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/section-type-coverage/groups/new-section-types/pre_clarifications.md`
3. `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/section-type-coverage/groups/new-section-types/reference_context.md`
4. Actual specs — read high/medium relevance specs from reference_context.md

- Read high/medium relevance specs listed in reference_context.md (under `/Users/chris.cheng/cclab/cclab-sdd/cclab/specs/`)

## Instructions

### Step 1: Systematic Contradiction Mining

For each high-relevance spec from reference_context.md:
1. Read the spec file
2. For each requirement in requirements.md, explicitly ask: "Does this spec define a convention or pattern that conflicts with this requirement?"
3. Look specifically for:
   - Naming conventions that differ from the user's proposal
   - Data formats or API patterns that would be inconsistent
   - Error handling approaches that conflict
   - Existing constraints that limit the proposed approach

### Step 2: Assumption Surfacing

List implicit assumptions from requirements.md that the referenced specs don't address:
- What does the user assume about error handling that specs don't define?
- What backward compatibility assumptions exist?
- What edge cases are not mentioned in either requirements or specs?

### Step 3: Decision

- **No conflicts found** after systematic check → Call artifact tool with `skipped: true`. Do NOT force unnecessary Q&A.
- **Conflicts found** → Use AskUserQuestion with concrete options (e.g., "Spec X uses pattern A, but you propose pattern B. Which should we use?" with specific choices), then call artifact tool with resolved questions/contradictions.

## CLI Commands

```
# Skip-fast path (no clarifications needed)
cclab sdd artifact create-post-clarifications section-type-coverage cclab/changes/section-type-coverage/payloads/create-post-clarifications.json

# With clarifications (write payload JSON with questions/contradictions first, then run)
cclab sdd artifact create-post-clarifications section-type-coverage cclab/changes/section-type-coverage/payloads/create-post-clarifications.json
```