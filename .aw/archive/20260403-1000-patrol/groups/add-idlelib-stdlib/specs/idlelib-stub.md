---
id: idlelib-stub
main_spec_ref: "crates/mamba/stdlib/idlelib.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, changes]
filled_sections: [overview, requirements, scenarios, changes]
create_complete: true
---

# Idlelib Stub

## Overview

Stub implementation of Python's `idlelib` package for Mamba. Registers the `idlelib` module namespace so `import idlelib` succeeds. All functional APIs raise `NotImplementedError` — no Tkinter GUI functionality is provided.

idlelib is CPython's IDLE editor internals package. It contains ~70 submodules for the IDLE IDE (editor, debugger, configuration, etc.). Since IDLE is rarely used in production and depends heavily on Tkinter, Mamba provides only a stub package that:

- Registers the top-level `idlelib` module with standard attributes (`__name__`, `__file__`, `__package__`)
- Exposes key submodule names as stub attributes that raise `NotImplementedError` on access
- Follows the existing `register()` + `register_module()` pattern per `native-implementations.md`
## Requirements

### R1: Module Registration

Register `idlelib` as a native stdlib module in Mamba's module registry.

**Acceptance criteria**
- `import idlelib` succeeds without error.
- Module has `__name__` set to `"idlelib"`.
- Module is registered via `register()` + `register_module("idlelib", attrs)` pattern.
- `idlelib_mod::register()` is called from `register_stdlib()` in `mod.rs`.
- `pub mod idlelib_mod;` is declared in `mod.rs` under the `// P3 stdlib modules` block.

### R2: Stub Attributes

Expose key idlelib submodule/function names as stub attributes.

**Acceptance criteria**
- The following attribute names are registered: `idle`, `run`, `idle_test`, `PyShell`, `config`, `colorizer`, `autocomplete`, `calltip`, `debugger`, `editor`, `filelist`, `outwin`, `rpc`.
- Each attribute resolves to a symbol string (e.g., `"mb_idlelib_idle"`).
- Calling any stub function raises `NotImplementedError` with a descriptive message (e.g., `"idlelib.idle is not implemented in Mamba"`).

### R3: Package Semantics

Behave as a Python package (directory-like module).

**Acceptance criteria**
- `idlelib.__package__` returns `"idlelib"`.
- `from idlelib import idle` succeeds (returns a stub).
- `idlelib.__path__` attribute is present (can be an empty list).

## Non-goals

- Full Tkinter GUI functionality.
- Actual IDLE editor, debugger, or configuration UI.
- Submodule file-level imports (e.g., `idlelib.idle_test.test_foo`) beyond top-level stubs.
- Widget rendering or event loop integration.
## Scenarios

### Scenario: Basic import succeeds
- **WHEN** `import idlelib` is executed.
- **THEN** No error is raised and `idlelib` is available in the namespace.
- **AND** `idlelib.__name__` equals `"idlelib"`.

### Scenario: Submodule attribute access returns stub
- **WHEN** `from idlelib import idle` is executed.
- **THEN** `idle` is bound to a stub value (symbol string).
- **AND** Calling `idle()` raises `NotImplementedError` with message containing `"idlelib.idle"`.

### Scenario: Package attributes present
- **WHEN** `idlelib.__package__` is accessed.
- **THEN** It returns `"idlelib"`.
- **AND** `idlelib.__path__` is present as an empty list.

### Scenario: Multiple stub attributes accessible
- **WHEN** `idlelib.PyShell`, `idlelib.config`, `idlelib.editor` are accessed.
- **THEN** Each returns a valid stub attribute (no `AttributeError`).

### Scenario: Module cached in sys.modules
- **WHEN** `import idlelib` is called twice.
- **THEN** The same module object is returned (sys.modules caching).
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

<!-- TODO -->

## Changes

```yaml
files:
  - path: crates/mamba/src/runtime/stdlib/idlelib_mod.rs
    action: create
    description: |
      New stub module file for idlelib.
      - `pub fn register()`: registers "idlelib" via `register_module("idlelib", attrs)`
      - Attrs map: idle, run, idle_test, PyShell, config, colorizer, autocomplete,
        calltip, debugger, editor, filelist, outwin, rpc → symbol strings
      - Package attrs: __package__ = "idlelib", __path__ = empty list
      - Stub functions: `mb_idlelib_*()` each raises NotImplementedError
      - Unit tests: test_register, test_stub_raises
    requirements: [R1, R2, R3]

  - path: crates/mamba/src/runtime/stdlib/mod.rs
    action: modify
    description: |
      - Add `pub mod idlelib_mod;` under `// P3 stdlib modules` block
      - Add `idlelib_mod::register();` in `register_stdlib()` function
    requirements: [R1]
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
