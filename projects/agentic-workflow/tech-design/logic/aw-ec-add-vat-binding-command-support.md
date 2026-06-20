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
    text: "EcBinding::command() returns `vat run` when tool is vat and dir is absent or blank."
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
    text: "Unknown-tool errors list `arena|rig|meter|vat`."
    kind: functional
    risk: medium
    verify: test
  fallback_unchanged:
    id: R4
    text: "resolve_project_ec_command still uses generated EC case commands for unbound EC categories."
    kind: regression
    risk: medium
    verify: test
elements:
  ec_binding_command_builds_arena_rig_meter_vat:
    kind: test
    type: "rs/#[test]"
  ec_binding_command_rejects_unknown_tool_and_missing_arg:
    kind: test
    type: "rs/#[test]"
  resolve_ec_command_dispatches_bound_category:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: ec_binding_command_builds_arena_rig_meter_vat, verifies: vat_default }
  - { from: ec_binding_command_builds_arena_rig_meter_vat, verifies: vat_named_runner }
  - { from: ec_binding_command_rejects_unknown_tool_and_missing_arg, verifies: tool_error }
  - { from: resolve_ec_command_dispatches_bound_category, verifies: fallback_unchanged }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "vat default yields vat run"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "vat runner yields vat run runner"
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
      text: "unbound category falls back to generated EC case command"
      risk: medium
      verifymethod: test
    }
    element ec_binding_command_builds_arena_rig_meter_vat {
      type: "rs/#[test]"
    }
    element ec_binding_command_rejects_unknown_tool_and_missing_arg {
      type: "rs/#[test]"
    }
    element resolve_ec_command_dispatches_bound_category {
      type: "rs/#[test]"
    }
    ec_binding_command_builds_arena_rig_meter_vat - verifies -> R1
    ec_binding_command_builds_arena_rig_meter_vat - verifies -> R2
    ec_binding_command_rejects_unknown_tool_and_missing_arg - verifies -> R3
    resolve_ec_command_dispatches_bound_category - verifies -> R4
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] contract-complete: `EcBinding::command()` has a deterministic vat branch, uses `dir` only as an optional runner id, preserves existing arena/rig/meter behavior, and reports unknown tools with the supported-tool set.
- [unit-test] contract-complete: Rust tests are named and mapped to the new vat default, vat named-runner, unknown-tool, and manifest-fallback assertions.
