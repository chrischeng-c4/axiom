# Task: Fill Section 'test_plan' for Spec 'jet-dev-server-v2-spec' (Change 'jet-dev-server-v2')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec via `cclab sdd workflow read-artifact jet-dev-server-v2` with scope="jet-dev-server-v2-spec"
2. Write content for **test_plan**: Define test cases using BDD Given/When/Then. Use sdd_generate_requirement_plus tool.
Begin with `<!-- type: test-plan lang: markdown -->`.
3. Write payload JSON to the change's payloads directory (do NOT write to repo root or CWD), then run: `cclab sdd artifact create-change-spec jet-dev-server-v2 <payload_path>`