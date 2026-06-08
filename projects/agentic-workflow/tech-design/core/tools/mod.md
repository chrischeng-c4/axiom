---
id: sdd-tools-mod
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Tool Registry Types

## Overview
<!-- type: overview lang: markdown -->

MCP tool registry types in `projects/agentic-workflow/src/tools/mod.rs`. Two shapes:

- `ToolRegistry` — `tools: Vec<ToolDefinition>` (private). No derives.
- `ToolDefinition` — `name`, `description`, `input_schema: Value`.
  Derives `[Clone]`.

Codegen replaces both type declarations. Module submodule
declarations, imports, the `impl ToolRegistry` block stay
hand-written.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ToolRegistry:
    type: object
    required: [tools]
    description: Registry of available MCP tools.
    properties:
      tools:
        type: array
        items: { type: object }
        x-rust-type: "Vec<ToolDefinition>"
        x-rust-visibility: private
        description: "Registered tools."
    x-rust-struct:
      derive: []

  ToolDefinition:
    type: object
    required: [name, description, input_schema]
    description: Tool definition for MCP protocol.
    properties:
      name:
        type: string
        description: "Tool name."
      description:
        type: string
        description: "Human-readable description."
      input_schema:
        type: object
        x-rust-type: "Value"
        description: "JSON Schema for tool input."
    x-rust-struct:
      derive: [Clone]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/mod.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ToolRegistry
      - ToolDefinition
    description: |
      Codegen replaces both type declarations.
  - path: projects/agentic-workflow/src/tools/mod.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written outside CODEGEN: module submodule declarations,
      `pub use` re-exports, imports, the `impl ToolRegistry` block.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- [overview] 2 structs; standard pattern.
- [schema] Both well-formed.
- [changes] Standard split.
