---
id: folder-agent-artifact-stability
summary: External contract for Folder Agent Artifact Stability.
fill_sections: [e2e-test]
---

# EC: Folder Agent Artifact Stability

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: folder-agent-artifact-stability
    capability_id: terminal-first-agent-workbench
    claim_id: folder-agent-artifact-production-stability
    contract_id: folder-agent-artifact-stability
    category: stability
    command: "cargo test -p workbench --test production_journey -- --nocapture"
    assertions:
      - "Twelve consecutive production Tauri IPC sessions each complete real PTY launch, input, resize, and one of interrupt, terminate, or normal-exit lifecycle modes."
      - "Every child process id observed at launch is reaped and no longer alive after the cycle; the selected canonical folder remains unchanged."
      - "Unavailable-agent recovery precedes a successful real launch, and repeated sessions retain Git, Markdown, and AW context behavior without leaked session state."
      - "Every session transcript remains at or below the 524288-byte production bound, with measured peak and lifecycle modes retained in ipc-journey.json."
```
