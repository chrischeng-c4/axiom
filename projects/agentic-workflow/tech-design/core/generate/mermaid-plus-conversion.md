---
id: mermaid-plus-conversion
type: spec
title: "Mermaid+ Conversion Algorithm"
version: 2
spec_type: algorithm
created_at: 2026-01-28T04:58:53.932575+00:00
updated_at: 2026-02-03T00:00:00.000000+00:00
requirements:
  total: 2
  ids:
    - R1
    - R2
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Mermaid+ Conversion Flow"
history:
  - timestamp: 2026-01-28T04:58:53.932575+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-03T00:00:00.000000+00:00
    agent: "human"
    tool: "manual_edit"
    action: "updated - removed XState references, documented implemented conversion"
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Codegen TDs support CB lifecycle generation and regenerable artifact production."
---

<spec>

# Mermaid+ Conversion Algorithm

## Overview
<!-- type: doc lang: markdown -->

This spec defines the conversion algorithm from structured definitions to Mermaid+ output. The implementation is in crates/cclab-sdd/src/diagrams/*/generator.rs for each diagram type. The algorithm:

1. Takes a structured definition (validated schema)
2. Generates YAML frontmatter from the definition
3. Generates Mermaid diagram syntax
4. Combines them in Mermaid+ format (frontmatter inside code block)

## Requirements
<!-- type: doc lang: markdown -->

### R1 - Core Conversion Algorithm

```yaml
id: R1
priority: high
status: implemented
```

Implement core conversion functions for each diagram type. The generators are located in:
- `crates/cclab-sdd/src/diagrams/state_plus/generator.rs`
- `crates/cclab-sdd/src/diagrams/flowchart_plus/generator.rs`
- `crates/cclab-sdd/src/diagrams/sequence_plus/generator.rs`
- etc.

### R2 - Output Format Compliance

```yaml
id: R2
priority: high
status: implemented
```

All generators must output Mermaid+ format with frontmatter INSIDE the code block:

```
```mermaid
---
<yaml frontmatter>
---
<mermaid diagram>
```
```

Validation warnings are appended as HTML comments after the code block.

## Acceptance Criteria
<!-- type: doc lang: markdown -->

### Scenario: Nested State Conversion

- **GIVEN** A definition with nested/compound states.
- **WHEN** Conversion algorithm executes.
- **THEN** Produces Mermaid code with `state "..." as id { ... }` syntax.

### Scenario: Frontmatter Inside Code Block

- **GIVEN** Any valid diagram definition.
- **WHEN** Conversion algorithm executes.
- **THEN** Output starts with ` ```mermaid\n---\n ` (frontmatter inside).

## Diagrams
<!-- type: doc lang: markdown -->

### Mermaid+ Conversion Flow

```mermaid
flowchart TB
    Start[Start Conversion]
    Parse[Parse Structured Definition]
    GenFrontmatter[Generate YAML Frontmatter]
    GenMermaid[Generate Mermaid Syntax]
    Combine[Combine: fence + frontmatter + diagram]
    AddWarnings{Has Warnings?}
    AppendWarnings[Append HTML Comment Warnings]
    End[Return Combined Output]
    Start --> Parse
    Parse --> GenFrontmatter
    GenFrontmatter --> GenMermaid
    GenMermaid --> Combine
    Combine --> AddWarnings
    AddWarnings -->|Yes| AppendWarnings
    AppendWarnings --> End
    AddWarnings -->|No| End
```

## Implementation Details
<!-- type: doc lang: markdown -->

Each generator follows this pattern:

```mermaid
flowchart TD
    Input[Definition + ValidationResult] --> GenFM[generate_frontmatter]
    Input --> GenMD[generate_mermaid]
    GenFM --> Combine[combine: fence + frontmatter + diagram]
    GenMD --> Combine
    Combine --> CheckWarn{has warnings?}
    CheckWarn -->|yes| Append[append HTML comment warnings]
    CheckWarn -->|no| Output[return Output]
    Append --> Output
```

Output format: ` ```mermaid ` → `---` → YAML frontmatter → `---` → Mermaid diagram → ` ``` ` → optional `<!-- Validation Warnings -->` HTML comment.

</spec>
