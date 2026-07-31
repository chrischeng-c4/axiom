---
id: folder-agent-artifact-efficiency
summary: External contract for Folder Agent Artifact Efficiency.
fill_sections: [e2e-test]
---

# EC: Folder Agent Artifact Efficiency

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: folder-agent-artifact-efficiency
    capability_id: terminal-first-agent-workbench
    claim_id: folder-agent-artifact-production-efficiency
    contract_id: folder-agent-artifact-efficiency
    category: efficiency
    command: "cargo test -p workbench --test production_journey -- --nocapture"
    assertions:
      - "Across the production Tauri IPC to real-PTY boundary, deterministic agent launch-to-OSC7-ready latency is at most 2000 milliseconds."
      - "Peak resident memory for the complete production_journey test process is at most 524288 KiB, measured by getrusage rather than reported by Workbench."
      - "The retained ipc-journey.json records measured and limit values under schema workbench.production-ipc.evidence.v1 and the gate rejects either limit violation."
```
