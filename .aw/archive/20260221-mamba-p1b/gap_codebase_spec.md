---
change_id: mamba-p1b
type: gap_codebase_spec
created_at: 2026-02-21T14:55:47.810655+00:00
updated_at: 2026-02-21T14:55:47.810655+00:00
---

# Gap Analysis: Codebase vs Spec

| Type | Gap | Severity | Source | Description |
| :--- | :--- | :--- | :--- | :--- |
| code_without_spec | **Value Representation & Nan-tagging** | High | `crates/mamba/src/runtime/value.rs` | `runtime/value.rs` implements Nan-tagging for `MbValue`, but no specification defines the specific tag bits (NAN_PREFIX, TAG_PTR, etc.) or the overall memory layout for values. `mamba-type-system` focuses on high-level generics/protocols. |
| code_without_spec | **MIR Instruction Set** | High | `crates/mamba/src/mir/mod.rs` | `mir/mod.rs` defines the `MirInst` instruction set and its components (`MirBinOp`, `MirUnaryOp`), but there is no specification detailing the semantics, format, or exhaustive list of instructions in the Mamba Intermediate Representation. |
| spec_without_code | **Standard Library Core** | High | `cclab-mamba/mamba-stdlib-core` | Specification exists for the core standard library, but no implementation files or modules are present in the analyzed codebase context. |
| spec_without_code | **String Runtime** | High | `cclab-mamba/mamba-string-runtime` | Specification exists for the runtime string implementation, but no implementation files are present in the analyzed codebase context. |
| code_without_spec | **Runtime Core & RC** | Medium | `crates/mamba/src/runtime/rc.rs` | `runtime/rc.rs` implements the reference counting mechanism (`MbObjectHeader`, `mb_retain`, `mb_release`), but these core primitives are not formally specified. `mamba-gc-runtime` assumes their existence but focuses on cycle detection. |
| spec_without_code | **Iteration Protocol** | Medium | `cclab-mamba/mamba-iteration-protocol` | Specification exists for the iteration protocol (iter/next), but no implementation logic is present in the analyzed codebase context. |
| spec_without_code | **Parser & Syntax** | Medium | `cclab-mamba/mamba-py312-syntax` | Specification exists for Python 3.12 syntax support, but no parser or AST-related implementation files are present in the analyzed codebase context. |
| spec_without_code | **Async Runtime** | Low | `cclab-mamba/mamba-async-runtime` | Specification exists for the async/await runtime, but no implementation files are present in the analyzed codebase context. |
