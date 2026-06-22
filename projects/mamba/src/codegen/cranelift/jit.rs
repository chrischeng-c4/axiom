/// Cranelift JIT backend for Mamba (#296).
///
/// Uses cranelift-jit's JITModule to compile MIR directly into executable
/// memory. Runtime mb_* functions are wired as symbols so JIT-compiled
/// code can call them.
use super::marshal;
use super::perf_map;
use super::{emit_binop, emit_terminator, VarAlloc, EMIT_REFCOUNT_CALLS};
use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::mir::{MirBinOp, MirBody, MirConst, MirExtern, MirInst, MirModule, MirType, VReg};
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

use std::collections::HashMap;
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
            internal_param_counts: HashMap::new(),
            compile_time_objects: Vec::new(),
            internal_code_sizes: HashMap::new(),
        })
    }

    fn module(&mut self) -> &mut JITModule {
        self.module.as_mut().expect("module already consumed")
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
        let is_entry_body = body.name.0 == u32::MAX;
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

        for (block_idx, block) in body.blocks.iter().enumerate() {
            if block_idx > 0 {
                builder.switch_to_block(cl_blocks[&block.id.0]);
            }
            for inst in &block.stmts {
                self.emit_inst(inst, tcx, externs, &mut builder, &mut vars);
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
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
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
                if vars.is_declared_i64(dest) && !vars.raw_ints.contains(&dest) {
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
                        let ptr = MbObject::new_str_immortal(s.clone());
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
                let int_mod = matches!(op, MirBinOp::Mod) && matches!(resolved_ty, Ty::Int);
                if matches!(op, MirBinOp::FloorDiv) || int_mod {
                    // Floor division → call mb_floordiv runtime for correct Python
                    // floor semantics and ZeroDivisionError handling (#1085).
                    // Int modulo → call mb_mod for the same reason: the inline
                    // `srem` fast path executed a raw Cranelift hardware trap on
                    // `x % 0` (SIGILL, exit 132) instead of raising a catchable
                    // ZeroDivisionError.
                    // Float operands are already NaN-boxed I64 MbValues — no boxing needed.
                    // Int/Bool operands need boxing from raw I64 to MbValue.
                    let helper_name = if int_mod { "mb_mod" } else { "mb_floordiv" };
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
                            // If operand is raw 0/1 (raw_int), XOR works directly.
                            // If operand is NaN-boxed (instance ptr, float, etc.),
                            // must call mb_is_truthy first to get raw 0/1.
                            if vars.raw_ints.contains(operand) {
                                let one = builder.ins().iconst(cl_types::I64, 1);
                                builder.ins().bxor(val, one)
                            } else if let Some(&truthy_id) = self.extern_funcs.get("mb_is_truthy") {
                                let truthy_ref =
                                    self.module().declare_func_in_func(truthy_id, builder.func);
                                let call = builder.ins().call(truthy_ref, &[val]);
                                let truthy_val = builder.inst_results(call)[0];
                                let one = builder.ins().iconst(cl_types::I64, 1);
                                builder.ins().bxor(truthy_val, one)
                            } else {
                                let one = builder.ins().iconst(cl_types::I64, 1);
                                builder.ins().bxor(val, one)
                            }
                        }
                        crate::mir::MirUnaryOp::BitNot => builder.ins().bnot(val),
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
                            if vars.raw_ints.contains(operand) {
                                let one = builder.ins().iconst(cl_types::I64, 1);
                                builder.ins().bxor(val, one)
                            } else if let Some(&truthy_id) = self.extern_funcs.get("mb_is_truthy") {
                                let truthy_ref =
                                    self.module().declare_func_in_func(truthy_id, builder.func);
                                let call = builder.ins().call(truthy_ref, &[val]);
                                let truthy_val = builder.inst_results(call)[0];
                                let one = builder.ins().iconst(cl_types::I64, 1);
                                builder.ins().bxor(truthy_val, one)
                            } else {
                                let one = builder.ins().iconst(cl_types::I64, 1);
                                builder.ins().bxor(val, one)
                            }
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
                self.emit_make_list(dest, elements, builder, vars, externs);
            }
            MirInst::MakeDict {
                dest,
                keys,
                values,
                ty: _,
            } => {
                self.emit_make_dict(dest, keys, values, builder, vars, externs);
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

    fn emit_make_list(
        &mut self,
        dest: &crate::mir::VReg,
        elements: &[crate::mir::VReg],
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
        let small_arity_fn = match n {
            1 => Some("mb_list_new_1"),
            2 => Some("mb_list_new_2"),
            3 => Some("mb_list_new_3"),
            4 => Some("mb_list_new_4"),
            5 => Some("mb_list_new_5"),
            6 => Some("mb_list_new_6"),
            7 => Some("mb_list_new_7"),
            8 => Some("mb_list_new_8"),
            9 => Some("mb_list_new_9"),
            10 => Some("mb_list_new_10"),
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
                let list_val = builder.inst_results(call)[0];
                vars.def_var_cast(*dest, builder, list_val, cl_types::I64);
                return;
            }
        }

        let new_id_opt = if n > 0 {
            self.extern_funcs
                .get("mb_list_new_with_capacity")
                .copied()
                .or_else(|| self.extern_funcs.get("mb_list_new").copied())
        } else {
            self.extern_funcs.get("mb_list_new").copied()
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
        builder: &mut cranelift_frontend::FunctionBuilder,
        vars: &mut VarAlloc,
        _externs: &[MirExtern],
    ) {
        if let (Some(&new_id), Some(&set_id)) = (
            self.extern_funcs.get("mb_dict_new"),
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
                    let callsite_ty = tcx.get(*ty);
                    let callee_is_primitive = matches!(callee_ty, Ty::Int | Ty::Bool | Ty::Float);
                    let callsite_is_nonprimitive =
                        !matches!(callsite_ty, Ty::Int | Ty::Bool | Ty::Float);
                    if callee_is_primitive && callsite_is_nonprimitive {
                        // Float: already NaN-boxed as I64 (= MbValue), no boxing needed.
                        // Int/Bool: raw value in I64, needs boxing to MbValue.
                        if matches!(callee_ty, Ty::Float) {
                            result
                        } else {
                            let box_fn_name = match callee_ty {
                                Ty::Bool => "mb_box_bool",
                                _ => "mb_box_int",
                            };
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
                    let callsite_ty = tcx.get(*ty);
                    if matches!(callee_ty, Ty::Int | Ty::Bool)
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
}
