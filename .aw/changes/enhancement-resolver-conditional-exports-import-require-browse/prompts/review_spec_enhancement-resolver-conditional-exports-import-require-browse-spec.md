# Task: Review Spec 'enhancement-resolver-conditional-exports-import-require-browse-spec' for Change 'enhancement-resolver-conditional-exports-import-require-browse'

## Instructions

1. **Run automated validation**:
   `score workflow validate-spec-completeness enhancement-resolver-conditional-exports-import-require-browse --spec-id enhancement-resolver-conditional-exports-import-require-browse-spec`
2. **Read the spec**:
   `.aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md`
3. **Read the proposal** for context routing
4. **Evaluate against checklist**:
   - Overview is substantive (>= 50 chars)
   - Requirements are well-defined with IDs and descriptions
   - At least one scenario per requirement
   - Diagrams are relevant and correct (if present)
   - API specs are valid (if present)
   - Changes list covers all affected files
   - No duplicate section types in this spec file
   - Sections follow dependency order: data → behavior → interface → test → changes
5. **CRITICAL: List ALL issues in a single pass.** You only get ONE review round before auto-approve. Report every problem NOW — do not hold back issues for a future round.
6. **Determine verdict**: APPROVED / REVIEWED / REJECTED
7. **Identify problem sections**: If not APPROVED, list which sections need work
8. Write the review

## Verdict Guidelines

- **APPROVED**: Passes all checklist items, spec is implementation-ready
- **REVIEWED**: Missing elements, unclear requirements, or insufficient scenarios
- **REJECTED**: Fundamental design problems, wrong approach

## CLI Commands

```
# Validate spec completeness
score workflow validate-spec-completeness enhancement-resolver-conditional-exports-import-require-browse --spec-id enhancement-resolver-conditional-exports-import-require-browse-spec

# Read spec
Read file: .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md

# Write review (write payload JSON first, then run)
score artifact review-change-spec enhancement-resolver-conditional-exports-import-require-browse .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/payloads/review-change-spec.json
```