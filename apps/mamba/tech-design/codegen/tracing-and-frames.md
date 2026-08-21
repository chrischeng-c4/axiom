# Tracing and frames — sys.settrace event emission

Codegen's contract for debugger/introspection support: emitting trace events
at the right frames without slowing the untraced fast path.

## Exception events fire in every unwinding frame

CPython fires the settrace `'exception'` event in EVERY frame an exception
unwinds through en route to a handler; mamba currently emits it once, at the
raising frame (`mb_traceback_capture_raise`). Invariant: every
function-exit-with-pending-exception path (the epilogue/unwind check each call
site already performs for propagation) emits the event for ITS frame when
tracing is active, with CPython's `(exc_type, exc_value, traceback)` tuple and
ordering — not only try-guarded call sites (`emit_try_exception_guard`).

Fast-path rule: gate on a cheap "tracing active" flag (the sys.settrace
activation state) BEFORE materializing any event arguments; the untraced path
must stay free. Prefer one shared unwind-event helper extern over inline
expansion at every call site (code-size). Tracked: #1535 (dominant cluster of
212/275 settrace xfails; jump-in-trace `f_lineno` and opcode tracing are
separate features, re-tally after landing).

## EC surface

`behavior/core/sys_settrace` (full-dir scan reports the xfail→green flip),
bdb + `std-libs/trace` baselines as regression guards.
