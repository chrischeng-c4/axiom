<!-- HANDWRITE-BEGIN gap="missing-generator:unit-test:c9775542" tracker="pending-tracker" reason="Bind the release external contract to the same production_journey Cargo command and retained evidence schema." -->
---
id: folder-agent-artifact-journey
summary: External contract for the complete Workbench folder-to-agent-to-artifact journey.
fill_sections: [e2e-test]
---

# EC: Folder Agent Artifact Journey

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: folder-agent-artifact-journey
    capability_id: terminal-first-agent-workbench
    claim_id: folder-agent-artifact-production-journey
    contract_id: folder-agent-artifact-journey
    category: behavior
    command: "cargo test -p workbench --test production_journey -- --nocapture"
    assertions:
      - "A canonical registered folder launches a deterministic process through the same real PTY session used by native Claude Code, Codex, and AGY adapters."
      - "Explicit OSC 7 telemetry updates active cwd before representative Markdown, Git, and configured AW context is rendered with source navigation and disclosed provenance."
      - "Unavailable agents recover without losing folder selection or read-only context."
      - "Desktop and constrained primary states are keyboard operable, readable, accessible, and placeholder free."
      - "apps/workbench/evidence/production-journey/v1/manifest.json identifies every assertion and retained artifact."
```
<!-- HANDWRITE-END -->
