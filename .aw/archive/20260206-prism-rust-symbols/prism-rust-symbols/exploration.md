---
id: prism-rust-symbols
type: exploration
created_at: 2026-02-06T07:42:30.732358+00:00
needs_clarification: false
---

# Codebase Exploration

# Codebase Exploration: Prism Rust & TypeScript Symbol Extraction

## Architecture Overview

Prism's symbol extraction pipeline:
```
File → MultiParser (tree-sitter) → ParsedFile → SymbolTableBuilder → SymbolTable → MCP/LSP
```

### Current State
- **Python**: Full symbol extraction via `SymbolTableBuilder::build_python()`
- **Rust**: Tree-sitter grammar loaded (`tree-sitter-rust 0.23`), parser works, but `SymbolTable::default()` returned
- **TypeScript**: Same as Rust — grammar loaded but no builder

### Key Discovery: Existing RustSymbolCollector
There is already a **comprehensive** Rust symbol collector at `crates/cclab-prism/src/types/rust_symbols.rs` (1023 lines) with:
- `RustSymbols` struct containing `structs`, `enums`, `traits`, `impls`, `functions`, `constants`, `type_aliases`
- `RustSymbolCollector` with visitor methods for all Rust node kinds
- Detailed type extraction (generics, where clauses, visibility, etc.)

This is a **separate system** from the `semantic/symbols.rs` SymbolTable. The types/ module focuses on **type analysis** while semantic/ focuses on **cross-language symbol indexing**. We can reference the collector's visitor logic as a pattern.

## Relevant Files

### Core Files to Modify
| File | Lines | Purpose |
|------|-------|---------|
| `crates/cclab-prism/src/semantic/symbols.rs` | 649 | Symbol table + Python builder — **needs Rust/TS builders** |
| `crates/cclab-prism/src/server/handler.rs` | 622 | Request handler — **language check at line 507** |
| `crates/cclab-prism/src/lsp/server.rs` | 1042 | LSP server — **language check at line 94** |

### Reference Files (read-only)
| File | Lines | Purpose |
|------|-------|---------|
| `crates/cclab-prism/src/types/rust_symbols.rs` | 1023 | Existing Rust visitor patterns |
| `crates/cclab-prism/src/types/ts_infer.rs` | 1380 | TypeScript node kind reference |
| `crates/cclab-prism/src/syntax/parser.rs` | 248 | Language enum, parser setup |
| `crates/cclab-prism/src/semantic/tests.rs` | ~200 | Test patterns to follow |

### MCP Interface (no changes needed)
| File | Lines | Purpose |
|------|-------|---------|
| `crates/cclab-prism/src/mcp/tools.rs` | 855 | prism_symbols tool — already language-agnostic |

## Data Model Analysis

### Symbol Struct (current)
```rust
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub location: Range,
    pub type_info: Option<TypeInfo>,
    pub doc: Option<String>,
    pub scope_id: usize,        // Flat scope tracking — NO parent reference
}
```

### Hierarchy Gap
User requested **hierarchical** symbols (e.g., `struct → impl → methods`). Current Symbol struct only has `scope_id` (a flat integer). To support hierarchy, we need to add:
- `parent_id: Option<SymbolId>` to the Symbol struct
- Update SymbolTableBuilder to track parent during scope push/pop
- Add `children()` or `parent()` accessor methods to SymbolTable

This is a **data model extension** that affects all languages (Python will also benefit).

### SymbolKind Coverage
Already defined: `Struct`, `Trait`, `Impl`, `Macro`, `Const`, `Static`, `Interface`, `TypeParameter`, `Enum`, `EnumMember`

**Missing for comprehensive coverage**:
- Rust: `Union`, `Lifetime`, `UseDeclaration`, `ExternCrate`, `Attribute`
- TypeScript: `TypeAlias` (reuse existing), `Namespace`, `Abstract`
- Common: `Method` (distinct from `Function` for impl/class methods)

## Impact Analysis

### Files to Create (new)
1. `crates/cclab-prism/src/semantic/symbols/mod.rs` — shared types (Symbol, SymbolTable, SymbolKind, etc.)
2. `crates/cclab-prism/src/semantic/symbols/python.rs` — Python builder (extracted from current)
3. `crates/cclab-prism/src/semantic/symbols/rust.rs` — New Rust builder
4. `crates/cclab-prism/src/semantic/symbols/typescript.rs` — New TypeScript builder

### Files to Modify
1. `handler.rs` — Change `if Python` to `match` on all languages
2. `lsp/server.rs` — Same change
3. `semantic/mod.rs` — Update module declarations

### Estimated: ~5-7 files modified, ~3-4 new files

## Technical Considerations

### File Size Constraint
`symbols.rs` is at 649 lines. Adding Rust + TS builders would push it past 1000. **Must split into directory module.**

### Tree-sitter Node Kinds (Rust)
From existing `rust_symbols.rs` pattern:
- `function_item`, `struct_item`, `enum_item`, `trait_item`, `impl_item`
- `const_item`, `static_item`, `type_item`, `mod_item`
- `macro_definition`, `macro_invocation`
- `use_declaration`, `extern_crate_declaration`
- `visibility_modifier`, `attribute_item`

### Tree-sitter Node Kinds (TypeScript)
From `ts_infer.rs` and tree-sitter-typescript grammar:
- `function_declaration`, `class_declaration`, `interface_declaration`
- `enum_declaration`, `type_alias_declaration`, `namespace_declaration`
- `variable_declaration`, `lexical_declaration`
- `method_definition`, `public_field_definition`
- `import_statement`, `export_statement`

### Scope/Hierarchy Approach
Use scope stack (already exists) + new `parent_id` field:
- `push_scope()` records the current symbol as parent
- `pop_scope()` restores previous parent
- Child symbols get `parent_id = Some(current_parent)`

## Spec Recommendations

### Spec 1: `symbol-model-extension` (spec_type: data-model)
- Extend Symbol struct with `parent_id`, `visibility`, `is_async`, `is_static` fields
- Add new SymbolKind variants
- Add hierarchy query methods to SymbolTable
- Refactor `symbols.rs` into directory module

### Spec 2: `rust-symbol-extraction` (spec_type: algorithm, depends on spec 1)
- Implement `build_rust()` with comprehensive visitor methods
- Handle all Rust tree-sitter node kinds
- Extract type info, doc comments, visibility
- Enable in handler.rs and lsp/server.rs

### Spec 3: `typescript-symbol-extraction` (spec_type: algorithm, depends on spec 1)
- Implement `build_typescript()` with visitor methods
- Handle all TypeScript/TSX node kinds
- Extract type info, JSDoc, export status
- Enable in handler.rs and lsp/server.rs

## Risk Assessment

1. **Medium: Breaking existing Python behavior** — Refactoring symbols.rs could affect Python extraction. Mitigate with existing tests.
2. **Low: MCP interface unchanged** — The MCP tools are language-agnostic; they just relay SymbolTable contents.
3. **Medium: Hierarchy design** — Adding `parent_id` changes the data model. Need backward compatibility for callers expecting flat lists.
4. **Low: tree-sitter version compatibility** — Already using compatible versions.

## Open Questions

None — all clarifications have been resolved. Ready to proceed to planning.
