---
change_id: mamba-features-305-316
type: gap_spec_knowledge
created_at: 2026-02-14T09:28:59.233134+00:00
updated_at: 2026-02-14T09:28:59.233134+00:00
---

# Gap Analysis: Spec vs Knowledge

## Responsibility Boundary Misalignments and Pattern Gaps

- **Async/Await Specification (Missing)**
  - Spec: NO spec found for Mamba-specific coroutine scheduling.
  - Knowledge: `orbit/bridge-internals.md` provides a mature pattern for async bridges that is not yet leveraged by any language-specific spec.
  - Severity: **High** (Essential for Feature #313).

- **Memory Management Specification (Missing)**
  - Spec: NO spec found for cycle-detecting GC.
  - Knowledge: `orbit/performance-tuning.md` and standard runtime practices identify the need for explicit cycle detection and GC management, which is not reflected in any existing Mamba spec.
  - Severity: **High** (Essential for Feature #315).

- **Backend Architecture Abstraction**
  - Spec: `mamba-jit-backend` is tightly coupled to Cranelift.
  - Knowledge: Feature #305 requires an LLVM backend for AOT. The knowledge of pluggable backends (from `CodegenBackend` trait in codebase) is not fully reflected in the spec architecture.
  - Severity: **Medium**.

- **Spec-to-Code Contract Adherence**
  - Spec: Existing specs like `mamba-py312-syntax` and `mamba-jit-backend` use basic flowcharts.
  - Knowledge: `spec-to-code/code-generator-contract.md` requires semantic diagrams (Flowchart Plus, Sequence Plus) for automated code generation.
  - Severity: **Medium**.
