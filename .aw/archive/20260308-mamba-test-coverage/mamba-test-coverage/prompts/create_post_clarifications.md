# Task: Post-Clarification for Group 'codegen-hir-mir-coverage' (Change 'mamba-test-coverage')

Issues: #744_test-coverage-codegen-jit-aot-llvm-target-95-98-li, #743_test-coverage-hir-mir-lowering-target-95-98-line-c, #747_test-coverage-name-resolution-target-95-98-line-co

## Context Sources

Read these files before analysis:
1. `/Users/chrischeng/projects/cclab/cclab/changes/mamba-test-coverage/groups/codegen-hir-mir-coverage/requirements.md`
2. `/Users/chrischeng/projects/cclab/cclab/changes/mamba-test-coverage/groups/codegen-hir-mir-coverage/pre_clarifications.md`
3. `/Users/chrischeng/projects/cclab/cclab/changes/mamba-test-coverage/groups/codegen-hir-mir-coverage/reference_context.md`
4. Actual specs — read high/medium relevance specs from reference_context.md

- Read high/medium relevance specs listed in reference_context.md (under `/Users/chrischeng/projects/cclab/cclab/specs/`)

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

## MCP Tools

```
mcp__cclab-mcp__sdd_artifact_create_post_clarifications(project_path="/Users/chrischeng/projects/cclab", change_id="mamba-test-coverage", group_id="codegen-hir-mir-coverage", skipped=true)
```

Or with clarifications:

```
mcp__cclab-mcp__sdd_artifact_create_post_clarifications(project_path="/Users/chrischeng/projects/cclab", change_id="mamba-test-coverage", group_id="codegen-hir-mir-coverage", questions=[{"topic": "...", "question": "...", "answer": "...", "rationale": "..."}], contradictions=[{"spec_id": "...", "requirement": "...", "conflict": "...", "resolution": "..."}])
```