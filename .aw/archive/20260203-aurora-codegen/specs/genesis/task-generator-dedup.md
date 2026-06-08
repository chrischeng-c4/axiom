---
id: task-generator-dedup
type: spec
title: "Task Generator Nested Spec Deduplication"
version: 1
spec_type: algorithm
spec_group: genesis
created_at: 2026-02-02T14:47:39.820443+00:00
updated_at: 2026-02-02T14:47:39.820443+00:00
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
      title: "Spec Deduplication Flow"
history:
  - timestamp: 2026-02-02T14:47:39.820443+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Task Generator Nested Spec Deduplication

## Overview

修正 task generator 在處理 nested specs 時產生重複 tasks 的問題。當同一個 spec 同時存在於 root 和 subdirectory 時，應該 merge 或 deduplicate，避免產生重複的 implementation 和 test tasks。

## Requirements

### R1 - Spec Path Normalization

```yaml
id: R1
priority: high
status: draft
```

將所有 spec paths 正規化，識別出同名的 specs (如 specs/foo.md 和 specs/bar/foo.md)

### R2 - Deduplication Strategy

```yaml
id: R2
priority: high
status: draft
```

當發現重複 spec 時，優先使用 nested spec (更具體)，或 merge 兩者的 requirements

### R3 - Task File Path Uniqueness

```yaml
id: R3
priority: high
status: draft
```

確保生成的 task file paths 唯一，不會有兩個 tasks 建立同一個檔案

## Acceptance Criteria

### Scenario: Deduplicate same-name specs

- **GIVEN** specs/template-engine.md 和 specs/aurora-codegen/template-engine.md 都存在
- **WHEN** 執行 task generation
- **THEN** 只產生一個 template-engine.rs task，不會有 Task 2.5 和 2.6 重複

### Scenario: Unique test tasks

- **GIVEN** 有 N 個 unique implementation tasks
- **WHEN** 生成 test tasks
- **THEN** 產生 N 個 unique test tasks，每個對應一個 impl task

## Diagrams

### Spec Deduplication Flow

```mermaid
flowchart TB
    start([Collect all specs])
    normalize[Normalize spec paths]
    group[Group by base name]
    check{Has duplicates?} 
    merge[Merge or pick nested]
    gen[Generate tasks]
    dedup_files[Deduplicate file paths]
    end([Output unique tasks])
    start --> normalize
    normalize --> group
    group --> check
    check -->|Yes| merge
    check -->|No| gen
    merge --> gen
    gen --> dedup_files
    dedup_files --> end
```

</spec>
