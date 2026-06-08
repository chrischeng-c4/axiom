# Task: Post-Clarification for Change 'bug-init-change-writes-score-changes-to-main-repo-inst'

## Context Sources

Read these files before analysis:
1. `/Users/chrischeng/projects/cclab/.score/changes/bug-init-change-writes-score-changes-to-main-repo-inst/user_input.md`
2. `/Users/chrischeng/projects/cclab/.score/changes/bug-init-change-writes-score-changes-to-main-repo-inst/pre_clarifications.md`
3. `/Users/chrischeng/projects/cclab/.score/changes/bug-init-change-writes-score-changes-to-main-repo-inst/reference_context.md`
4. Actual specs — read high/medium relevance specs from reference_context.md

- List specs under `/Users/chrischeng/projects/cclab/.score/tech_design/` using Glob and read the most relevant ones

## Instructions

### Step 1: Systematic Contradiction Mining

For each high-relevance spec from reference_context.md:
1. Read the spec file
2. For each requirement, explicitly ask: "Does this spec define a convention or pattern that conflicts with this requirement?"
3. Look specifically for:
   - Naming conventions that differ from the user's proposal
   - Data formats or API patterns that would be inconsistent
   - Error handling approaches that conflict
   - Existing constraints that limit the proposed approach

### Step 2: Assumption Surfacing

List implicit assumptions from user input that the referenced specs don't address.

### Step 3: Scope Summary (MANDATORY)

Write a Scope Summary with cross-references:

- **Problem**: ref to user_input.md sections that define the gap
- **Success Criteria**: acceptance criteria + pre_clarifications answers that confirmed behavior
- **Boundary**: in scope, out of scope, constraints

Use → refs to point to specific sections — do NOT duplicate content.

### Step 4: Decision

- **No conflicts found** → Call artifact tool with `skipped: true` + `scope_summary`.
- **Conflicts found** → Use AskUserQuestion, then call artifact tool with resolved questions/contradictions + `scope_summary`.

## CLI Commands

```
score artifact create-post-clarifications bug-init-change-writes-score-changes-to-main-repo-inst .score/changes/bug-init-change-writes-score-changes-to-main-repo-inst/payloads/create-post-clarifications.json
```