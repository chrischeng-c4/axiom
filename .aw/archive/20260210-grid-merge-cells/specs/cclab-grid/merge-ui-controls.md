---
id: merge-ui-controls
type: spec
title: "Merge/Unmerge UI Controls"
version: 1
spec_type: utility
spec_group: cclab-grid
created_at: 2026-02-10T03:43:28.138829+00:00
updated_at: 2026-02-10T03:43:28.138829+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-10T03:43:28.138829+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Merge/Unmerge UI Controls

## Overview

Enhance the toolbar merge button to act as a toggle (merge if unmerged, unmerge if merged) with visual active state. The context menu already has merge/unmerge items. This spec also covers sort protection UI feedback when merges block the operation.

## Requirements

### R1 - Toolbar merge button toggles

```yaml
id: R1
priority: high
status: draft
```

When selection is within a merged region, clicking the merge button calls unmergeCells. When selection is multi-cell and not merged, it calls mergeCells. Button shows active/pressed state when selection is merged.

### R2 - Merge button state syncs on selection change

```yaml
id: R2
priority: high
status: draft
```

On selection change, check if the active cell is part of a merged region via getMergeInfo. Update merge button active state accordingly.

### R3 - Sort protection feedback

```yaml
id: R3
priority: medium
status: draft
```

When user triggers sort (via menu or context menu) on a range overlapping merged cells, show an alert or disable the sort option with a tooltip explaining the conflict.

### R4 - Single-cell merge attempt feedback

```yaml
id: R4
priority: low
status: draft
```

When user tries to merge a single cell, the merge button does nothing (no action, no error). The button should only be enabled for multi-cell selections.

## Acceptance Criteria

### Scenario: Toggle merge on unmerged selection

- **GIVEN** Cells A1:C1 are selected and not merged
- **WHEN** User clicks merge button in toolbar
- **THEN** Cells A1:C1 are merged, merge button shows active state

### Scenario: Toggle unmerge on merged selection

- **GIVEN** Cells A1:C1 are merged, user selects A1
- **WHEN** User clicks merge button in toolbar
- **THEN** Cells A1:C1 are unmerged, merge button shows inactive state

### Scenario: Merge button shows active for merged cell

- **GIVEN** Cells B2:D4 are merged
- **WHEN** User clicks on B2
- **THEN** Merge button in toolbar shows active/pressed appearance

### Scenario: Sort blocked by merge shows feedback

- **GIVEN** Cells A1:A3 are merged, user selects column A
- **WHEN** User clicks Data > Sort A to Z
- **THEN** Alert shown: 'Cannot sort range containing merged cells'

</spec>
