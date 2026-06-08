# Task: Fill Section 'openrpc' for Spec 'merge-tool-openrpc' (Change '1136')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec via `cclab sdd workflow read-artifact 1136` with scope="merge-tool-openrpc"
2. Write content for **openrpc**: Write OpenRPC 1.3 JSON. Begin with `<!-- type: rpc-api lang: json -->`.
3. Write payload JSON to the change's payloads directory (do NOT write to repo root or CWD), then run: `cclab sdd artifact create-change-spec 1136 <payload_path>`