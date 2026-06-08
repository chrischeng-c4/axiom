---
change_id: taipan-295-297
type: gap_spec_knowledge
created_at: 2026-02-13T07:25:18.283908+00:00
updated_at: 2026-02-13T07:25:18.283908+00:00
---

# Gap Analysis: Spec vs Knowledge

## Knowledge Patterns Not Reflected in Spec

- **GIL Management for JIT Execution**: The critical `GIL Release Pattern` documented in `orbit/bridge-internals.md` is not reflected in any Taipan spec. When the JIT backend executes Taipan code, it must properly manage the Python GIL if it interacts with any Python-owned resources, yet this is currently missing from `taipan-backend-cranelift.md` and `taipan-cli-integration.md`. (Severity: High)
- **FFI Error Handling for Raise**: The `FFI Error Handling Pattern` (bridging panics/errors to structured exceptions) is not documented in `taipan-backend-cranelift.md` for the `Raise` instruction. The spec should define how the backend calls the runtime exception handler. (Severity: Medium)

## Responsibility Boundary Misalignments

- **Execution Context Misalignment**: `taipan-cli-integration.md` describes the `run` command as "spawning the resulting process", which is an AOT-centric view. The JIT knowledge (and project goal) implies in-memory execution, requiring a boundary realignment where the CLI directly invokes the JIT-compiled entry point. (Severity: High)
