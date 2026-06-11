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
  branch: { kind: decision, label: "binding.tool" }
  arena: { kind: process, label: "arena: require spec, emit arena run --spec <spec>" }
  rig: { kind: process, label: "rig: require dir, emit rig run --dir <dir>" }
  meter: { kind: process, label: "meter: require meter, emit meter run --target <meter>" }
  vat_runner: { kind: decision, label: "vat binding has dir runner id?" }
  vat_named: { kind: terminal, label: "emit vat run <dir>" }
  vat_default: { kind: terminal, label: "emit vat run" }
  unknown: { kind: terminal, label: "error: expected arena|rig|meter|vat" }
edges:
  - { from: start, to: branch }
  - { from: branch, to: arena, label: "arena" }
  - { from: branch, to: rig, label: "rig" }
  - { from: branch, to: meter, label: "meter" }
  - { from: branch, to: vat_runner, label: "vat" }
  - { from: vat_runner, to: vat_named, label: "yes" }
  - { from: vat_runner, to: vat_default, label: "no" }
  - { from: branch, to: unknown, label: "other" }
---
flowchart TD
  start([EcBinding::command]) --> branch{binding.tool}
  branch -->|arena| arena[require spec; arena run --spec]
  branch -->|rig| rig[require dir; rig run --dir]
  branch -->|meter| meter[require meter; meter run --target]
  branch -->|vat| vat_runner{dir runner id present?}
  vat_runner -->|yes| vat_named([vat run runner])
  vat_runner -->|no| vat_default([vat run])
  branch -->|other| unknown([error: expected arena|rig|meter|vat])
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
