---
id: conductor-mamba-p0-spec
main_spec_ref: "cclab-mamba/conductor-mamba-p0-spec.md"
---

# Conductor Mamba P0 Spec

## Overview
<!-- type: overview lang: markdown -->

This spec outlines the P0 capabilities necessary for running the Conductor project on the Mamba engine. It addresses critical native integration and Python language gaps. Specifically, it specifies the creation of a `cclab-mamba-registry` crate containing the `MambaModule` trait and distributed slice for module registration; updating the `mamba.toml` configuration to support dynamic crate loading mapping via `[crates]`; adding standardized helpers for `MbValue` and Rust type conversions; and resolving critical language feature blockers such as import aliases, relative imports, bare type annotations, dict unpacking, and advanced f-strings.
## Requirements
<!-- type: overview lang: markdown -->

### R1: Mamba Registry Crate (`cclab-mamba-registry`)
- Introduce the `cclab-mamba-registry` crate containing the `MambaModule` trait and the `MAMBA_MODULES` distributed slice.
- Implement `ModuleRegistrar` and `RuntimeSymbol` structures.
- Add the `rt_sym!` macro for simplified runtime symbol declaration.

### R2: `mamba.toml` Configuration Updates
- Replace `[dependencies]` with a new `[crates]` table within `MambaConfig`.
- Support fields `crate` (crate name), `version`, `path`, `expose` (whitelisted symbols), and `module` (import path) under each crate entry.
- Add `[paths]` configuration to specify `search` paths.
- Implement validation to ensure `version` or `path` is present and `expose` is non-empty.

### R3: `MbValue` Conversion Helpers
- Implement `FromMbValue` and `IntoMbValue` traits in `cclab-mamba-registry` for primitive types (`i64`, `bool`, `f64`), `String`, `Option`, `Vec`, and `HashMap`.
- Provide `mb_wrap_native<T>` and `mb_unwrap_native<T>` for opaque Rust structures via `Box<dyn Any>`.
- Introduce `MbConvError` for type conversion failure handling.

### R4: Mamba Language Feature Blockers
- **Imports**: Resolve compiler support for import aliases (`import sys as system`) and relative imports (`from . import models`).
- **Annotations**: Fix support for bare type annotations (e.g., `id: int` without default value).
- **Literals & Strings**: Support dictionary unpacking in literals (`{**defaults, **overrides}`) and advanced PEP-701 f-strings.
- **Classes & Context**: Fix or implement `__init_subclass__` behavior, metaclass + generics combination, and PEP-617 parenthesized `with` statements.
- **Typing & Reflection**: Ensure standard library supports runtime type reflection (`get_type_hints`), the `ast` and `inspect` modules, `tomllib`, and `@runtime_checkable` Protocols.
## Scenarios
<!-- type: overview lang: markdown -->

### Scenario: MambaModule Registration and Symbol Resolution
- **WHEN** the Mamba compiler links an auto-registered binding crate (e.g. `cclab-pg-mamba`) utilizing the `MAMBA_MODULES` distributed slice.
- **THEN** the module is correctly recognized, its attributes are collected via `ModuleRegistrar`, and `rt_sym!` declared functions are exposed for JIT execution.

### Scenario: Config Processing with [crates]
- **WHEN** the user provides a `mamba.toml` file with a `[crates]` table containing properties `path`, `expose`, and an explicit `module` prefix.
- **THEN** the compiler correctly deserializes the configuration, defaults the missing `crate` name properly, and restricts script-level imports to only the symbols listed in `expose`.

### Scenario: MbValue Round-Trip Conversions
- **WHEN** converting native Rust types (such as `HashMap<String, Option<i64>>` or `Box<dyn Any>`) into `MbValue` and back.
- **THEN** the `FromMbValue` and `IntoMbValue` implementations execute successfully without data loss, correctly wrapping/unwrapping via `mb_wrap_native` and `mb_unwrap_native` for opaque structs.

### Scenario: Import Aliases and Relative Imports
- **WHEN** a Mamba script utilizes `import x as y` and `from . import z`.
- **THEN** the parser and lowerer process the instructions without emitting an XFAIL or NotImplemented error, resolving the appropriate module paths based on the multi-package hierarchy.

### Scenario: Bare Type Annotations in ORM
- **WHEN** a Mamba script defines a class with bare type annotations (e.g., `id: int` without assignments).
- **THEN** the class parses and compiles successfully, allowing ORM frameworks like SQLAlchemy or Pydantic (using `__init_subclass__`) to reflect upon the fields using runtime type hints.
## Diagrams
<!-- type: overview lang: markdown -->

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- score-td-placeholder -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- score-td-placeholder -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- score-td-placeholder -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- score-td-placeholder -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- score-td-placeholder -->

## API Spec
<!-- type: overview lang: markdown -->

### REST API
<!-- type: rest-api lang: yaml -->
<!-- score-td-placeholder -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- score-td-placeholder -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- score-td-placeholder -->

### CLI
<!-- type: cli lang: yaml -->
<!-- score-td-placeholder -->

### Schema
<!-- type: schema lang: json -->
<!-- score-td-placeholder -->

### Config
<!-- type: config lang: json -->
<!-- score-td-placeholder -->

## Test Plan
<!-- type: test_plan lang: markdown -->

- **Registry Crate**: Introduce comprehensive unit tests in `cclab-mamba-registry` asserting the `ModuleRegistrar` and `MambaModule` trait macros correctly register expected symbols.
- **Config Parsing**: Add schema validation tests in `crates/mamba/src/config/schema.rs` parsing various edge-cases of the `[crates]` and `[paths]` sections to ensure missing values correctly fall back to expected defaults.
- **Type Conversions**: Develop rigorous round-trip unit tests within `crates/cclab-mamba-registry/src/convert.rs` validating `MbValue` to Rust native mappings and vice versa, including the downcasting of `Box<dyn Any>`.
- **Language Blockers**: Remove or update existing `XFAIL` status markers in the Mamba testing suite (`crates/mamba/tests/...`) for features like `import aliases`, `relative imports`, `dict unpacking`, and `bare type annotations`. Validate success across realistic Conductor project sample snippets.
## Changes
<!-- type: overview lang: markdown -->

### `crates/cclab-mamba-registry` (CREATE)
- **`src/lib.rs`**: Define `MambaModule` trait, `RuntimeSymbol`, `ModuleRegistrar`, and `MAMBA_MODULES` distributed slice macro.
- **`src/convert.rs`**: Implement `FromMbValue` and `IntoMbValue` traits and blanket implementations for primary data types.

### `crates/cclab-mamba` (MODIFY)
- **`src/config/schema.rs`**: Update `MambaConfig` to substitute `dependencies` with `crates`, defining the `CrateConfig` struct and schema properties (`path`, `expose`, `module`).
- **Parser/Lowerer (e.g. `src/parser/` and `src/lower/`)**: Add logic to properly parse, expand, and enforce typing rules on import aliases, relative imports, bare annotations, dict unpacking, and PEP-701 string forms to resolve missing feature support.
- **Runtime Integration**: Expose integration capabilities so ORMs can rely on `__init_subclass__` and generic reflection behaviors.
## Wireframe
<!-- type: wireframe lang: yaml -->

```yaml
# score-td-placeholder
```

## Component
<!-- type: component lang: yaml -->

```yaml
# score-td-placeholder
```

## Design Token
<!-- type: design-token lang: yaml -->

```yaml
# score-td-placeholder
```

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->
