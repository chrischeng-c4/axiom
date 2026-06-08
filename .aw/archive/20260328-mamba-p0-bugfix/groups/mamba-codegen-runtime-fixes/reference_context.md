---
change: mamba-p0-bugfix
group: mamba-codegen-runtime-fixes
date: 2026-03-25
written_by: artifact_cli
review_verdict: pass
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| hir-to-mir | lower | high | BUILTIN_MAP routes list->mb_list_from_iterable, tuple->mb_tuple_from_iterable, set->mb_set_from_iterable — zero-arg calls with 0 args must short-circuit to mb_list_new/mb_tuple_new/mb_set_new before dispatching to the iterable variant, R1: Comprehension lowering uses mb_list_new()/mb_set_new()/mb_dict_new() directly — the zero-param construction pattern exists and works, fix location: hir_to_mir.rs Call lowering for builtin names with empty args — add arity check before BUILTIN_MAP lookup |
| cranelift | codegen | high | R5: Function Signature Generation — Signature built from MirExtern.params; zero-param externs produce a Signature with no AbiParams, R2: MIR Instruction Translation — emit_extern_call builds arg_vals from args slice; when args=[] the call instruction emits zero operands; mismatch with wrong sig causes verifier error, declare_extern iterates ext.params to push AbiParam — if wrong extern (mb_list_from_iterable with 1 param) is called with 0 args the IR is malformed, R3: Runtime Symbol Wiring — mb_list_new/mb_tuple_new/mb_set_new are declared with [], I64 and wired; calling them with zero args is valid once routing is fixed in lowering |
| string-ops | runtime | high | mb_str_slice_full(s, start, stop, step): negative step branch calls clamp_rev_str(stop.as_int().unwrap_or(-1), len), Bug root: clamp_rev_str(-1, 6) = -1+6 = 5 (normalises -1 as last char index), making e_idx=5 and s_idx=clamp_rev_str(5,6)=5, loop condition 5>5 is immediately false — empty result, Correct behaviour: absent stop with negative step means iterate down to index 0 inclusive — sentinel must bypass clamp_rev_str or use -1 as literal lower bound in loop condition, R1: slice methods must handle full Python slice semantics including step=-1 reversal (s[::-1] == reversed string) |
| mamba-thread-safe-spec | runtime | high | R5: thread_local! state in closure.rs (3 statics), generator.rs (5 statics), exception.rs (1), class.rs (3 statics), iter.rs (1) — when cargo test runs multiple conformance tests in parallel these thread-locals are shared/aliased across OS threads causing SIGBUS, R3: No-GIL — all allocations/ref-adjustments must not depend on thread-local GIL; conformance tests run in separate Rust threads via cargo test -j N, R7: Atomic Global Counters — ID allocation in generator.rs uses thread_local increments; must migrate to AtomicU64, gc.rs uses GC_TEST_LOCK: LazyLock<Mutex<()>> to serialise GC tests — similar serialisation pattern may be needed for conformance tests as short-term fix |
| cranelift-jit | codegen | high | R1: JIT Module Initialization — CraneliftJitBackend holds Option<JITModule>; JITModule contains an ISA object with internal mutable state; concurrent instantiation from multiple cargo test threads races, R4: Executable Memory Management — JIT finalise_module() marks pages executable via mprotect; concurrent calls from separate threads can alias the same memory region, R2: Symbol Table Population — register_symbols iterates RUNTIME_SYMBOLS static; if ISA builder has internal thread-local state concurrent init causes SIGBUS on aarch64 (bus error on misaligned executable page), fix: either wrap JITModule init in a global Mutex, or ensure each test thread creates an isolated JITModule with no shared state |
| builtins | runtime | medium | R6: Symbol Registration — mb_builtin_* registered in symbols.rs with correct MirExtern (param count + return type), Constructor dispatch: list()/tuple()/set() with args route to mb_list_from_iterable etc; zero-arg case needs separate _new variant, R5: P1 builtins — all use extern C calling convention with explicit param counts |
| list-ops | runtime | medium | R5: Construction — mb_list_new() creates empty list (0 params, I64 return); mb_list_from_values(values, count) creates from C array, mb_list_new registered as rt_sym!("mb_list_new", ..., [], I64) — zero-param signature is correct; calling it via CallExtern with args=[] is valid |
| symbols | runtime | medium | R2: MirExtern Declarations — mb_list_new: params=[], return=I64; mb_tuple_new: params=[], return=I64; mb_set_new: params=[], return=I64 — all zero-param constructors already registered correctly, R1: Symbol Name-to-Address Mapping — all mb_* registered; the constructors exist; the bug is in routing not in symbol registration, R4: Naming convention — mb_<category>_new pattern for zero-arg constructors |
| gc | runtime | medium | GC state already uses LazyLock<Mutex<GcState>> and GC_TEST_LOCK for test serialisation — established pattern for safe concurrent access, thread_local! at gc.rs:70 for per-thread GC registration — this pattern is what needs to be replicated safely for runtime state, Safepoint-based STW design in spec provides model for coordinating concurrent thread pause |
| cclab-mamba-fix-xfail-spec | testing | medium | Conformance test runner: cargo test -p mamba --test conformance_tests (multi-threaded by default), Single-threaded workaround -- --test-threads=1 passes for #1114; fix must make multi-threaded safe, xfail fixtures for list()/tuple()/set() and string[::-1] will be unblocked once #1109 and #1111 are fixed |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| no-arg-constructor-codegen-fix | modify | crates/mamba/lower/hir-to-mir.md | overview, requirements, scenarios, changes |
| string-reverse-slice-fix | modify | crates/mamba/runtime/string-ops.md | overview, requirements, scenarios, changes |
| sigbus-runtime-thread-safety-fix | modify | crates/mamba/runtime/thread-safe-runtime.md | overview, requirements, state-machine, scenarios, changes |
| sigbus-jit-concurrency-fix | modify | crates/mamba/codegen/cranelift-jit.md | overview, requirements, scenarios, changes |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: mamba-p0-bugfix

**Verdict**: pass

### Summary

Two critical spec content mismatches and one significant requirement ID mismapping. Spec plan paths and structure are correct, but key_requirements for string-ops and cclab-mamba-fix-xfail-spec cite content that does not exist in the actual specs.

### Issues

No issues found.
