# Task: Fill Section 'test-plan' for Spec 'result-backend' (Change 'queue-test-coverage')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec via `cclab sdd workflow read-artifact queue-test-coverage` with scope="result-backend"
2. Write content for **test-plan**: Define test cases using BDD Given/When/Then. Use sdd_generate_requirement_plus tool.
Begin with `<!-- type: test-plan lang: markdown -->`.
3. Write payload JSON to the change's payloads directory (do NOT write to repo root or CWD), then run: `cclab sdd artifact create-change-spec queue-test-coverage <payload_path>`