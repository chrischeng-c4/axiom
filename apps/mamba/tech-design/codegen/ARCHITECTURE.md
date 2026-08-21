# codegen — architecture (as-is, 2026-07-15)

Scope: the compile pipeline AST→HIR→MIR→native (`src/lower/`, `src/codegen/cranelift/`) plus the `--emit` dump tooling.
Runtime semantics (class registry, exceptions, GC) belong to their own domains; this domain owns *emission and sequencing*.

## Responsibilities

- AST→HIR lowering (`src/lower/ast_to_hir.rs:AstLowerer`): desugar (incl. `pep695.rs`), hoist `def`/`class` into `HirModule.functions/classes`, synthesize locals, emit textual `ClassDefPlaceholder`/`FuncDefPlaceholder` markers.
- HIR→MIR lowering (`src/lower/hir_to_mir.rs:HirToMir`): block/VReg SSA-ish emission, class-registration sequencing via `pending_class_*` queues, exception/`with`/`finally` plumbing, `match` pattern tests, trace-event seams.
- Cranelift JIT backend (`src/codegen/cranelift/jit.rs:CraneliftJitBackend`): MIR→executable memory, extern thunks, NaN-boxed I64 ABI, refcount call emission, inline fast paths (recursion guard, unboxing).
- AOT/alt backends: `cranelift/mod.rs:CraneliftBackend` (+`aot.rs:emit_main` object-file `main`), `codegen/llvm.rs`, wasm — all behind `codegen/mod.rs:CodegenBackend`.
- Debug tooling: `mamba build --emit ast|hir|mir` (`src/main.rs:61`) — dumps `{:#?}` and returns early at pipeline checkpoints (`driver/mod.rs:177,198,211`; `check` only honors `Ast`).

## Key structures & invariants

| Structure | Rule that must hold |
|---|---|
| `ast_to_hir.rs:3533 next_local_sym` (starts `1_000_000`) | SymbolId ≥ 1_000_000 = lowerer-synthetic local, NEVER in `SymbolTable` — every `symbols.get_symbol` path must guard on the threshold (`hir_to_mir.rs:925`) or it panics. REPL advances past pre-seeded ids (`ast_to_hir.rs:3493`). |
| `hir_to_mir.rs:1598-1653 pending_class_*` family (`pending_classes`, `_values`, `_cell_syms`, `_runtime_key_values`, `pending_runtime_class_bases`/`_base_lists`, `_slots` (#1492), `_attrs` (#1686), `_body_stmts`, `_finalizers`, `pending_abstract_methods`, `_decorators` (#1690), `pending_dataclass_fields`, `_docs`) | Drained ONLY at the class's textual `ClassDefPlaceholder` (or the fallback loop). Fixed drain order: register → runtime bases → slots → body stmts → attrs → finalizers → dataclass fields → decorators → bind (`hir_to_mir.rs:5670-5777`). Slots strictly after `mb_class_update_bases` — see `../object-model/class-construction.md` §Registration pipeline. |
| Class runtime key `__mamba_user_class__:{file}:{sym.0}:{name}` (`hir_to_mir.rs:944`) | Made execution-unique via `mb_class_runtime_key`; cached VReg fetched through `class_runtime_key_value(sym, fallback)` (`hir_to_mir.rs:6295`), never re-derived ad hoc. |
| `hir_to_mir.rs:1615 classes_needing_textual_registration` (#82) | `emit_pending_class_registrations(None)` (`:5782`) must skip these and kwargs-classes; `Some(sym)` selects one class. |
| `jit.rs:108 internal_funcs: HashMap<u32, FuncId>` | Keyed by `SymbolId.0`; declared name `_mb_<sym>`; `__main__` sentinel = `SymbolId(u32::MAX)` (`hir_to_mir.rs:3562`); entry = LAST body in `MirModule` (`jit.rs:3043`). |
| `jit.rs:131 internal_param_counts` / `:102 extern_param_counts` | Arity guards: call args reshaped (truncate / NaN-boxed-None pad) to the DECLARED signature before Cranelift sees them (#1696, #2098) — verifier rejects become conservative runtime behavior. |
| `jit.rs:107 extern_addrs` → per-extern thunks | All extern calls go through same-module `call_indirect` thunks (arm64 `Arm64Call` ±128MB BL range bug, `declare_extern` doc `:306`). |
| `jit.rs:33 JIT_LOCK` | Global mutex; compile+execute must be serialized across threads (concurrent finalize → mprotect race → SIGBUS on aarch64). Acquired by `tests/harness/cpython/pipeline.rs:75`. `new()` does NOT need it. |
| `jit.rs:41 CACHED_ISA` / `:61 CACHED_RT_SYMBOLS` / `:75 warm_jit_caches` | Process-global, immutable, forced in the `test-batch` zygote parent pre-fork (`main.rs:800-812`) for COW inheritance. |
| ABI: every VReg is I64 (`jit.rs:298 mamba_to_cl_type`) | NaN-boxed MbValue; floats bitcast I64↔F64; raw int/bool tracked out-of-band in `VarAlloc.raw_ints`/`native_bools` (`cranelift/mod.rs:603`). |
| `cranelift/mod.rs:14 EMIT_REFCOUNT_CALLS = true` | Release/retain emission; entry body (`__main__`) emits NO releases (#1663 UAF, `jit.rs:488`); loop-carried VRegs pre-seeded with None so release-before-overwrite fires per iteration (#1013/#2111, `jit.rs:502`). |

## Control flow

1. `main.rs` `build|run|check` → `driver/mod.rs:CompilerSession::{build,run,run_source}`.
2. `parser::parse` → `lower::pep695::desugar_module` → [`--emit ast` dump, return].
3. `TypeChecker::check_module` (+ `check_dependencies` for imports).
4. `lower::lower_module` (`ast_to_hir.rs:3440`): per-statement walk; `collect_class_stmt` (`:5087`) allocates synthetic `class_sym`, lowers body, queues facts, emits placeholder to top_level or to the caller's stream (nested-in-control-flow) → [`--emit hir`].
5. `lower_hir_to_mir_with_symbols_src` (`hir_to_mir.rs:1420`): pre-pass registers all class names/runtime keys (`:906-954`) so forward refs resolve; primes func introspection maps; lowers method/function bodies; `lower_top_level` (`:3114`) emits `mb_register_builtins`, `__file__`, FUNC_NAMES/DOCS/argcount priming, then the stmt loop; `__main__` body is appended LAST → [`--emit mir`].
6. Class emission inside the stmt loop: `HirStmt::ClassDefPlaceholder` arm (`:5670`) drains the queue family in the fixed order above; fallback loop for hand-built HIR without placeholders (`:3485-3499`).
7. `match` lowering: per-case `emit_pattern_test` (`:7459`, dispatched at `:8170`) — recursive; each sub-test branches to `fail_block`; `Capture` unboxes prims when subject is boxed (#827); `Or` merges bindings via pre-allocated merge VRegs.
8. Backend: `CraneliftJitBackend::codegen` (`jit.rs:2969`) — Phase 1 declare externs, Phase 2 forward-declare internals, Phase 3 `compile_function` per body, `finalize_definitions`, register variadic/kwargs/boxed-return ptrs + perf map (`MAMBA_PERF_MAP=1`, #2094), return entry ptr.
9. `execute_jit_entry` (`driver/mod.rs:244`): transmute → call → drain pending exception/uncaught traceback → `cleanup_all_runtime_state`.

## Known hazards

- **JIT without JIT_LOCK**: any new threaded harness that compiles+runs without the lock — intermittent SIGBUS on aarch64, looks like a bisect-resistant flake.
- **Backend Drop leaks by design** (`jit.rs:161`): calling `free_memory()` or freeing `compile_time_objects` — global registries hold pointers into old JIT pages/literals; freeing = delayed UAF in later same-process tests.
- **Arity-guard masking** (`jit.rs:2436`, `:102`): truncate/pad silently "fixes" wrong-arity calls — a lowering bug downgrades from verifier abort to wrong runtime values; don't trust a clean compile as proof of correct call shape.
- **Synthetic-sym lookups**: new code that calls `symbols.get_symbol(sym)` without the `>= 1_000_000` guard — index-out-of-bounds panic only on pattern/synthetic-local paths.
- **Eager class-side-effect emission**: emitting attrs/decorators/dataclass fields before the stmt loop resolves imports against an empty module (#1686/#1690 defect class) — anything class-scoped must ride the placeholder.
- **Slots/bases ordering**: `mb_register_slots` before `mb_class_update_bases` drops inherited slots — see `../object-model/class-construction.md` §Registration pipeline.
- **Entry-body releases** (`jit.rs:488`): removing the `!is_entry_body` guard resurfaces `stdlib/re` UAF (#1663); the perf motivation is already covered elsewhere.
- **Recursion fast path** (`jit.rs:2688`): inline increment commits ONLY on the fast path; slow path defers wholly to `mb_recursion_enter` (no double count, byte-identical raise). Changing either side desyncs depth or the error message.
- **`--emit` on `check`**: only `Ast` is honored (`driver/mod.rs:139`); use `build --emit hir|mir` for later stages.
- **Entry = last body** (`jit.rs:3043`): reordering `MirModule.bodies` silently changes the program entry.
- **Last-expression capture** (`hir_to_mir.rs:3465`): only a trailing *Call* expr becomes `__main__`'s return; typed-prim results route through `mb_unbox_*_if_boxed` (`:3529`) — new terminal shapes must follow it.
- **settrace exception under-emission**: event fires once at the raising frame, not per unwound frame — the open fix is `tracing-and-frames.md` §Exception events fire in every unwinding frame (this dir); its seams are `emit_try_exception_guard` (`hir_to_mir.rs:12425`) and the epilogue unwind checks.
- **`sym_to_vreg` stale-raw-for-Any read** (`hir_to_mir.rs:1573`, tracked: #1794): the VReg cache holds no raw/boxed tag, so an Any-typed read of a module-scope symbol last cached raw (e.g. an accumulator widened after its first assignment) can hand a raw bit pattern to a callsite expecting boxed — `box_operand` treats Any as already-boxed and skips it, so NaN-box decoding misreads the raw int as a float. Full mechanism and the `global_synced_syms` fix: `value-representation.md`.
- **Boxed-return double-boxing at dynamic-dispatch boundaries** (`module.rs:1225` `register_boxed_return_func`, tracked: #1794): a function whose return value is already a boxed `MbValue` must be registered, or an indirect/dynamic caller re-boxes it as if raw (e.g. `dispatch_thread_jit_frame`, `asyncio_mod.rs:803`) — same misread family as the hazard above. Full registry chain: `value-representation.md`.

## Extension points

| Adding | Where it plugs in |
|---|---|
| New runtime helper callable from generated code | `src/runtime/symbols.rs` (`runtime_symbols` + `runtime_externs`), then `MirInst::CallExtern { name }` from lowering — no backend change needed. |
| Inline fast path for a hot extern | Name-matched intercept at the top of `emit_extern_call` (`jit.rs:2530+`); pattern: `mb_recursion_enter/leave` (`:2688/:2785`), `mb_is_stop_iter` (`:2817`). Keep the FFI fallback path intact. |
| New class-body construct needing runtime emission | New `pending_class_*` queue on `HirToMir` + drain calls in BOTH the placeholder arm (`:5670`) and the fallback loop (`:3485`) — one call site is a latent ordering bug. |
| New `match` pattern kind | `HirPattern` variant + arm in `emit_pattern_test` (`:7459`) + binding collection in `collect_pattern_bindings` (`:13388`). |
| New codegen backend | Implement `codegen/mod.rs:20 CodegenBackend`; register in `driver/config.rs:15 Backend` + selection in `CompilerSession::build` (`driver/mod.rs:217`). |
| New IR dump stage | `driver/config.rs:23 EmitMode` + a checkpoint in `CompilerSession::{check,build}`. |
| Trace/settrace events | `emit_try_exception_guard`/`emit_exception_propagate*` (`hir_to_mir.rs:12425/:12455`) and `pending_trace_return_arg` (`:1666`) are the sanctioned seams (used by #1535). |

## EC surface

Per `apps/mamba/external-contracts/README.md`: codegen is the substrate — **every run-dimension fixture** (`behavior/errors/real_world/surface/_regression/security/concurrency`) exercises it; a compile reject there is a defect by definition. Proof-bearing subsets:

- `behavior/core/sys_settrace/` + `std-libs/{bdb,trace}` — trace-event emission (gate of `1535-…md`, this dir).
- `_regression/core/{class_system,mro_super,language}`, `behavior/core/descr` — class registration/drain ordering (shared with object-model).
- `bench/` + C2 perf pins (`tests/harness/cpython/config/perf/pins/*.toml`) — recursion-guard fast paths, rc-elision, loop-carried preseed cost.
- `stability/`, `gc/` soaks + corpus-wide absence of hang/SIGTRAP/SIGBUS — leak boundedness and JIT_LOCK discipline.
- Full gate: `cargo test -p mamba --release --test conformance` (~3 min, oracle = python3.12 byte-diff).
