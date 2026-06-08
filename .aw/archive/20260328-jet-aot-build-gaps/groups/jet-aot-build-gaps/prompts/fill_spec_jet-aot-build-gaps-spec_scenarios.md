# Task: Fill Section 'scenarios' for Spec 'jet-aot-build-gaps-spec' (Change 'jet-aot-build-gaps')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec via `cclab sdd workflow read-artifact jet-aot-build-gaps` with scope="jet-aot-build-gaps-spec"
2. Write content for **scenarios**: Write acceptance scenarios:
### Scenario: Name
**GIVEN** precondition
**WHEN** action
**THEN** outcome
Begin with `<!-- type: scenarios lang: markdown -->`.
3. Write payload JSON to the change's payloads directory (do NOT write to repo root or CWD), then run: `cclab sdd artifact create-change-spec jet-aot-build-gaps <payload_path>`