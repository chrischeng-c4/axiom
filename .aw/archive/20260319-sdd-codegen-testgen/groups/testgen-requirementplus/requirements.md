---
change: sdd-codegen-testgen
group: testgen-requirementplus
date: 2026-03-19
---

# Requirements

RequirementPlus schema has BDD-style fields (`given`, `when`, `then`, `test_type`, `verification`) and traceability relationships (`Satisfies`, `Verifies`, `Refines`, `Traces`), but no generator produces actual test files from them. This group implements the test generation pipeline:

1. **Test scaffold generator**: Consume `RequirementPlus` spec and emit test file skeletons. Map `ElementDef` fields to test structure: `test_type` selects framework (unit/integration/e2e), `given`/`when`/`then` map to setup/action/assertion comments (and optionally assertion stubs). Traceability comment links generated test back to the spec element.

2. **Rust target (#[test])**: First supported output language. Example output:
   ```rust
   #[test]
   fn test_login_returns_token() {
       // Given: user exists
       // When: login called
       // Then: returns token
       todo!("implement")
   }
   ```

3. **Coverage validation**: Parse RequirementPlus and check every `Requirement` node has at least one `Verifies` relationship to an `Element`. Report uncovered requirements as warnings (or errors in strict mode). Suitable as a CI gate.

4. **cclab-probe integration** (if in scope): Hook generated tests or coverage report into cclab-probe.

**Acceptance criteria**: Generator produces test file skeleton from RequirementPlus. BDD given/when/then fields map to test structure. Coverage report flags requirements without Verifies relationships. At least Rust target working.
