---
change: mamba-noarg-constructor
group: noarg-constructor-codegen
date: 2026-03-28
written_by: artifact_cli
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| hir-to-mir | lower | high | BUILTIN_EXTERNS table maps 'list' → mb_list_from_iterable, 'tuple' → mb_tuple_from_iterable, 'set' → mb_set_from_iterable, 'dict' → mb_dict_from_pairs — all take 1 argument (the iterable), Zero-arg arity guard at hir_to_mir.rs:3447-3468: when boxed_args.is_empty() and extern_name matches an iterable variant, redirect to _new variant (mb_list_new, mb_tuple_new, mb_set_new, mb_dict_new) with args=[], Fix committed in bc5921e9: CallExtern with args=[] is emitted for _new variants; the Cranelift signature has 0 AbiParams, matching the call site — verifier error eliminated, R1 comprehension lowering already calls mb_list_new()/mb_set_new()/mb_dict_new() directly — the zero-param construction pattern exists and works, Tests added in no_arg_constructor_tests.rs: S1 list()→empty, S2 tuple()→empty, S3 set()→empty, S7 dict()→empty, plus one-arg paths (S4/S5/S6) still route to _from_iterable correctly |
| cranelift | codegen | high | R5: Function Signature Generation — Signature built from MirExtern param_count; zero-param externs produce a Signature with no AbiParams (no i64 arguments), R2: MIR Instruction Translation — emit_extern_call builds arg_vals from the args slice; when args=[] the call instruction has zero operands; calling a 1-param extern with 0 args makes IR malformed and the Cranelift verifier rejects it, R3: Runtime Symbol Wiring — mb_list_new/mb_tuple_new/mb_set_new/mb_dict_new are registered with [], I64 signature; calling them with zero args is valid once routing is fixed in lowering, declare_extern iterates ext.params to push AbiParam — if wrong extern (mb_list_from_iterable with 1 param) is called with 0 args the generated Cranelift IR is malformed |
| cranelift-jit | codegen | medium | R2: Runtime Symbol Table Population — mb_list_new, mb_tuple_new, mb_set_new, mb_dict_new are all registered with their physical addresses before any user code is compiled, R3: Function Finalization — Cranelift verifier runs during finalization; a call with wrong arity triggers VerifierError, aborting JIT compilation, JIT_LOCK mutex serializes JIT test execution — no_arg_constructor_tests.rs acquires JIT_LOCK at start of each test to prevent concurrent JIT compilation races |
| list-ops | runtime | medium | R5: Construction — mb_list_new() creates empty list (0 params, I64 return); registered as rt_sym!("mb_list_new", ..., [], I64) — zero-param signature is correct, mb_list_from_iterable(val: MbValue) takes 1 MbValue and iterates it to build a list — this 1-arg function cannot be called with 0 args without triggering a Cranelift verifier error |
| set-ops | runtime | medium | R1: Set Creation — mb_set_new() creates empty set (0 params, I64 return); registered as rt_sym!("mb_set_new", ..., [], I64), mb_set_from_iterable(args: MbValue) is in builtins.rs (not set_ops.rs) and takes 1 MbValue argument — must not be called with 0 args |
| symbols | runtime | medium | R2: MirExtern Declarations — mb_list_new: params=[], return=I64; mb_tuple_new: params=[], return=I64; mb_set_new: params=[], return=I64; mb_dict_new: params=[], return=I64 — all zero-param constructors correctly registered, R1: Symbol Name-to-Address Mapping — mb_list_from_iterable registered with [I64], I64; mb_tuple_from_iterable with [I64], I64; mb_dict_from_pairs with [I64], I64 — the bug was in routing (lowering), not in symbol registration, R4: Naming convention — mb_<category>_new for zero-arg constructor, mb_<category>_from_iterable for one-arg conversion constructor |
| mir | mir | medium | R1: CallExtern instruction — dest, name (extern symbol), args (register list), ty; args must match the extern's declared parameter count or Cranelift verifier will reject the IR, R5: Extern Declarations — each MirExtern specifies symbol name; param count must match the args slice length at the CallExtern call site |
| builtins | runtime | low | mb_set_from_iterable(args: MbValue) defined in builtins.rs (not set_ops.rs); takes exactly 1 argument — calling with 0 args causes arity mismatch, R6: Symbol Registration — all mb_builtin_* and constructor variants registered in symbols.rs with correct param counts |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| no-arg-constructor-codegen-fix | modify | crates/mamba/lower/hir-to-mir.md | overview, requirements, scenarios, changes |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: mamba-noarg-constructor

**Verdict**: needs_revision

### Summary

Reference context covers the affected areas and relevance scores are reasonable. Two significant issues block passing: (1) the mir spec key requirements incorrectly claim CallExtern is in mir.md R1 — it is not; CallExtern exists in source (mir/mod.rs:49) but is completely absent from the spec, making this a pre-existing spec gap presented as documented coverage; (2) the spec plan is incomplete — mir.md must also be modified to add CallExtern to R1, since CallExtern is the exact instruction that triggers the verifier error and the spec should not remain silent on it. Additionally, hir-to-mir key requirements reference source code line numbers, commit hashes, and test scenario labels rather than actual spec R-IDs, and the tuple-ops spec entry is missing despite tuple() being one of the four constructors being fixed.

### Issues

No issues found.
