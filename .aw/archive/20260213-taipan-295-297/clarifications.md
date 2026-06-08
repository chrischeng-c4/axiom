---
change: taipan-295-297
date: 2026-02-13
---

# Clarifications

## Q1: Git Workflow
- **Question**: Which git workflow should we use?
- **Answer**: in_place — work on current feat/taipan branch
- **Rationale**: Continuing from taipan-283-294 work on the same branch. All runtime modules and pipeline are already there.

## Q2: Codegen Scope
- **Question**: Which Cranelift codegen placeholders need to be replaced?
- **Answer**: GetAttr, SetAttr, GetItem, SetItem, MakeList, MakeDict, MakeTuple, Raise — all should emit FFI calls to the corresponding tp_* runtime functions instead of returning iconst(0).
- **Rationale**: These are the 8 MirInst variants added in taipan-283-294 that currently have placeholder codegen. They need real extern calls to the runtime.

## Q3: JIT Backend
- **Question**: Which JIT approach should we use?
- **Answer**: Cranelift JITModule with cranelift-jit crate. Create a JIT backend alongside the existing AOT ObjectModule backend. Wire tp_* runtime functions as symbols in the JIT module.
- **Rationale**: Cranelift already supports JITModule which compiles to executable memory. This is the natural extension of our existing Cranelift ObjectModule backend.

## Q4: CLI Integration
- **Question**: How should 'cclab taipan run' work?
- **Answer**: Use JIT backend: parse → type check → lower → codegen via JITModule → call __main__ entry point. Replace the current 'JIT execution not yet implemented' message.
- **Rationale**: The CLI already has the run subcommand scaffolded in crates/cclab-cli/src/taipan.rs. Just need to wire it to the JIT backend.

