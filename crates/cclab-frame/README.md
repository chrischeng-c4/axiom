# Cclab Frame

## Brief

Cclab Frame is the Rust DataFrame and Series library for cclab data workflows.

It owns pandas-like tabular data structures, typed values, single and
multi-indexing, row/column selection, null handling, analytical operations,
reshape/window helpers, and tabular IO. The public surface is a Rust library
API; this crate does not expose a standalone CLI surface.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| DataFrame Series Core | - | implemented | passing | conformance | not_ready | DataFrame/Series/Value/Index APIs with pandas-like indexing, null handling, sorting, apply, and stats |
| Analytical Operations | - | implemented | passing | conformance | not_ready | GroupBy, joins, reshape, rolling, expanding, and EWM operations |
| Frame IO And Workbook | - | implemented | passing | conformance | not_ready | CSV, columnar, workbook, and optional JSON IO surfaces |

### DataFrame Series Core

ID: dataframe-series-core
Type: DeveloperTool
Surfaces: Rust API: `cclab_frame::frame::{DataFrame, Series, Value, Index, MultiIndex, FrameError}`
EC Dimensions: behavior: `cargo test -p cclab-frame` - construction, shape, indexing, selection, sorting, null handling, arithmetic, conversion, statistics, duplicate handling, and record/dict transforms
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Frame provides pandas-like DataFrame and Series primitives with typed values, single and multi-indexing, positional/label access, null handling, row/column transforms, sorting, statistics, and conversion helpers.
Gate Inventory: `cargo test -p cclab-frame`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| DataFrame and Series core contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-frame` |

### Analytical Operations

ID: analytical-operations
Type: DeveloperTool
Surfaces: Rust API: `cclab_frame::frame::ops::{GroupBy, JoinType, AggFunc}` plus DataFrame reshape/window methods
EC Dimensions: behavior: `cargo test -p cclab-frame` - groupby aggregates/transform/filter, joins, pivot/crosstab, stack/unstack, rolling, expanding, and EWM behavior
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Frame exposes analytical DataFrame operations for grouped aggregation, joins, pivoting, reshaping, rolling windows, expanding windows, and exponentially weighted calculations.
Gate Inventory: `cargo test -p cclab-frame`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Analytical operations contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-frame` |

### Frame IO And Workbook

ID: frame-io-and-workbook
Type: DeveloperTool
Surfaces:
- Rust API: `cclab_frame::frame::io::{read_csv, write_csv, read_columnar, write_columnar, Workbook}` - CSV, columnar, and workbook IO helpers.
- Cargo feature: `io-extra` - JSON read/write helpers.
EC Dimensions:
- behavior: `cargo test -p cclab-frame` - CSV parse/read/write, columnar file roundtrip, workbook sheet behavior, and IO error paths.
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Frame reads and writes tabular data through CSV, columnar, workbook, and optional JSON feature surfaces while preserving the DataFrame value model.
Gate Inventory:
- `cargo test -p cclab-frame`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Frame IO contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-frame` |
