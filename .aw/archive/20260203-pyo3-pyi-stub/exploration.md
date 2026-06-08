---
id: pyo3-pyi-stub
type: exploration
created_at: 2026-01-30T03:38:45.669148+00:00
needs_clarification: false
---

# Codebase Exploration

### Codebase Analysis for PyO3 Stub Generator

#### Current Architecture
- **Prism** (`cclab-prism`) uses tree-sitter for multi-language parsing.
- Existing generators are located in `src/gen/python` and `src/gen/rust`.
- Generators typically implement the `CodeGenerator` trait and operate on `SpecIR`.
- CLI commands are defined in `cclab-cli` and dispatched to Prism.
- MCP tools are defined in `cclab-prism/src/mcp/tools.rs`.

#### Implementation Strategy
- A new module `src/gen/python/pyo3.rs` will be created.
- Since this generator operates directly on Rust source code rather than `SpecIR`, it will use `MultiParser` to traverse the Rust AST.
- It will target specific PyO3 attributes: `#[pyclass]`, `#[pyfunction]`, and `#[pymethods]`.
- Type mapping will be implemented to convert Rust types (e.g., `Vec<T>`) to Python type hints (e.g., `list[T]`).
- Docstrings will be extracted from Rust documentation comments.

#### Impact Analysis
- **cclab-cli**: Renaming `Argus` to `Prism` and adding `gen-stub` subcommand.
- **cclab-prism**: New generation module, updated MCP tools, and entry points in `lib.rs`.
- **Dependencies**: Reuses existing `tree-sitter-rust`.

#### Recommendations
- Use a dedicated `PyO3Analyzer` to separate AST traversal from code generation.
- Ensure the CLI supports both individual files and directory-wide scanning.
- Expose the generator via MCP to allow LLMs to generate stubs during development.
