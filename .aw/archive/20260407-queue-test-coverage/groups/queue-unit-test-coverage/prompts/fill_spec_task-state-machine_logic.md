# Task: Fill Section 'logic' for Spec 'task-state-machine' (Change 'queue-test-coverage')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec via `cclab sdd workflow read-artifact queue-test-coverage` with scope="task-state-machine"
2. Write content for **logic**: Draw a Mermaid flowchart. Begin with `<!-- type: logic lang: mermaid -->`.
3. Write payload JSON to the change's payloads directory (do NOT write to repo root or CWD), then run: `cclab sdd artifact create-change-spec queue-test-coverage <payload_path>`