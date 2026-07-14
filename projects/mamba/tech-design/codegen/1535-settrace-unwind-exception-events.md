# #1535 — sys.settrace 'exception' event must fire in every unwinding frame

Status: OPEN (p1). Design for implementation. First TD of the codegen/ context.

## Mechanism

mamba emits the settrace `'exception'` event once, at the raising frame
(`mb_traceback_capture_raise`, runtime/stdlib/traceback_mod.rs:~1153). CPython
fires it in EVERY frame the exception unwinds through en route to a handler.
3-frame discriminator (caller→passthrough(no try)→boom): CPython emits
exception×3, mamba ×1. Dominant cluster of 212/275 settrace xfails.

## Invariant

Every function-exit path that can return with a pending exception emits the
`'exception'` trace event for ITS frame (when tracing is active), with
CPython's arg tuple (exc_type, exc_value, traceback) and ordering. Not only
try-guarded call sites (`emit_try_exception_guard`, lower/hir_to_mir.rs:~12084).

## Fix direction

Codegen-wide: at function-exit-with-pending-exception (the epilogue/unwind
check every call site already performs for propagation), add a
tracing-gated event emission. Keep the fast path free: gate on a cheap
"tracing active" check before any argument materialization (the existing
sys.settrace machinery from #891's slices has the activation flag). Beware
per-call-site code-size blowup — prefer one shared unwind-event helper extern
over inline expansion.

## Out of scope (same 212-cluster, different features)

jump-in-trace (`frame.f_lineno` assignment), opcode-level tracing. Re-tally
the cluster after landing; file the remainder separately.

## Verification contract

3-frame probe byte-identical vs python3.12; `behavior/core/sys_settrace/`
full-dir re-scan (expect a large xfail→green flip — report exact count);
bdb (65 PASS) + std-libs/trace (15 PASS) baselines no regression; #891's AC1
probes stay green. Full gate before/after reading.
