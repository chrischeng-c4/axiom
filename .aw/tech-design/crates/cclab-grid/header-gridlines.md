---
id: header-gridlines
type: spec
title: "Header Gridline Separators"
version: 1
spec_type: algorithm
spec_group: cclab-grid
merge_strategy: new
created_at: 2026-02-10T02:51:57.491206+00:00
updated_at: 2026-02-10T02:51:57.491206+00:00
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
      title: "Header Gridline Rendering Flow"
history:
  - timestamp: 2026-02-10T02:51:57.491206+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Header Gridline Separators

## Overview

Add visual separator gridlines between each column header and each row header in the canvas renderer. Currently, only two border lines exist (after the row header column and after the column header row). This spec defines rendering individual separators to match Google Sheets visual style.

## Requirements

### R1 - Column header vertical separators

```yaml
id: R1
priority: high
status: draft
```

Render vertical gridlines between each visible column header cell, using the same gridLineColor and gridLineWidth as the data grid. Lines extend from y=0 to y=headerHeight.

### R2 - Row header horizontal separators

```yaml
id: R2
priority: high
status: draft
```

Render horizontal gridlines between each visible row header cell, using the same gridLineColor and gridLineWidth as the data grid. Lines extend from x=0 to x=headerWidth.

### R3 - Scroll-aware rendering

```yaml
id: R3
priority: high
status: draft
```

Header gridlines must account for scroll offset so they align with the data grid lines. Column header separators align with vertical grid lines; row header separators align with horizontal grid lines.

## Acceptance Criteria

### Scenario: Column headers show separators

- **GIVEN** Grid is rendered with 10 visible columns
- **WHEN** renderHeaders() is called
- **THEN** Vertical separator lines are drawn between each column header (A|B, B|C, etc.) matching data gridline positions

### Scenario: Row headers show separators

- **GIVEN** Grid is rendered with 20 visible rows
- **WHEN** renderHeaders() is called
- **THEN** Horizontal separator lines are drawn between each row header (1|2, 2|3, etc.) matching data gridline positions

### Scenario: Separators align after scroll

- **GIVEN** User has scrolled down 5 rows and right 3 columns
- **WHEN** renderHeaders() is called
- **THEN** Header separators align with visible data grid lines at their correct scroll-adjusted positions

## Diagrams

### Header Gridline Rendering Flow

```mermaid
flowchart TB
    start[renderHeaders()]
    bg[Draw header backgrounds]
    colsep[Loop visible columns: draw vertical separator]
    rowsep[Loop visible rows: draw horizontal separator]
    text[Draw header text labels]
    borders[Draw main header border lines]
    end[Headers complete]
    start --> bg
    bg --> colsep
    colsep --> rowsep
    rowsep --> text
    text --> borders
    borders --> end
```

</spec>