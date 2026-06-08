---
id: lens-full-upgrade-spec
main_spec_ref: ~
merge_strategy: new
filled_sections: [overview, requirements, scenarios, changes]
---

# Lens Full Upgrade Spec

## Overview


Full upgrade of cclab-lens covering 7 major areas:

**1. Disk Cache** — Persistent AST index to .cclab/lens/cache/ using bincode. Content hash staleness check. Warm load on daemon startup. Background write after analysis. Hash-mismatch-only invalidation.

**2. Lint Rule Densification** — Expand TS/JS/Rust/CSS from 5 to 15+ rules each. Per-rule severity grading (bug=Error, style=Warning, suggestion=Info).

**3. Auto-Fix** — Diagnostic carries optional Fix (text edits). Single apply + fix-all batch mode. LSP code actions. MCP tool: lens_fix.

**4. Go Type Inference** — Interface satisfaction, full generic constraint inference, method set analysis, type assertion validation, channel type tracking, composite literal inference.

**5. New Languages** — TOML (serde-based), SQL (tree-sitter, PostgreSQL+MySQL, injection detection), Proto/gRPC (tree-sitter, proto3), GraphQL (tree-sitter, schema validation).

**6. Cross-File Import Graph** — Project-wide dependency graph from imports. Circular dependency detection. Unused file detection. Auto-detect entry points. Python/TS/Rust/Go import resolution. MCP tool: lens_import_graph.

**7. Formatter** — Unified interface wrapping rustfmt/prettier/gofmt/black/terraform-fmt. Binary detection. Diff mode. LSP format-on-save. Warn+skip if missing. MCP tool: lens_format.

### Key Decisions
- Cache: hash-mismatch-only invalidation (no LRU/TTL)
- Lint severity: per-rule grading
- Auto-fix: single + fix-all
- Go generics: full constraint inference
- SQL: PostgreSQL + MySQL dialects
- Proto: proto3 only
- Entry points: auto-detect (main.py, index.ts, main.rs, main.go)
- Formatter: warn+skip if binary not found
## Requirements


### R1 - Disk Cache (DiskCache struct)

```yaml
id: R1
priority: high
status: draft
```

- `DiskCache` struct: cache_dir, manifest (RwLock<CacheManifest>)
- `PersistedEntry`: version, content_hash, semantic_model, diagnostics (bincode serialized)
- `CacheManifest`: version, entries HashMap<String, ManifestEntry>
- `load(path, current_hash)` → Option<PersistedEntry>
- `store(path, entry)` → background write
- `flush_manifest()` on shutdown
- Integration: handler.rs ensure_analyzed cache-miss → disk → full parse
- Storage: `{project}/.cclab/lens/cache/{path_hash}.idx` + `manifest.bin`

### R2 - TypeScript Lint Expansion (5→15+)

```yaml
id: R2
priority: high
status: draft
```

New rules: TS006 no-non-null-assertion, TS007 no-floating-promises (unhandled async), TS008 strict-boolean-expressions, TS009 prefer-const, TS010 no-unnecessary-type-assertion, TS011 no-inferrable-types, TS012 prefer-optional-chain, TS013 no-empty-interface, TS014 no-duplicate-enum-values, TS015 consistent-type-imports.

### R3 - JavaScript Lint Expansion (5→15+)

```yaml
id: R3
priority: high
status: draft
```

New rules: JS006 no-eval, JS007 no-implied-eval, JS008 no-proto, JS009 no-with, JS010 no-alert, JS011 no-debugger, JS012 no-caller, JS013 no-extend-native, JS014 no-new-wrappers, JS015 no-throw-literal.

### R4 - Rust Lint Expansion (5→15+)

```yaml
id: R4
priority: high
status: draft
```

New rules: RS006 unwrap-usage (prefer expect or ?), RS007 todo-unimplemented (leftover markers), RS008 large-enum-variant, RS009 needless-return, RS010 redundant-clone, RS011 manual-map (if-let → map), RS012 single-match (match → if-let), RS013 missing-docs-public, RS014 wildcard-imports, RS015 dbg-macro-left.

### R5 - CSS Lint Expansion (5→15+)

```yaml
id: R5
priority: high
status: draft
```

New rules: CSS011 no-important, CSS012 no-universal-selector, CSS013 no-id-selectors, CSS014 shorthand-property-overrides, CSS015 z-index-max (>9999), CSS016 no-duplicate-selectors, CSS017 declaration-no-important, CSS018 no-descending-specificity, CSS019 color-no-invalid-hex, CSS020 font-family-no-missing-generic.

### R6 - Auto-Fix Framework

```yaml
id: R6
priority: high
status: draft
```

- `Fix` struct: description, text_edits Vec<TextEdit>, is_preferred
- `TextEdit`: range, new_text
- Add `fix: Option<Fix>` field to `Diagnostic`
- Implement fixes for: unused imports (remove line), var→const (JS), ==→=== (JS), trailing whitespace, missing semicolons
- `apply_fix(source, fix)` → String
- `apply_all_fixes(source, diagnostics)` → String (non-overlapping edits)
- LSP: code action provider returning fixes
- MCP tool: `lens_fix(path, rule_id?, apply_all?)`

### R7 - Go Type Inference Deep

```yaml
id: R7
priority: medium
status: draft
```

- Interface satisfaction: check if struct's method set satisfies interface
- Generic type params: resolve `func Foo[T constraint](x T)` calls
- Type constraint inference: infer T from argument types against constraints
- Method set analysis: value receiver vs pointer receiver rules
- Type assertion validation: `x.(Type)` check if plausible
- Channel type tracking: `chan T`, `<-chan T`, `chan<- T` direction
- Composite literal inference: `Point{X: 1, Y: 2}` → struct field types

### R8 - TOML Checker

```yaml
id: R8
priority: medium
status: draft
```

- Language::Toml variant, `.toml` extension
- Line-based or serde_toml parsing (no tree-sitter needed)
- Rules: TM001 syntax error, TM002 duplicate key, TM003 unknown section (Cargo.toml/pyproject.toml schema), TM004 deprecated key, TM005 type mismatch
- Symbol builder: sections, keys, values

### R9 - SQL Support

```yaml
id: R9
priority: medium
status: draft
```

- Language::Sql variant, `.sql` extension
- tree-sitter-sql grammar
- Rules: SQ001 syntax error, SQ002 SELECT *, SQ003 missing WHERE on UPDATE/DELETE, SQ004 implicit join (use explicit JOIN), SQ005 deprecated function
- PostgreSQL-specific: PG001 serial→identity, PG002 text vs varchar
- MySQL-specific: MY001 engine specification, MY002 charset specification
- Injection detection: scan Python/JS/Go string literals for SQL patterns
- Symbol builder: tables, columns, procedures

### R10 - Proto/gRPC Support

```yaml
id: R10
priority: medium
status: draft
```

- Language::Proto variant, `.proto` extension
- tree-sitter-protobuf grammar (if available, else line-based)
- proto3 only
- Rules: PB001 syntax error, PB002 field number uniqueness, PB003 reserved field usage, PB004 missing package, PB005 service without rpc, PB006 repeated without message type, PB007 import not used
- Symbol builder: messages, fields, services, rpcs, enums

### R11 - GraphQL Support

```yaml
id: R11
priority: medium
status: draft
```

- Language::GraphQL variant, `.graphql`, `.gql` extension
- tree-sitter-graphql grammar (if available, else line-based)
- Rules: GQ001 syntax error, GQ002 undefined type reference, GQ003 deprecated field usage (@deprecated), GQ004 deep nesting (>5 levels), GQ005 missing field descriptions, GQ006 unused fragment, GQ007 duplicate field in selection
- Symbol builder: types, queries, mutations, subscriptions, fragments

### R12 - Cross-File Import Graph

```yaml
id: R12
priority: high
status: draft
```

- `ImportGraph` struct: nodes (files), edges (imports), entry_points
- Build from parsed files' import statements (Python/TS/Rust/Go)
- `detect_entry_points()`: scan for main.py, index.ts, main.rs, main.go, etc.
- `find_circular_dependencies()` → Vec<Vec<PathBuf>> (cycles)
- `find_unused_files()` → Vec<PathBuf> (no inbound from entries)
- `resolve_import(from, import_path, language)` → Option<PathBuf>
- Incremental: update on file change
- MCP tool: `lens_import_graph(path?, mode=full|circular|unused)`

### R13 - Formatter Integration

```yaml
id: R13
priority: medium
status: draft
```

- `FormatterRegistry`: language→FormatterConfig mapping
- `FormatterConfig`: binary_name, args, check_args (diff mode)
- Detection: `which {binary}` on init
- Formatters: rustfmt (Rust), prettier (JS/TS/HTML/CSS/JSON/YAML/MD/GraphQL), gofmt (Go), black (Python), terraform fmt (HCL), pg_format (SQL)
- `format(path)` → FormattedResult { original, formatted, diff }
- `format_check(path)` → bool (would change?)
- LSP: textDocument/formatting handler
- MCP tool: `lens_format(path, check_only?, language?)`
- Missing binary: log warning, skip silently
## Scenarios


### S1 - Disk Cache Warm Load

```yaml
id: S1
covers: [R1]
```

**Given** daemon was previously running and analyzed 500 files
**When** daemon restarts
**Then** `DiskCache::new()` loads manifest.bin, and `ensure_analyzed()` hits disk cache for unchanged files (content_hash match), re-parses only tree-sitter Tree (~1ms/file), skipping full semantic analysis

### S2 - Disk Cache Miss on Changed File

```yaml
id: S2
covers: [R1]
```

**Given** file was modified since last analysis
**When** `ensure_analyzed()` is called
**Then** content_hash mismatch triggers full `check_file()`, result stored to in-memory + background disk write

### S3 - Lint Expansion Catches New Issues

```yaml
id: S3
covers: [R2, R3, R4, R5]
```

**Given** TypeScript file contains `x!.foo`, `let a = 1` (inferrable type), `import type { X }` mixed with value imports
**When** lint check runs
**Then** TS006 (no-non-null-assertion), TS011 (no-inferrable-types), TS015 (consistent-type-imports) all fire with correct severity

### S4 - Auto-Fix Single Apply

```yaml
id: S4
covers: [R6]
```

**Given** diagnostic with Fix (e.g., JS unused import → remove line)
**When** `apply_fix(source, fix)` called
**Then** source returned with import line removed, no other changes

### S5 - Auto-Fix Batch Apply

```yaml
id: S5
covers: [R6]
```

**Given** file has 3 fixable diagnostics (non-overlapping ranges)
**When** `apply_all_fixes(source, diagnostics)` called
**Then** all 3 fixes applied in reverse-offset order, result is valid source

### S6 - Go Interface Satisfaction

```yaml
id: S6
covers: [R7]
```

**Given** Go file: `type Writer interface { Write([]byte) (int, error) }` and `type MyWriter struct{}` with `func (w MyWriter) Write(p []byte) (int, error)`
**When** type inference runs
**Then** MyWriter recognized as satisfying Writer interface

### S7 - TOML Cargo.toml Lint

```yaml
id: S7
covers: [R8]
```

**Given** Cargo.toml with duplicate `[dependencies]` section
**When** TOML checker runs
**Then** TM002 (duplicate key) diagnostic emitted

### S8 - SQL Injection Detection

```yaml
id: S8
covers: [R9]
```

**Given** Python file containing `cursor.execute(f"SELECT * FROM users WHERE id = {user_id}")`
**When** lint check runs on .py file
**Then** SQL injection warning emitted for f-string SQL pattern

### S9 - Proto Field Number Uniqueness

```yaml
id: S9
covers: [R10]
```

**Given** proto3 file with message having two fields with same number
**When** proto checker runs
**Then** PB002 diagnostic emitted with both field locations

### S10 - GraphQL Deep Nesting

```yaml
id: S10
covers: [R11]
```

**Given** GraphQL query with 6 levels of nested field selections
**When** GraphQL checker runs
**Then** GQ004 (deep nesting >5) warning emitted

### S11 - Circular Import Detection

```yaml
id: S11
covers: [R12]
```

**Given** Python files: a.py imports b, b.py imports c, c.py imports a
**When** `find_circular_dependencies()` called
**Then** returns cycle [a.py, b.py, c.py]

### S12 - Unused File Detection

```yaml
id: S12
covers: [R12]
```

**Given** project with entry point main.py, orphan file utils_old.py not imported by anything
**When** `find_unused_files()` called
**Then** utils_old.py appears in unused list

### S13 - Format Check

```yaml
id: S13
covers: [R13]
```

**Given** unformatted Rust file, rustfmt available
**When** `format_check(path)` called
**Then** returns false (would change), `format(path)` returns diff

### S14 - Missing Formatter Graceful Skip

```yaml
id: S14
covers: [R13]
```

**Given** Go file, gofmt not installed
**When** `format(path)` called
**Then** logs warning, returns FormattedResult with no changes (skip)
## Diagrams

### Sequence Diagram
<!-- TODO -->

### Flowchart
<!-- TODO -->

### Class Diagram
<!-- TODO -->

### State Diagram
<!-- TODO -->

### ERD
<!-- TODO -->

## API Spec

### OpenAPI 3.1
<!-- TODO -->

### OpenRPC 1.3
<!-- TODO -->

### AsyncAPI 2.6
<!-- TODO -->

### Serverless Workflow 0.8
<!-- TODO -->

## Test Plan

<!-- TODO -->

## Changes


### Group 1: Disk Cache (R1)

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-lens/Cargo.toml` | modify | Add `bincode = "1.3"` dependency |
| `crates/cclab-lens/src/server/disk_cache.rs` | new | `DiskCache`, `PersistedEntry`, `CacheManifest`, load/store/flush |
| `crates/cclab-lens/src/server/mod.rs` | modify | Add `pub mod disk_cache;` |
| `crates/cclab-lens/src/server/handler.rs` | modify | Add `disk_cache` field, integrate in `ensure_analyzed`/`check_file` |
| `crates/cclab-lens/src/server/daemon.rs` | modify | Call `flush_cache()` on shutdown |
| `crates/cclab-lens/src/storage.rs` | modify | Add `resolve_cache_dir()` |
| `crates/cclab-lens/src/types/cache.rs` | modify | Add `Serialize, Deserialize` to `ContentHash` |

### Group 2: Lint Densification (R2-R5)

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-lens/src/lint/typescript.rs` | modify | Add TS006-TS015 rules |
| `crates/cclab-lens/src/lint/javascript.rs` | modify | Add JS006-JS015 rules |
| `crates/cclab-lens/src/lint/rust_lint.rs` | modify | Add RS006-RS015 rules |
| `crates/cclab-lens/src/lint/css.rs` | modify | Add CSS011-CSS020 rules |

### Group 3: Auto-Fix (R6)

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-lens/src/lint/autofix.rs` | new | `Fix`, `TextEdit`, `apply_fix`, `apply_all_fixes` |
| `crates/cclab-lens/src/lint/mod.rs` | modify | Add `pub mod autofix;`, add `fix` field to `Diagnostic` |
| `crates/cclab-lens/src/lsp/server.rs` | modify | Add code action provider |
| `crates/cclab-lens/src/lint/javascript.rs` | modify | Attach fixes to JS fixable rules |
| `crates/cclab-lens/src/lint/typescript.rs` | modify | Attach fixes to TS fixable rules |

### Group 4: Go Type Inference (R7)

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-lens/src/semantic/types/go.rs` | modify | Interface satisfaction, generics, method sets, channels |

### Group 5: New Languages (R8-R11)

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-lens/Cargo.toml` | modify | Add tree-sitter-sql, toml crate deps |
| `crates/cclab-lens/src/syntax/parser.rs` | modify | Add Toml, Sql, Proto, GraphQL language variants |
| `crates/cclab-lens/src/lint/toml.rs` | new | TomlChecker TM001-TM005 |
| `crates/cclab-lens/src/lint/sql.rs` | new | SqlChecker SQ001-SQ005, PG001-PG002, MY001-MY002, injection detection |
| `crates/cclab-lens/src/lint/proto.rs` | new | ProtoChecker PB001-PB007 |
| `crates/cclab-lens/src/lint/graphql.rs` | new | GraphqlChecker GQ001-GQ007 |
| `crates/cclab-lens/src/lint/mod.rs` | modify | Register new checkers |
| `crates/cclab-lens/src/semantic/symbols/mod.rs` | modify | Add symbol builders for new languages |
| `crates/cclab-lens/src/semantic/symbols/toml_sym.rs` | new | TOML symbol extraction |
| `crates/cclab-lens/src/semantic/symbols/sql_sym.rs` | new | SQL symbol extraction |
| `crates/cclab-lens/src/semantic/symbols/proto_sym.rs` | new | Proto symbol extraction |
| `crates/cclab-lens/src/semantic/symbols/graphql_sym.rs` | new | GraphQL symbol extraction |
| `crates/cclab-lens/src/lib.rs` | modify | Add new languages to check_file routing |

### Group 6: Cross-File Import Graph (R12)

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-lens/src/graph/mod.rs` | new | `ImportGraph` struct, build, query methods |
| `crates/cclab-lens/src/graph/resolve.rs` | new | Per-language import resolution |
| `crates/cclab-lens/src/lib.rs` | modify | Add `pub mod graph;` |

### Group 7: Formatter (R13)

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-lens/src/format/mod.rs` | new | `FormatterRegistry`, `FormatterConfig`, format/check |
| `crates/cclab-lens/src/format/detect.rs` | new | Binary detection via `which` |
| `crates/cclab-lens/src/lib.rs` | modify | Add `pub mod format;` |
| `crates/cclab-lens/src/lsp/server.rs` | modify | Add textDocument/formatting handler |
# Reviews