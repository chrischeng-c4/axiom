---
id: merge-row-col-shift
type: spec
title: "Merge Region Adjustment on Row/Column Operations"
version: 1
spec_type: algorithm
spec_group: cclab-grid
created_at: 2026-02-10T03:42:52.876611+00:00
updated_at: 2026-02-10T03:42:52.876611+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Merge Adjustment on Row Insert"
history:
  - timestamp: 2026-02-10T03:42:52.876611+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Merge Region Adjustment on Row/Column Operations

## Overview

When rows or columns are inserted or deleted, all merged_ranges in the Sheet must be adjusted to maintain correct coordinates. This includes shifting merge boundaries, expanding/shrinking merges that contain the insertion/deletion point, and removing merges that are fully deleted. Additionally, sort operations must validate against merged regions to prevent corruption.

## Requirements

### R1 - Insert rows shifts merge ranges down

```yaml
id: R1
priority: high
status: draft
```

When rows are inserted at position P, all merged ranges with startRow >= P shift down by count. Merged ranges spanning P (startRow < P <= endRow) expand their endRow by count.

### R2 - Delete rows adjusts merge ranges

```yaml
id: R2
priority: high
status: draft
```

When rows are deleted at position P with count N: merges fully within [P, P+N) are removed, merges partially overlapping shrink, merges below shift up by N.

### R3 - Insert columns shifts merge ranges right

```yaml
id: R3
priority: high
status: draft
```

When columns are inserted at position P, all merged ranges with startCol >= P shift right by count. Merged ranges spanning P expand their endCol by count.

### R4 - Delete columns adjusts merge ranges

```yaml
id: R4
priority: high
status: draft
```

When columns are deleted at position P with count N: merges fully within [P, P+N) are removed, merges partially overlapping shrink, merges below shift left by N.

### R5 - Sort validates against merged regions

```yaml
id: R5
priority: medium
status: draft
```

Before sorting a range, check if any merged regions overlap the sort range. If overlap exists, block the sort and return an error or warning.

## Acceptance Criteria

### Scenario: Insert row inside merged region expands it

- **GIVEN** Cells A1:A3 are merged (rows 0-2)
- **WHEN** Insert 1 row at row 1
- **THEN** Merged region becomes A1:A4 (rows 0-3), merge endRow increased by 1

### Scenario: Insert row below merged region shifts it

- **GIVEN** Cells B5:C6 are merged (rows 4-5)
- **WHEN** Insert 2 rows at row 2
- **THEN** Merged region becomes B7:C8 (rows 6-7), both startRow and endRow shifted by 2

### Scenario: Delete rows fully containing a merge removes it

- **GIVEN** Cells A2:B3 are merged (rows 1-2)
- **WHEN** Delete rows 1-2
- **THEN** Merge is completely removed from merged_ranges

### Scenario: Delete rows partially overlapping a merge shrinks it

- **GIVEN** Cells A1:A5 are merged (rows 0-4)
- **WHEN** Delete rows 3-4
- **THEN** Merged region becomes A1:A3 (rows 0-2), endRow shrunk

### Scenario: Sort blocked on merged range

- **GIVEN** Cells A1:A3 are merged
- **WHEN** User sorts column A range that overlaps the merge
- **THEN** Sort is blocked, returns false or error indicating merge conflict

## Diagrams

### Merge Adjustment on Row Insert

```mermaid
flowchart TB
    start[insert_rows(at, count)]
    loop[For each merge in merged_ranges]
    below[merge.startRow >= at?]
    shift[Shift both start/end row += count]
    spans[merge spans insertion point?]
    expand[Expand endRow += count]
    skip[No change]
    done[Continue to next merge]
    start --> loop
    loop --> below
    below -->|yes| shift
    below -->|no| spans
    spans -->|yes| expand
    spans -->|no| skip
    shift --> done
    expand --> done
    skip --> done
```

</spec>