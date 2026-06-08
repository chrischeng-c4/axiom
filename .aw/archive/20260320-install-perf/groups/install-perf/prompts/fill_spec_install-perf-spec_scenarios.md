# Task: Fill Section 'scenarios' for Spec 'install-perf-spec' (Change 'install-perf')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec: `cclab/changes/install-perf/specs/install-perf-spec.md`
2. Write content for **scenarios**: Write acceptance scenarios:
### Scenario: Name
**GIVEN** precondition
**WHEN** action
**THEN** outcome
Begin with `<!-- type: scenarios lang: markdown -->`.
3. Write payload JSON then run: `cclab sdd artifact create-change-spec install-perf cclab/changes/install-perf/payloads/create-change-spec.json`