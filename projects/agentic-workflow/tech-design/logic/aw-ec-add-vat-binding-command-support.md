---
id: aw-ec-add-vat-binding-command-support
summary: Add vat as a supported AW EC binding tool so project EC categories can run vat-managed environment runners through aw health --verify-ec.
fill_sections: [logic, unit-test]
capability_refs:
  - id: project-local-td-and-ec-gates
    role: primary
    gap: ec-tool-binding-dispatch
    claim: ec-tool-binding-dispatch
    coverage: partial
    rationale: "Extends the existing EC binding dispatch from arena/rig/meter to include vat-managed runner environments."
---

# TD: aw EC vat binding command support

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: aw-ec-vat-binding-command
entry: start
nodes:
  start: { kind: start, label: "EcBinding::command()" }
  branch: { kind: decision, label: "binding.tool.as_str()" }
  arena: { kind: process, label: "require spec; return arena run --spec <spec>" }
  rig: { kind: process, label: "require dir; return rig run --dir <dir>" }
  meter: { kind: process, label: "require meter; return meter run --target <meter>" }
  vat_runner: { kind: decision, label: "self.dir has non-empty runner id?" }
  vat_named: { kind: terminal, label: "return vat run <runner>" }
  vat_default: { kind: terminal, label: "return vat run" }
  unknown: { kind: terminal, label: "bail unknown ec binding tool; expected arena|rig|meter|vat" }
edges:
  - { from: start, to: branch }
  - { from: branch, to: arena, label: "arena" }
  - { from: branch, to: rig, label: "rig" }
  - { from: branch, to: meter, label: "meter" }
  - { from: branch, to: vat_runner, label: "vat" }
  - { from: vat_runner, to: vat_named, label: "Some(dir) and not blank" }
  - { from: vat_runner, to: vat_default, label: "None or blank" }
  - { from: branch, to: unknown, label: "other" }
---
flowchart TD
  start([EcBinding::command]) --> branch{tool}
  branch -->|arena| arena[require spec; arena run --spec]
  branch -->|rig| rig[require dir; rig run --dir]
  branch -->|meter| meter[require meter; meter run --target]
  branch -->|vat| vat_runner{dir runner id?}
  vat_runner -->|present| vat_named([vat run runner])
  vat_runner -->|absent| vat_default([vat run])
  branch -->|other| unknown([error expected arena|rig|meter|vat])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: aw-ec-vat-binding-unit-tests
requirements:
  vat_default:
    id: R1
    text: "EcBinding::command() returns `vat run` when tool is vat and no runner id is configured."
    kind: functional
    risk: high
    verify: test
  vat_named_runner:
    id: R2
    text: "EcBinding::command() returns `vat run <runner>` when tool is vat and dir carries the runner id."
    kind: functional
    risk: high
    verify: test
  tool_error:
    id: R3
    text: "Unknown-tool errors list vat alongside arena, rig, and meter."
    kind: functional
    risk: medium
    verify: test
  fallback_unchanged:
    id: R4
    text: "Unbound EC categories continue to fall back to the manifest command."
    kind: regression
    risk: medium
    verify: test
elements:
  test_command_builder:
    kind: test
    type: "rs/#[test]"
  test_unknown_tool:
    kind: test
    type: "rs/#[test]"
  test_dispatch_fallback:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: test_command_builder, verifies: vat_default }
  - { from: test_command_builder, verifies: vat_named_runner }
  - { from: test_unknown_tool, verifies: tool_error }
  - { from: test_dispatch_fallback, verifies: fallback_unchanged }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "vat without runner yields vat run"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "vat with runner yields vat run runner"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "unknown-tool error includes vat"
      risk: medium
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "unbound category falls back to manifest command"
      risk: medium
      verifymethod: test
    }
    element test_command_builder {
      type: "rs/#[test]"
    }
    element test_unknown_tool {
      type: "rs/#[test]"
    }
    element test_dispatch_fallback {
      type: "rs/#[test]"
    }
    test_command_builder - verifies -> R1
    test_command_builder - verifies -> R2
    test_unknown_tool - verifies -> R3
    test_dispatch_fallback - verifies -> R4
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] applicable: the change is a small extension of the existing EC command builder; the vat branch is deterministic, preserves the existing arena/rig/meter branches, and keeps unknown tools as a failed EC command.
- [unit-test] applicable: R1-R4 cover the new default vat command, named vat runner command, unknown-tool error text, and unchanged manifest fallback behavior.
