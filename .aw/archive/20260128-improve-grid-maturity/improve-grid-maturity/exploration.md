---
id: improve-grid-maturity
type: exploration
created_at: 2026-01-28T07:51:43.985612+00:00
needs_clarification: false
---

# Codebase Exploration

# Exploration: Improve Grid Maturity

## Architecture Overview
The `cclab-grid` system is divided into several crates:
- `cclab-grid-core`: Contains the fundamental data structures (`Workbook`, `Sheet`, `Cell`, `CellFormat`).
- `cclab-grid-formula`: Handles formula parsing and evaluation.
- `cclab-grid-server` & `cclab-grid-db`: Handle persistence and collaborative features.

The current maturity is estimated at 65%. To reach 95%, we need to fill gaps in styling, formula power, I/O support, and performance validation.

## Relevant Files
- `crates/cclab-grid-core/src/format.rs`: Needs expansion for borders, pattern fills, and themes.
- `crates/cclab-grid-core/src/workbook.rs`: Needs a theme model.
- `crates/cclab-grid-formula/src/functions/`: Needs `INDEX` and improved `lookup.rs`.
- `crates/cclab-grid-formula/src/evaluator.rs`: Needs support for array formula evaluation (spilling).
- `crates/cclab-grid-formula/src/dependency.rs`: Needs robust circular dependency detection.
- `crates/cclab-grid-core/src/chunk.rs`: Review for 100k+ row performance.

## Impact Analysis
- **Core Data Model**: Adding borders and themes to `CellFormat` and `Workbook` will affect serialization but should be backward compatible if using `Option` or `Default`.
- **Formula Engine**: Array formulas are a significant architectural shift. Currently, one cell = one value. Spilling requires a cell to "occupy" or "influence" neighboring cells.
- **I/O**: A new crate `cclab-grid-io` will be added to avoid bloating `core` with heavy dependencies like `calamine` or `rust_xlsxwriter`.

## Technical Considerations
- **Array Formulas**: We should implement a "spill" mechanism where an evaluator can return a range of values, and the sheet manager handles the overflow into neighboring cells, potentially marking them as "spilled".
- **Borders**: Borders should probably be stored as part of the `CellFormat`. However, adjacent cells share borders. A common approach is to store them per-cell (Top, Bottom, Left, Right) and let the UI resolve overlaps.
- **100k+ Rows**: The current `ChunkedGrid` (using `morton_encode`) is designed for sparsity and large ranges. We need to benchmark its memory overhead and access patterns for dense 100k+ row datasets.

## Spec Recommendations
- `grid-io-spec`: Covers XLSX/CSV/ODS integration using external libraries.
- `grid-styling-spec`: Defines the border and theme model.
- `grid-formula-array-spec`: Details the array formula evaluation and spilling logic.
- `grid-formula-functions-spec`: Requirements for `INDEX`, `VLOOKUP` improvements, etc.
- `grid-performance-spec`: Benchmarking suite and circular dependency detection logic.

