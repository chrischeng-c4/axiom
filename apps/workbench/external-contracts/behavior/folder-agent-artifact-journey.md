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
      - "The production configure_builder Tauri IPC handler resolves a canonical registered folder and launches a deterministic agent executable through the same real PTY command boundary used by Claude Code, Codex, and AGY; only the executable is substituted."
      - "The composed IPC journey sends input, resizes, interrupts, terminates, observes OSC 7 cwd, and renders Git, Markdown, and configured AW context with canonical source navigation."
      - "Unavailable-agent errors cross the production IPC boundary without losing the selected folder, and a subsequent available agent launch succeeds."
      - "Jet rejects invalid production bridge arguments and asserts recorded launch agent and canonical cwd, terminal input, context root, and context target values before accepting keyboard, desktop, constrained, and placeholder-free evidence."
      - "apps/workbench/evidence/production-journey/v1/manifest.json binds these assertions to ipc-journey.json, the PTY transcript, context summary, and screenshots."
```
