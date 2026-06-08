---
id: mamba-p1b
type: proposal
version: 2
created_at: 2026-02-21T15:12:38.996243+00:00
updated_at: 2026-02-21T15:12:38.996243+00:00
iteration: 1
scope: minor
spec_plan:
  - id: mamba-oop-features
    title: "Advanced OOP Features"
    depends: [mamba-oop-model]
    context_refs:
      codebase: ["crates/mamba/src/runtime/class.rs", "crates/mamba/src/runtime/rc.rs"]
      spec: ["issue_383_feat-mamba-super-runtime-implementation.md", "issue_406_mamba-descriptor-protocol-get-set-delete.md", "issue_382_feat-mamba-isinstance-issubclass-and-type-narrowin.md", "issue_384_feat-mamba-property-classmethod-staticmethod-decor.md", "issue_407_mamba-metaclasses-and-abc-abstract-base-classes.md", "issue_408_mamba-reflection-builtins-hasattr-getattr-setattr-.md"]
      knowledge: ["spec:cclab-mamba/mamba-oop-model.md"]
    gap_repairs:
      - { source: gap_spec_knowledge, gap_index: 0 }
      - { source: gap_spec_knowledge, gap_index: 1 }
      - { source: gap_spec_knowledge, gap_index: 4 }
      - { source: gap_spec_knowledge, gap_index: 6 }
    affected_code: ["crates/mamba/src/runtime/class.rs", "crates/mamba/src/runtime/symbols.rs"]
  - id: mamba-runtime-types
    title: "Runtime Types (Bytes)"
    depends: [mamba-runtime-p1]
    context_refs:
      codebase: ["crates/mamba/src/runtime/value.rs"]
      spec: ["issue_405_mamba-bytes-bytearray-type-and-binary-data-operati.md"]
      knowledge: ["spec:cclab-mamba/mamba-runtime-p1.md"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 0 }
    affected_code: ["crates/mamba/src/runtime/value.rs", "crates/mamba/src/runtime/object.rs"]
  - id: mamba-system-modules
    title: "System Modules & Integration"
    depends: [mamba-import-system]
    context_refs:
      codebase: ["crates/mamba/src/runtime/module.rs"]
      spec: ["issue_421_mamba-module-package-system-init-py-sys-path-relat.md", "issue_424_mamba-os-path-and-extended-os-module.md"]
      knowledge: ["spec:cclab-mamba/mamba-import-system.md", "spec:cclab-mamba/mamba-stdlib-core.md"]
    gap_repairs:
      - { source: gap_spec_knowledge, gap_index: 2 }
      - { source: gap_spec_knowledge, gap_index: 3 }
    affected_code: ["crates/mamba/src/runtime/module.rs", "crates/mamba/src/stdlib/os.rs"]
  - id: mamba-codegen-flow
    title: "Control Flow Codegen"
    depends: [mamba-codegen-logic]
    context_refs:
      codebase: ["crates/mamba/src/mir/mod.rs", "crates/mamba/src/codegen/cranelift/mod.rs"]
      spec: ["issue_385_feat-mamba-context-manager-protocol-with-statement.md", "issue_422_mamba-assert-and-del-statement-codegen.md"]
      knowledge: ["spec:cclab-mamba/mamba-codegen-logic.md"]
    gap_repairs:
      - { source: gap_spec_knowledge, gap_index: 5 }
    affected_code: ["crates/mamba/src/mir/mod.rs", "crates/mamba/src/codegen/cranelift/mod.rs"]
history:
  - timestamp: 2026-02-21T15:12:38.996243+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: mamba-p1b

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((mamba-p1b))  
    OOP Features
      super()
      descriptors
      decorators
      metaclasses
      reflection
      isinstance
    Runtime Types
      bytes
      bytearray
    System Integration
      Modules
      Packages
      os module
    Control Flow
      with statement
      assert
      del
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  mamba_oop_features["mamba-oop-features\n codebase: crates/mamba/src/runtime/class.rs, crates/mamba/src/runtime/rc.rs\n gaps: spec_knowledge#0, spec_knowledge#1, spec_knowledge#4, spec_knowledge#6"]
  mamba_runtime_types["mamba-runtime-types\n codebase: crates/mamba/src/runtime/value.rs\n gaps: codebase_spec#0"]
  mamba_system_modules["mamba-system-modules\n codebase: crates/mamba/src/runtime/module.rs\n gaps: spec_knowledge#2, spec_knowledge#3"]
  mamba_codegen_flow["mamba-codegen-flow\n codebase: crates/mamba/src/mir/mod.rs, crates/mamba/src/codegen/cranelift/mod.rs\n gaps: spec_knowledge#5"]

  mamba_oop_model --> mamba_oop_features
  mamba_runtime_p1 --> mamba_runtime_types
  mamba_import_system --> mamba_system_modules
  mamba_codegen_logic --> mamba_codegen_flow
```

## Spec Execution Order

1. **mamba-codegen-flow** — Control Flow Codegen
   - depends: mamba-codegen-logic
   - code: crates/mamba/src/mir/mod.rs, crates/mamba/src/codegen/cranelift/mod.rs
2. **mamba-oop-features** — Advanced OOP Features
   - depends: mamba-oop-model
   - code: crates/mamba/src/runtime/class.rs, crates/mamba/src/runtime/symbols.rs
3. **mamba-runtime-types** — Runtime Types (Bytes)
   - depends: mamba-runtime-p1
   - code: crates/mamba/src/runtime/value.rs, crates/mamba/src/runtime/object.rs
4. **mamba-system-modules** — System Modules & Integration
   - depends: mamba-import-system
   - code: crates/mamba/src/runtime/module.rs, crates/mamba/src/stdlib/os.rs

</proposal>
