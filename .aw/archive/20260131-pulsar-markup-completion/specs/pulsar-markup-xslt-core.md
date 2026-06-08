---
id: pulsar-markup-xslt-core
type: spec
title: "XSLT Core Instructions"
version: 1
spec_type: algorithm
created_at: 2026-01-31T02:49:20.697875+00:00
updated_at: 2026-01-31T02:49:20.697875+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "XSLT Template Matching Flow"
history:
  - timestamp: 2026-01-31T02:49:20.697875+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# XSLT Core Instructions

## Overview

Specifies core XSLT instructions missing from the current implementation, enabling complex transformations through conditional logic and template application.

## Requirements

### R1 - XSLT Template Application

```yaml
id: R1
priority: medium
status: draft
```

Implement xsl:apply-templates with select and mode support.

### R2 - XSLT Conditional Branches

```yaml
id: R2
priority: medium
status: draft
```

Implement xsl:choose, xsl:when, and xsl:otherwise for conditional logic.

### R3 - XSLT Node Copying

```yaml
id: R3
priority: medium
status: draft
```

Implement xsl:copy and xsl:copy-of for node duplication.

## Acceptance Criteria

### Scenario: Apply Templates Matching

- **GIVEN** A source document and an XSLT with multiple templates.
- **WHEN** xsl:apply-templates is encountered.
- **THEN** The transformer should execute the template that best matches each selected node.

### Scenario: Choose/When Execution

- **GIVEN** An XSLT stylesheet with conditional branches.
- **WHEN** xsl:choose is processed.
- **THEN** Only the first xsl:when with a true condition, or the xsl:otherwise, should be executed.

### Scenario: Deep Copy Nodes

- **GIVEN** An XSLT using xsl:copy-of to select a subtree.
- **WHEN** xsl:copy-of is executed.
- **THEN** The selected nodes and all their descendants should be recursively copied to the output.

## Diagrams

### XSLT Template Matching Flow

```mermaid
flowchart TB
    start(Apply Templates to Node)
    find_matching_templates[Find all templates matching node tag/type and mode]
    sort_by_priority[Sort matches by XSLT priority rules]
    apply_best_match[Execute the template with highest priority]
    end(Finish Template Application)
    start --> find_matching_templates
    find_matching_templates --> sort_by_priority
    sort_by_priority --> apply_best_match
    apply_best_match --> end
```

</spec>
