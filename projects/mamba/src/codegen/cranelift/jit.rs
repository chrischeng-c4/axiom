/// Cranelift JIT backend for Mamba (#296).
///
/// Uses cranelift-jit's JITModule to compile MIR directly into executable
/// memory. Runtime mb_* functions are wired as symbols so JIT-compiled
/// code can call them.
use super::marshal;
use super::perf_map;
use super::{emit_binop, emit_terminator, VarAlloc, EMIT_REFCOUNT_CALLS};
use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::mir::{
    analyze_literal_escapes, LiteralEscapeAnalysis, LiteralEscapeClassification, LiteralEscapeKind,
    MirBinOp, MirBody, MirConst, MirExtern, MirInst, MirModule, MirType, VReg,
};
use crate::runtime::rc::MbObject;
use crate::runtime::symbols::{runtime_externs, runtime_symbols};
use crate::runtime::value::MbValue;
use crate::types::{Ty, TypeContext, TypeId};

use cranelift_codegen::ir::{
    types as cl_types, AbiParam, Function, InstBuilder, MemFlags, Signature,
};
use cranelift_codegen::isa::{CallConv, OwnedTargetIsa};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};

use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

/// Global lock to serialize JIT compilation + execution across test threads.
/// Concurrent JITModule finalization causes SIGBUS on aarch64 due to mprotect
/// races. Callers (e.g. conformance runner) acquire this before JIT pipeline.
pub static JIT_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// Process-global native ISA with mamba's JIT flags, built ONCE. `OwnedTargetIsa`
/// is `Arc<dyn TargetIsa>` (immutable after construction), so cloning is a cheap
/// refcount bump — `CraneliftJitBackend::new_with_externals` rebuilt the ISA
/// (CPU feature detection + ISA construction) on EVERY call, which dominates a
/// process-per-fixture conformance sweep. Sharing one read-only ISA also makes
/// it safe across the `mamba test-batch` zygote fork boundary (inherited COW).
static CACHED_ISA: LazyLock<OwnedTargetIsa> = LazyLock::new(|| {
    let mut flags_builder = settings::builder();
    // JIT needs PIC disabled for direct calls; speed opt level matches the
    // previous per-backend configuration exactly.
    flags_builder
        .set("use_colocated_libcalls", "false")
        .unwrap();
    flags_builder.set("is_pic", "false").unwrap();
    flags_builder.set("opt_level", "speed").unwrap();
    cranelift_native::builder()
        .expect("no native ISA")
        .finish(settings::Flags::new(flags_builder))
        .expect("native ISA finish")
});

/// Process-global runtime symbol table (name → addr-as-usize), built ONCE from
/// `runtime_symbols()`. The original code rebuilt the several-hundred-entry
/// `Vec<RuntimeSymbol>` on every backend construction; memoizing the (name,
/// addr) pairs avoids that per-fixture. `usize` (not `*const u8`) keeps the
/// static `Send + Sync`; call sites cast back to `*const u8`.
static CACHED_RT_SYMBOLS: LazyLock<Vec<(&'static str, usize)>> = LazyLock::new(|| {
    runtime_symbols()
        .into_iter()
        .map(|s| (s.name, s.addr as usize))
        .collect()
});

/// Force the process-global JIT caches (native ISA + runtime symbol table) to
/// build NOW. `mamba test-batch` calls this in the parent BEFORE forking, so
/// each worker COW-inherits the already-built, read-only caches instead of
/// rebuilding them — in fork-per-fixture each child compiles exactly one
/// fixture, so without this the LazyLocks would rebuild once per child and
/// amortize nothing. The ISA is `Arc<dyn TargetIsa>` and the symbol table is
/// immutable, so inheriting them across the fork boundary is sound.
pub fn warm_jit_caches() {
    LazyLock::force(&CACHED_ISA);
    LazyLock::force(&CACHED_RT_SYMBOLS);
}

/// Which native arithmetic op the raw-int overflow-check helper emits.
#[derive(Copy, Clone)]
enum RawIntOp {
    Add,
    Sub,
    Mul,
}

pub struct CraneliftJitBackend {
    module: Option<JITModule>,
    extern_funcs: HashMap<String, FuncId>,
    /// Declared parameter count per extern thunk. Captured at
    /// `declare_extern` time from `ext.params.len()`. The MIR-level
    /// `externs` slice passed to `emit_extern_call` is per-module-pass
    /// and may not contain entries for already-declared externs reused
    /// across passes; without this map the per-call arity guard at
    /// lines ~1650 silently skips reshape and emits a mismatched-arity
    /// `call fnN(...)` that the Cranelift verifier rejects (the
    /// #2098 fingerprint: `call fn22(v54, v55, v56, v52, v53)` against
    /// a declared `(i64) -> i64` for `assertRaises`-style forwarding).
    /// Keyed by extern name so the guard can recover the declared
    /// param count without relying on the current pass's externs slice.
    extern_param_counts: HashMap<String, usize>,
    /// Absolute addresses of extern symbols (`mb_*` runtime + binding-crate
    /// symbols). Used to emit per-extern thunk functions that call the symbol
    /// via `call_indirect` against an absolute i64 — sidesteps the cranelift
    /// arm64 `Reloc::Arm64Call` ±128MB BL immediate-range bug.
    extern_addrs: HashMap<String, *const u8>,
    internal_funcs: HashMap<u32, FuncId>,
    /// Declared return TypeId per internal function for NaN-boxing promotion
    internal_return_tys: HashMap<u32, TypeId>,
    /// Bodies whose non-None returns are native bool producers even when the
    /// MIR-level `return_ty` remains Int-shaped. This lets the JIT preserve
    /// bool semantics across internal calls without widening the ABI.
    internal_native_bool_returns: HashSet<u32>,
    /// Declared parameter count per internal function. Captured at
    /// `declare_internal` time from `body.params.len()` and used as a
    /// defensive arity guard in `emit_internal_call` so a call site whose
    /// `MirInst::Call { args }` length diverges from the registered
    /// signature is reshaped (truncated or zero-padded) before being
    /// handed to Cranelift. Prevents the verifier reject
    /// `mismatched argument count for v? = call fnN(...) got K, expected N`
    /// observed in #1696 — codegen used to blindly pass `args.len()`
    /// operands against a sig declared with `body.params.len()` ABI
    /// params. Zero-pad with NaN-boxed None (`0`) on under-arity; trunc
    /// to declared count on over-arity. The behaviour is conservative:
    /// the call may still be semantically wrong, but it no longer
    /// aborts the entire JIT module — the runtime path that surfaced
    /// the mismatch (cross-method state during cpython test_bool
    /// compilation) is allowed to proceed to a downstream failure or
    /// success rather than failing the whole compilation unit.
    internal_param_counts: HashMap<u32, usize>,
    /// Compile-time allocated objects (string/bytes literals embedded in code).
    /// Owned by the backend; freed on Drop (#1129 R5).
    compile_time_objects: Vec<*mut MbObject>,
    /// Compiled-code size (in bytes) per internal function, captured at
    /// `define_function` time. Used after `finalize_definitions` to emit
    /// `/tmp/perf-<pid>.map` records when `MAMBA_PERF_MAP=1` (#2094).
    /// Populated only when the env var is set on entry to `compile_function`,
    /// otherwise stays empty so non-profiling runs pay no cost.
    internal_code_sizes: HashMap<u32, u32>,
}

/// Drop handler for CraneliftJitBackend.
///
/// Cranelift's JITModule intentionally leaks mmapped code pages on Drop
/// (to keep function pointers valid). We previously tried calling
/// `free_memory()` to reclaim those pages but it crashes prior-test
/// state on aarch64/macOS — global runtime registries (mb_register_builtins,
/// module symbol table, GC roots) hold pointers into JIT code from earlier
/// runs in the same process; freeing those pages creates dangling references
/// that fault when later tests touch the global state. The leak is bounded:
/// each compilation creates ~4-16 KB of executable pages. With the
/// `selinux-fix` feature (enabled in projects/mamba/Cargo.toml) those pages
/// live in `memmap2`-managed anonymous mmap regions rather than libsystem's
/// heap, so the leak does not pressure malloc bookkeeping.
///
/// Compile-time objects (immortal strings/bytes) are also intentionally
/// leaked. GC-tracked containers and runtime state may still reference
/// them via borrowed MbValue copies. Freeing them creates dangling
/// pointers that cause use-after-free when GC sweeps those containers.
impl Drop for CraneliftJitBackend {
    fn drop(&mut self) {
        // JITModule is dropped without calling free_memory().
        // Cranelift leaks the mmapped code pages by design.
        drop(self.module.take());
        // Compile-time objects leak — clear the Vec without freeing.
        self.compile_time_objects.clear();
    }
}

impl CraneliftJitBackend {
    /// Create a JIT backend with only built-in runtime symbols (single-file mode).
    pub fn new() -> crate::error::Result<Self> {
        Self::new_with_externals(&[])
    }

    /// Create a JIT backend and also inject `external_syms` — symbols from
    /// native binding crates collected by [`register_external_modules`].
    ///
    /// Each entry is `(name, raw_fn_ptr)`.  The names must match the symbols
    /// that the compiled Mamba code calls via `MirExtern`.
    ///
    /// [`register_external_modules`]: crate::driver::register_external_modules
    pub fn new_with_externals(external_syms: &[(&str, *const u8)]) -> crate::error::Result<Self> {
        // Reuse the process-global ISA (Arc clone) instead of rebuilding it —
        // CPU feature detection + ISA construction is constant per machine and
        // was previously re-paid on every backend construction.
        let isa = CACHED_ISA.clone();

        let mut jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        // Collect absolute addresses for thunk emission (see `declare_extern`).
        let mut extern_addrs: HashMap<String, *const u8> =
            HashMap::with_capacity(CACHED_RT_SYMBOLS.len() + external_syms.len());

        // Wire all mb_* runtime symbols into the JIT module (from the memoized
        // table — no per-call `runtime_symbols()` Vec rebuild).
        for &(name, addr) in CACHED_RT_SYMBOLS.iter() {
            let ptr = addr as *const u8;
            extern_addrs.insert(name.to_string(), ptr);
            jit_builder.symbol(name, ptr);
        }

        // Wire external (binding-crate) symbols supplied by the caller.
        for &(name, ptr) in external_syms {
            extern_addrs.insert(name.to_string(), ptr);
            jit_builder.symbol(name, ptr);
        }

        let module = JITModule::new(jit_builder);
        Ok(Self {
            module: Some(module),
            extern_funcs: HashMap::new(),
            extern_param_counts: HashMap::new(),
            extern_addrs,
            internal_funcs: HashMap::new(),
            internal_return_tys: HashMap::new(),
            internal_native_bool_returns: HashSet::new(),
            internal_param_counts: HashMap::new(),
            compile_time_objects: Vec::new(),
            internal_code_sizes: HashMap::new(),
        })
    }

    fn module(&mut self) -> &mut JITModule {
        self.module.as_mut().expect("module already consumed")
    }

    fn body_returns_native_bool(body: &MirBody, tcx: &TypeContext) -> bool {
        let mut bool_vregs = HashSet::new();
        let mut none_vregs = HashSet::new();
        for block in &body.blocks {
            for inst in &block.stmts {
                match inst {
                    MirInst::LoadConst {
                        dest,
                        value: MirConst::Bool(_),
                        ..
                    } => {
                        bool_vregs.insert(*dest);
                    }
                    MirInst::LoadConst {
                        dest,
                        value: MirConst::None,
                        ..
                    } => {
                        none_vregs.insert(*dest);
                    }
                    MirInst::BinOp { dest, ty, .. }
                    | MirInst::UnaryOp { dest, ty, .. }
                    | MirInst::CallExtern {
                        dest: Some(dest),
                        ty,
                        ..
                    }
                    | MirInst::Call {
                        dest: Some(dest),
                        ty,
                        ..
                    } if matches!(tcx.get(*ty), Ty::Bool) => {
                        bool_vregs.insert(*dest);
                    }
                    MirInst::Copy { dest, source } if bool_vregs.contains(source) => {
                        bool_vregs.insert(*dest);
                    }
                    MirInst::Copy { dest, source } if none_vregs.contains(source) => {
                        none_vregs.insert(*dest);
                    }
                    _ => {}
                }
            }
        }

        let mut saw_bool_return = false;
        for block in &body.blocks {
            if let crate::mir::Terminator::Return(Some(vreg)) = &block.terminator {
                if bool_vregs.contains(vreg) {
                    saw_bool_return = true;
                } else if !none_vregs.contains(vreg) {
                    return false;
                }
            }
        }
        saw_bool_return
    }

    /// Get the finalized function pointer for an internal function by SymbolId (#1190).
    ///
    /// Returns the raw function pointer if the function was compiled and finalized,
    /// or `None` if the SymbolId was not found. The pointer is NaN-boxed with TAG_FUNC
    /// to produce an MbValue suitable for use as a module attribute.
    pub fn get_func_ptr(&self, sym_id: u32) -> Option<*const u8> {
        let func_id = self.internal_funcs.get(&sym_id)?;
        let jit_module = self.module.as_ref()?;
        Some(jit_module.get_finalized_function(*func_id))
    }

    fn mamba_to_cl_type(_ty: &crate::types::Ty) -> cranelift_codegen::ir::Type {
        // All VRegs use I64 (NaN-boxed MbValue). Float arithmetic uses
        // bitcast I64↔F64 wrappers around native fadd/fsub/fmul/fdiv.
        // This avoids type mismatches when MIR reuses a VReg across different
        // types (e.g., `total = 0.0; total = total + bar()` where bar returns MbValue).
        cl_types::I64
    }

    /// Declare an extern symbol as a local thunk that calls the symbol via
    /// `call_indirect` against the absolute address — sidesteps cranelift's
    /// arm64 `Reloc::Arm64Call` ±128MB BL immediate-range bug.
    ///
    /// Background: cranelift on arm64 emits direct `BL <symbol>` for
    /// `Linkage::Import` calls, whose 26-bit signed immediate caps the
    /// caller↔target distance at ±128MB. mmap'd JIT pages on macOS/aarch64
    /// frequently land further than that from the host process's runtime
    /// symbol catalog, tripping the assertion at
    /// `cranelift-jit/src/compiled_blob.rs:90`. `is_pic = true` does not help
    /// (cranelift-jit's `write_plt_entry_bytes` only supports x86_64).
    ///
    /// The thunk lives in the same `JITModule` code region as its callers, so
    /// the `BL <thunk>` from any JIT'd function always fits in 26 bits.
    /// Inside the thunk, `iconst.i64 <addr>` + `call_indirect` lowers to
    /// `MOVZ/MOVK/BLR` — register-form, no immediate range.
    fn declare_extern(&mut self, ext: &MirExtern) -> crate::error::Result<FuncId> {
        if let Some(&existing) = self.extern_funcs.get(&ext.name) {
            return Ok(existing);
        }

        let mut sig = Signature::new(CallConv::SystemV);
        for param_ty in &ext.params {
            sig.params
                .push(AbiParam::new(marshal::mir_type_to_cl(param_ty)));
        }
        if ext.return_type != MirType::Void {
            sig.returns
                .push(AbiParam::new(marshal::mir_type_to_cl(&ext.return_type)));
        }

        let addr = *self.extern_addrs.get(&ext.name).ok_or_else(|| {
            crate::error::MambaError::codegen(format!(
                "extern '{}' has no registered address (not in runtime_symbols nor external_syms)",
                ext.name
            ))
        })?;

        let thunk_name = format!("__thunk_{}", ext.name);
        let thunk_id = self
            .module()
            .declare_function(&thunk_name, Linkage::Local, &sig)
            .map_err(|e| {
                crate::error::MambaError::codegen(format!("declare thunk '{thunk_name}': {e}"))
            })?;

        let mut func = Function::with_name_signature(
            cranelift_codegen::ir::UserFuncName::user(0, thunk_id.as_u32()),
            sig.clone(),
        );
        let mut fb_ctx = cranelift_frontend::FunctionBuilderContext::new();
        let mut builder = cranelift_frontend::FunctionBuilder::new(&mut func, &mut fb_ctx);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        let args: Vec<_> = builder.block_params(entry).to_vec();
        let target = builder.ins().iconst(cl_types::I64, addr as i64);
        let sig_ref = builder.import_signature(sig);
        let call = builder.ins().call_indirect(sig_ref, target, &args);

        if ext.return_type != MirType::Void {
            let results = builder.inst_results(call).to_vec();
            builder.ins().return_(&results);
        } else {
            builder.ins().return_(&[]);
        }
        builder.finalize();

        let mut ctx = cranelift_codegen::Context::for_function(func);
        self.module()
            .define_function(thunk_id, &mut ctx)
            .map_err(|e| {
                crate::error::MambaError::codegen(format!("define thunk '{thunk_name}': {e}"))
            })?;

        self.extern_funcs.insert(ext.name.clone(), thunk_id);
        self.extern_param_counts
            .insert(ext.name.clone(), ext.params.len());
        Ok(thunk_id)
    }

    fn declare_internal(
        &mut self,
        body: &MirBody,
        tcx: &TypeContext,
    ) -> crate::error::Result<FuncId> {
        let mut sig = Signature::new(CallConv::SystemV);
        for (_, ty_id) in &body.params {
            sig.params
                .push(AbiParam::new(Self::mamba_to_cl_type(tcx.get(*ty_id))));
        }
        sig.returns.push(AbiParam::new(Self::mamba_to_cl_type(
            tcx.get(body.return_ty),
        )));
        let func_name = format!("_mb_{}", body.name.0);
        let func_id = self
            .module()
            .declare_function(&func_name, Linkage::Export, &sig)
            .map_err(|e| crate::error::MambaError::codegen(format!("declare: {e}")))?;
        self.internal_funcs.insert(body.name.0, func_id);
        self.internal_return_tys.insert(body.name.0, body.return_ty);
        if Self::body_returns_native_bool(body, tcx) {
            self.internal_native_bool_returns.insert(body.name.0);
        }
        self.internal_param_counts
            .insert(body.name.0, body.params.len());
        Ok(func_id)
    }

    fn compile_function(
        &mut self,
        body: &MirBody,
        tcx: &TypeContext,
        externs: &[MirExtern],
    ) -> crate::error::Result<()> {
        let func_id = self.internal_funcs[&body.name.0];
        let mut sig = Signature::new(CallConv::SystemV);
        for (_, ty_id) in &body.params {
            sig.params
                .push(AbiParam::new(Self::mamba_to_cl_type(tcx.get(*ty_id))));
        }
        let ret_ty = Self::mamba_to_cl_type(tcx.get(body.return_ty));
        sig.returns.push(AbiParam::new(ret_ty));

        let mut func = Function::with_name_signature(
            cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32()),
            sig,
        );
        let mut fb_ctx = cranelift_frontend::FunctionBuilderContext::new();
        let mut builder = cranelift_frontend::FunctionBuilder::new(&mut func, &mut fb_ctx);
        let mut vars = VarAlloc::new();
        let literal_escapes = analyze_literal_escapes(body);
        let is_entry_body = body.name.0 == u32::MAX;
        // Per-block "definitely assigned on all incoming paths" VReg sets,
        // consumed by `emit_terminator`'s Return epilogue via
        // `vars.live_filter` so large branch-local release sets do not force
        // Cranelift to synthesize enormous SSA block-param lists.
        let must_assign = super::compute_must_assign(body);

        // Map MIR BlockIds to Cranelift blocks by ID (not by array index).
        let mut cl_blocks: std::collections::HashMap<u32, cranelift_codegen::ir::Block> =
            std::collections::HashMap::new();
        for block in &body.blocks {
            let cl_block = builder.create_block();
            cl_blocks.insert(block.id.0, cl_block);
        }

        let entry_cl = cl_blocks[&body.blocks[0].id.0];
        builder.append_block_params_for_function_params(entry_cl);
        builder.switch_to_block(entry_cl);

        // Track parameter VRegs — these are borrowed from the caller and must
        // NOT be released by the callee's epilogue. The caller owns them.
        let mut param_vregs = std::collections::HashSet::new();
        for (i, (vreg, ty_id)) in body.params.iter().enumerate() {
            let cl_type = Self::mamba_to_cl_type(tcx.get(*ty_id));
            let var = vars.get(*vreg, &mut builder, cl_type);
            let param_val = builder.block_params(entry_cl)[i];
            builder.def_var(var, param_val);
            param_vregs.insert(*vreg);
            // Tag Int/Bool params as raw_ints so subsequent CheckedAdd/Sub/Mul
            // can take the native overflow-checked fast path. The fits_48 check
            // inside emit_raw_int_op_with_overflow_check rejects NaN-boxed bits
            // (inline int or BigInt pointer) and routes them to the runtime
            // slow path that handles boxed inputs via reg_to_mbvalue.
            if matches!(
                tcx.get(*ty_id),
                crate::types::Ty::Int | crate::types::Ty::Bool
            ) {
                vars.raw_ints.insert(*vreg);
            }
        }

        // Resolve mb_release_value FuncRef for return-time cleanup (#1129 R3).
        // #1663 T4c5 iter-5 (mitigation): re-apply the `is_entry_body` guard
        // dropped in T4c4. See `codegen/cranelift/mod.rs` (~line 322) for the
        // full rationale — the entry-body release surfaces a residual UAF in
        // `stdlib/re/*` fixtures, and the original perf motivation (#1274)
        // is already met via a separate idempotency fix (28cb58070), so the
        // 4× bench gate stays green without entry-body release.
        let release_func_ref = if EMIT_REFCOUNT_CALLS && !is_entry_body {
            let release_id = self.extern_funcs.get("mb_release_value").copied();
            release_id.map(|id| self.module().declare_func_in_func(id, builder.func))
        } else {
            None
        };
        // Retain parameters returned by value (see emit_terminator for rationale).
        let retain_func_ref = if EMIT_REFCOUNT_CALLS {
            let retain_id = self.extern_funcs.get("mb_retain_value").copied();
            retain_id.map(|id| self.module().declare_func_in_func(id, builder.func))
        } else {
            None
        };

        // #1013 / #2111 (Subset A iteration-retention amplifier): pre-seed
        // VRegs written inside a loop body with a harmless `MbValue::none()`
        // sentinel before the loop's own blocks compile. This makes
        // `VarAlloc::is_declared_i64` true from the start, so `emit_inst`'s
        // existing release-before-overwrite gate (see its doc comment,
        // below) now fires on every dynamic loop iteration instead of never
        // — the single-static-write-site shape that previously defeated it.
        // See `compute_loop_carried_vregs` for the full rationale, the
        // back-edge/natural-loop discovery, and why Int/Bool/Float/None/
        // Never VRegs are excluded (they're never heap-backed in this JIT
        // and pre-seeding them would only tax numeric hot loops). Emitted
        // while `entry_cl` is still open (not yet terminated) — safe to
        // append instructions here before the main block-emission loop
        // switches blocks and starts appending terminators.
        if EMIT_REFCOUNT_CALLS {
            let loop_carried = super::compute_loop_carried_vregs(body, tcx);
            if !loop_carried.is_empty() {
                let none_bits = MbValue::none().to_bits() as i64;
                for vreg in loop_carried {
                    if !crate::runtime::rc::should_preseed_loop_owner_slot(
                        param_vregs.contains(&vreg),
                    ) {
                        // Never override a caller-borrowed param's original
                        // value (#1018) — a param VReg is already handled
                        // by the params loop above.
                        continue;
                    }
                    let dv = vars.get(vreg, &mut builder, cl_types::I64);
                    let sentinel = builder.ins().iconst(cl_types::I64, none_bits);
                    builder.def_var(dv, sentinel);
                }
            }
        }

        for (block_idx, block) in body.blocks.iter().enumerate() {
            if block_idx > 0 {
                builder.switch_to_block(cl_blocks[&block.id.0]);
            }
            for inst in &block.stmts {
                self.emit_inst(
                    inst,
                    tcx,
                    externs,
                    &literal_escapes,
                    !is_entry_body,
                    &mut builder,
                    &mut vars,
                    &param_vregs,
                );
            }
            // Scope the Return epilogue's release loop to VRegs that are
            // definitely assigned on every path reaching THIS terminator
            // (empty/missing entry => no filtering, same as before).
            vars.live_filter = must_assign.get(&block.id.0).cloned();
            if std::env::var("MAMBA_TRACE_RETURN_RELEASES").is_ok() {
                let release_count = match &block.terminator {
                    crate::mir::Terminator::Return(Some(vreg)) => {
                        vars.releasable_i64_vregs(&param_vregs, Some(*vreg)).len()
                    }
                    crate::mir::Terminator::Return(None) => {
                        vars.releasable_i64_vregs(&param_vregs, None).len()
                    }
                    _ => 0,
                };
                if release_count > 0 {
                    eprintln!(
                        "[return-release body={} block={} candidates={} vars={}]",
                        body.name.0,
                        block.id.0,
                        release_count,
                        vars.map.len()
                    );
                }
            }
            emit_terminator(
                &block.terminator,
                &cl_blocks,
                ret_ty,
                &mut builder,
                &mut vars,
                release_func_ref,
                retain_func_ref,
                &param_vregs,
            );
        }

        // Seal all blocks after emission so that loop headers see
        // back-edges when Cranelift constructs SSA phi nodes.
        builder.seal_all_blocks();
        builder.finalize();
        // Dump Cranelift IR for the last (main) function body
        let mut ctx = cranelift_codegen::Context::for_function(func);
        // #1663 T2: env-gated CLIF dump for IR inspection (e.g. counting
        // mb_release_value calls in __main__ to confirm H1 leak path).
        // Tag with the MIR body id so __main__ (u32::MAX = 4294967295) is greppable.
        if std::env::var("MAMBA_DUMP_CLIF").is_ok() {
            let rel = self
                .extern_funcs
                .get("mb_release_value")
                .map(|id| id.as_u32());
            let ret = self
                .extern_funcs
                .get("mb_retain_value")
                .map(|id| id.as_u32());
            eprintln!(
                "[clif-dump body.name.0={} mb_release_value=u0:{:?} mb_retain_value=u0:{:?}]:\n{}",
                body.name.0,
                rel,
                ret,
                ctx.func.display()
            );
        }
        self.module()
            .define_function(func_id, &mut ctx)
            .map_err(|e| {
                eprintln!(
                    "DEBUG: Verifier fail for func_id={} body_name={}: {e:#?}",
                    func_id.as_u32(),
                    body.name.0
                );
                // Print the IR for debugging
                eprintln!("IR:\n{}", ctx.func.display());
                crate::error::MambaError::codegen(format!("define: {e}"))
            })?;
        // #2094: capture compiled code size for perf-map emission. Only
        // populate the map when MAMBA_PERF_MAP is enabled so the hot path
        // pays no allocation cost during ordinary JIT runs.
        if perf_map::is_enabled() {
            if let Some(cc) = ctx.compiled_code() {
                self.internal_code_sizes
                    .insert(body.name.0, cc.code_info().total_size);
            }
        }
        Ok(())
    }

    fn emit_inst(
        &mut self,
        inst: &MirInst,
        tcx: &TypeContext,
        externs: &[MirExtern],
        literal_escapes: &LiteralEscapeAnalysis,
        allow_untracked_literals: bool,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
        param_vregs: &std::collections::HashSet<VReg>,
    ) {
        // Release old dest value before overwriting (#1129 R2).
        // Every instruction that writes to a dest VReg must release
        // the previous value to maintain refcount balance.
        //
        // #2111 carve-out (Subset A iteration-retention amplifier):
        // This pre-write release closes the rebind-release leak ONLY for
        // VRegs that are reused across loop iterations (i.e. the
        // `sym_to_vreg`-mapped VReg in `hir_to_mir::HirStmt::Assign`'s
        // `Copy { dest: orig_vreg, … }` back-edge). Fresh per-iter VRegs
        // synthesised inside loop bodies (the `args_list` VReg from
        // method-call lowering at `hir_to_mir.rs:4515-4518`, boxed-arg
        // temporaries, intermediate Call results) get a NEW VReg on every
        // iteration, so they never appear here on a back-edge and bypass
        // rebind release entirely. They are only released at function
        // epilogue (`emit_terminator` Return path) — and for module-scope
        // (`__main__`) code the `is_entry_body` guard at line 333 above
        // skips that epilogue release to dodge the #1663 T4c5 iter-5
        // BigInt double-free regression. Net effect: hot-loop allocations
        // at module scope accumulate monotonically with ITERS, matching
        // the linear memory growth signature in #2111. Fix surface is a
        // per-loop-back-edge release sweep for fresh VRegs introduced
        // inside the loop body.
        if EMIT_REFCOUNT_CALLS {
            let dest_vreg: Option<VReg> = match inst {
                MirInst::LoadConst { dest, .. }
                | MirInst::Copy { dest, .. }
                | MirInst::BinOp { dest, .. }
                | MirInst::UnaryOp { dest, .. }
                | MirInst::GetAttr { dest, .. }
                | MirInst::GetItem { dest, .. }
                | MirInst::MakeList { dest, .. }
                | MirInst::MakeDict { dest, .. }
                | MirInst::MakeTuple { dest, .. }
                | MirInst::LoadGlobal { dest, .. }
                | MirInst::LoadCell { dest, .. }
                | MirInst::MakeCell { dest, .. }
                | MirInst::LoadCapture { dest, .. }
                | MirInst::CheckedAdd { dest, .. }
                | MirInst::CheckedSub { dest, .. }
                | MirInst::CheckedMul { dest, .. } => Some(*dest),
                // Call/CallExtern have Option<VReg> dest
                MirInst::Call {
                    dest: Some(dest), ..
                }
                | MirInst::CallExtern {
                    dest: Some(dest), ..
                } => Some(*dest),
                _ => None,
            };
            if let Some(dest) = dest_vreg {
                // Only release if variable was already declared as I64.
                // F64 variables never hold heap pointers — skip them.
                // First-time writes (var not yet declared) are also skipped
                // (default 0 would be a no-op release anyway).
                // Skip raw_ints — the previous value is a raw i64, not a
                // heap pointer, so mb_release_value's as_ptr check would
                // bail out anyway.
                // #1018: also skip parameter VRegs. Parameters are borrowed
                // from the caller (see the `param_vregs` construction above,
                // and `emit_terminator`'s Return epilogue, which excludes
                // them from its release sweep for the same reason) — the
                // callee must never release a param's ORIGINAL value, even
                // when reassigning the param's own VReg (e.g. `args =
                // args[1:]`, or a tuple-unpack target that reuses the
                // param's VReg via `sym_to_vreg`). Without this check, the
                // first such reassignment released the caller's live
                // reference out from under it, causing a double free once
                // the caller (or callee's own later cleanup) released the
                // same object again — surfacing as nondeterministic heap
                // corruption (SIGSEGV/SIGBUS/SIGTRAP/capacity-overflow/
                // wrong-value, depending on what reused the freed memory).
                if crate::runtime::rc::should_release_local_slot(
                    crate::runtime::rc::LocalSlotReleaseRule {
                        declared_i64: vars.is_declared_i64(dest),
                        raw_value: vars.raw_ints.contains(&dest),
                        native_bool: vars.native_bools.contains(&dest),
                        assigned_on_path: true,
                        borrowed_param: param_vregs.contains(&dest),
                        return_value: false,
                    },
                ) {
                    if let Some(&release_id) = self.extern_funcs.get("mb_release_value") {
                        let release_ref =
                            self.module().declare_func_in_func(release_id, builder.func);
                        let dv = vars.get(dest, builder, cl_types::I64);
                        let old_val = builder.use_var(dv);
                        builder.ins().call(release_ref, &[old_val]);
                    }
                }
            }
        }

        match inst {
            MirInst::LoadConst { dest, value, ty } => {
                let cl_type = Self::mamba_to_cl_type(tcx.get(*ty));
                let var = vars.get(*dest, builder, cl_type);
                let val = match value {
                    MirConst::Int(v) => {
                        vars.raw_ints.insert(*dest);
                        builder.ins().iconst(cl_types::I64, *v)
                    }
                    MirConst::BigInt(s) => {
                        let val = crate::runtime::bigint_ops::bigint_immortal_from_literal(s);
                        builder.ins().iconst(cl_types::I64, val.to_bits() as i64)
                    }
                    MirConst::Float(v) => {
                        // Store as I64 (NaN-boxed): raw IEEE 754 bits as u64.
                        // MbValue::from_float stores raw bits for normal floats.
                        builder
                            .ins()
                            .iconst(cl_types::I64, MbValue::from_float(*v).to_bits() as i64)
                    }
                    MirConst::Bool(v) => {
                        vars.raw_ints.insert(*dest);
                        builder.ins().iconst(cl_types::I64, *v as i64)
                    }
                    MirConst::None => builder
                        .ins()
                        .iconst(cl_types::I64, MbValue::none().to_bits() as i64),
                    MirConst::NotImplemented => builder
                        .ins()
                        .iconst(cl_types::I64, MbValue::not_implemented().to_bits() as i64),
                    MirConst::Ellipsis => builder
                        .ins()
                        .iconst(cl_types::I64, MbValue::ellipsis().to_bits() as i64),
                    MirConst::Str(s) => {
                        // Allocate immortal string at JIT compile time (#1129 R4).
                        let ptr = if let Some(codepoints) =
                            crate::lexer::token::decode_surrogate_escape_markers(s)
                        {
                            crate::runtime::string_ops::new_surrogate_codepoints_str_immortal(
                                codepoints,
                            )
                        } else {
                            MbObject::new_str_immortal(s.clone())
                        };
                        self.compile_time_objects.push(ptr);
                        let str_val = MbValue::from_ptr(ptr);
                        builder
                            .ins()
                            .iconst(cl_types::I64, str_val.to_bits() as i64)
                    }
                    MirConst::Bytes(data) => {
                        // Allocate immortal bytes at JIT compile time (#1129 R4).
                        let ptr = MbObject::new_bytes_immortal(data.clone());
                        self.compile_time_objects.push(ptr);
                        let bytes_val = MbValue::from_ptr(ptr);
                        builder
                            .ins()
                            .iconst(cl_types::I64, bytes_val.to_bits() as i64)
                    }
                    MirConst::FuncRef(sym) => {
                        // Load function address for class method / lambda / async body (#313 R1).
                        // Stored as TAG_FUNC (4) so mb_map/mb_filter can distinguish from heap ptrs.
                        if let Some(&func_id) = self.internal_funcs.get(&sym.0) {
                            let fref = self.module().declare_func_in_func(func_id, builder.func);
                            let raw_addr = builder.ins().func_addr(cl_types::I64, fref);
                            // NaN-box with TAG_FUNC=4: NAN_PREFIX | (4 << 48) | addr
                            let tag_prefix = builder
                                .ins()
                                .iconst(cl_types::I64, 0xFFFC_0000_0000_0000u64 as i64);
                            builder.ins().bor(raw_addr, tag_prefix)
                        } else {
                            builder.ins().iconst(cl_types::I64, 0)
                        }
                    }
                    MirConst::ExternFuncRef(name) => {
                        // Load address of a runtime extern function (e.g. "mb_abs", "mb_str").
                        // Stored as TAG_FUNC (4) so mb_map/mb_filter can call them safely.
                        if let Some(&func_id) = self.extern_funcs.get(name.as_str()) {
                            let fref = self.module().declare_func_in_func(func_id, builder.func);
                            let raw_addr = builder.ins().func_addr(cl_types::I64, fref);
                            // NaN-box with TAG_FUNC=4: NAN_PREFIX | (4 << 48) | addr
                            let tag_prefix = builder
                                .ins()
                                .iconst(cl_types::I64, 0xFFFC_0000_0000_0000u64 as i64);
                            builder.ins().bor(raw_addr, tag_prefix)
                        } else {
                            builder.ins().iconst(cl_types::I64, 0)
                        }
                    }
                };
                builder.def_var(var, val);
            }
            MirInst::BinOp {
                dest,
                op,
                lhs,
                rhs,
                ty,
            } => {
                let resolved_ty = tcx.get(*ty);
                let use_primitive = match op {
                    MirBinOp::Is | MirBinOp::IsNot => true,
                    MirBinOp::In | MirBinOp::NotIn => false,
                    _ => matches!(resolved_ty, Ty::Int | Ty::Float | Ty::Bool),
                };
                let is_mod = matches!(op, MirBinOp::Mod)
                    && matches!(resolved_ty, Ty::Int | Ty::Float | Ty::Bool);
                if matches!(op, MirBinOp::FloorDiv) || is_mod {
                    // Floor division → call mb_floordiv runtime for correct Python
                    // floor semantics and ZeroDivisionError handling (#1085).
                    // Modulo → call mb_mod for the same reason: the inline `srem`
                    // fast path executed a raw Cranelift hardware trap on `x % 0`
                    // (SIGILL, exit 132) instead of raising a catchable
                    // ZeroDivisionError, and the inline float `%` path returned NaN
                    // for `1.0 % 0.0` instead of raising ZeroDivisionError (#35).
                    // Routing through the runtime also gives BigInt operands a
                    // correct `%` / `//` result instead of operating on the raw
                    // tagged pointer bits.
                    // Float operands are already NaN-boxed I64 MbValues — no boxing needed.
                    // Int/Bool operands need boxing from raw I64 to MbValue.
                    let helper_name = if is_mod { "mb_mod" } else { "mb_floordiv" };
                    let floordiv_id = self.extern_funcs.get(helper_name).copied();
                    let is_float = matches!(resolved_ty, Ty::Float);
                    let box_id = if is_float {
                        None
                    } else {
                        let box_fn_name = match resolved_ty {
                            Ty::Bool => "mb_box_bool",
                            _ => "mb_box_int",
                        };
                        self.extern_funcs.get(box_fn_name).copied()
                    };
                    if let Some(func_id) = floordiv_id {
                        let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                        let l = vars.use_as_i64(*lhs, builder);
                        let r = vars.use_as_i64(*rhs, builder);
                        let (l_boxed, r_boxed) = if let Some(bid) = box_id {
                            let fref = self.module().declare_func_in_func(bid, builder.func);
                            let lc = builder.ins().call(fref, &[l]);
                            let rc = builder.ins().call(fref, &[r]);
                            (builder.inst_results(lc)[0], builder.inst_results(rc)[0])
                        } else {
                            (l, r)
                        };
                        let call = builder.ins().call(func_ref, &[l_boxed, r_boxed]);
                        let result_bits = builder.inst_results(call)[0];
                        // For Int operands: mb_floordiv/mb_mod return NaN-boxed MbValue,
                        // but subsequent primitive ops expect raw i64. Unbox inline-int
                        // results (tag=1) to raw i64, keep BigInt NaN-boxed.
                        if !is_float {
                            let tag_raw = builder.ins().ushr_imm(result_bits, 48);
                            let tag = builder.ins().band_imm(tag_raw, 7);
                            let tag_int = builder.ins().iconst(cl_types::I64, 1);
                            let is_inline = builder.ins().icmp(
                                cranelift_codegen::ir::condcodes::IntCC::Equal,
                                tag,
                                tag_int,
                            );
                            let payload_mask = builder
                                .ins()
                                .iconst(cl_types::I64, 0x0000_FFFF_FFFF_FFFFi64);
                            let payload = builder.ins().band(result_bits, payload_mask);
                            let shifted = builder.ins().ishl_imm(payload, 16);
                            let unboxed = builder.ins().sshr_imm(shifted, 16);
                            let result = builder.ins().select(is_inline, unboxed, result_bits);
                            vars.def_var_cast(*dest, builder, result, cl_types::I64);
                        } else {
                            vars.def_var_cast(*dest, builder, result_bits, cl_types::I64);
                        }
                    } else {
                        let zero = builder.ins().iconst(cl_types::I64, 0);
                        vars.def_var_cast(*dest, builder, zero, cl_types::I64);
                    }
                } else if matches!(op, MirBinOp::BitAnd | MirBinOp::BitOr | MirBinOp::BitXor)
                    && matches!(resolved_ty, Ty::Int)
                {
                    // Bitwise AND/OR/XOR of two *genuinely* raw (non-NaN-boxed)
                    // i64 operands is provably safe to compute natively (the
                    // redundant sign/tag bits never force BigInt promotion —
                    // bitwise ops never grow beyond the operands' own width).
                    // BUT `vars.raw_ints` is not a strict "fits in 48 bits"
                    // guarantee: it's also set for call-results/params from a
                    // statically Ty::Int callee (jit.rs `emit_internal_call`,
                    // ~line 2089) *unconditionally* on the static type, even
                    // when that callee's *actual* returned bit pattern is a
                    // NaN-boxed BigInt heap pointer (produced by its own
                    // internal CheckedAdd/Sub/Mul slow path once a literal
                    // exceeds the 48-bit inline threshold — e.g. `return
                    // 9_000_000_000_000_000_000 + i`). CheckedAdd/Sub/Mul
                    // tolerate that mistagging because their *native op result*
                    // gets an explicit overflow/fits-check that (empirically)
                    // always reroutes a garbage computation to the BigInt-aware
                    // runtime, whose `mb_box_int`/`reg_to_mbvalue` tag-detection
                    // then self-corrects. Bitwise AND/OR/XOR have no analogous
                    // "does this look wrong" signal — any bit pattern is a
                    // "valid" result — so a raw_ints-gated *native* band/bor/bxor
                    // has no safety net and silently corrupts a mistagged
                    // NaN-boxed pointer's bits (confirmed via a `total ^=
                    // f(i)` repro with `f` returning a promoted-to-BigInt Int).
                    // Route unconditionally through the runtime instead: v1
                    // per #1090's own guidance ("routing through CallExtern
                    // unconditionally is acceptable v1 if pin-neutral,
                    // measure") — `mb_box_int` is idempotent/tag-aware on
                    // already-boxed input, so this is correct for every case.
                    let helper_name = match op {
                        MirBinOp::BitAnd => "mb_bitand",
                        MirBinOp::BitOr => "mb_bitor",
                        _ => "mb_bitxor",
                    };
                    self.emit_checked_bitwise_op(dest, lhs, rhs, helper_name, builder, vars);
                } else if matches!(op, MirBinOp::LShift) && matches!(resolved_ty, Ty::Int) {
                    // Left shift can promote to BigInt even when BOTH operands
                    // are already proven inline (`1 << 64`) — unlike AND/OR/XOR/
                    // RSHIFT above/below (which dropped their raw_ints-gated
                    // native fast path entirely, see those arms' comments), an
                    // operand-tag check alone is not enough here; this needs an
                    // overflow-checked fast path (#1090).
                    if vars.raw_ints.contains(lhs) && vars.raw_ints.contains(rhs) {
                        self.emit_raw_lshift_with_overflow_check(dest, lhs, rhs, builder, vars);
                    } else {
                        self.emit_checked_bitwise_op(dest, lhs, rhs, "mb_lshift", builder, vars);
                    }
                } else if matches!(op, MirBinOp::RShift) && matches!(resolved_ty, Ty::Int) {
                    // Right shift of a *genuinely* raw inline base always stays
                    // inline (magnitude only shrinks toward zero) — but same
                    // hazard as AND/OR/XOR above: `raw_ints` doesn't guarantee
                    // the operand isn't a mistagged NaN-boxed BigInt pointer
                    // from a promoted call-result/param, and `sshr` has no
                    // overflow-style check to catch that after the fact
                    // (confirmed via repro: `total >>= 1` after XORing in a
                    // promoted-to-BigInt call result produced garbage/a raw
                    // function pointer leaking into output). Route
                    // unconditionally through the BigInt-aware runtime
                    // (#1085) — `mb_box_int` is idempotent/tag-aware so this
                    // is correct for every case.
                    self.emit_checked_bitwise_op(dest, lhs, rhs, "mb_rshift", builder, vars);
                } else if matches!(op, MirBinOp::Pow) && matches!(resolved_ty, Ty::Int) {
                    // Integer power → call mb_pow_int runtime function
                    if let Some(&func_id) = self.extern_funcs.get("mb_pow_int") {
                        let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                        let l = vars.use_as_i64(*lhs, builder);
                        let r = vars.use_as_i64(*rhs, builder);
                        let call = builder.ins().call(func_ref, &[l, r]);
                        let result = builder.inst_results(call)[0];
                        vars.def_var_cast(*dest, builder, result, cl_types::I64);
                    } else {
                        let zero = builder.ins().iconst(cl_types::I64, 0);
                        vars.def_var_cast(*dest, builder, zero, cl_types::I64);
                    }
                } else if matches!(op, MirBinOp::Pow) && matches!(resolved_ty, Ty::Float) {
                    // Float power → call mb_pow_float (f64, f64) -> f64.
                    // emit_binop has no `(Pow, Float)` arm; without this it
                    // fell through to `iadd`, treating the f64 bit pattern
                    // as an i64 and producing garbage like 2.0**3.0 ≈ -1e-308.
                    if let Some(&func_id) = self.extern_funcs.get("mb_pow_float") {
                        let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                        let l = vars.use_as(*lhs, cl_types::F64, builder);
                        let r = vars.use_as(*rhs, cl_types::F64, builder);
                        let call = builder.ins().call(func_ref, &[l, r]);
                        let result = builder.inst_results(call)[0];
                        vars.def_var_cast(*dest, builder, result, cl_types::F64);
                    } else {
                        let zero = builder.ins().f64const(0.0);
                        vars.def_var_cast(*dest, builder, zero, cl_types::F64);
                    }
                } else if matches!(op, MirBinOp::In | MirBinOp::NotIn) {
                    if let Some(&func_id) = self.extern_funcs.get("mb_obj_contains") {
                        let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                        let r = vars.use_as_i64(*rhs, builder);
                        let l = vars.use_as_i64(*lhs, builder);
                        let call = builder.ins().call(func_ref, &[r, l]);
                        let result = builder.inst_results(call)[0];
                        let final_result = if matches!(op, MirBinOp::NotIn) {
                            let one = builder.ins().iconst(cl_types::I64, 1);
                            builder.ins().bxor(result, one)
                        } else {
                            result
                        };
                        vars.def_var_cast(*dest, builder, final_result, cl_types::I64);
                    } else {
                        let zero = builder.ins().iconst(cl_types::I64, 0);
                        vars.def_var_cast(*dest, builder, zero, cl_types::I64);
                    }
                } else if matches!(
                    op,
                    MirBinOp::Eq
                        | MirBinOp::NotEq
                        | MirBinOp::Lt
                        | MirBinOp::Gt
                        | MirBinOp::LtEq
                        | MirBinOp::GtEq
                ) {
                    // #1131: rich comparisons of Int/Bool-typed operands do
                    // NOT get pre-dispatched to a boxed CallExtern at
                    // lowering time the way Float/Str comparisons do (those
                    // never reach a MIR BinOp at all — confirmed via MIR
                    // dump). Instead lowering unboxes both operands (e.g.
                    // `mb_unbox_int_if_boxed`) and emits a raw BinOp here —
                    // whose `ty` field is always the *result* type (Bool),
                    // never useful for gating. Comparing the raw i64 bits
                    // directly (the old behavior) is wrong whenever an
                    // operand is secretly a NaN-boxed BigInt: either a
                    // genuine large int that unboxing left boxed (the
                    // unbox helper's fallback literally returns
                    // `val.to_bits() as i64` — the *same* NaN-boxed bits,
                    // just relabeled), or a mistagged call-result (#1090:
                    // "raw_ints tags LIE for call-results; comparisons have
                    // no self-check signal").
                    self.emit_checked_int_compare(dest, lhs, rhs, op, builder, vars);
                } else if use_primitive {
                    let cl_type = Self::mamba_to_cl_type(resolved_ty);
                    // use_as handles I64→F64 bitcast when operand came from runtime call
                    let l = vars.use_as(*lhs, cl_type, builder);
                    let r = vars.use_as(*rhs, cl_type, builder);
                    let dv = vars.get(*dest, builder, cl_type);
                    let result = emit_binop(builder, op, resolved_ty, l, r);
                    builder.def_var(dv, result);
                    // Propagate raw_int: if both operands are raw i64 and result
                    // is Int, the result is also raw i64.
                    if matches!(resolved_ty, Ty::Int | Ty::Bool)
                        && vars.raw_ints.contains(lhs)
                        && vars.raw_ints.contains(rhs)
                    {
                        vars.raw_ints.insert(*dest);
                    }
                } else if let Some(&func_id) = self.extern_funcs.get("mb_dispatch_binop") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let opcode = builder.ins().iconst(cl_types::I64, op.to_opcode());
                    let l = vars.use_as_i64(*lhs, builder);
                    let r = vars.use_as_i64(*rhs, builder);
                    let call = builder.ins().call(func_ref, &[opcode, l, r]);
                    let result = builder.inst_results(call)[0];
                    vars.def_var_cast(*dest, builder, result, cl_types::I64);
                } else {
                    let cl_type = Self::mamba_to_cl_type(resolved_ty);
                    let l = vars.use_as(*lhs, cl_type, builder);
                    let r = vars.use_as(*rhs, cl_type, builder);
                    let dv = vars.get(*dest, builder, cl_type);
                    let result = emit_binop(builder, op, resolved_ty, l, r);
                    builder.def_var(dv, result);
                }
            }
            MirInst::Copy { dest, source } => {
                // Copy with auto-bitcast: source and dest may have different types
                // (e.g., I64 from runtime call copied into F64 variable, or vice versa).
                let src_type = vars.declared_type(*source).unwrap_or(cl_types::I64);
                let sv = vars.get(*source, builder, src_type);
                let val = builder.use_var(sv);
                vars.def_var_cast(*dest, builder, val, src_type);
                // Propagate raw_int through copies — must remove first so
                // a vreg that was previously raw doesn't stay raw when the
                // source is NaN-boxed (e.g., after FloorDiv runtime call).
                vars.raw_ints.remove(dest);
                if vars.raw_ints.contains(source) {
                    vars.raw_ints.insert(*dest);
                }
                if EMIT_REFCOUNT_CALLS && !vars.raw_ints.contains(source) {
                    // Retain the new value — Copy is aliasing, both source
                    // and dest now reference the same object (#1129 R2).
                    // Only retain I64 (pointer) values, not F64 (floats).
                    // Use I64 version for retain since mb_retain_value expects I64.
                    // Skip raw_ints sources: mb_retain_value as_ptr-checks
                    // the NaN tag and is a no-op for raw i64s.
                    let i64_val = vars.use_as_i64(*source, builder);
                    if src_type == cl_types::I64 || vars.declared_type(*dest) == Some(cl_types::I64)
                    {
                        if let Some(&retain_id) = self.extern_funcs.get("mb_retain_value") {
                            let retain_ref =
                                self.module().declare_func_in_func(retain_id, builder.func);
                            builder.ins().call(retain_ref, &[i64_val]);
                        }
                    }
                }
            }
            MirInst::Call {
                dest,
                func,
                args,
                ty,
            } => {
                self.emit_internal_call(dest, func.0, args, ty, tcx, builder, vars);
            }
            MirInst::CallExtern {
                dest,
                name,
                args,
                ty,
            } => {
                self.emit_extern_call(dest, name, args, ty, tcx, externs, builder, vars);
            }
            MirInst::UnaryOp {
                dest,
                op,
                operand,
                ty,
            } => {
                let resolved_ty = tcx.get(*ty);
                let is_primitive = matches!(resolved_ty, Ty::Int | Ty::Float | Ty::Bool);
                if is_primitive {
                    let cl_type = Self::mamba_to_cl_type(resolved_ty);
                    let val = vars.use_as(*operand, cl_type, builder);
                    let dv = vars.get(*dest, builder, cl_type);
                    let result = match op {
                        crate::mir::MirUnaryOp::Pos => val,
                        crate::mir::MirUnaryOp::Neg => {
                            // Use fneg for floats (with I64↔F64 bitcast), ineg for integers/bools.
                            if matches!(resolved_ty, Ty::Float) {
                                let fval =
                                    builder.ins().bitcast(cl_types::F64, MemFlags::new(), val);
                                let neg = builder.ins().fneg(fval);
                                builder.ins().bitcast(cl_types::I64, MemFlags::new(), neg)
                            } else {
                                builder.ins().ineg(val)
                            }
                        }
                        crate::mir::MirUnaryOp::Not => {
                            // Python `not x` evaluates truthiness then inverts.
                            // Raw ints are not necessarily 0/1 (`not 5` is
                            // False), so compare the truth value to zero.
                            let truth_value = if vars.raw_ints.contains(operand) {
                                val
                            } else if let Some(&truthy_id) = self.extern_funcs.get("mb_is_truthy") {
                                let truthy_ref =
                                    self.module().declare_func_in_func(truthy_id, builder.func);
                                let call = builder.ins().call(truthy_ref, &[val]);
                                builder.inst_results(call)[0]
                            } else {
                                val
                            };
                            super::emit_logical_not(builder, truth_value)
                        }
                        crate::mir::MirUnaryOp::BitNot => {
                            if matches!(resolved_ty, Ty::Int) {
                                // Always route through the BigInt-aware dunder
                                // dispatch (op_code 3 = `__invert__`) for
                                // Ty::Int — same mistagging hazard as the
                                // AND/OR/XOR/RSHIFT `BinOp` arms above:
                                // `vars.raw_ints` doesn't guarantee `operand`
                                // isn't a mistagged NaN-boxed BigInt pointer
                                // (from a promoted call-result/param whose
                                // static type is Int but whose actual runtime
                                // value overflowed 48 bits internally), and a
                                // raw `bnot` has no overflow-style check to
                                // catch that after the fact. Box first
                                // (`mb_box_int` is idempotent/tag-aware on
                                // already-boxed input) so the dispatch always
                                // receives a properly NaN-boxed MbValue
                                // regardless of whether `operand` was
                                // genuinely raw or mistagged (#1090).
                                let boxed_val = if let Some(&box_id) =
                                    self.extern_funcs.get("mb_box_int")
                                {
                                    let box_ref =
                                        self.module().declare_func_in_func(box_id, builder.func);
                                    let call = builder.ins().call(box_ref, &[val]);
                                    builder.inst_results(call)[0]
                                } else {
                                    val
                                };
                                if let Some(&func_id) = self.extern_funcs.get("mb_dispatch_unaryop")
                                {
                                    let func_ref =
                                        self.module().declare_func_in_func(func_id, builder.func);
                                    let opcode =
                                        builder.ins().iconst(cl_types::I64, op.to_opcode());
                                    let call = builder.ins().call(func_ref, &[opcode, boxed_val]);
                                    builder.inst_results(call)[0]
                                } else {
                                    builder.ins().bnot(val)
                                }
                            } else {
                                // Bool/Float operand: `!x` of any inline value
                                // always stays inline (never at BigInt-
                                // promotion risk), so the native op is exact.
                                builder.ins().bnot(val)
                            }
                        }
                    };
                    builder.def_var(dv, result);
                    // Not always produces raw 0/1 — mark for direct branching
                    if matches!(op, crate::mir::MirUnaryOp::Not) {
                        vars.raw_ints.insert(*dest);
                        vars.native_bools.insert(*dest);
                    }
                } else if let Some(&func_id) = self.extern_funcs.get("mb_dispatch_unaryop") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let opcode = builder.ins().iconst(cl_types::I64, op.to_opcode());
                    let val = vars.use_as_i64(*operand, builder);
                    let call = builder.ins().call(func_ref, &[opcode, val]);
                    let result = builder.inst_results(call)[0];
                    vars.def_var_cast(*dest, builder, result, cl_types::I64);
                } else {
                    let cl_type = Self::mamba_to_cl_type(resolved_ty);
                    let val = vars.use_as(*operand, cl_type, builder);
                    let dv = vars.get(*dest, builder, cl_type);
                    let result = match op {
                        crate::mir::MirUnaryOp::Pos => val,
                        crate::mir::MirUnaryOp::Neg => {
                            if matches!(resolved_ty, Ty::Float) {
                                let fval =
                                    builder.ins().bitcast(cl_types::F64, MemFlags::new(), val);
                                let neg = builder.ins().fneg(fval);
                                builder.ins().bitcast(cl_types::I64, MemFlags::new(), neg)
                            } else {
                                builder.ins().ineg(val)
                            }
                        }
                        crate::mir::MirUnaryOp::Not => {
                            let truth_value = if vars.raw_ints.contains(operand) {
                                val
                            } else if let Some(&truthy_id) = self.extern_funcs.get("mb_is_truthy") {
                                let truthy_ref =
                                    self.module().declare_func_in_func(truthy_id, builder.func);
                                let call = builder.ins().call(truthy_ref, &[val]);
                                builder.inst_results(call)[0]
                            } else {
                                val
                            };
                            super::emit_logical_not(builder, truth_value)
                        }
                        crate::mir::MirUnaryOp::BitNot => builder.ins().bnot(val),
                    };
                    builder.def_var(dv, result);
                    if matches!(op, crate::mir::MirUnaryOp::Not) {
                        vars.raw_ints.insert(*dest);
                        vars.native_bools.insert(*dest);
                    }
                }
            }
            // Object operations — emit real FFI calls to runtime
            MirInst::GetAttr {
                dest,
                object,
                attr,
                ty: _,
            } => {
                self.emit_getattr(dest, object, attr, builder, vars, externs);
            }
            MirInst::SetAttr {
                object,
                attr,
                value,
            } => {
                self.emit_setattr(object, attr, value, builder, vars, externs);
            }
            MirInst::GetItem {
                dest,
                object,
                index,
                ty: _,
            } => {
                self.emit_getitem(dest, object, index, builder, vars, externs);
            }
            MirInst::SetItem {
                object,
                index,
                value,
            } => {
                self.emit_setitem(object, index, value, builder, vars, externs);
            }
            MirInst::MakeList {
                dest,
                elements,
                ty: _,
            } => {
                self.emit_make_list(
                    dest,
                    elements,
                    literal_escapes,
                    allow_untracked_literals,
                    builder,
                    vars,
                    externs,
                );
            }
            MirInst::MakeDict {
                dest,
                keys,
                values,
                ty: _,
            } => {
                self.emit_make_dict(
                    dest,
                    keys,
                    values,
                    literal_escapes,
                    allow_untracked_literals,
                    builder,
                    vars,
                    externs,
                );
            }
            MirInst::MakeTuple {
                dest,
                elements,
                ty: _,
            } => {
                self.emit_make_tuple(dest, elements, builder, vars, externs);
            }
            MirInst::Raise { value } => {
                if let Some(vreg) = value {
                    let v = vars.get(*vreg, builder, cl_types::I64);
                    let _val = builder.use_var(v);
                }
                builder
                    .ins()
                    .trap(cranelift_codegen::ir::TrapCode::user(1).unwrap());
                // `trap` is a terminator; subsequent insts in this MIR block
                // need somewhere to live. Open a fresh (unreachable) cranelift
                // block so the verifier's "single terminator per block" rule
                // holds.
                let dead = builder.create_block();
                builder.switch_to_block(dead);
                builder.seal_block(dead);
            }
            MirInst::LoadGlobal { dest, name, .. } => {
                // Call mb_global_get_id(symbol_id) → MbValue
                if let Some(&func_id) = self.extern_funcs.get("mb_global_get_id") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let id_val = builder.ins().iconst(cl_types::I64, name.0 as i64);
                    let call = builder.ins().call(func_ref, &[id_val]);
                    let result = builder.inst_results(call)[0];
                    vars.def_var_cast(*dest, builder, result, cl_types::I64);
                }
            }
            MirInst::StoreGlobal { name, value } => {
                // mb_global_set_id owns retaining the new value and releasing the overwritten value.
                if let Some(&func_id) = self.extern_funcs.get("mb_global_set_id") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let id_val = builder.ins().iconst(cl_types::I64, name.0 as i64);
                    let val = vars.use_as_i64(*value, builder);
                    builder.ins().call(func_ref, &[id_val, val]);
                }
            }
            MirInst::DeleteGlobal { name } => {
                if let Some(&func_id) = self.extern_funcs.get("mb_global_del_id") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let id_val = builder.ins().iconst(cl_types::I64, name.0 as i64);
                    builder.ins().call(func_ref, &[id_val]);
                }
            }
            MirInst::LoadCell { dest, cell_idx, .. } => {
                if let Some(&func_id) = self.extern_funcs.get("mb_cell_get") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let idx_val = builder.ins().iconst(cl_types::I64, *cell_idx as i64);
                    let call = builder.ins().call(func_ref, &[idx_val]);
                    let result = builder.inst_results(call)[0];
                    vars.def_var_cast(*dest, builder, result, cl_types::I64);
                }
            }
            MirInst::StoreCell { cell_idx, value } => {
                // mb_cell_set owns retaining the new value and releasing the overwritten value.
                if let Some(&func_id) = self.extern_funcs.get("mb_cell_set") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let idx_val = builder.ins().iconst(cl_types::I64, *cell_idx as i64);
                    let val = vars.use_as_i64(*value, builder);
                    builder.ins().call(func_ref, &[idx_val, val]);
                }
            }
            MirInst::MakeCell { dest, value, .. } => {
                if let Some(&func_id) = self.extern_funcs.get("mb_cell_new") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let val = vars.use_as_i64(*value, builder);
                    let call = builder.ins().call(func_ref, &[val]);
                    let result = builder.inst_results(call)[0];
                    vars.def_var_cast(*dest, builder, result, cl_types::I64);
                }
            }
            MirInst::LoadCapture {
                dest, capture_idx, ..
            } => {
                if let Some(&func_id) = self.extern_funcs.get("mb_closure_get_capture") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    // closure_handle is passed as first hidden parameter (vreg 0 by convention)
                    let closure_var = vars.get(VReg(0), builder, cl_types::I64);
                    let closure_val = builder.use_var(closure_var);
                    let idx_val = builder.ins().iconst(cl_types::I64, *capture_idx as i64);
                    let call = builder.ins().call(func_ref, &[closure_val, idx_val]);
                    let result = builder.inst_results(call)[0];
                    vars.def_var_cast(*dest, builder, result, cl_types::I64);
                }
            }
            MirInst::CheckedAdd {
                dest,
                lhs,
                rhs,
                ty: _,
            } => {
                if vars.raw_ints.contains(lhs) && vars.raw_ints.contains(rhs) {
                    self.emit_raw_int_op_with_overflow_check(
                        dest,
                        lhs,
                        rhs,
                        RawIntOp::Add,
                        "mb_bigint_add",
                        builder,
                        vars,
                    );
                } else {
                    self.emit_checked_int_op(dest, lhs, rhs, "mb_bigint_add", builder, vars);
                }
            }
            MirInst::CheckedSub {
                dest,
                lhs,
                rhs,
                ty: _,
            } => {
                if vars.raw_ints.contains(lhs) && vars.raw_ints.contains(rhs) {
                    self.emit_raw_int_op_with_overflow_check(
                        dest,
                        lhs,
                        rhs,
                        RawIntOp::Sub,
                        "mb_bigint_sub",
                        builder,
                        vars,
                    );
                } else {
                    self.emit_checked_int_op(dest, lhs, rhs, "mb_bigint_sub", builder, vars);
                }
            }
            MirInst::CheckedMul {
                dest,
                lhs,
                rhs,
                ty: _,
            } => {
                if vars.raw_ints.contains(lhs) && vars.raw_ints.contains(rhs) {
                    self.emit_raw_int_op_with_overflow_check(
                        dest,
                        lhs,
                        rhs,
                        RawIntOp::Mul,
                        "mb_bigint_mul",
                        builder,
                        vars,
                    );
                } else {
                    self.emit_checked_int_op(dest, lhs, rhs, "mb_bigint_mul", builder, vars);
                }
            }
        }
    }

    /// Emit overflow-checked integer arithmetic via BigInt runtime ABI (#833).
    ///
    /// Pass raw register values directly to mb_bigint_{add,sub,mul}.
    /// The ABI functions handle both raw i64 and NaN-boxed BigInt inputs
    /// via `reg_to_mbvalue()`. Returns NaN-boxed MbValue bits.
    ///
    /// For inline int results (tag=1): unbox to raw i64 so subsequent primitive
    /// ops work correctly. For BigInt pointer results (tag=0): keep NaN-boxed
    /// bits — subsequent checked ops and mb_box_int both handle this.
    fn emit_checked_int_op(
        &mut self,
        dest: &crate::mir::VReg,
        lhs: &crate::mir::VReg,
        rhs: &crate::mir::VReg,
        func_name: &str,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
    ) {
        use cranelift_codegen::ir::InstBuilder;

        const PAYLOAD_MASK: i64 = 0x0000_FFFF_FFFF_FFFFi64;
        const TAG_INT_VAL: i64 = 1i64;

        let lv = vars.get(*lhs, builder, cl_types::I64);
        let rv = vars.get(*rhs, builder, cl_types::I64);
        let l = builder.use_var(lv);
        let r = builder.use_var(rv);

        if let Some(&func_id) = self.extern_funcs.get(func_name) {
            let func_ref = self.module().declare_func_in_func(func_id, builder.func);
            // Pass raw register values — the ABI function handles both
            // raw i64 and NaN-boxed BigInt inputs via reg_to_mbvalue().
            let call = builder.ins().call(func_ref, &[l, r]);
            let result_bits = builder.inst_results(call)[0];

            // Check result tag: (result_bits >> 48) & 7
            let tag_raw = builder.ins().ushr_imm(result_bits, 48);
            let tag = builder.ins().band_imm(tag_raw, 7);
            let tag_int_const = builder.ins().iconst(cl_types::I64, TAG_INT_VAL);
            let is_inline = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                tag_int_const,
            );

            // Unbox inline-int path: sign-extend from 48-bit payload
            let pm = builder.ins().iconst(cl_types::I64, PAYLOAD_MASK);
            let result_payload = builder.ins().band(result_bits, pm);
            let shifted = builder.ins().ishl_imm(result_payload, 16);
            let unboxed = builder.ins().sshr_imm(shifted, 16);

            // Select: if inline → unboxed raw i64, else → NaN-boxed BigInt bits
            let result = builder.ins().select(is_inline, unboxed, result_bits);

            vars.def_var_cast(*dest, builder, result, cl_types::I64);
        } else {
            // Fallback: raw wrapping arithmetic (BigInt runtime unavailable)
            let result = match func_name {
                "mb_bigint_sub" => builder.ins().isub(l, r),
                "mb_bigint_mul" => builder.ins().imul(l, r),
                _ => builder.ins().iadd(l, r),
            };
            vars.def_var_cast(*dest, builder, result, cl_types::I64);
        }
    }

    /// Emit raw-int CheckedAdd/Sub/Mul with INT48 overflow detection (#1212 §5b).
    ///
    /// Both operands are already in `vars.raw_ints` (raw i64 values that fit
    /// in 48-bit signed). The native op may produce a result outside that range
    /// — silent wrap would be a Py3.12 conformance hole (Python ints are
    /// unbounded). We branch on overflow: fast path returns the native result;
    /// slow path calls `mb_bigint_{add,sub,mul}` which returns either an inline
    /// MbValue (tag=1, payload re-fits) or a NaN-boxed BigInt heap pointer
    /// (tag=0). The merge produces a single i64 that downstream code MUST
    /// treat as potentially boxed — `dest` is removed from `raw_ints`.
    ///
    /// Cost on the no-overflow hot path:
    /// - Add/Sub: native iadd/isub + 3-instr 48-bit fits-check + brif + jump.
    /// - Mul: native imul + smulhi + 4-instr fits-check + brif + jump.
    fn emit_raw_int_op_with_overflow_check(
        &mut self,
        dest: &crate::mir::VReg,
        lhs: &crate::mir::VReg,
        rhs: &crate::mir::VReg,
        op: RawIntOp,
        func_name: &str,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
    ) {
        use cranelift_codegen::ir::condcodes::IntCC;
        use cranelift_codegen::ir::InstBuilder;

        const PAYLOAD_MASK: i64 = 0x0000_FFFF_FFFF_FFFFi64;
        const TAG_INT_VAL: i64 = 1i64;

        let lv = vars.get(*lhs, builder, cl_types::I64);
        let rv = vars.get(*rhs, builder, cl_types::I64);
        let l = builder.use_var(lv);
        let r = builder.use_var(rv);

        // Fast-path native arithmetic.
        let raw_result = match op {
            RawIntOp::Add => builder.ins().iadd(l, r),
            RawIntOp::Sub => builder.ins().isub(l, r),
            RawIntOp::Mul => builder.ins().imul(l, r),
        };

        // 48-bit signed fits-check: (raw_result << 16) >>s 16 == raw_result.
        let shifted = builder.ins().ishl_imm(raw_result, 16);
        let restored = builder.ins().sshr_imm(shifted, 16);
        let fits_48 = builder.ins().icmp(IntCC::Equal, raw_result, restored);

        // For multiplication, also check native i64 didn't wrap:
        // smulhi(l, r) must equal the sign-extension of the low half.
        let no_overflow = match op {
            RawIntOp::Mul => {
                let hi = builder.ins().smulhi(l, r);
                let sign_lo = builder.ins().sshr_imm(raw_result, 63);
                let no_i64_ovf = builder.ins().icmp(IntCC::Equal, hi, sign_lo);
                builder.ins().band(no_i64_ovf, fits_48)
            }
            _ => fits_48,
        };

        let fast_block = builder.create_block();
        let slow_block = builder.create_block();
        let merge_block = builder.create_block();
        let merged_param = builder.append_block_param(merge_block, cl_types::I64);

        builder
            .ins()
            .brif(no_overflow, fast_block, &[], slow_block, &[]);

        // Fast block: pass native raw_result through.
        builder.switch_to_block(fast_block);
        builder.seal_block(fast_block);
        builder.ins().jump(merge_block, &[raw_result.into()]);

        // Slow block: call mb_bigint_*; select inline-unboxed vs boxed bits.
        builder.switch_to_block(slow_block);
        builder.seal_block(slow_block);
        let slow_value = if let Some(&func_id) = self.extern_funcs.get(func_name) {
            let func_ref = self.module().declare_func_in_func(func_id, builder.func);
            let call = builder.ins().call(func_ref, &[l, r]);
            let result_bits = builder.inst_results(call)[0];

            let tag_raw = builder.ins().ushr_imm(result_bits, 48);
            let tag = builder.ins().band_imm(tag_raw, 7);
            let tag_int_const = builder.ins().iconst(cl_types::I64, TAG_INT_VAL);
            let is_inline = builder.ins().icmp(IntCC::Equal, tag, tag_int_const);

            let pm = builder.ins().iconst(cl_types::I64, PAYLOAD_MASK);
            let result_payload = builder.ins().band(result_bits, pm);
            let shifted2 = builder.ins().ishl_imm(result_payload, 16);
            let unboxed = builder.ins().sshr_imm(shifted2, 16);

            builder.ins().select(is_inline, unboxed, result_bits)
        } else {
            // Runtime missing — fall back to wrapping result (legacy behavior).
            raw_result
        };
        builder.ins().jump(merge_block, &[slow_value.into()]);

        // Merge block: phi the chosen value into dest.
        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);
        let dv = vars.get(*dest, builder, cl_types::I64);
        builder.def_var(dv, merged_param);

        // Keep dest in raw_ints. In the fast path (no overflow) merged_param is a
        // raw INT48; in the slow path the select already unboxes inline returns,
        // and downstream arithmetic guards via runtime fits_48, so correctness is
        // preserved either way. Trade-off: on actual INT48 overflow the slow path
        // returns a NaN-boxed BigInt pointer and subsequent release/retain calls
        // on this VReg are skipped (fire 36 policy), leaking that BigInt. This is
        // acceptable because (a) overflow is rare in hot loops, (b) the leak is
        // bounded by the number of overflowing CheckedOps, and (c) keeping
        // raw_ints status enables fast-path chaining for the 99.99% case.
        vars.raw_ints.insert(*dest);
    }

    /// Unbox an inline-int (tag=1) NaN-boxed `MbValue` result back to a raw
    /// sign-extended i64; pass a BigInt heap pointer (tag=0) through
    /// unchanged. Shared by the bitwise runtime call sites below — factors
    /// the tag-check-and-sign-extend sequence duplicated inline at the
    /// FloorDiv/Mod call site and in `emit_checked_int_op` above (#1090).
    fn unbox_if_inline(
        builder: &mut cranelift_frontend::FunctionBuilder,
        result_bits: cranelift_codegen::ir::Value,
    ) -> cranelift_codegen::ir::Value {
        use cranelift_codegen::ir::InstBuilder;

        const PAYLOAD_MASK: i64 = 0x0000_FFFF_FFFF_FFFFi64;
        const TAG_INT_VAL: i64 = 1i64;

        let tag_raw = builder.ins().ushr_imm(result_bits, 48);
        let tag = builder.ins().band_imm(tag_raw, 7);
        let tag_int_const = builder.ins().iconst(cl_types::I64, TAG_INT_VAL);
        let is_inline = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            tag,
            tag_int_const,
        );
        let pm = builder.ins().iconst(cl_types::I64, PAYLOAD_MASK);
        let result_payload = builder.ins().band(result_bits, pm);
        let shifted = builder.ins().ishl_imm(result_payload, 16);
        let unboxed = builder.ins().sshr_imm(shifted, 16);
        builder.ins().select(is_inline, unboxed, result_bits)
    }

    /// Emit a BigInt-aware bitwise binop (`mb_bitand`/`mb_bitor`/`mb_bitxor`/
    /// `mb_lshift`/`mb_rshift`) for a statically Ty::Int operand pair where
    /// at least one operand may be NaN-boxed (inline int or BigInt heap
    /// pointer) — the slow path reached when the both-operands-provably-raw
    /// fast path in `emit_inst`'s `MirInst::BinOp` arm doesn't apply (#1090).
    ///
    /// Boxes both operands via `mb_box_int` first (a no-op on an already
    /// NaN-boxed value — see `mb_box_int`'s own idempotency guard), calls
    /// `func_name`, then unboxes an inline-int result back to raw i64 so
    /// downstream primitive ops keep working — mirrors the FloorDiv/Mod
    /// box + inline-unbox convention in `emit_inst` above.
    fn emit_checked_bitwise_op(
        &mut self,
        dest: &crate::mir::VReg,
        lhs: &crate::mir::VReg,
        rhs: &crate::mir::VReg,
        func_name: &str,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
    ) {
        use cranelift_codegen::ir::InstBuilder;

        let l = vars.use_as_i64(*lhs, builder);
        let r = vars.use_as_i64(*rhs, builder);
        let box_id = self.extern_funcs.get("mb_box_int").copied();
        let (l_boxed, r_boxed) = if let Some(bid) = box_id {
            let fref = self.module().declare_func_in_func(bid, builder.func);
            let lc = builder.ins().call(fref, &[l]);
            let rc = builder.ins().call(fref, &[r]);
            (builder.inst_results(lc)[0], builder.inst_results(rc)[0])
        } else {
            (l, r)
        };

        if let Some(&func_id) = self.extern_funcs.get(func_name) {
            let func_ref = self.module().declare_func_in_func(func_id, builder.func);
            let call = builder.ins().call(func_ref, &[l_boxed, r_boxed]);
            let result_bits = builder.inst_results(call)[0];
            let result = Self::unbox_if_inline(builder, result_bits);
            vars.def_var_cast(*dest, builder, result, cl_types::I64);
        } else {
            let zero = builder.ins().iconst(cl_types::I64, 0);
            vars.def_var_cast(*dest, builder, zero, cl_types::I64);
        }
    }

    /// Emit `LShift` for a statically Ty::Int, both-operands-raw_ints-tagged
    /// pair with overflow detection, mirroring
    /// `emit_raw_int_op_with_overflow_check`'s CheckedMul-style fast/slow
    /// branch (#1090). A raw INT48 base and a raw (small) shift count can
    /// still promote to BigInt — even `1 << 64` starts from two operands
    /// that individually fit inline — so this needs an actual overflow
    /// check, not just a tag check (unlike AND/OR/XOR/RSHIFT above, which
    /// dropped their raw_ints-gated native fast path entirely: those ops
    /// have no analogous "does the result look wrong" signal to fall back
    /// on if `raw_ints` mistakenly tags a promoted-to-BigInt call-result/
    /// param as raw — see the AND/OR/XOR branch's comment above for the
    /// repro. This overflow check (the `(x<<n)>>n==x` shift-invertibility
    /// identity + the 48-bit fits-check) gives LShift's fast path the same
    /// kind of safety net CheckedAdd/Sub/Mul already rely on for that same
    /// mistagging risk: a garbage/mistagged operand's native shift result
    /// essentially never coincidentally survives both checks, so it
    /// reroutes to the tag-aware runtime slow path below.
    fn emit_raw_lshift_with_overflow_check(
        &mut self,
        dest: &crate::mir::VReg,
        lhs: &crate::mir::VReg,
        rhs: &crate::mir::VReg,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
    ) {
        use cranelift_codegen::ir::condcodes::IntCC;
        use cranelift_codegen::ir::InstBuilder;

        let l = vars.use_as_i64(*lhs, builder);
        let r = vars.use_as_i64(*rhs, builder);

        // Native `ishl`/`sshr` mask their shift-count operand to the
        // register width (hardware SHL/SAR semantics), so a count >= 64 (or
        // negative, which looks huge as unsigned) makes the raw computation
        // below meaningless — route those to the runtime unconditionally.
        let sixty_four = builder.ins().iconst(cl_types::I64, 64);
        let r_in_range = builder.ins().icmp(IntCC::UnsignedLessThan, r, sixty_four);

        let raw_result = builder.ins().ishl(l, r);
        // Overflow check #1 (64-bit): shifting the raw result back right by
        // the same count must recover `l` exactly, or high bits were lost
        // off the top of the 64-bit register — the standard
        // `(x << n) >> n == x` shift-overflow identity, the shift analogue
        // of `emit_raw_int_op_with_overflow_check`'s `smulhi` check.
        let restored = builder.ins().sshr(raw_result, r);
        let no_64_overflow = builder.ins().icmp(IntCC::Equal, restored, l);
        // Overflow check #2 (48-bit inline range): same fits-check
        // CheckedAdd/Sub/Mul use above.
        let shifted16 = builder.ins().ishl_imm(raw_result, 16);
        let restored16 = builder.ins().sshr_imm(shifted16, 16);
        let fits_48 = builder.ins().icmp(IntCC::Equal, raw_result, restored16);
        let no_overflow_1 = builder.ins().band(r_in_range, no_64_overflow);
        let no_overflow = builder.ins().band(no_overflow_1, fits_48);

        let fast_block = builder.create_block();
        let slow_block = builder.create_block();
        let merge_block = builder.create_block();
        let merged_param = builder.append_block_param(merge_block, cl_types::I64);

        builder
            .ins()
            .brif(no_overflow, fast_block, &[], slow_block, &[]);

        // Fast block: pass the native shift result through.
        builder.switch_to_block(fast_block);
        builder.seal_block(fast_block);
        builder.ins().jump(merge_block, &[raw_result.into()]);

        // Slow block: box operands, call `mb_lshift` (handles arbitrary-
        // precision promotion, e.g. `1 << 64` → BigInt 2**64), unbox an
        // inline-int result.
        builder.switch_to_block(slow_block);
        builder.seal_block(slow_block);
        let slow_value = if let Some(&func_id) = self.extern_funcs.get("mb_lshift") {
            let func_ref = self.module().declare_func_in_func(func_id, builder.func);
            let box_id = self.extern_funcs.get("mb_box_int").copied();
            let (l_boxed, r_boxed) = if let Some(bid) = box_id {
                let fref = self.module().declare_func_in_func(bid, builder.func);
                let lc = builder.ins().call(fref, &[l]);
                let rc = builder.ins().call(fref, &[r]);
                (builder.inst_results(lc)[0], builder.inst_results(rc)[0])
            } else {
                (l, r)
            };
            let call = builder.ins().call(func_ref, &[l_boxed, r_boxed]);
            let result_bits = builder.inst_results(call)[0];
            Self::unbox_if_inline(builder, result_bits)
        } else {
            builder.ins().iconst(cl_types::I64, 0)
        };
        builder.ins().jump(merge_block, &[slow_value.into()]);

        // Merge block: phi the chosen value into dest. `dest` is
        // intentionally NOT marked raw_ints — the slow path may return a
        // NaN-boxed BigInt pointer (same convention as CheckedAdd/Sub/Mul).
        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);
        let dv = vars.get(*dest, builder, cl_types::I64);
        builder.def_var(dv, merged_param);
    }

    /// #1131: runtime-tag-tested rich comparison for Int/Bool-typed operands.
    ///
    /// `l`/`r` arrive here as whatever bits lowering already produced for
    /// this BinOp's lhs/rhs (typically the output of `mb_unbox_int_if_boxed`,
    /// which passes genuine small ints and i64-exact BigInts through as a
    /// real raw i64 — but for a BigInt too large even for i64 falls back to
    /// `val.to_bits() as i64`, i.e. the *same* NaN-boxed bit pattern,
    /// unchanged, just relabeled as if it were raw). A native icmp on that
    /// bit pattern is nonsense (e.g. compares pointer/tag bits, or treats a
    /// NaN-boxed value's sign bit as a huge negative number).
    ///
    /// Fast path (native icmp): the `(x << 16) >>s 16 == x` identity — the
    /// same one `mb_box_int`'s Fire-51 inline path and
    /// `emit_raw_lshift_with_overflow_check` use — is true iff `x` is a
    /// genuine sign-extended 48-bit-range i64. It is *never* true for a
    /// NaN-boxed bit pattern of any tag (NAN_PREFIX forces the top 13 bits
    /// to 1, which can only equal 16 copies of bit 47 if tag==7 *and*
    /// payload's MSB is set — Ellipsis's payload is always 0, so this never
    /// happens in practice). So `both_fit` true guarantees neither operand
    /// is secretly NaN-boxed, and a native signed icmp is exact.
    ///
    /// Slow path: box both operands via `mb_box_int_for_compare` (idempotent —
    /// a genuinely raw i64 outside 48 bits gets freshly BigInt-promoted; bits
    /// that are already a valid NaN-boxed pointer/int/bool/None, including the
    /// `mb_unbox_int_if_boxed` fallback case above, are passed through
    /// unchanged) and route through the existing BigInt-aware `mb_eq`/`mb_ne`/
    /// `mb_lt`/`mb_gt`/`mb_le`/`mb_ge` runtime comparators (already
    /// implemented, already registered as externs — only this call-site
    /// wiring was missing).
    ///
    /// Uses the comparison-scoped `mb_box_int_for_compare`, NOT the generic
    /// `mb_box_int` (#1133): `mb_box_int`'s `tag<=4` acceptance test lets a
    /// raw i64 that merely *aliases* TAG_FUNC(4) (e.g. `-2**50`) or
    /// TAG_PTR(0) (e.g. `-2**51`) pass through unchanged instead of being
    /// promoted to BigInt, corrupting the comparison. A comparison operand
    /// can never legitimately be a real function value, so the accepted tag
    /// set is narrowed at this consuming site instead of loosening the
    /// shared `mb_box_int` (which other callers, e.g. #1084's decorator
    /// passthrough, still need the wider set for).
    fn emit_checked_int_compare(
        &mut self,
        dest: &crate::mir::VReg,
        lhs: &crate::mir::VReg,
        rhs: &crate::mir::VReg,
        op: &MirBinOp,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
    ) {
        use cranelift_codegen::ir::condcodes::IntCC;
        use cranelift_codegen::ir::InstBuilder;

        let l = vars.use_as_i64(*lhs, builder);
        let r = vars.use_as_i64(*rhs, builder);

        let l_shifted = builder.ins().ishl_imm(l, 16);
        let l_restored = builder.ins().sshr_imm(l_shifted, 16);
        let l_fits_48 = builder.ins().icmp(IntCC::Equal, l, l_restored);
        let r_shifted = builder.ins().ishl_imm(r, 16);
        let r_restored = builder.ins().sshr_imm(r_shifted, 16);
        let r_fits_48 = builder.ins().icmp(IntCC::Equal, r, r_restored);
        let both_fit = builder.ins().band(l_fits_48, r_fits_48);

        let maybe_fast_block = builder.create_block();
        let fast_block = builder.create_block();
        let slow_block = builder.create_block();
        let merge_block = builder.create_block();
        let merged_param = builder.append_block_param(merge_block, cl_types::I64);

        builder
            .ins()
            .brif(both_fit, maybe_fast_block, &[], slow_block, &[]);

        // Raw i64 cell handles are small positive ints, so they satisfy the
        // native-int fast-path range check above. But Python cell comparison is
        // by contents, not by handle id (#896), so live cell handles must route
        // through the runtime rich comparator.
        builder.switch_to_block(maybe_fast_block);
        builder.seal_block(maybe_fast_block);
        if vars.raw_ints.contains(lhs) && vars.raw_ints.contains(rhs) {
            builder.ins().jump(fast_block, &[]);
        } else if let Some(&cell_id) = self.extern_funcs.get("mb_cell_handle_raw_is_live") {
            let cell_ref = self.module().declare_func_in_func(cell_id, builder.func);
            let lc = builder.ins().call(cell_ref, &[l]);
            let l_cell_raw = builder.inst_results(lc)[0];
            let rc = builder.ins().call(cell_ref, &[r]);
            let r_cell_raw = builder.inst_results(rc)[0];
            let zero = builder.ins().iconst(cl_types::I64, 0);
            let l_is_cell = builder.ins().icmp(IntCC::NotEqual, l_cell_raw, zero);
            let r_is_cell = builder.ins().icmp(IntCC::NotEqual, r_cell_raw, zero);
            let either_cell = builder.ins().bor(l_is_cell, r_is_cell);
            builder
                .ins()
                .brif(either_cell, slow_block, &[], fast_block, &[]);
        } else {
            builder.ins().jump(fast_block, &[]);
        }

        // Fast: neither operand is NaN-boxed — native signed icmp is exact.
        builder.switch_to_block(fast_block);
        builder.seal_block(fast_block);
        let cc = match op {
            MirBinOp::Eq => IntCC::Equal,
            MirBinOp::NotEq => IntCC::NotEqual,
            MirBinOp::Lt => IntCC::SignedLessThan,
            MirBinOp::Gt => IntCC::SignedGreaterThan,
            MirBinOp::LtEq => IntCC::SignedLessThanOrEqual,
            MirBinOp::GtEq => IntCC::SignedGreaterThanOrEqual,
            _ => unreachable!("emit_checked_int_compare only called for rich comparisons"),
        };
        let cmp = builder.ins().icmp(cc, l, r);
        let fast_result = builder.ins().uextend(cl_types::I64, cmp);
        builder.ins().jump(merge_block, &[fast_result.into()]);

        // Slow: box both (idempotent/tag-aware) and call the BigInt-aware
        // runtime comparator; unbox its NaN-boxed bool result (the bool
        // value lives in bit 0, see `MbValue::from_bool`) back to a raw 0/1
        // so downstream consumers (branch terminators, `mb_box_bool`
        // inserted at lowering time for StoreGlobal/print) see the same
        // raw-bool convention as the fast path.
        builder.switch_to_block(slow_block);
        builder.seal_block(slow_block);
        let helper_name = match op {
            MirBinOp::Eq => "mb_eq",
            MirBinOp::NotEq => "mb_ne",
            MirBinOp::Lt => "mb_lt",
            MirBinOp::Gt => "mb_gt",
            MirBinOp::LtEq => "mb_le",
            MirBinOp::GtEq => "mb_ge",
            _ => unreachable!("emit_checked_int_compare only called for rich comparisons"),
        };
        let slow_value = if let (Some(&box_id), Some(&func_id)) = (
            self.extern_funcs.get("mb_box_int_for_compare"),
            self.extern_funcs.get(helper_name),
        ) {
            let box_ref = self.module().declare_func_in_func(box_id, builder.func);
            let lc = builder.ins().call(box_ref, &[l]);
            let l_boxed = builder.inst_results(lc)[0];
            let rc = builder.ins().call(box_ref, &[r]);
            let r_boxed = builder.inst_results(rc)[0];
            let func_ref = self.module().declare_func_in_func(func_id, builder.func);
            let call = builder.ins().call(func_ref, &[l_boxed, r_boxed]);
            let result_bits = builder.inst_results(call)[0];
            builder.ins().band_imm(result_bits, 1)
        } else {
            builder.ins().iconst(cl_types::I64, 0)
        };
        builder.ins().jump(merge_block, &[slow_value.into()]);

        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);
        let dv = vars.get(*dest, builder, cl_types::I64);
        builder.def_var(dv, merged_param);
        vars.native_bools.insert(*dest);
    }

    // ── Object operation FFI calls ──

    fn emit_getattr(
        &mut self,
        dest: &crate::mir::VReg,
        object: &crate::mir::VReg,
        attr: &str,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
        _externs: &[MirExtern],
    ) {
        // mb_getattr(obj, attr_str) -> MbValue
        if let Some(&func_id) = self.extern_funcs.get("mb_getattr") {
            let func_ref = self.module().declare_func_in_func(func_id, builder.func);
            let obj_v = vars.get(*object, builder, cl_types::I64);
            let obj_val = builder.use_var(obj_v);
            // Emit attribute name as an immortal string constant (#1129 R4/R5).
            let ptr = MbObject::new_str_immortal(attr.to_string());
            self.compile_time_objects.push(ptr);
            let attr_str = MbValue::from_ptr(ptr);
            let attr_val = builder
                .ins()
                .iconst(cl_types::I64, attr_str.to_bits() as i64);
            let call = builder.ins().call(func_ref, &[obj_val, attr_val]);
            let result = builder.inst_results(call)[0];
            vars.def_var_cast(*dest, builder, result, cl_types::I64);
        } else {
            let zero = builder.ins().iconst(cl_types::I64, 0);
            vars.def_var_cast(*dest, builder, zero, cl_types::I64);
        }
    }

    fn emit_setattr(
        &mut self,
        object: &crate::mir::VReg,
        attr: &str,
        value: &crate::mir::VReg,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
        _externs: &[MirExtern],
    ) {
        if let Some(&func_id) = self.extern_funcs.get("mb_setattr") {
            let func_ref = self.module().declare_func_in_func(func_id, builder.func);
            let obj_v = vars.get(*object, builder, cl_types::I64);
            let obj_val = builder.use_var(obj_v);
            // Emit attribute name as an immortal string constant (#1129 R4/R5).
            let ptr = MbObject::new_str_immortal(attr.to_string());
            self.compile_time_objects.push(ptr);
            let attr_str = MbValue::from_ptr(ptr);
            let attr_val = builder
                .ins()
                .iconst(cl_types::I64, attr_str.to_bits() as i64);
            let val = vars.use_as_i64(*value, builder);
            builder.ins().call(func_ref, &[obj_val, attr_val, val]);
        }
    }

    fn emit_getitem(
        &mut self,
        dest: &crate::mir::VReg,
        object: &crate::mir::VReg,
        index: &crate::mir::VReg,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
        _externs: &[MirExtern],
    ) {
        // Always use runtime-dispatched getitem for correct list/dict/tuple/str dispatch
        let func_name = "mb_obj_getitem";
        if let Some(&func_id) = self.extern_funcs.get(func_name) {
            let func_ref = self.module().declare_func_in_func(func_id, builder.func);
            let obj_val = vars.use_as_i64(*object, builder);
            let idx_val = vars.use_as_i64(*index, builder);
            let call = builder.ins().call(func_ref, &[obj_val, idx_val]);
            let result = builder.inst_results(call)[0];
            vars.def_var_cast(*dest, builder, result, cl_types::I64);
        } else {
            let zero = builder.ins().iconst(cl_types::I64, 0);
            vars.def_var_cast(*dest, builder, zero, cl_types::I64);
        }
    }

    fn emit_setitem(
        &mut self,
        object: &crate::mir::VReg,
        index: &crate::mir::VReg,
        value: &crate::mir::VReg,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
        _externs: &[MirExtern],
    ) {
        // Always use runtime-dispatched setitem for correct list/dict dispatch
        let func_name = "mb_obj_setitem";
        if let Some(&func_id) = self.extern_funcs.get(func_name) {
            let func_ref = self.module().declare_func_in_func(func_id, builder.func);
            let obj_val = vars.use_as_i64(*object, builder);
            let idx_val = vars.use_as_i64(*index, builder);
            let val = vars.use_as_i64(*value, builder);
            builder.ins().call(func_ref, &[obj_val, idx_val, val]);
        }
    }

    fn literal_can_skip_gc_track(
        literal_escapes: &LiteralEscapeAnalysis,
        vreg: VReg,
        expected_kind: LiteralEscapeKind,
    ) -> bool {
        matches!(
            literal_escapes.get(vreg),
            Some(info)
                if info.kind == expected_kind
                    && info.classification == LiteralEscapeClassification::NonEscaping
        )
    }

    fn fixed_arity_list_ctor_name(len: usize, skip_gc_track: bool) -> Option<&'static str> {
        match (len, skip_gc_track) {
            (1, false) => Some("mb_list_new_1"),
            (2, false) => Some("mb_list_new_2"),
            (3, false) => Some("mb_list_new_3"),
            (4, false) => Some("mb_list_new_4"),
            (5, false) => Some("mb_list_new_5"),
            (6, false) => Some("mb_list_new_6"),
            (7, false) => Some("mb_list_new_7"),
            (8, false) => Some("mb_list_new_8"),
            (9, false) => Some("mb_list_new_9"),
            (10, false) => Some("mb_list_new_10"),
            (1, true) => Some("mb_list_new_1_untracked"),
            (2, true) => Some("mb_list_new_2_untracked"),
            (3, true) => Some("mb_list_new_3_untracked"),
            (4, true) => Some("mb_list_new_4_untracked"),
            (5, true) => Some("mb_list_new_5_untracked"),
            (6, true) => Some("mb_list_new_6_untracked"),
            (7, true) => Some("mb_list_new_7_untracked"),
            (8, true) => Some("mb_list_new_8_untracked"),
            (9, true) => Some("mb_list_new_9_untracked"),
            (10, true) => Some("mb_list_new_10_untracked"),
            _ => None,
        }
    }

    fn emit_make_list(
        &mut self,
        dest: &crate::mir::VReg,
        elements: &[crate::mir::VReg],
        literal_escapes: &LiteralEscapeAnalysis,
        allow_untracked_literals: bool,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
        _externs: &[MirExtern],
    ) {
        // For non-empty literals, pre-size the backing Vec via
        // mb_list_new_with_capacity(N). Avoids 2-3 Vec growths inside the
        // mb_list_append loop on small literals.
        // NaN-box constant: NAN_PREFIX | (TAG_INT << TAG_SHIFT) | N
        //                 = 0xFFF8_0000_0000_0000 | (1 << 48) | N
        //                 = 0xFFF9_0000_0000_0000 | N
        const NAN_INT_PREFIX: u64 = 0xFFF9_0000_0000_0000;
        let n = elements.len();

        // Fast path: small literals (1..=10) collapse into a single FFI call
        // via mb_list_new_N instead of `new_with_capacity` + N appends.
        // n=1 is the hottest case (every method call lowers a 1-element
        // args list). 8 is the AArch64 SysV register-pass limit — past
        // that args spill to the stack but a single FFI dispatch is
        // still cheaper than 1+N. 10 covers the list_sort_builtin shape
        // (`data = [9, 3, 7, 1, 5, 8, 2, 6, 4, 0]`).
        let skip_gc_track = allow_untracked_literals
            && Self::literal_can_skip_gc_track(literal_escapes, *dest, LiteralEscapeKind::List);
        let small_arity_fn = Self::fixed_arity_list_ctor_name(n, skip_gc_track);
        if let Some(fn_name) = small_arity_fn {
            if let Some(&fn_id) = self.extern_funcs.get(fn_name) {
                let fn_ref = self.module().declare_func_in_func(fn_id, builder.func);
                let arg_vals: Vec<_> = elements
                    .iter()
                    .map(|e| vars.use_as_i64(*e, builder))
                    .collect();
                let call = builder.ins().call(fn_ref, &arg_vals);
                let list_val = builder.inst_results(call)[0];
                vars.def_var_cast(*dest, builder, list_val, cl_types::I64);
                return;
            }
        }

        let new_id_opt = if n > 0 {
            self.extern_funcs
                .get(if skip_gc_track {
                    "mb_list_new_with_capacity_untracked"
                } else {
                    "mb_list_new_with_capacity"
                })
                .copied()
                .or_else(|| {
                    self.extern_funcs
                        .get(if skip_gc_track {
                            "mb_list_new_untracked"
                        } else {
                            "mb_list_new"
                        })
                        .copied()
                })
        } else {
            self.extern_funcs
                .get(if skip_gc_track {
                    "mb_list_new_untracked"
                } else {
                    "mb_list_new"
                })
                .copied()
        };
        // The freshly-allocated list has no other references yet, so the
        // RwLock try_write/write fallback in mb_list_append is wasted —
        // mb_list_append_unchecked uses unwrap_unchecked + skips retain
        // for inline scalars. Safe here because the list is private until
        // we publish it via def_var_cast below.
        let append_id_opt = self
            .extern_funcs
            .get("mb_list_append_unchecked")
            .copied()
            .or_else(|| self.extern_funcs.get("mb_list_append").copied());
        if let (Some(new_id), Some(append_id)) = (new_id_opt, append_id_opt) {
            let new_ref = self.module().declare_func_in_func(new_id, builder.func);
            let list_val = if n > 0 {
                let cap_val = builder
                    .ins()
                    .iconst(cl_types::I64, (NAN_INT_PREFIX | (n as u64)) as i64);
                let call = builder.ins().call(new_ref, &[cap_val]);
                builder.inst_results(call)[0]
            } else {
                let call = builder.ins().call(new_ref, &[]);
                builder.inst_results(call)[0]
            };

            let append_ref = self.module().declare_func_in_func(append_id, builder.func);
            for elem in elements {
                let elem_val = vars.use_as_i64(*elem, builder);
                builder.ins().call(append_ref, &[list_val, elem_val]);
            }

            vars.def_var_cast(*dest, builder, list_val, cl_types::I64);
        } else {
            let zero = builder.ins().iconst(cl_types::I64, 0);
            vars.def_var_cast(*dest, builder, zero, cl_types::I64);
        }
    }

    fn emit_make_dict(
        &mut self,
        dest: &crate::mir::VReg,
        keys: &[crate::mir::VReg],
        values: &[crate::mir::VReg],
        literal_escapes: &LiteralEscapeAnalysis,
        allow_untracked_literals: bool,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
        _externs: &[MirExtern],
    ) {
        let ctor_name = if allow_untracked_literals
            && Self::literal_can_skip_gc_track(literal_escapes, *dest, LiteralEscapeKind::Dict)
        {
            "mb_dict_new_untracked"
        } else {
            "mb_dict_new"
        };
        if let (Some(&new_id), Some(&set_id)) = (
            self.extern_funcs.get(ctor_name),
            self.extern_funcs.get("mb_dict_setitem"),
        ) {
            let new_ref = self.module().declare_func_in_func(new_id, builder.func);
            let call = builder.ins().call(new_ref, &[]);
            let dict_val = builder.inst_results(call)[0];

            let set_ref = self.module().declare_func_in_func(set_id, builder.func);
            for (k, v) in keys.iter().zip(values.iter()) {
                let key_val = vars.use_as_i64(*k, builder);
                let val_val = vars.use_as_i64(*v, builder);
                builder.ins().call(set_ref, &[dict_val, key_val, val_val]);
            }

            vars.def_var_cast(*dest, builder, dict_val, cl_types::I64);
        } else {
            let zero = builder.ins().iconst(cl_types::I64, 0);
            vars.def_var_cast(*dest, builder, zero, cl_types::I64);
        }
    }

    fn emit_make_tuple(
        &mut self,
        dest: &crate::mir::VReg,
        elements: &[crate::mir::VReg],
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
        _externs: &[MirExtern],
    ) {
        // Build as list (pre-sized when N > 0), then convert to tuple.
        // See emit_make_list for the NaN-boxed int constant rationale.
        const NAN_INT_PREFIX: u64 = 0xFFF9_0000_0000_0000;
        let n = elements.len();

        // Fast path (#2128): small tuples (1..=8) collapse into a single
        // FFI call via mb_tuple_new_N. The previous list+convert path
        // gc_track'd the intermediate List, dominating runtime for
        // primitive-tuple returns and producing the ~150-220x penalty
        // vs CPython documented on the issue. mb_tuple_new_N allocates
        // the tuple directly; new_tuple already elides gc_track when
        // every element is non-cycle-capable, matching the new_complex
        // / new_bytes contract called out by the issue's "suggested fix".
        // 8 = AArch64 SysV register-pass limit (same threshold as the
        // list fast path).
        let small_arity_fn = match n {
            1 => Some("mb_tuple_new_1"),
            2 => Some("mb_tuple_new_2"),
            3 => Some("mb_tuple_new_3"),
            4 => Some("mb_tuple_new_4"),
            5 => Some("mb_tuple_new_5"),
            6 => Some("mb_tuple_new_6"),
            7 => Some("mb_tuple_new_7"),
            8 => Some("mb_tuple_new_8"),
            _ => None,
        };
        if let Some(fn_name) = small_arity_fn {
            if let Some(&fn_id) = self.extern_funcs.get(fn_name) {
                let fn_ref = self.module().declare_func_in_func(fn_id, builder.func);
                let arg_vals: Vec<_> = elements
                    .iter()
                    .map(|e| vars.use_as_i64(*e, builder))
                    .collect();
                let call = builder.ins().call(fn_ref, &arg_vals);
                let tuple_val = builder.inst_results(call)[0];
                vars.def_var_cast(*dest, builder, tuple_val, cl_types::I64);
                return;
            }
        }

        // n == 0: direct empty-tuple allocator — also bypasses the
        // intermediate list (#2128). new_tuple(Vec::new()) sees no
        // cycle-capable elements and skips gc_track.
        if n == 0 {
            if let Some(&fn_id) = self.extern_funcs.get("mb_tuple_new") {
                let fn_ref = self.module().declare_func_in_func(fn_id, builder.func);
                let call = builder.ins().call(fn_ref, &[]);
                let tuple_val = builder.inst_results(call)[0];
                vars.def_var_cast(*dest, builder, tuple_val, cl_types::I64);
                return;
            }
        }

        let new_id = if n > 0 {
            self.extern_funcs
                .get("mb_list_new_with_capacity")
                .copied()
                .or_else(|| self.extern_funcs.get("mb_list_new").copied())
        } else {
            self.extern_funcs.get("mb_list_new").copied()
        };
        // Same private-list rationale as emit_make_list — the intermediate
        // list never escapes to user code.
        let append_id = self
            .extern_funcs
            .get("mb_list_append_unchecked")
            .copied()
            .or_else(|| self.extern_funcs.get("mb_list_append").copied());
        let convert_id = self.extern_funcs.get("mb_list_to_tuple").copied();
        if let (Some(new_id), Some(append_id), Some(convert_id)) = (new_id, append_id, convert_id) {
            let new_ref = self.module().declare_func_in_func(new_id, builder.func);
            let list_val = if n > 0 {
                let cap_val = builder
                    .ins()
                    .iconst(cl_types::I64, (NAN_INT_PREFIX | (n as u64)) as i64);
                let call = builder.ins().call(new_ref, &[cap_val]);
                builder.inst_results(call)[0]
            } else {
                let call = builder.ins().call(new_ref, &[]);
                builder.inst_results(call)[0]
            };
            let app_ref = self.module().declare_func_in_func(append_id, builder.func);
            for elem in elements {
                let elem_val = vars.use_as_i64(*elem, builder);
                builder.ins().call(app_ref, &[list_val, elem_val]);
            }
            let conv_ref = self.module().declare_func_in_func(convert_id, builder.func);
            let conv_call = builder.ins().call(conv_ref, &[list_val]);
            let tuple_val = builder.inst_results(conv_call)[0];
            vars.def_var_cast(*dest, builder, tuple_val, cl_types::I64);
        } else {
            let zero = builder.ins().iconst(cl_types::I64, 0);
            vars.def_var_cast(*dest, builder, zero, cl_types::I64);
        }
    }

    fn emit_inline_box_bool(
        builder: &mut cranelift_frontend::FunctionBuilder,
        raw: cranelift_codegen::ir::Value,
    ) -> cranelift_codegen::ir::Value {
        use cranelift_codegen::ir::condcodes::IntCC;
        use cranelift_codegen::ir::InstBuilder;

        const NAN_PREFIX_MASK: i64 = 0xFFF8_0000_0000_0000_u64 as i64;
        const TAG_MASK: i64 = 0x0007_0000_0000_0000;
        const BOOL_TAG_BITS: i64 = 0x0002_0000_0000_0000;
        const BOX_BOOL_FALSE: i64 = 0xFFFA_0000_0000_0000_u64 as i64;

        let prefix_bits = builder.ins().band_imm(raw, NAN_PREFIX_MASK);
        let has_prefix = builder
            .ins()
            .icmp_imm(IntCC::Equal, prefix_bits, NAN_PREFIX_MASK);
        let tag_bits = builder.ins().band_imm(raw, TAG_MASK);
        let is_bool_tag = builder
            .ins()
            .icmp_imm(IntCC::Equal, tag_bits, BOOL_TAG_BITS);
        let is_boxed_bool = builder.ins().band(has_prefix, is_bool_tag);
        let truthy = builder.ins().icmp_imm(IntCC::NotEqual, raw, 0);
        let truthy_i64 = builder.ins().uextend(cl_types::I64, truthy);
        let boxed = builder.ins().bor_imm(truthy_i64, BOX_BOOL_FALSE);
        builder.ins().select(is_boxed_bool, raw, boxed)
    }

    fn emit_internal_call(
        &mut self,
        dest: &Option<crate::mir::VReg>,
        sym_id: u32,
        args: &[crate::mir::VReg],
        ty: &crate::types::ty::TypeId,
        tcx: &TypeContext,
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
    ) {
        if let Some(&callee_id) = self.internal_funcs.get(&sym_id) {
            let func_ref = self.module().declare_func_in_func(callee_id, builder.func);
            // Bitcast F64 args to I64 — internal functions use I64 ABI for all params.
            let mut arg_vals: Vec<_> = args.iter().map(|a| vars.use_as_i64(*a, builder)).collect();
            // #1696 arity guard: reshape `arg_vals` to match the declared
            // signature so a call site whose `MirInst::Call { args }` length
            // diverges from `body.params.len()` no longer trips the Cranelift
            // verifier (`mismatched argument count for v? = call fnN(...)
            // got K, expected N`). Truncate on over-arity, pad with
            // NaN-boxed None (`iconst.i64 0`) on under-arity. See the
            // `internal_param_counts` field docs for the why.
            if let Some(&declared) = self.internal_param_counts.get(&sym_id) {
                if arg_vals.len() > declared {
                    arg_vals.truncate(declared);
                } else {
                    while arg_vals.len() < declared {
                        let pad = builder.ins().iconst(cl_types::I64, 0);
                        arg_vals.push(pad);
                    }
                }
            }
            let call = builder.ins().call(func_ref, &arg_vals);
            if let Some(dest_vreg) = dest {
                let cl_type = Self::mamba_to_cl_type(tcx.get(*ty));
                let actual_dest_type = vars.declared_type(*dest_vreg).unwrap_or(cl_type);
                let var = vars.get(*dest_vreg, builder, actual_dest_type);
                let result = builder.inst_results(call)[0];
                // NaN-box the result when the callee has a primitive return type but
                // the call-site expects a non-primitive (Dynamic/Any) value.
                let boxed = if let Some(&callee_ty_id) = self.internal_return_tys.get(&sym_id) {
                    let callee_ty = tcx.get(callee_ty_id);
                    let callee_native_bool = self.internal_native_bool_returns.contains(&sym_id);
                    let callsite_ty = tcx.get(*ty);
                    let callee_is_bool = callee_native_bool || matches!(callee_ty, Ty::Bool);
                    let callee_is_primitive =
                        callee_is_bool || matches!(callee_ty, Ty::Int | Ty::Float);
                    let callsite_is_nonprimitive =
                        !matches!(callsite_ty, Ty::Int | Ty::Bool | Ty::Float);
                    if callee_is_primitive && callsite_is_nonprimitive {
                        // Float: already NaN-boxed as I64 (= MbValue), no boxing needed.
                        // Int/Bool: raw value in I64, needs boxing to MbValue.
                        if matches!(callee_ty, Ty::Float) {
                            result
                        } else if callee_is_bool {
                            Self::emit_inline_box_bool(builder, result)
                        } else {
                            let box_fn_name = "mb_box_int";
                            if let Some(&box_func_id) = self.extern_funcs.get(box_fn_name) {
                                let box_ref = self
                                    .module()
                                    .declare_func_in_func(box_func_id, builder.func);
                                let box_call = builder.ins().call(box_ref, &[result]);
                                builder.inst_results(box_call)[0]
                            } else {
                                result
                            }
                        }
                    } else {
                        result
                    }
                } else {
                    result
                };
                // Bitcast I64 result to F64 if dest variable is F64
                if actual_dest_type == cl_types::F64 && actual_dest_type != cl_type {
                    let cast = builder.ins().bitcast(cl_types::F64, MemFlags::new(), boxed);
                    builder.def_var(var, cast);
                } else {
                    builder.def_var(var, boxed);
                }
                // Propagate raw_ints when callee returns Int/Bool AND call-site
                // type is also Int/Bool — `boxed = result` (raw i64), no
                // mb_box_int wrap was applied. Lets recursive callers of
                // typed-int functions feed the result directly into the
                // CheckedAdd/Sub/Mul fast path without re-unboxing.
                if let Some(&callee_ty_id) = self.internal_return_tys.get(&sym_id) {
                    let callee_ty = tcx.get(callee_ty_id);
                    let callee_native_bool = self.internal_native_bool_returns.contains(&sym_id);
                    let callee_is_bool = callee_native_bool || matches!(callee_ty, Ty::Bool);
                    let callsite_ty = tcx.get(*ty);
                    if callee_is_bool && matches!(callsite_ty, Ty::Int | Ty::Bool) {
                        vars.raw_ints.insert(*dest_vreg);
                        vars.native_bools.insert(*dest_vreg);
                    } else if matches!(callee_ty, Ty::Int)
                        && matches!(callsite_ty, Ty::Int | Ty::Bool)
                    {
                        vars.raw_ints.insert(*dest_vreg);
                    }
                }
            }
        } else if let Some(dest_vreg) = dest {
            let cl_type = Self::mamba_to_cl_type(tcx.get(*ty));
            let actual_dest_type = vars.declared_type(*dest_vreg).unwrap_or(cl_type);
            let var = vars.get(*dest_vreg, builder, actual_dest_type);
            let zero = builder.ins().iconst(cl_types::I64, 0);
            if actual_dest_type == cl_types::F64 {
                let cast = builder.ins().bitcast(cl_types::F64, MemFlags::new(), zero);
                builder.def_var(var, cast);
            } else {
                builder.def_var(var, zero);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_extern_call(
        &mut self,
        dest: &Option<crate::mir::VReg>,
        name: &str,
        args: &[crate::mir::VReg],
        ty: &crate::types::ty::TypeId,
        tcx: &TypeContext,
        externs: &[MirExtern],
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
    ) {
        // Lever A v2: inline `mb_is_stop_iter` as `icmp_imm eq <SENTINEL_BITS>`
        // — saves one FFI thunk per yield in for-loop / comprehension lowering.
        // The dest is marked native_bool so the Branch terminator consumes the
        // i8 result directly without band_imm. Sentinel bits computed from
        // value.rs::TAG_STOP_ITER=6: NAN_PREFIX(0xFFF8…) | (6 << 48) =
        // 0xFFFE_0000_0000_0000. `mb_is_stop_iter` remains registered as a
        // runtime symbol so non-JIT paths (AOT, debug) still link.
        //
        // #1073 bounded slice: inline `mb_box_bool` in the common compare /
        // truthiness boxing path. Preserve `mb_box_bool`'s one special case
        // (already-boxed bool stays bit-identical) and otherwise synthesize
        // `MbValue::from_bool(raw != 0)` directly in IR. This removes one
        // extern thunk from typed-bool producer chains without changing the
        // broader call ABI.
        if name == "mb_box_bool" && args.len() == 1 {
            if let Some(dest_vreg) = dest {
                let raw = vars.use_as_i64(args[0], builder);
                let result = Self::emit_inline_box_bool(builder, raw);

                vars.raw_ints.remove(dest_vreg);
                vars.native_bools.remove(dest_vreg);
                vars.def_var_cast(*dest_vreg, builder, result, cl_types::I64);
                return;
            }
        }

        // Fire 51: inline `mb_box_int` for raw_int args. The fast path is a
        // single bor: when the arg is a genuine raw INT48 (no NAN_PREFIX
        // set), boxing is `(arg & PAYLOAD_MASK) | (NAN_PREFIX | TAG_INT<<48)`
        // which is just two and/or-imm pairs — no FFI thunk. Branches to
        // the FFI thunk only when NAN_PREFIX is already set on the input
        // (rare: only when a CheckedOp's overflow path left NaN-boxed
        // BigInt bits in this VReg via emit_raw_int_op_with_overflow_check's
        // inline-unbox select). Saves ~10-20 ns per yield in generator
        // bodies that yield typed-int locals.
        if name == "mb_box_int" && args.len() == 1 {
            if let Some(dest_vreg) = dest {
                if vars.native_bools.contains(&args[0]) {
                    let raw = vars.use_as_i64(args[0], builder);
                    let result = Self::emit_inline_box_bool(builder, raw);
                    vars.raw_ints.remove(dest_vreg);
                    vars.native_bools.remove(dest_vreg);
                    vars.def_var_cast(*dest_vreg, builder, result, cl_types::I64);
                    return;
                }
                if vars.raw_ints.contains(&args[0]) {
                    use cranelift_codegen::ir::condcodes::IntCC;
                    use cranelift_codegen::ir::InstBuilder;
                    const PAYLOAD_MASK: i64 = 0x0000_FFFF_FFFF_FFFFi64;
                    // NAN_PREFIX | (TAG_INT(=1) << TAG_SHIFT(=48))
                    const BOX_INT_TEMPLATE: i64 = 0xFFF9_0000_0000_0000_u64 as i64;

                    let av = vars.get(args[0], builder, cl_types::I64);
                    let arg_val = builder.use_var(av);

                    // Single combined check: fits in INT48 (signed). Pattern
                    // `(x << 16) >>s 16 == x` succeeds iff the high 16 bits
                    // are sign-extension of bit 47, which simultaneously
                    // rejects (a) NaN-boxed values (high bits 0xFFF8) and
                    // (b) raw i64 values outside ±2^47 (where mb_box_int
                    // would BigInt-promote — failing fits-48 here is what
                    // a regression in `1 << 62 == 0` taught us). Roughly
                    // matches the fits_48 pattern in
                    // emit_raw_int_op_with_overflow_check.
                    let shifted = builder.ins().ishl_imm(arg_val, 16);
                    let restored = builder.ins().sshr_imm(shifted, 16);
                    let fits_48 = builder.ins().icmp(IntCC::Equal, arg_val, restored);

                    let fast_block = builder.create_block();
                    let slow_block = builder.create_block();
                    let merge_block = builder.create_block();
                    let merged_param = builder.append_block_param(merge_block, cl_types::I64);

                    builder
                        .ins()
                        .brif(fits_48, fast_block, &[], slow_block, &[]);

                    // Fast: raw INT48 → format inline.
                    builder.switch_to_block(fast_block);
                    builder.seal_block(fast_block);
                    let payload = builder.ins().band_imm(arg_val, PAYLOAD_MASK);
                    let boxed = builder.ins().bor_imm(payload, BOX_INT_TEMPLATE);
                    builder.ins().jump(merge_block, &[boxed.into()]);

                    // Slow: NaN-boxed BigInt input — call mb_box_int FFI which
                    // does retain_if_ptr and pass-through.
                    builder.switch_to_block(slow_block);
                    builder.seal_block(slow_block);
                    let slow_result = if let Some(&func_id) = self.extern_funcs.get("mb_box_int") {
                        let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                        let call = builder.ins().call(func_ref, &[arg_val]);
                        builder.inst_results(call)[0]
                    } else {
                        arg_val
                    };
                    builder.ins().jump(merge_block, &[slow_result.into()]);

                    builder.switch_to_block(merge_block);
                    builder.seal_block(merge_block);
                    let dv = vars.get(*dest_vreg, builder, cl_types::I64);
                    builder.def_var(dv, merged_param);
                    return;
                }
            }
        }

        // Inline-only integer unbox: tag=1 boxed ints become raw i64; raw
        // i64 values and boxed BigInt bits pass through unchanged. This is
        // used by typed await/result paths that feed CheckedAdd/Sub/Mul,
        // whose BigInt-aware ABI can consume boxed BigInt bits on overflow
        // fallback. Do not apply this to mb_unbox_int_if_boxed; that helper
        // must extract i64-fitting BigInts for comparison and entry-return
        // semantics.
        if name == "mb_unbox_inline_int_if_boxed" && args.len() == 1 {
            if let Some(dest_vreg) = dest {
                let actual_type = vars.declared_type(args[0]).unwrap_or(cl_types::I64);
                let av = vars.get(args[0], builder, actual_type);
                let arg = builder.use_var(av);
                let bits = if actual_type == cl_types::F64 {
                    builder.ins().bitcast(cl_types::I64, MemFlags::new(), arg)
                } else {
                    arg
                };
                let result = Self::unbox_if_inline(builder, bits);
                let dv = vars.get(*dest_vreg, builder, cl_types::I64);
                builder.def_var(dv, result);
                vars.raw_ints.insert(*dest_vreg);
                vars.native_bools.remove(dest_vreg);
                return;
            }
        }

        // #1010: inline the recursion-depth guard's fast path.
        // `mb_recursion_enter` is emitted once at the top of every function
        // body (hir_to_mir's prologue) and, together with the
        // `mb_has_exception` check that used to follow it, was the dominant
        // remaining per-call overhead after #959's rc-elision work. Fetch
        // this thread's depth/limit cell addresses via two trivial
        // leaf-function calls (no branching, no closures — see
        // `mb_recursion_depth_ptr`/`mb_recursion_limit_ptr`), then do the
        // actual load+increment+compare inline; only fall back to the real
        // `mb_recursion_enter` FFI call (unchanged, so the raise — exact
        // message and limit value — stays byte-identical) on the rare,
        // near-the-limit slow path. Dest is marked native_bool/raw_int so
        // downstream consumers (the exception-propagate branch this feeds)
        // read the 0/1 result directly.
        if name == "mb_recursion_enter" && args.is_empty() {
            if let (Some(dest_vreg), Some(&depth_ptr_id), Some(&limit_ptr_id), Some(&enter_id)) = (
                dest,
                self.extern_funcs.get("mb_recursion_depth_ptr"),
                self.extern_funcs.get("mb_recursion_limit_ptr"),
                self.extern_funcs.get("mb_recursion_enter"),
            ) {
                use cranelift_codegen::ir::condcodes::IntCC;
                use cranelift_codegen::ir::InstBuilder;

                let depth_ptr_ref = self
                    .module()
                    .declare_func_in_func(depth_ptr_id, builder.func);
                let depth_ptr_call = builder.ins().call(depth_ptr_ref, &[]);
                let depth_ptr = builder.inst_results(depth_ptr_call)[0];
                vars.recursion_depth_ptr = Some(depth_ptr);

                let limit_ptr_ref = self
                    .module()
                    .declare_func_in_func(limit_ptr_id, builder.func);
                let limit_ptr_call = builder.ins().call(limit_ptr_ref, &[]);
                let limit_ptr = builder.inst_results(limit_ptr_call)[0];

                let mem_flags = MemFlags::trusted();
                let current = builder.ins().load(cl_types::I64, mem_flags, depth_ptr, 0);
                let limit = builder.ins().load(cl_types::I64, mem_flags, limit_ptr, 0);
                let next = builder.ins().iadd_imm(current, 1);
                let exceeded = builder.ins().icmp(IntCC::SignedGreaterThan, next, limit);

                let fast_block = builder.create_block();
                let slow_block = builder.create_block();
                let merge_block = builder.create_block();
                let merge_param = builder.append_block_param(merge_block, cl_types::I8);

                builder
                    .ins()
                    .brif(exceeded, slow_block, &[], fast_block, &[]);

                // Fast: depth has headroom — commit the increment inline
                // and report ok. This is the overwhelming common case; it
                // never leaves JIT-generated code beyond the two pointer
                // fetches above.
                builder.switch_to_block(fast_block);
                builder.seal_block(fast_block);
                builder.ins().store(mem_flags, next, depth_ptr, 0);
                let ok_true = builder.ins().iconst(cl_types::I8, 1);
                builder.ins().jump(merge_block, &[ok_true.into()]);

                // Slow: would exceed the limit — defer entirely to
                // `mb_recursion_enter` itself. We never wrote `next` on
                // this path, so there is no double count; it reloads
                // current/limit fresh, finds the same over-limit condition,
                // raises RecursionError exactly as before, and returns the
                // (false) ok flag.
                builder.switch_to_block(slow_block);
                builder.seal_block(slow_block);
                let enter_ref = self.module().declare_func_in_func(enter_id, builder.func);
                let slow_call = builder.ins().call(enter_ref, &[]);
                let slow_raw = builder.inst_results(slow_call)[0];
                let slow_bit = builder.ins().band_imm(slow_raw, 1);
                let slow_ok = builder.ins().ireduce(cl_types::I8, slow_bit);
                builder.ins().jump(merge_block, &[slow_ok.into()]);

                builder.switch_to_block(merge_block);
                builder.seal_block(merge_block);
                let dv = vars.get(*dest_vreg, builder, cl_types::I8);
                builder.def_var(dv, merge_param);
                vars.raw_ints.insert(*dest_vreg);
                vars.native_bools.insert(*dest_vreg);
                return;
            }
        }

        // #1073 / #1437: mirror the safe part of the recursion-leave fast
        // path inline. Fetch this thread's TLS depth-cell address at runtime
        // via `mb_recursion_depth_ptr`, then store `current.saturating_sub(1)`
        // back.
        // Keep the generic extern-call path as the fallback when the helper
        // symbol is unavailable or the MIR shape differs.
        if name == "mb_recursion_leave" && args.is_empty() {
            if let Some(depth_ptr) = vars.recursion_depth_ptr {
                use cranelift_codegen::ir::condcodes::IntCC;
                use cranelift_codegen::ir::InstBuilder;

                let mem_flags = MemFlags::trusted();
                let current = builder.ins().load(cl_types::I64, mem_flags, depth_ptr, 0);
                let decremented = builder.ins().iadd_imm(current, -1);
                let is_i64_min = builder.ins().icmp_imm(IntCC::Equal, current, i64::MIN);
                let next = builder.ins().select(is_i64_min, current, decremented);
                builder.ins().store(mem_flags, next, depth_ptr, 0);
                return;
            } else if let Some(&depth_ptr_id) = self.extern_funcs.get("mb_recursion_depth_ptr") {
                use cranelift_codegen::ir::condcodes::IntCC;
                use cranelift_codegen::ir::InstBuilder;

                let depth_ptr_ref = self
                    .module()
                    .declare_func_in_func(depth_ptr_id, builder.func);
                let depth_ptr_call = builder.ins().call(depth_ptr_ref, &[]);
                let depth_ptr = builder.inst_results(depth_ptr_call)[0];

                let mem_flags = MemFlags::trusted();
                let current = builder.ins().load(cl_types::I64, mem_flags, depth_ptr, 0);
                let decremented = builder.ins().iadd_imm(current, -1);
                let is_i64_min = builder.ins().icmp_imm(IntCC::Equal, current, i64::MIN);
                let next = builder.ins().select(is_i64_min, current, decremented);
                builder.ins().store(mem_flags, next, depth_ptr, 0);
                return;
            }
        }

        if name == "mb_is_stop_iter" && args.len() == 1 {
            if let Some(dest_vreg) = dest {
                const SENTINEL_BITS: i64 = 0xFFFE_0000_0000_0000_u64 as i64;
                let actual_type = vars.declared_type(args[0]).unwrap_or(cl_types::I64);
                let av = vars.get(args[0], builder, actual_type);
                let raw = builder.use_var(av);
                let arg = if actual_type == cl_types::F64 {
                    builder.ins().bitcast(cl_types::I64, MemFlags::new(), raw)
                } else {
                    raw
                };
                let is_eq = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    arg,
                    SENTINEL_BITS,
                );
                let dv = vars.get(*dest_vreg, builder, cl_types::I8);
                builder.def_var(dv, is_eq);
                vars.native_bools.insert(*dest_vreg);
                vars.raw_ints.insert(*dest_vreg);
                return;
            }
        }
        let ext = externs.iter().find(|e| e.name == name);
        if let Some(&func_id) = self.extern_funcs.get(name) {
            let func_ref = self.module().declare_func_in_func(func_id, builder.func);
            let mut arg_vals: Vec<_> = args
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    // Load the variable with its actual declared type, then marshal
                    // to the extern's expected param type. This handles F64→I64 bitcast
                    // when a float VReg is passed to a runtime function expecting MbValue.
                    let actual_type = vars.declared_type(*a).unwrap_or(cl_types::I64);
                    let v = vars.get(*a, builder, actual_type);
                    let val = builder.use_var(v);
                    if let Some(ext) = ext {
                        if i < ext.params.len() {
                            return marshal::marshal_arg(builder, val, actual_type, &ext.params[i]);
                        }
                    }
                    // No extern info — if F64, bitcast to I64 (safe default for MbValue)
                    if actual_type == cl_types::F64 {
                        return builder.ins().bitcast(cl_types::I64, MemFlags::new(), val);
                    }
                    val
                })
                .collect();
            // #1696 / #2098 arity guard: the extern thunk's Cranelift
            // signature was registered with `ext.params.len()` AbiParams
            // in `declare_extern`. A MIR `CallExtern { args }` whose length
            // diverges from that would emit a mismatched-arity
            // `call fnN(...)` that the verifier rejects with
            // `mismatched argument count for v? = call fnN(...): got K,
            // expected N` (#1696: 3-arg site vs 2-arg sig in cpython
            // test_bool; #2098: 5-arg site vs 1-arg sig in
            // `assertRaises(struct.error, struct.calcsize, 'Z')`).
            //
            // The declared count is sourced from `ext.params.len()` when
            // the current MIR pass's externs slice carries the entry; the
            // #2098 fingerprint surfaced because that slice is per-pass
            // and is empty for externs reused across passes. Fall back to
            // `self.extern_param_counts`, which is keyed at
            // `declare_extern` time and survives across passes.
            //
            // Reshape `arg_vals` so the call always matches the registered
            // sig: truncate on over-arity, zero-pad with NaN-boxed None on
            // under-arity. Conservative — the call may produce a wrong
            // runtime value, but the JIT module is no longer aborted by
            // the verifier so downstream code continues compiling.
            let declared = ext
                .map(|e| e.params.len())
                .or_else(|| self.extern_param_counts.get(name).copied());
            if let Some(declared) = declared {
                if arg_vals.len() > declared {
                    arg_vals.truncate(declared);
                } else {
                    while arg_vals.len() < declared {
                        let pad = builder.ins().iconst(cl_types::I64, 0);
                        arg_vals.push(pad);
                    }
                }
            }
            let call = builder.ins().call(func_ref, &arg_vals);
            if let Some(dest_vreg) = dest {
                vars.raw_ints.remove(dest_vreg);
                vars.native_bools.remove(dest_vreg);
                let cl_type = Self::mamba_to_cl_type(tcx.get(*ty));
                // Use actual declared type if variable already exists (may be F64 from earlier assignment)
                let actual_dest_type = vars.declared_type(*dest_vreg).unwrap_or(cl_type);
                let var = vars.get(*dest_vreg, builder, actual_dest_type);
                if let Some(ext) = ext {
                    if ext.return_type != MirType::Void {
                        let raw = builder.inst_results(call)[0];
                        let val = marshal::unmarshal_return(
                            builder,
                            raw,
                            &ext.return_type,
                            actual_dest_type,
                        );
                        builder.def_var(var, val);
                    } else {
                        let none_bits = builder
                            .ins()
                            .iconst(cl_types::I64, MbValue::none().to_bits() as i64);
                        if actual_dest_type == cl_types::F64 {
                            let cast =
                                builder
                                    .ins()
                                    .bitcast(cl_types::F64, MemFlags::new(), none_bits);
                            builder.def_var(var, cast);
                        } else {
                            builder.def_var(var, none_bits);
                        }
                    }
                } else {
                    let none_bits = builder
                        .ins()
                        .iconst(cl_types::I64, MbValue::none().to_bits() as i64);
                    if actual_dest_type == cl_types::F64 {
                        let cast = builder
                            .ins()
                            .bitcast(cl_types::F64, MemFlags::new(), none_bits);
                        builder.def_var(var, cast);
                    } else {
                        builder.def_var(var, none_bits);
                    }
                }
                if name == "mb_unbox_int_if_boxed" || name == "mb_unbox_bool_if_boxed" {
                    vars.raw_ints.insert(*dest_vreg);
                    if name == "mb_unbox_bool_if_boxed" {
                        vars.native_bools.insert(*dest_vreg);
                    }
                }
            }
        } else if let Some(dest_vreg) = dest {
            vars.raw_ints.remove(dest_vreg);
            vars.native_bools.remove(dest_vreg);
            let cl_type = Self::mamba_to_cl_type(tcx.get(*ty));
            let actual_dest_type = vars.declared_type(*dest_vreg).unwrap_or(cl_type);
            let var = vars.get(*dest_vreg, builder, actual_dest_type);
            let zero = builder.ins().iconst(cl_types::I64, 0);
            if actual_dest_type == cl_types::F64 {
                let cast = builder.ins().bitcast(cl_types::F64, MemFlags::new(), zero);
                builder.def_var(var, cast);
            } else {
                builder.def_var(var, zero);
            }
        }
    }
}

impl CodegenBackend for CraneliftJitBackend {
    fn codegen(
        &mut self,
        module: &MirModule,
        tcx: &TypeContext,
    ) -> crate::error::Result<CodegenOutput> {
        // Merge user externs with runtime externs
        let rt_externs = runtime_externs();
        let all_externs: Vec<MirExtern> = module
            .externs
            .iter()
            .chain(rt_externs.iter())
            .cloned()
            .collect();

        // Phase 1: Declare all extern functions
        for ext in &all_externs {
            self.declare_extern(ext)?;
        }
        // Phase 2: Forward-declare all internal functions
        for body in &module.bodies {
            self.declare_internal(body, tcx)?;
        }
        // Phase 3: Compile function bodies
        for body in &module.bodies {
            self.compile_function(body, tcx, &all_externs)?;
        }

        // Finalize — commit code to executable memory
        let jit_module = self.module.as_mut().expect("module already consumed");
        jit_module
            .finalize_definitions()
            .map_err(|e| crate::error::MambaError::codegen(format!("finalize: {e}")))?;

        // Register variadic function addresses so mb_call_spread can detect them.
        // A body is variadic if its SymbolId was registered by the lowerer (has_star_args=true).
        // #2094: also emit /tmp/perf-<pid>.map records when MAMBA_PERF_MAP=1
        // so samply / Instruments / linux perf can resolve JIT'd frames by
        // name instead of bare hex.
        let perf_map_on = perf_map::is_enabled();
        for body in &module.bodies {
            if let Some(&func_id) = self.internal_funcs.get(&body.name.0) {
                let ptr = jit_module.get_finalized_function(func_id);
                if crate::runtime::module::is_variadic_symbol(body.name.0) {
                    crate::runtime::module::register_variadic_func(ptr as u64);
                }
                if crate::runtime::module::is_kwargs_symbol(body.name.0) {
                    crate::runtime::module::register_kwargs_func(ptr as u64);
                }
                if crate::runtime::module::is_boxed_return_symbol(body.name.0) {
                    crate::runtime::module::register_boxed_return_func(ptr as u64);
                }
                if perf_map_on {
                    let size = self
                        .internal_code_sizes
                        .get(&body.name.0)
                        .copied()
                        .unwrap_or(0) as usize;
                    // Symbol name mirrors the cranelift declared name
                    // (`declare_internal`): `_mb_<symbol-id>`. The MIR layer
                    // does not retain source identifiers down to this point,
                    // so the id is the most stable handle; profiler output
                    // shows distinct frames per Mamba function regardless.
                    let sym = if body.name.0 == u32::MAX {
                        "_mb_main".to_string()
                    } else {
                        format!("_mb_{}", body.name.0)
                    };
                    perf_map::record(ptr, size, &sym);
                }
            }
        }

        // Find the __main__ entry point (last body is typically __main__)
        let entry_id = if let Some(body) = module.bodies.last() {
            self.internal_funcs.get(&body.name.0).copied()
        } else {
            None
        };

        if let Some(func_id) = entry_id {
            let entry_ptr = jit_module.get_finalized_function(func_id);
            Ok(CodegenOutput::Jit { entry: entry_ptr })
        } else {
            Err(crate::error::MambaError::codegen(
                "no entry point found".to_string(),
            ))
        }
    }

    fn name(&self) -> &str {
        "cranelift-jit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::CodegenBackend;
    use crate::mir::{
        BasicBlock, BlockId, MirBody, MirConst, MirInst, MirModule, Terminator, VReg,
    };
    use crate::resolve::SymbolId;
    use crate::runtime::closure::cleanup_all_closures;
    use crate::runtime::gc::{gc_clear_all_state, gc_get_full_stats};
    use crate::types::TypeContext;

    #[test]
    fn test_new_returns_ok() {
        let result = CraneliftJitBackend::new();
        assert!(result.is_ok(), "CraneliftJitBackend::new() should succeed");
    }

    #[test]
    fn test_new_with_externals_empty_returns_ok() {
        let result = CraneliftJitBackend::new_with_externals(&[]);
        assert!(result.is_ok(), "new_with_externals(&[]) should succeed");
    }

    #[test]
    fn test_name_is_cranelift_jit() {
        let backend = CraneliftJitBackend::new().unwrap();
        assert_eq!(backend.name(), "cranelift-jit");
    }

    // ── JIT_LOCK tests (sigbus-jit-concurrency-fix) ─────────────────────────

    /// Helper: acquire JIT_LOCK, tolerating poison from other test threads.
    fn acquire_jit_lock() -> std::sync::MutexGuard<'static, ()> {
        JIT_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn run_zero_arg_body_and_alloc_delta(module: MirModule, tcx: &TypeContext) -> (i64, usize) {
        let _guard = acquire_jit_lock();
        cleanup_all_closures();
        gc_clear_all_state();
        let before_alloc = gc_get_full_stats().3;

        let mut backend = CraneliftJitBackend::new().unwrap();
        let output = backend.codegen(&module, tcx).unwrap();
        let result = match output {
            crate::codegen::CodegenOutput::Jit { entry } => unsafe {
                let func: extern "C" fn() -> i64 = std::mem::transmute(entry);
                func()
            },
            _ => panic!("expected Jit output"),
        };

        let after_alloc = gc_get_full_stats().3;
        unsafe {
            crate::runtime::rc::release_if_ptr(MbValue::from_bits(result as u64));
        }
        cleanup_all_closures();
        gc_clear_all_state();
        (result, after_alloc.saturating_sub(before_alloc))
    }

    fn zero_arg_body(return_ty: TypeId, blocks: Vec<BasicBlock>) -> MirBody {
        MirBody {
            name: SymbolId(0),
            params: vec![],
            return_ty,
            blocks,
        }
    }

    fn entry_zero_arg_body(return_ty: TypeId, blocks: Vec<BasicBlock>) -> MirBody {
        MirBody {
            name: SymbolId(u32::MAX),
            params: vec![],
            return_ty,
            blocks,
        }
    }

    /// S2/R1: JIT_LOCK exists and is acquirable from external callers.
    #[test]
    fn jit_lock_is_acquirable() {
        let guard = acquire_jit_lock();
        // Lock acquired successfully — drop releases it.
        drop(guard);
    }

    /// S5/R2: Lock is released when MutexGuard is dropped (simulating error path).
    /// After acquiring and dropping the lock, a second acquisition must succeed
    /// without deadlock.
    #[test]
    fn jit_lock_released_on_drop() {
        {
            let _guard = acquire_jit_lock();
            // Simulate work or error; guard drops at scope exit.
        }
        // Must be reacquirable — proves the lock was released.
        let guard2 = acquire_jit_lock();
        drop(guard2);
    }

    /// S3/R4: Uncontended lock acquisition adds negligible overhead (<1ms).
    #[test]
    fn jit_lock_uncontended_overhead_is_negligible() {
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _guard = acquire_jit_lock();
        }
        let elapsed = start.elapsed();
        // 1000 acquisitions should complete well under 1 second.
        assert!(
            elapsed.as_millis() < 1000,
            "1000 uncontended lock acquisitions took {}ms — expected <1000ms",
            elapsed.as_millis()
        );
    }

    /// S2/R1: JIT_LOCK serializes concurrent access — two threads never hold
    /// the lock simultaneously.
    #[test]
    fn jit_lock_serializes_concurrent_threads() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..4 {
            let active = Arc::clone(&active);
            let max_active = Arc::clone(&max_active);
            handles.push(std::thread::spawn(move || {
                let _guard = acquire_jit_lock();
                let prev = active.fetch_add(1, Ordering::SeqCst);
                // Record the max concurrent holders.
                max_active.fetch_max(prev + 1, Ordering::SeqCst);
                // Simulate JIT work.
                std::thread::sleep(std::time::Duration::from_millis(5));
                active.fetch_sub(1, Ordering::SeqCst);
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        // At most 1 thread held the lock at any time.
        assert_eq!(
            max_active.load(Ordering::SeqCst),
            1,
            "more than one thread held JIT_LOCK concurrently"
        );
    }

    /// S5: Mutex is released even when a thread panics while holding it.
    /// Uses a local LazyLock<Mutex<()>> (same type as JIT_LOCK) to demonstrate
    /// the recovery pattern without poisoning the global JIT_LOCK.
    #[test]
    fn jit_lock_pattern_recoverable_after_panic() {
        static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

        // Spawn a thread that acquires the lock and panics — poisons it.
        let handle = std::thread::spawn(|| {
            let _guard = TEST_LOCK.lock().unwrap();
            panic!("intentional test panic to poison the lock");
        });
        // The thread panicked — join returns Err.
        let _ = handle.join();
        // Lock is poisoned but recoverable via into_inner().
        let guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        drop(guard);
        // This proves the LazyLock<Mutex<()>> pattern used by JIT_LOCK
        // releases the lock on panic and is recoverable.
    }

    /// S4/R5: CraneliftJitBackend::new() works WITHOUT acquiring JIT_LOCK —
    /// the lock is external / opt-in, not required for single-threaded usage.
    #[test]
    fn jit_backend_works_without_lock() {
        // Do NOT acquire JIT_LOCK — backend should still work.
        let backend = CraneliftJitBackend::new();
        assert!(
            backend.is_ok(),
            "CraneliftJitBackend::new() should work without JIT_LOCK"
        );
    }

    // ── Pre-existing codegen tests ────────────────────────────────────────────

    /// #2094: when `MAMBA_PERF_MAP=1` is set, the JIT pipeline must append
    /// at least one `<addr-hex> <size-hex> <symbol>` line to
    /// `/tmp/perf-<pid>.map` per finalized function.
    ///
    /// Uses the JIT_LOCK to serialize against other JIT tests so the
    /// process-wide env var flip does not leak into them. The marker is
    /// a per-test unique symbol prefix carried by the compiled function's
    /// MIR SymbolId (chosen to never collide with __main__ = u32::MAX).
    #[test]
    fn perf_map_written_when_env_set() {
        let _guard = acquire_jit_lock();
        // Also serialize against the perf_map module's env-touching tests
        // so they don't flip MAMBA_PERF_MAP back to unset between this
        // test's `set_var` and the JIT pipeline's `is_enabled()` check.
        let _env_guard = perf_map::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var("MAMBA_PERF_MAP").ok();
        // SAFETY: serialized by JIT_LOCK + TEST_ENV_LOCK above; restored
        // on scope exit below.
        unsafe { std::env::set_var("MAMBA_PERF_MAP", "1") };
        // Restore env on drop, including on panic.
        struct Restore(Option<String>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match &self.0 {
                    Some(v) => unsafe { std::env::set_var("MAMBA_PERF_MAP", v) },
                    None => unsafe { std::env::remove_var("MAMBA_PERF_MAP") },
                }
            }
        }
        let _restore = Restore(prev);

        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        // Pick a SymbolId that is unlikely to collide with any other test:
        // a random non-MAX 32-bit id derived from nanoseconds. The codegen
        // emits "_mb_<id>" so we can grep for it.
        let sym = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u32 & 0x7fff_ffff)
            .unwrap_or(123_456_789))
        .max(1);
        let mir = MirModule {
            bodies: vec![MirBody {
                name: SymbolId(sym),
                params: vec![],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(0),
                        value: MirConst::Int(7),
                        ty: int_ty,
                    }],
                    terminator: Terminator::Return(Some(VReg(0))),
                }],
            }],
            externs: vec![],
        };
        let mut backend = CraneliftJitBackend::new().unwrap();
        backend.codegen(&mir, &tcx).expect("codegen ok");

        let path = format!("/tmp/perf-{}.map", std::process::id());
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("expected perf map at {path}: {e}"));
        let needle = format!("_mb_{sym}");
        let line = body
            .lines()
            .find(|l| l.ends_with(&needle))
            .unwrap_or_else(|| {
                panic!("no perf-map line ending in {needle:?} found in {path}:\n{body}")
            });
        // Format must be: <addr-hex> <size-hex> <symbol>
        let mut parts = line.split_whitespace();
        let addr = parts.next().expect("addr");
        let size = parts.next().expect("size");
        let name = parts.next().expect("name");
        assert!(parts.next().is_none(), "extra fields on line: {line:?}");
        assert!(
            u64::from_str_radix(addr, 16).is_ok(),
            "addr {addr:?} not hex"
        );
        let size_n =
            u64::from_str_radix(size, 16).unwrap_or_else(|_| panic!("size {size:?} not hex"));
        assert!(size_n > 0, "expected non-zero compiled size");
        assert_eq!(name, needle);
    }

    #[test]
    fn test_codegen_minimal_function_returns_42() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let mir = MirModule {
            bodies: vec![MirBody {
                name: SymbolId(0),
                params: vec![],
                return_ty: int_ty,
                blocks: vec![BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::LoadConst {
                        dest: VReg(0),
                        value: MirConst::Int(42),
                        ty: int_ty,
                    }],
                    terminator: Terminator::Return(Some(VReg(0))),
                }],
            }],
            externs: vec![],
        };
        let mut backend = CraneliftJitBackend::new().unwrap();
        let output = backend.codegen(&mir, &tcx).unwrap();
        match output {
            crate::codegen::CodegenOutput::Jit { entry } => {
                let result = unsafe {
                    let func: extern "C" fn() -> i64 = std::mem::transmute(entry);
                    func()
                };
                assert_eq!(result, 42);
            }
            _ => panic!("expected Jit output"),
        }
    }

    #[test]
    fn test_codegen_internal_bool_return_boxes_to_any() {
        let _guard = acquire_jit_lock();
        let tcx = TypeContext::new();
        let bool_ty = tcx.bool();
        let any_ty = tcx.any();
        let mir = MirModule {
            bodies: vec![
                MirBody {
                    name: SymbolId(0),
                    params: vec![],
                    return_ty: bool_ty,
                    blocks: vec![BasicBlock {
                        id: BlockId(0),
                        stmts: vec![MirInst::LoadConst {
                            dest: VReg(0),
                            value: MirConst::Bool(true),
                            ty: bool_ty,
                        }],
                        terminator: Terminator::Return(Some(VReg(0))),
                    }],
                },
                MirBody {
                    name: SymbolId(1),
                    params: vec![],
                    return_ty: any_ty,
                    blocks: vec![BasicBlock {
                        id: BlockId(0),
                        stmts: vec![MirInst::Call {
                            dest: Some(VReg(0)),
                            func: SymbolId(0),
                            args: vec![],
                            ty: any_ty,
                        }],
                        terminator: Terminator::Return(Some(VReg(0))),
                    }],
                },
            ],
            externs: vec![],
        };
        let mut backend = CraneliftJitBackend::new().unwrap();
        let output = backend.codegen(&mir, &tcx).unwrap();
        match output {
            crate::codegen::CodegenOutput::Jit { entry } => {
                let result = unsafe {
                    let func: extern "C" fn() -> i64 = std::mem::transmute(entry);
                    func()
                };
                assert_eq!(result, MbValue::from_bool(true).to_bits() as i64);
            }
            _ => panic!("expected Jit output"),
        }
    }

    #[test]
    fn test_codegen_internal_native_bool_return_boxes_through_mb_box_int() {
        let _guard = acquire_jit_lock();
        let tcx = TypeContext::new();
        let bool_ty = tcx.bool();
        let int_ty = tcx.int();
        let any_ty = tcx.any();
        let mir = MirModule {
            bodies: vec![
                MirBody {
                    name: SymbolId(0),
                    params: vec![],
                    return_ty: int_ty,
                    blocks: vec![
                        BasicBlock {
                            id: BlockId(0),
                            stmts: vec![MirInst::LoadConst {
                                dest: VReg(0),
                                value: MirConst::Bool(true),
                                ty: bool_ty,
                            }],
                            terminator: Terminator::Return(Some(VReg(0))),
                        },
                        BasicBlock {
                            id: BlockId(1),
                            stmts: vec![MirInst::LoadConst {
                                dest: VReg(1),
                                value: MirConst::None,
                                ty: any_ty,
                            }],
                            terminator: Terminator::Return(Some(VReg(1))),
                        },
                    ],
                },
                MirBody {
                    name: SymbolId(1),
                    params: vec![],
                    return_ty: any_ty,
                    blocks: vec![BasicBlock {
                        id: BlockId(0),
                        stmts: vec![
                            MirInst::Call {
                                dest: Some(VReg(0)),
                                func: SymbolId(0),
                                args: vec![],
                                ty: int_ty,
                            },
                            MirInst::CallExtern {
                                dest: Some(VReg(1)),
                                name: "mb_box_int".to_string(),
                                args: vec![VReg(0)],
                                ty: any_ty,
                            },
                        ],
                        terminator: Terminator::Return(Some(VReg(1))),
                    }],
                },
            ],
            externs: vec![],
        };
        let mut backend = CraneliftJitBackend::new().unwrap();
        let output = backend.codegen(&mir, &tcx).unwrap();
        match output {
            crate::codegen::CodegenOutput::Jit { entry } => {
                let result = unsafe {
                    let func: extern "C" fn() -> i64 = std::mem::transmute(entry);
                    func()
                };
                assert_eq!(result, MbValue::from_bool(true).to_bits() as i64);
            }
            _ => panic!("expected Jit output"),
        }
    }

    #[test]
    fn test_non_escaping_list_and_dict_literals_skip_gc_tracking() {
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let none_ty = tcx.none();

        let list_body = zero_arg_body(
            none_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![MirInst::MakeList {
                    dest: VReg(0),
                    elements: vec![],
                    ty: any_ty,
                }],
                terminator: Terminator::Return(None),
            }],
        );
        let (_, list_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![list_body],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            list_delta, 0,
            "non-escaping list literal must skip gc_track"
        );

        let dict_body = zero_arg_body(
            none_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![MirInst::MakeDict {
                    dest: VReg(0),
                    keys: vec![],
                    values: vec![],
                    ty: any_ty,
                }],
                terminator: Terminator::Return(None),
            }],
        );
        let (_, dict_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![dict_body],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            dict_delta, 0,
            "non-escaping dict literal must skip gc_track"
        );
    }

    #[test]
    fn test_entry_body_literals_remain_tracked() {
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let none_ty = tcx.none();

        let list_body = entry_zero_arg_body(
            none_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![MirInst::MakeList {
                    dest: VReg(0),
                    elements: vec![],
                    ty: any_ty,
                }],
                terminator: Terminator::Return(None),
            }],
        );
        let (_, list_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![list_body],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            list_delta, 1,
            "entry-body list literal must remain tracked because __main__ has no release epilogue"
        );

        let dict_body = entry_zero_arg_body(
            none_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![MirInst::MakeDict {
                    dest: VReg(0),
                    keys: vec![],
                    values: vec![],
                    ty: any_ty,
                }],
                terminator: Terminator::Return(None),
            }],
        );
        let (_, dict_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![dict_body],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            dict_delta, 1,
            "entry-body dict literal must remain tracked because __main__ has no release epilogue"
        );
    }

    #[test]
    fn test_returned_called_global_cell_aggregate_and_unknown_literals_remain_tracked() {
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let none_ty = tcx.none();
        let bool_ty = tcx.bool();

        let returned_list = zero_arg_body(
            any_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![MirInst::MakeList {
                    dest: VReg(0),
                    elements: vec![],
                    ty: any_ty,
                }],
                terminator: Terminator::Return(Some(VReg(0))),
            }],
        );
        let (_, returned_list_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![returned_list],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            returned_list_delta, 1,
            "returned list literal must remain tracked"
        );

        let returned_dict = zero_arg_body(
            any_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![MirInst::MakeDict {
                    dest: VReg(0),
                    keys: vec![],
                    values: vec![],
                    ty: any_ty,
                }],
                terminator: Terminator::Return(Some(VReg(0))),
            }],
        );
        let (_, returned_dict_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![returned_dict],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            returned_dict_delta, 1,
            "returned dict literal must remain tracked"
        );

        let called_list = zero_arg_body(
            none_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::MakeList {
                        dest: VReg(0),
                        elements: vec![],
                        ty: any_ty,
                    },
                    MirInst::CallExtern {
                        dest: None,
                        name: "mb_is_truthy".to_string(),
                        args: vec![VReg(0)],
                        ty: bool_ty,
                    },
                ],
                terminator: Terminator::Return(None),
            }],
        );
        let (_, called_list_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![called_list],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            called_list_delta, 1,
            "called list literal must remain tracked"
        );

        let called_dict = zero_arg_body(
            none_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::MakeDict {
                        dest: VReg(0),
                        keys: vec![],
                        values: vec![],
                        ty: any_ty,
                    },
                    MirInst::CallExtern {
                        dest: None,
                        name: "mb_is_truthy".to_string(),
                        args: vec![VReg(0)],
                        ty: bool_ty,
                    },
                ],
                terminator: Terminator::Return(None),
            }],
        );
        let (_, called_dict_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![called_dict],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            called_dict_delta, 1,
            "called dict literal must remain tracked"
        );

        let global_list = zero_arg_body(
            none_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::MakeList {
                        dest: VReg(0),
                        elements: vec![],
                        ty: any_ty,
                    },
                    MirInst::StoreGlobal {
                        name: SymbolId(17),
                        value: VReg(0),
                    },
                ],
                terminator: Terminator::Return(None),
            }],
        );
        let (_, global_list_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![global_list],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            global_list_delta, 1,
            "global-stored list literal must remain tracked"
        );

        let global_dict = zero_arg_body(
            none_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::MakeDict {
                        dest: VReg(0),
                        keys: vec![],
                        values: vec![],
                        ty: any_ty,
                    },
                    MirInst::StoreGlobal {
                        name: SymbolId(23),
                        value: VReg(0),
                    },
                ],
                terminator: Terminator::Return(None),
            }],
        );
        let (_, global_dict_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![global_dict],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            global_dict_delta, 1,
            "global-stored dict literal must remain tracked"
        );

        let cell_list = zero_arg_body(
            none_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::MakeList {
                        dest: VReg(0),
                        elements: vec![],
                        ty: any_ty,
                    },
                    MirInst::MakeCell {
                        dest: VReg(1),
                        value: VReg(0),
                        ty: any_ty,
                    },
                ],
                terminator: Terminator::Return(None),
            }],
        );
        let (_, cell_list_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![cell_list],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            cell_list_delta, 1,
            "cell-captured list literal must remain tracked"
        );

        let cell_dict = zero_arg_body(
            none_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::MakeDict {
                        dest: VReg(0),
                        keys: vec![],
                        values: vec![],
                        ty: any_ty,
                    },
                    MirInst::MakeCell {
                        dest: VReg(1),
                        value: VReg(0),
                        ty: any_ty,
                    },
                ],
                terminator: Terminator::Return(None),
            }],
        );
        let (_, cell_dict_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![cell_dict],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            cell_dict_delta, 1,
            "cell-captured dict literal must remain tracked"
        );

        let aggregate_list = zero_arg_body(
            none_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::MakeList {
                        dest: VReg(0),
                        elements: vec![],
                        ty: any_ty,
                    },
                    MirInst::MakeList {
                        dest: VReg(1),
                        elements: vec![VReg(0)],
                        ty: any_ty,
                    },
                ],
                terminator: Terminator::Return(None),
            }],
        );
        let (_, aggregate_list_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![aggregate_list],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            aggregate_list_delta, 1,
            "inner aggregate list literal must remain tracked while the outer local skips tracking"
        );

        let aggregate_dict = zero_arg_body(
            none_ty,
            vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::MakeDict {
                        dest: VReg(0),
                        keys: vec![],
                        values: vec![],
                        ty: any_ty,
                    },
                    MirInst::MakeList {
                        dest: VReg(1),
                        elements: vec![VReg(0)],
                        ty: any_ty,
                    },
                ],
                terminator: Terminator::Return(None),
            }],
        );
        let (_, aggregate_dict_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![aggregate_dict],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            aggregate_dict_delta, 1,
            "inner aggregate dict literal must remain tracked while the outer local skips tracking"
        );

        let unknown_list = zero_arg_body(
            none_ty,
            vec![
                BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::MakeList {
                        dest: VReg(0),
                        elements: vec![],
                        ty: any_ty,
                    }],
                    terminator: Terminator::Branch {
                        cond: VReg(0),
                        then_block: BlockId(1),
                        else_block: BlockId(2),
                    },
                },
                BasicBlock {
                    id: BlockId(1),
                    stmts: vec![],
                    terminator: Terminator::Return(None),
                },
                BasicBlock {
                    id: BlockId(2),
                    stmts: vec![],
                    terminator: Terminator::Return(None),
                },
            ],
        );
        let (_, unknown_list_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![unknown_list],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            unknown_list_delta, 1,
            "unknown-use list literal must remain tracked"
        );

        let unknown_dict = zero_arg_body(
            none_ty,
            vec![
                BasicBlock {
                    id: BlockId(0),
                    stmts: vec![MirInst::MakeDict {
                        dest: VReg(0),
                        keys: vec![],
                        values: vec![],
                        ty: any_ty,
                    }],
                    terminator: Terminator::Branch {
                        cond: VReg(0),
                        then_block: BlockId(1),
                        else_block: BlockId(2),
                    },
                },
                BasicBlock {
                    id: BlockId(1),
                    stmts: vec![],
                    terminator: Terminator::Return(None),
                },
                BasicBlock {
                    id: BlockId(2),
                    stmts: vec![],
                    terminator: Terminator::Return(None),
                },
            ],
        );
        let (_, unknown_dict_delta) = run_zero_arg_body_and_alloc_delta(
            MirModule {
                bodies: vec![unknown_dict],
                externs: vec![],
            },
            &tcx,
        );
        assert_eq!(
            unknown_dict_delta, 1,
            "unknown-use dict literal must remain tracked"
        );
    }

    #[test]
    fn test_stored_literals_remain_tracked() {
        let _guard = acquire_jit_lock();
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let int_ty = tcx.int();
        let none_ty = tcx.none();

        cleanup_all_closures();
        gc_clear_all_state();

        let list_container = MbValue::from_ptr(MbObject::new_dict());
        gc_clear_all_state();
        let before_list_alloc = gc_get_full_stats().3;
        let list_body = MirBody {
            name: SymbolId(0),
            params: vec![(VReg(0), any_ty)],
            return_ty: none_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Int(0),
                        ty: int_ty,
                    },
                    MirInst::MakeList {
                        dest: VReg(2),
                        elements: vec![],
                        ty: any_ty,
                    },
                    MirInst::SetItem {
                        object: VReg(0),
                        index: VReg(1),
                        value: VReg(2),
                    },
                ],
                terminator: Terminator::Return(None),
            }],
        };
        let mut backend = CraneliftJitBackend::new().unwrap();
        let output = backend
            .codegen(
                &MirModule {
                    bodies: vec![list_body],
                    externs: vec![],
                },
                &tcx,
            )
            .unwrap();
        let list_result = match output {
            crate::codegen::CodegenOutput::Jit { entry } => unsafe {
                let func: extern "C" fn(i64) -> i64 = std::mem::transmute(entry);
                func(list_container.to_bits() as i64)
            },
            _ => panic!("expected Jit output"),
        };
        let after_list_alloc = gc_get_full_stats().3;
        unsafe {
            crate::runtime::rc::release_if_ptr(MbValue::from_bits(list_result as u64));
            crate::runtime::rc::release_if_ptr(list_container);
        }
        cleanup_all_closures();
        gc_clear_all_state();
        assert_eq!(
            after_list_alloc.saturating_sub(before_list_alloc),
            1,
            "stored list literal must remain tracked"
        );

        let dict_container = MbValue::from_ptr(MbObject::new_dict());
        gc_clear_all_state();
        let before_dict_alloc = gc_get_full_stats().3;
        let dict_body = MirBody {
            name: SymbolId(0),
            params: vec![(VReg(0), any_ty)],
            return_ty: none_ty,
            blocks: vec![BasicBlock {
                id: BlockId(0),
                stmts: vec![
                    MirInst::LoadConst {
                        dest: VReg(1),
                        value: MirConst::Int(0),
                        ty: int_ty,
                    },
                    MirInst::MakeDict {
                        dest: VReg(2),
                        keys: vec![],
                        values: vec![],
                        ty: any_ty,
                    },
                    MirInst::SetItem {
                        object: VReg(0),
                        index: VReg(1),
                        value: VReg(2),
                    },
                ],
                terminator: Terminator::Return(None),
            }],
        };
        let mut backend = CraneliftJitBackend::new().unwrap();
        let output = backend
            .codegen(
                &MirModule {
                    bodies: vec![dict_body],
                    externs: vec![],
                },
                &tcx,
            )
            .unwrap();
        let dict_result = match output {
            crate::codegen::CodegenOutput::Jit { entry } => unsafe {
                let func: extern "C" fn(i64) -> i64 = std::mem::transmute(entry);
                func(dict_container.to_bits() as i64)
            },
            _ => panic!("expected Jit output"),
        };
        let after_dict_alloc = gc_get_full_stats().3;
        unsafe {
            crate::runtime::rc::release_if_ptr(MbValue::from_bits(dict_result as u64));
            crate::runtime::rc::release_if_ptr(dict_container);
        }
        cleanup_all_closures();
        gc_clear_all_state();
        assert_eq!(
            after_dict_alloc.saturating_sub(before_dict_alloc),
            1,
            "stored dict literal must remain tracked"
        );
    }
}
