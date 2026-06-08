---
id: nebula-rust-bulk-write
type: exploration
created_at: 2026-01-31T10:42:46.484962+00:00
needs_clarification: false
---

# Codebase Exploration

The goal of this change is to migrate the core logic of `bulk_write` operations in `cclab.nebula` from Python to Rust. Currently, the Python implementation handles operation building through a fluent API and then converts these operations into a list of dictionaries. These dictionaries are passed to the Rust backend, which (currently missing) triggers a Python-side fallback that executes operations one-by-one.

### Architecture Overview
- **Python Layer (`python/cclab/nebula/bulk.py`):** Defines `BulkOperation` and its subclasses (`InsertOne`, `UpdateOne`, etc.) with a fluent API.
- **Engine Layer (`python/cclab/nebula/_engine.py`):** Acts as a bridge. It checks for the existence of `_rust.Document.bulk_write`.
- **Rust Backend (`crates/cclab-nucleus`):** Currently lacks the `bulk_write` implementation in `RustDocument`. However, `crates/cclab-nucleus/src/nebula/types.rs` already contains an `ExtractedBulkOp` enum and `BulkWriteResultWrapper` struct, indicating a previous attempt or placeholder for this feature.

### Impact Analysis
- **`crates/cclab-nucleus`:**
    - `src/nebula/document.rs`: Add `bulk_write` as a `#[staticmethod]`.
    - `src/nebula/types.rs`: Enhance `ExtractedBulkOp` and implement `FromPyObject` for it (or a new `BulkOperation` enum).
    - `src/nebula/conversion.rs`: Add conversion logic for bulk operations.
- **`python/cclab/nebula`:**
    - `bulk.py`: Potentially modify `to_dict()` or replace it with a structure that Rust can easily consume via `FromPyObject`.
    - `_engine.py`: Ensure `bulk_write` correctly calls the Rust backend.

### Technical Considerations
- **Enum-based Design:** Using a Rust `enum` for `BulkOperation` with PyO3's `#[derive(FromPyObject)]` will allow for type-safe and efficient conversion of operations from Python.
- **Breaking API Changes:** Since destructive changes are allowed, we can optimize the communication format between Python and Rust. Instead of a generic `Dict[str, Any]`, we can use a more structured tuple or tagged dictionary that maps directly to the Rust enum variants.
- **Performance:** Moving the loop and BSON conversion to Rust will significantly improve performance, especially for large bulk writes, by releasing the GIL and potentially utilizing parallel execution where appropriate (though `mongodb` driver's `bulk_write` is usually a single network call).

### Recommendations
1. Define a `BulkOperation` enum in Rust that covers all MongoDB bulk write actions.
2. Implement `FromPyObject` for `BulkOperation` to handle conversion from Python dicts/objects.
3. Implement `RustDocument::bulk_write` in `document.rs` to execute the operations using the `mongodb` crate.
4. Update Python classes to provide the expected structure for the Rust enum.

