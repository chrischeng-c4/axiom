---
id: auto-gen-python-cclab
type: proposal
version: 2
created_at: 2026-01-30T14:45:00+08:00
updated_at: 2026-01-30T14:45:00+08:00
author: claude
status: proposed
iteration: 2
summary: "Move ORM logic to Rust, Python becomes thin auto-generable wrapper"
impact:
  scope: major
  affected_files: 30+
  new_files: 5+
affected_specs:
  - id: titan-orm-rust
    path: specs/titan-orm-rust.md
  - id: python-codegen
    path: specs/python-codegen.md
---

# Change: auto-gen-python-cclab (v2 - Expanded Scope)

## Summary

Move ORM logic (QueryBuilder, Transaction, Session, Relationships) from Python to Rust.
Python becomes a thin wrapper with only type definitions (Table, Column) that can be auto-generated.

## Architecture

```
┌─────────────────┐
│  Spec Files     │  JSON Schema / OpenRPC (source of truth)
└────────┬────────┘
         │ cclab prism gen-rust
         ▼
┌─────────────────┐
│  Rust Code      │  #[pyclass] structs, QueryBuilder, Transaction
│  (cclab-titan)  │  All business logic lives here
└────────┬────────┘
         │ cclab prism gen-python
         ▼
┌─────────────────┐
│  Python Code    │  Table/Column definitions + .pyi stubs
│  (cclab.titan)  │  Thin wrapper, auto-generated
└─────────────────┘
```

## What Changes

### Phase 1: Rust ORM Enhancement (cclab-titan)

Move Python ORM logic to Rust:

| Component | From (Python) | To (Rust) |
|-----------|---------------|-----------|
| QueryBuilder | `python/cclab/titan/query.py` | `crates/cclab-titan/src/query/builder.rs` |
| Transaction | `python/cclab/titan/transactions.py` | `crates/cclab-titan/src/transaction.rs` |
| Session | `python/cclab/titan/session.py` | `crates/cclab-titan/src/session.rs` |
| Relationships | `python/cclab/titan/relationships.py` | `crates/cclab-titan/src/relations.rs` |
| LoadingStrategy | `python/cclab/titan/loading.py` | `crates/cclab-titan/src/loading.rs` |

### Phase 2: PyO3 Bindings (cclab-nucleus)

Expose Rust ORM via PyO3:

```rust
// crates/cclab-nucleus/src/titan/mod.rs
#[pyclass]
pub struct QueryBuilder { ... }

#[pymethods]
impl QueryBuilder {
    fn select(&self, columns: Vec<String>) -> PyResult<Self> { ... }
    fn where_clause(&self, ...) -> PyResult<Self> { ... }
    fn order_by(&self, ...) -> PyResult<Self> { ... }
    fn limit(&self, n: usize) -> PyResult<Self> { ... }
    async fn to_list(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> { ... }
}
```

### Phase 3: Python Codegen

Create generator to produce Python type definitions:

```python
# Auto-generated: python/cclab/titan/models.py
from cclab._nucleus import titan as _titan

class Table(metaclass=TableMeta):
    """Base class for ORM tables - delegates to Rust"""
    __rust_class__ = _titan.Table

class Column:
    """Column definition - delegates to Rust"""
    def __init__(self, type_: type, ...):
        self._rust = _titan.Column(type_.__name__, ...)
```

### Phase 4: Naming Consistency

Rename Python modules to match Rust crates:

| Rust Crate | Python Module (Old) | Python Module (New) |
|------------|---------------------|---------------------|
| cclab-titan | cclab.postgres | cclab.titan |
| cclab-nebula | cclab.mongodb | cclab.nebula |

### Phase 5: Remove Prism Python Bindings

- Remove `crates/cclab-nucleus/src/prism/` (if exists)
- Prism is MCP server + CLI only, no Python bindings needed

## New CLI Commands

```bash
# Generate Rust code from spec
cclab prism gen-rust <spec-file> --output <rust-file>

# Generate Python wrapper from Rust (enhanced from existing gen-stub)
cclab prism gen-python <rust-crate> --output <python-dir>
  --include-stubs     # Also generate .pyi files
  --thin-wrapper      # Generate minimal Python (type defs only)
```

## Impact

- **Scope**: major
- **Affected Crates**: cclab-titan, cclab-nucleus, cclab-prism
- **Affected Python**: python/cclab/titan/, python/cclab/__init__.py
- **New Tools**: gen-rust, gen-python (enhanced)
- **Breaking Change**: Yes - Python API will change (users import from cclab.titan instead of cclab.postgres)

## Migration Guide

```python
# Before
from cclab.postgres import Table, Column, init
from cclab.postgres.query import QueryBuilder

# After
from cclab.titan import Table, Column, init
# QueryBuilder is now internal, use Table.find() API
```

## Success Criteria

1. ✅ All ORM logic executes in Rust (zero Python byte handling)
2. ✅ Python layer is < 500 lines (down from ~13,500)
3. ✅ `cclab prism gen-python` can regenerate Python from Rust
4. ✅ All existing tests pass with new architecture
5. ✅ Performance improvement (Rust execution)
