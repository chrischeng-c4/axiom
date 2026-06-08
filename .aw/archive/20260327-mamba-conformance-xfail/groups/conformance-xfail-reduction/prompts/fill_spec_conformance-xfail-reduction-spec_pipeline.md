# Task: Fill Section 'pipeline' for Spec 'conformance-xfail-reduction-spec' (Change 'mamba-conformance-xfail')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec via `cclab sdd workflow read-artifact mamba-conformance-xfail` with scope="conformance-xfail-reduction-spec"
2. Write content for **pipeline**: Fill in this section with appropriate content.
3. Write payload JSON to the change's payloads directory (do NOT write to repo root or CWD), then run: `cclab sdd artifact create-change-spec mamba-conformance-xfail <payload_path>`