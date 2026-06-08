# Task: Fill Section 'overview' for Spec 'jet-aot-build-gaps-spec' (Change 'jet-aot-build-gaps')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec via `cclab sdd workflow read-artifact jet-aot-build-gaps` with scope="jet-aot-build-gaps-spec"
2. Write content for **overview**: Write a comprehensive overview (>= 50 chars) describing what this spec covers.
Begin with `<!-- type: overview lang: markdown -->` on its own line.
3. Write payload JSON to the change's payloads directory (do NOT write to repo root or CWD), then run: `cclab sdd artifact create-change-spec jet-aot-build-gaps <payload_path>`