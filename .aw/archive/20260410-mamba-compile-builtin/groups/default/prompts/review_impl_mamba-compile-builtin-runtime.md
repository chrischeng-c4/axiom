# Task: Review Implementation of Spec 'mamba-compile-builtin-runtime' for Change 'mamba-compile-builtin'

## Pre-Review Step (MANDATORY)

Before evaluating any checklist items:
1. Read spec: `.score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md`
2. Find the `## Test Plan` section (if present) and note whether it exists and how many test cases it defines.

## Alignment Report

| File | Kind | Message |
|------|------|---------|
| /Users/chris.cheng/cclab/main/.score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md | missing_section_annotation | Section 'Overview' at line 12 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md | missing_section_annotation | Section 'Requirements' at line 28 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md | missing_section_annotation | Section 'Diagrams' at line 88 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md | missing_section_annotation | Section 'API Spec' at line 170 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md | missing_section_annotation | Section 'Changes' at line 236 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chris.cheng/cclab/main/.score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |

## Instructions

3. Read implementation diff: `.score/changes/mamba-compile-builtin/implementation.md`
4. List changed files via `score workflow list-changed-files mamba-compile-builtin`
5. Review code changes against spec requirements
6. Evaluate ALL checklist items below
7. **CRITICAL: List ALL issues in a single pass.** You only get ONE review round before auto-approve. Report every problem NOW.
8. Write review via the artifact CLI command

## Checklist

### Hard Checklist (MUST ALL PASS for APPROVED)

- [HARD] Code matches all spec requirements
- [HARD] If spec has `## Test Plan` section: diff contains at least one `#[test]` function
- [HARD] Existing tests still pass (no regressions introduced)

### Soft Checklist (Issues → REVIEWED verdict)

- Code quality and readability
- Error handling completeness
- Performance considerations
- Documentation where needed

## HARD REJECT RULE

**IF** the spec has a `## Test Plan` section
**AND** the implementation diff contains zero `#[test]` or `#[cfg(test)]` blocks
**THEN** verdict MUST be `REJECTED` — no exceptions, regardless of other checklist results.

This rule overrides all other considerations.

## Verdict Guidelines

- **APPROVED**: All hard checklist items pass, code matches spec, tests pass
- **REVIEWED**: Hard checklist passes but has fixable soft issues
- **REJECTED**: Any hard checklist item fails (including the hard reject rule above)

## CLI Commands

```
# Read spec and implementation
Read file: .score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md
Read file: .score/changes/mamba-compile-builtin/implementation.md

# List changed files
score workflow list-changed-files mamba-compile-builtin

# Write review (write payload JSON first, then run)
score artifact review-change-implementation mamba-compile-builtin .score/changes/mamba-compile-builtin/payloads/review-change-implementation.json
```