---
number: 933
title: "feat(sdd): test generation from RequirementPlus specs"
state: open
labels: [enhancement, P1, crate:sdd]
group: "specir-and-test-codegen"
---

# #933 — feat(sdd): test generation from RequirementPlus specs

## Summary

RequirementPlus schema already has BDD-style fields (`given`, `when`, `then`, `test_type`, `verification`) and requirement-element traceability (`Satisfies`, `Verifies`, `Refines`), but no generator consumes them to produce actual test files. The test-plan section creates a requirementDiagram — a map of what to test — but never scaffolds the tests.

## Current State

**What exists:**
- `RequirementDefPlus` with `risk`, `verification` (Analysis/Inspection/Test/Demonstration)
- `ElementDef` with `test_type` (unit/integration/e2e), BDD `given`/`when`/`then`
- `ReqRelationshipDef` with `Satisfies`/`Verifies`/`Refines`/`Traces`
- Mermaid requirementDiagram rendering

**What's missing:**
- Test file scaffold generation (e.g., `#[test]` / `def test_` / `it("should...")`)
- Assertion content derived from `then` field
- Test-to-spec traceability comments in generated tests
- Integration with `cclab-probe` (referenced in draft `test-generation.md` spec)
- CI gate: "all requirements have at least one Verifies element"

## Proposal

### Test scaffold from RequirementPlus

```
RequirementPlus spec
    ↓
ElementDef { test_type: "unit", given: "user exists", when: "login called", then: "returns token" }
    ↓
#[test]
fn test_login_returns_token() {
    // Given: user exists
    let user = create_test_user();
    // When: login called
    let result = login(&user);
    // Then: returns token
    assert!(result.token.is_some());
}
```

### Coverage validation

- Parse RequirementPlus from spec
- Check every `Requirement` has at least one `Verifies` relationship to an `Element`
- Report uncovered requirements as warnings (or errors in strict mode)

## Acceptance Criteria

- [ ] Generator produces test file skeleton from RequirementPlus
- [ ] BDD given/when/then fields map to test structure
- [ ] Coverage report: requirements without Verifies relationships flagged
- [ ] At least Rust target (`#[test]`) working
