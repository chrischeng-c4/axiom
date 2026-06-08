---
change: taipan-283-294
date: 2026-02-13
---

# Clarifications

## Q1: Git Workflow
- **Question**: Which git workflow to use for this change?
- **Answer**: in_place — work on current feat/taipan branch
- **Rationale**: All 12 features are part of the same Taipan Phase 2 effort, keeping them on the existing feature branch avoids branch proliferation.

## Q2: Implementation Scope
- **Question**: Implementation depth — these 12 features are massive. What's the target scope?
- **Answer**: Full CPython parity — comprehensive implementation matching Python 3.12 semantics for all 12 features
- **Rationale**: Taipan aims to be a production-grade Python compiler. Full CPython parity ensures compatibility with real-world Python code.

## Q3: Compiler Phases
- **Question**: Which compiler phases should each feature touch?
- **Answer**: Full pipeline — Lower + Codegen + Runtime for each feature, end-to-end compilable
- **Rationale**: Each feature must work end-to-end from source to native code. Partial implementations would leave features unusable.

## Q4: Async/Generator Model
- **Question**: Should async/await (#293) and generators (#290) use stackful coroutines or state machines?
- **Answer**: State machine transformation at MIR level for both. Generators store state in TpObject with ObjKind::Generator. Async functions use the same state machine approach + embed a tokio runtime in the Taipan runtime library. FFI calls bridge Cranelift-generated code to tokio for I/O.
- **Rationale**: State machines are more memory-efficient and avoid platform-specific stack management. Tokio provides battle-tested async I/O without reinventing the wheel. The FFI bridge is natural since Taipan already has a robust FFI system.

## Q5: Issues Covered
- **Question**: Which issues are covered by this change?
- **Answer**: All 12 Taipan issues: #283 (exception handling), #284 (string ops), #285 (list/dict/tuple ops), #286 (iterator/for-loop), #287 (class/methods/inheritance), #288 (dynamic dispatch/operator overloading), #289 (closure/nested functions), #290 (generators/yield), #291 (comprehensions), #292 (module import), #293 (async/await), #294 (decorators)
- **Rationale**: These form a cohesive set of Phase 2 features that build the Taipan runtime and extend the compiler to handle real Python programs.

