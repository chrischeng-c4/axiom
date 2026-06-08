# Task: Fill Section 'requirements' for Spec 'jet-aot-build-gaps-spec' (Change 'jet-aot-build-gaps')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec via `cclab sdd workflow read-artifact jet-aot-build-gaps` with scope="jet-aot-build-gaps-spec"
2. Write content for **requirements**: Write requirements in markdown:
### R1: Title

Description.

**Priority**: high/medium/low
Begin with `<!-- type: requirements lang: markdown -->`.
3. Write payload JSON to the change's payloads directory (do NOT write to repo root or CWD), then run: `cclab sdd artifact create-change-spec jet-aot-build-gaps <payload_path>`