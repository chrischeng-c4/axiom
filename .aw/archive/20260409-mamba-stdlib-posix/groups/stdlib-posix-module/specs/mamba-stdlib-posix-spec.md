---
id: mamba-stdlib-posix-spec
main_spec_ref: "crates/mamba/stdlib/posix.md"
merge_strategy: new
filled_sections: [overview, requirements, test-plan, changes]
fill_sections: [overview, requirements, test-plan, changes]
create_complete: true
---

# Mamba Stdlib Posix Spec

## Overview

Native `posix` module for Mamba, implementing CPython's low-level POSIX system call interface.

In CPython, `posix` is the C module that provides raw POSIX syscall wrappers. The `os` module then imports and re-exports these with a portable Python API. In Mamba, `os_mod.rs` already implements the high-level `os` interface. This module registers the underlying `posix` namespace so that `import posix` works as in CPython.

The module re-exports functions from `os_mod.rs` where possible (getpid, getcwd, getenv, listdir, mkdir, remove, rename, makedirs, rmdir, walk) and adds POSIX-specific functions:
- `posix.uname()` -- returns a 5-tuple via `libc::uname` on Unix
- `posix.environ` -- live dict populated from `std::env::vars()`
- `posix.getuid()` / `posix.getgid()` -- process user/group IDs via libc

Source: `posix_mod.rs` (new file)

Registration: `posix` module name via `register_module("posix", attrs)`.
## Requirements

| ID | Requirement | Priority |
|----|------------|----------|
| R1 | Create `posix_mod.rs` with `pub fn register()` that registers the `posix` module via `register_module("posix", attrs)` following the `os_mod.rs` pattern | P0 |
| R2 | Re-export os_mod functions: `getpid`, `getcwd`, `getenv`, `listdir`, `mkdir`, `remove`, `rename`, `makedirs`, `rmdir`, `walk`, `cpu_count` as dispatch wrappers delegating to `os_mod` implementations | P0 |
| R3 | Implement `posix.environ` as a dict populated from `std::env::vars()` at registration time (each key-value pair as MbValue strings) | P0 |
| R4 | Implement `posix.uname()` using `libc::uname` on Unix targets, returning a 5-element tuple of strings (sysname, nodename, release, version, machine) | P1 |
| R5 | Implement `posix.getuid()` and `posix.getgid()` using `libc::getuid()` / `libc::getgid()` returning integers | P1 |
| R6 | Register module in `stdlib/mod.rs`: add `pub mod posix_mod;` declaration and `posix_mod::register();` call in `register_stdlib()` | P0 |
| R7 | Include `posix.name` constant set to `"posix"` on Unix | P2 |
| R8 | Unit tests: test_posix_getpid, test_posix_getcwd, test_posix_environ, test_posix_uname, test_posix_getuid_getgid | P0 |
## Scenarios
<!-- type: scenarios lang: markdown -->

<!-- TODO -->

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

| Test | Type | Validates |
|------|------|----------|
| `test_posix_getpid` | unit | R2: getpid() returns a positive integer |
| `test_posix_getcwd` | unit | R2: getcwd() returns a non-empty string |
| `test_posix_getenv_existing` | unit | R2: getenv("PATH") returns non-empty string |
| `test_posix_getenv_missing` | unit | R2: getenv with missing key returns default |
| `test_posix_environ_populated` | unit | R3: environ dict is non-empty and contains PATH |
| `test_posix_uname` | unit | R4: uname() returns 5-element tuple of strings |
| `test_posix_getuid` | unit | R5: getuid() returns non-negative integer |
| `test_posix_getgid` | unit | R5: getgid() returns non-negative integer |
| `test_posix_listdir` | unit | R2: listdir(".") returns non-empty list |
| `test_posix_mkdir_remove` | unit | R2: mkdir + remove roundtrip works |
| `test_no_regression` | integration | R6: cargo test -p mamba passes |
## Changes

```yaml
files:
  - path: crates/mamba/src/runtime/stdlib/posix_mod.rs
    action: create
    description: |
      New posix module implementing POSIX system call wrappers.
      - Dispatch wrappers delegating to os_mod functions
      - posix.environ dict from std::env::vars()
      - posix.uname() via libc::uname
      - posix.getuid()/getgid() via libc
      - Unit tests in #[cfg(test)] mod tests
  - path: crates/mamba/src/runtime/stdlib/mod.rs
    action: modify
    description: |
      Add `pub mod posix_mod;` declaration and
      `posix_mod::register();` call in register_stdlib().
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
