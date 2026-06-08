---
change: mamba-conformance-p0
group: mamba-p0-fixes
date: 2026-03-24
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| fix-conformance-xfails-spec | testing | high | R1: Codegen IR calling-convention fixes (classmethod, descriptor __get__, getattr/setattr/delattr, super(), stacked decorators) in codegen/cranelift/mod.rs — SIGBUS root cause, R2: CallExtern return propagation fix in lower/hir_to_mir.rs — stdlib functions return None because return slot is discarded, R7: Walrus operator := scope fix in resolve/pass.rs — bind to enclosing function scope not comprehension scope, Full Changes section listing all modified files for P0 fixes: cranelift/mod.rs, hir_to_mir.rs, resolve/pass.rs, runtime/class.rs, Test plan: cargo test -p mamba --test conformance_tests with xfail elimination targets |
| cranelift | codegen | high | R5: Function signature generation — parameter count and calling convention for classmethod/descriptor/__get__/getattr/setattr/delattr (root cause of SIGBUS crashes), R2: MIR instruction translation — invalid Cranelift IR emission for class-related operations causing JIT verifier failures, R4: Value marshaling — argument passing conventions that produce calling-convention mismatches for lambda/with/decorator, Source file: codegen/cranelift/mod.rs (796 LOC) is the direct target for SIGBUS fixes |
| hir-to-mir | lower | high | R2: CallExtern lowering for module-level function calls — return value must be stored to register and propagated instead of discarded (stdlib None returns fix), R3: Pattern matching lowering — integer literal patterns emit wrong constant value in match/case dispatch, R1: Comprehension lowering — iterator scope setup in MIR that interacts with comprehension scope isolation, Source file: lower/hir_to_mir.rs (1,728 LOC) targeted for CallExtern and match fixes |
| mamba-global-nonlocal-spec | resolve | high | Variable classification state machine — how walrus := targets are classified (Local vs enclosing scope), Scope chain lookup with class-scope skipping — needed for walrus fix (skip comprehension and class scopes when binding walrus target), R2: Nonlocal statement processing — walk enclosing function scopes, skip class scopes (same logic needed for walrus), Source file: resolve/pass.rs targeted for both comprehension scope isolation and walrus := binding |
| class | runtime | high | R1: Type-tagged dispatch in mb_call_method — module Dict callable dispatch path that causes stdlib functions to return None, R6: Attribute access model — __getattr__/__setattr__/__delattr__ dispatch (related to R1.3 SIGBUS fix for getattr/setattr/delattr), R4: super() support via MRO — super() deduplication fix (R1.4) involves super() dispatch correctness, Source file: runtime/class.rs (1,238 LOC) targeted for module Dict dispatch fix |
| cranelift-jit | codegen | medium | R2: Runtime symbol table population — mb_* function addresses wired before compilation (context for SIGBUS analysis), R3: Function finalization — JIT code execution flow for understanding SIGBUS at call sites, R5: REPL incremental compilation — symbol linking relevant to super() duplicate-symbol fix (R1.4) |
| bytes-ops | runtime | medium | R4: Common sequence methods — replace, strip/lstrip/rstrip, startswith/endswith for bytes and bytearray (incomplete implementations causing xfails), Source file: runtime/bytes_ops.rs targeted for R3 conformance fixes |
| exception | runtime | medium | R2/R4: Exception instantiation with __cause__ and __context__ fields — exception chaining fix, R4: Thread-local exception state — mb_raise, mb_get_exception used in raise-from lowering, ExceptionGroup/except* (#755) intentionally xfailed — must not be changed |
| cclab-mamba-fix-xfail-spec | testing | medium | Historical context: previous xfail fix round (class system, generators, ExceptionGroup), Fixture structure under tests/fixtures/conformance/ and xfail marker conventions, Test run command: cargo test -p mamba --test conformance_tests |
| test-harness | testing | low | R1: Directive dispatch logic — RUN: jit directive runs Cranelift JIT backend, R3: Recursive fixture discovery under tests/fixtures/conformance/ tree, Fixture file path convention and error reporting format |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| mamba-p0-fixes-main | modify | crates/mamba/testing/mamba-py312-conformance.md | overview, changes |

