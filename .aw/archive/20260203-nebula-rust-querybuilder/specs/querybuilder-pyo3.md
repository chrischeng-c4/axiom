---
id: querybuilder-pyo3
type: spec
title: "PyO3 QueryBuilder Bindings"
version: 1
spec_type: integration
created_at: 2026-02-01T07:01:39.866659+00:00
updated_at: 2026-02-01T07:01:39.866659+00:00
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
history:
  - timestamp: 2026-02-01T07:01:39.866659+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# PyO3 QueryBuilder Bindings

## Overview

\u5be6\u4f5c PyO3 bindings \u5c07 Rust QueryBuilder \u548c QueryExpr \u66b4\u9732\u7d66 Python\u3002\u5305\u542b RustQueryBuilder \u548c RustQueryExpr PyO3 classes\uff0c\u652f\u63f4 chainable API \u548c async to_list() \u57f7\u884c\u3002

## Requirements

### R1 - RustQueryExpr PyO3 class

```yaml
id: R1
priority: high
status: draft
```

建立 #[pyclass] RustQueryExpr 提供靜態方法: eq(), ne(), gt(), gte(), lt(), lte(), in_(), nin(), exists(), regex(), and_(), or_()

### R2 - RustQueryBuilder PyO3 class

```yaml
id: R2
priority: high
status: draft
```

建立 #[pyclass] RustQueryBuilder 提供: new(), filter(), sort(), skip(), limit(), projection() 方法，每個方法回傳新的 RustQueryBuilder

### R3 - Async to_list

```yaml
id: R3
priority: high
status: draft
```

RustQueryBuilder 提供 async fn to_list() -> Vec<PyDict> 執行 MongoDB 查詢並回傳結果

### R4 - Async count

```yaml
id: R4
priority: medium
status: draft
```

RustQueryBuilder 提供 async fn count() -> u64 回傳符合條件的文件數量

### R5 - 錯誤處理

```yaml
id: R5
priority: medium
status: draft
```

將 Rust 錯誤轉換為 Python exceptions (PyValueError, PyRuntimeError)

## Acceptance Criteria

### Scenario: Python 使用 QueryExpr

- **GIVEN** Python 建立查詢條件
- **WHEN** 呼叫 RustQueryExpr.eq('name', 'Alice')
- **THEN** 回傳可用於 filter() 的 RustQueryExpr

### Scenario: Python chainable query

- **GIVEN** Python 建立查詢
- **WHEN** 呼叫 RustQueryBuilder('users').filter(expr).sort([('name', 1)]).limit(10)
- **THEN** 回傳新的 RustQueryBuilder 包含所有設定

### Scenario: Async to_list

- **GIVEN** QueryBuilder 已設定
- **WHEN** 呼叫 await builder.to_list()
- **THEN** 執行 MongoDB 查詢並回傳 list[dict]

## Flow Diagram

```mermaid
```mermaid
sequenceDiagram
    participant Python
    participant PyO3
    participant Rust
    participant MongoDB

    Python->>PyO3: RustQueryExpr.eq('name', 'Alice')
    PyO3->>Rust: QueryExpr::Eq { field, value }
    Rust-->>PyO3: RustQueryExpr wrapper
    PyO3-->>Python: RustQueryExpr object

    Python->>PyO3: builder.filter(expr).sort([...]).limit(10)
    PyO3->>Rust: Clone + modify QueryBuilder
    Rust-->>PyO3: New RustQueryBuilder
    PyO3-->>Python: RustQueryBuilder object

    Python->>PyO3: await builder.to_list()
    PyO3->>Rust: build_filter() + build_options()
    Rust->>MongoDB: find(filter, options)
    MongoDB-->>Rust: Cursor<Document>
    Rust->>PyO3: Convert to Vec<PyDict>
    PyO3-->>Python: list[dict]
```
```

</spec>
