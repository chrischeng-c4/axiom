# Task: Post-Clarification for Group 'conformance-xfail-reduction' (Change 'mamba-conformance-xfail')

## Context Sources

Read these files before analysis:
1. `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-conformance-xfail/groups/conformance-xfail-reduction/requirements.md`
2. `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-conformance-xfail/groups/conformance-xfail-reduction/pre_clarifications.md`
3. `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-conformance-xfail/groups/conformance-xfail-reduction/reference_context.md`
4. Actual specs — read high/medium relevance specs from reference_context.md

- List specs under `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/specs/` using Glob and read the most relevant ones

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

### Step 3: Scope Summary (MANDATORY)

Whether or not contradictions were found, write a Scope Summary with cross-references:

- **Problem**: ref to requirements.md sections that define the gap (e.g., \"→ requirements.md § R1-R3\")
- **Success Criteria**: ref to requirements.md acceptance criteria + pre_clarifications answers that confirmed behavior
- **Boundary**: in scope (ref to spec_plan entries), out of scope (ref to clarification answers that excluded things), constraints (ref to contradiction resolutions)

Use → refs to point to specific sections — do NOT duplicate content.

### Step 4: Decision

- **No conflicts found** after systematic check → Call artifact tool with `skipped: true` + `scope_summary`. Do NOT force unnecessary Q&A.
- **Conflicts found** → Use AskUserQuestion with concrete options, then call artifact tool with resolved questions/contradictions + `scope_summary`.

## CLI Commands

```
# Skip-fast path (no clarifications needed)
cclab sdd artifact create-post-clarifications mamba-conformance-xfail cclab/changes/mamba-conformance-xfail/payloads/create-post-clarifications.json

# With clarifications (write payload JSON with questions/contradictions first, then run)
cclab sdd artifact create-post-clarifications mamba-conformance-xfail cclab/changes/mamba-conformance-xfail/payloads/create-post-clarifications.json
```