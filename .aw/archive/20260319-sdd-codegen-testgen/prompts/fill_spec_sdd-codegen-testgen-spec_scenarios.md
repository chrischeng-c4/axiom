# Task: Fill Section 'scenarios' for Spec 'sdd-codegen-testgen-spec' (Change 'sdd-codegen-testgen')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec: `cclab/changes/sdd-codegen-testgen/specs/sdd-codegen-testgen-spec.md`
2. Write content for **scenarios**: Write acceptance scenarios:
### Scenario: Name
**GIVEN** precondition
**WHEN** action
**THEN** outcome
Begin with `<!-- type: scenarios lang: markdown -->`.
3. Write payload JSON then run: `cclab sdd artifact create-change-spec sdd-codegen-testgen cclab/changes/sdd-codegen-testgen/payloads/create-change-spec.json`