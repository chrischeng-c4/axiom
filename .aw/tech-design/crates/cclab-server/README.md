# CCLab Development Guide

## MCP Tools
<!-- type: doc lang: markdown -->

cclab-server 提供 37 個 MCP tools：

### SDD Tools (`sdd_*`) - 23 tools

- **Proposal**: create, append_review
- **Spec**: create, validate, list
- **Tasks**: create, get_task
- **Knowledge**: read, list, write
- **Implementation**: read_all_requirements, list_changed_files, create_review
- **Mermaid diagrams**: flowchart, sequence, class, state, erd, mindmap, requirement, journey
- **API specs**: openapi, asyncapi, openrpc, serverless_workflow

### Lens Tools (`lens_*`) - 14 tools

- **Analysis**: check, diagnostics, symbols, hover, definition, references
- **Navigation**: type_at
- **Config**: get_config, set_python_paths, configure_venv, detect_environment
- **Index**: index_status, invalidate, list_modules
- **Generation**: generate_from_spec, spec_to_mermaid, code_to_mermaid

## Build Commands
<!-- type: doc lang: markdown -->

```bash
# Build CLI (debug)
cargo build -p cclab-cli

# Install to ~/.cargo/bin
cargo build -p cclab-cli && cp target/debug/cclab ~/.cargo/bin/

# Build Python extension
cargo run -p cclab-server

# Run tests
cargo test -p cclab-lens --lib
cargo test -p cclab-sdd --lib
```

## Spec-Driven Development
<!-- type: doc lang: markdown -->

使用 SDD 工作流開發 cclab 自己：

1. `/cclab:sdd:run-change <id> "<description>"` - 建立 proposal、實作、歸檔


## File Size Limits
<!-- type: doc lang: markdown -->

- **Maximum**: 1000 lines (must split)
- **Consider splitting**: 500+ lines

## Testing Strategy
<!-- type: doc lang: markdown -->

- **Unit tests**: `#[cfg(test)] mod tests` 在各模組內
- **Integration tests**: `tests/` 目錄
- **Test patterns**:
  - Security sanitization tests
  - Round-trip conversion tests
  - Thread-safety tests
  - Error handling tests

## Conventions
<!-- type: doc lang: markdown -->

- **Naming**: snake_case (Rust standard)
- **Module structure**: One module per file, re-export in mod.rs
- **Documentation**: Rustdoc with `//!` for module docs, `///` for items
- **Imports**: Group by std → external → internal

## Project Statistics
<!-- type: doc lang: markdown -->

| Metric | Value |
|--------|-------|
| Total Crates | 35 |
| Rust Source Files | 646 |
| Total Lines | ~221,546 |
| MCP Tools | 37 |
| Test Coverage | 67+ tests (P0 sprint) |
