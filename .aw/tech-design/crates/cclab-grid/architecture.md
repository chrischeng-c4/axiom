# cclab-grid Architecture

## Overview
<!-- type: overview lang: markdown -->

Unified spreadsheet engine crate (post-consolidation). Combines core data structures, formula evaluation, undo/redo history, database persistence, and collaboration server into a single crate with feature-gated modules.

## Module Structure
<!-- type: dependency lang: mermaid -->

```mermaid
classDiagram
    class core {
        +cell: Cell, CellValue, CellContent
        +sheet: Sheet, FilterState
        +workbook: Workbook, WorkbookMetadata
        +range: CellCoord, CellRange
        +chunk: ChunkedGrid, Chunk, ChunkCoord
        +spatial: SpatialIndex, FenwickTree, morton_encode/decode
        +gap_buffer: GapBuffer
        +format: CellFormat, Color, CellBorders
        +error: CellError, RusheetError
        +conditional_format: ConditionalFormattingRule
        +validation: DataValidationRule
        +search: SearchEngine
        +state: SpreadsheetState
    }

    class formula {
        +ast: Expr, BinaryOp, UnaryOp
        +lexer: Lexer, Token
        +parser: Parser
        +parser_nom: NomParser
        +evaluator: Evaluator, CrossSheetEvaluator
        +dependency: DependencyGraph
        +reference_shifter: shift_formula_rows/cols
        +functions: math, text, logical, lookup, datetime
    }

    class history {
        +command: Command trait, CommandBox
        +stack: HistoryManager
    }

    class db {
        <<feature: db>>
        +storage: cell_store, wal, morton
        +query: range, spatial
        +snapshot: store
    }

    class server {
        <<feature: server>>
        +api: health, workbooks
        +collab: document, websocket
        +config: ServerConfig
        +db: models
    }

    formula --> core : uses CellValue, CellError
    history --> core : uses Sheet, CellCoord
    history --> formula : uses shift_formula_rows/cols
    db --> core : uses Cell, CellCoord
    server --> core : uses Sheet, Workbook
    server --> db : persistence layer
```

## Data Model
<!-- type: db-model lang: mermaid -->

```mermaid
erDiagram
    Workbook ||--o{ Sheet : contains
    Workbook {
        String name
        usize active_sheet_index
        WorkbookMetadata metadata
    }

    Sheet ||--o{ Cell : "sparse via ChunkedGrid"
    Sheet ||--o{ CellRange : merged_ranges
    Sheet ||--o{ FilterState : active_filters
    Sheet ||--o{ ConditionalFormattingRule : conditional_formatting
    Sheet ||--o{ DataValidationRule : data_validation
    Sheet {
        String name
        ChunkedGrid_Cell cells
        HashMap_u32_f64 row_heights
        HashMap_u32_f64 col_widths
        u32 frozen_rows
        u32 frozen_cols
        SpatialIndex spatial
    }

    Cell {
        CellContent content
        CellFormat format
    }

    CellContent ||--o| CellValue : computed_value
    CellContent {
        enum_Value_or_Formula variant
    }

    CellValue {
        enum_variant type
        f64_or_String_or_bool_or_CellError payload
    }

    CellFormat {
        bool bold
        bool italic
        bool underline
        bool strikethrough
        Option_u8 font_size
        Option_String font_family
        Option_Color text_color
        Option_Color background_color
        Option_FillPattern fill_pattern
        Option_CellBorders borders
        HorizontalAlign horizontal_align
        VerticalAlign vertical_align
        Option_String number_format
        bool wrap_text
    }
```

## Feature Gates
<!-- type: config lang: json -->

```json
{
  "$id": "grid-feature-gates",
  "type": "object",
  "properties": {
    "default": {
      "description": "core + formula + history (always included)",
      "const": ["core", "formula", "history"]
    },
    "db": {
      "description": "Morton-encoded persistence, WAL, range queries, spatial queries"
    },
    "server": {
      "description": "Axum web server, CRDT collaboration, WebSocket handlers"
    }
  }
}
```

## Key Design Decisions
<!-- type: overview lang: markdown -->

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Sparse storage | ChunkedGrid (HashMap of 64x64 Chunks) | Only non-empty cells stored; cache-friendly via Morton encoding |
| Cell indexing | Morton/Z-order curve within chunks | Improved cache locality for range queries |
| Position lookup | Fenwick Tree (SpatialIndex) | O(log N) row/col position lookups with variable heights/widths |
| Row/col insertion | GapBuffer (logical-to-physical BTreeMap) | Avoids expensive data movement on insert/delete |
| Formula parser | Dual: hand-rolled Lexer + NomParser | NomParser is primary; Lexer used for tokenization |
| Undo/redo | Command pattern with HistoryManager | Mergeable commands, bounded stack size |
| Serialization | serde JSON (Workbook, Sheet, Cell) | Full round-trip via `to_json()`/`from_json()` |
