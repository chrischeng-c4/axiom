# Task: Fill Section 'test_plan' for Spec 'mamba-py312-conformance-spec' (Change 'mamba-py312-conformance')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec via `cclab sdd workflow read-artifact mamba-py312-conformance` with scope="mamba-py312-conformance-spec"
2. Write content for **test_plan**: Define test cases using BDD Given/When/Then. Use sdd_generate_requirement_plus tool.
Begin with `<!-- type: test-plan lang: markdown -->`.
3. Write payload JSON to the change's payloads directory (do NOT write to repo root or CWD), then run: `cclab sdd artifact create-change-spec mamba-py312-conformance <payload_path>`