---
change_id: mamba-features-305-316
type: gap_codebase_knowledge
created_at: 2026-02-14T09:28:24.614008+00:00
updated_at: 2026-02-14T09:28:24.614008+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Pattern Mismatches and Convention Violations

- **Async Runtime Implementation (runtime/async_rt.rs)**
  - Code: Uses a simple synchronous loop in `mb_await` and `mb_coroutine_step`.
  - Knowledge: `orbit/bridge-internals.md` specifies a Tokio-backed runtime with `PythonWaker` and GIL-safe polling.
  - Severity: **High** (Feature #313 requires a scalable async execution model).

- **GIL Management (runtime/async_rt.rs, runtime/class.rs)**
  - Code: NO visible usage of `py.allow_threads` or GIL release strategies during potentially blocking or runtime operations.
  - Knowledge: `orbit/bridge-internals.md` mandates GIL release before waiting on cross-thread synchronization to avoid deadlocks.
  - Severity: **High**.

- **Error Propagation (runtime/async_rt.rs)**
  - Code: Many functions return `MbValue::none()` on failure or simply ignore error conditions.
  - Knowledge: `orbit/bridge-internals.md` defines a structured Error Flow with translation to Python exceptions.
  - Severity: **Medium**.

- **Memory Management (runtime/rc.rs)**
  - Code: Simple reference counting with `mb_retain`/`mb_release`.
  - Knowledge: `orbit/performance-tuning.md` mentions circular reference detection and `gc.disable()` patterns for latency-critical paths. The current code lacks explicit cycle detection infrastructure.
  - Severity: **High** (Feature #315 requires cycle-detecting GC).
