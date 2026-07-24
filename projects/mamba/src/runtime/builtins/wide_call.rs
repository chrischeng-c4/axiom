//! Wide-arity JIT call dispatch shim compiler (#1950).
//!
//! `dispatch_jit_frame` (`runtime/builtins/mod.rs`) calls a JIT-compiled
//! function through a raw `usize` address at runtime, which in Rust requires
//! transmuting that address to a concrete `extern "C" fn(..)` type — and
//! Rust fn-pointer types are monomorphic per arity, so a hand-written match
//! can only cover a fixed, hardcoded set of arities (previously 0..=16,
//! #1754). Genuinely-direct calls to statically-known callees never hit this
//! limit (Cranelift's own `call`/`call_indirect` already accept an arbitrary
//! runtime-length argument `Vec` — see `declare_internal`/`compile_function`
//! in `codegen/cranelift/jit.rs`); only the *dynamic* dispatcher (`*args`
//! spread, `**kwargs` spread, indirect/unknown callees) needs a Rust-typed
//! function pointer at all.
//!
//! Rather than raising past a hardcoded ceiling (the old behavior) or
//! hand-rolling a per-arch assembly trampoline, this module asks Cranelift
//! itself to author one small "loader shim" IR function per DISTINCT wide
//! arity actually requested at runtime — never a fixed max. Each shim has
//! the uniform signature `(target_addr: i64, args_ptr: i64) -> i64`: it
//! loads `n` consecutive 8-byte words from `args_ptr`, `call_indirect`s
//! `target_addr` with them (`CallConv::SystemV`, n x I64 -> I64 — matches
//! `mamba_to_cl_type`'s uniform I64 ABI for every MbValue slot regardless of
//! static mamba type, `codegen/cranelift/jit.rs:313-319`), and returns the
//! raw result bits. Every shim shares ONE Rust-callable type regardless of
//! `n`, so `dispatch_jit_frame` needs only a single match arm for "n > 16"
//! instead of one per arity. Not perf-gated (#1950 design guidance): this
//! path is an extra `call_indirect` hop plus a one-time-per-(thread, arity)
//! JIT compile on first hit only.
//!
//! Fully self-contained: its own tiny `JITModule`, independent of the
//! compiled program's own backend (`driver/mod.rs`'s `CraneliftJitBackend`
//! is a stack-local value, not reachable from deep runtime code — see
//! `tech-design/calling-convention/ARCHITECTURE.md`). No extern runtime
//! symbols are wired in because shim bodies never call back into `mb_*` —
//! they only load, call_indirect, and return.
//!
//! The `JITModule` lives in a `thread_local!` (not a process-global
//! `static`) so this module never has to argue about `JITModule: Send`;
//! each thread that ever needs a wide-arity shim lazily builds its own tiny
//! module (redundant compilation across threads is possible but harmless —
//! this path is explicitly not perf-gated). The known "concurrent JITModule
//! finalization SIGBUS-races on aarch64 mprotect" hazard
//! (`codegen/cranelift/jit.rs:30-33`) is cross-thread regardless of which
//! `JITModule` instance is finalizing, so shim finalization still holds the
//! same process-wide `JIT_LOCK` the main backend uses.

use cranelift_codegen::ir::{
    types as cl_types, AbiParam, Function, InstBuilder, MemFlags, Signature,
};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use std::cell::RefCell;
use std::collections::HashMap;

/// Uniform Rust-callable type for every compiled shim, regardless of arity:
/// `(target_fn_addr, args_base_ptr) -> raw_result_bits`. `args_base_ptr`
/// points at a contiguous run of `MbValue` (each an 8-byte NaN-boxed word,
/// `#[repr(transparent)]` over `u64` — `runtime/value.rs:67`), so the shim
/// treats it as a flat array of 8-byte words without depending on
/// `MbValue`'s Rust type at all.
type WideShimFn = extern "C" fn(usize, usize) -> u64;

/// One dedicated JITModule for shim compilation, plus the arity -> compiled
/// shim address cache.
struct ShimJit {
    module: JITModule,
    compiled: HashMap<usize, usize>,
}

impl ShimJit {
    fn new() -> Self {
        let mut flags_builder = settings::builder();
        flags_builder
            .set("use_colocated_libcalls", "false")
            .unwrap();
        flags_builder.set("is_pic", "false").unwrap();
        flags_builder.set("opt_level", "speed").unwrap();
        let isa = cranelift_native::builder()
            .expect("no native ISA")
            .finish(settings::Flags::new(flags_builder))
            .expect("native ISA finish");
        let jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(jit_builder);
        Self {
            module,
            compiled: HashMap::new(),
        }
    }

    /// Compile (or fetch the cached) loader shim for exactly `n` args.
    fn shim_for_arity(&mut self, n: usize) -> usize {
        if let Some(&addr) = self.compiled.get(&n) {
            return addr;
        }

        // Shim's own signature: (target_addr: i64, args_ptr: i64) -> i64.
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(cl_types::I64)); // target_addr
        sig.params.push(AbiParam::new(cl_types::I64)); // args_ptr
        sig.returns.push(AbiParam::new(cl_types::I64));

        let name = format!("__wide_call_shim_{n}");
        let func_id = self
            .module
            .declare_function(&name, Linkage::Local, &sig)
            .expect("declare wide-call shim");

        let mut func = Function::with_name_signature(
            cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32()),
            sig,
        );
        let mut fb_ctx = cranelift_frontend::FunctionBuilderContext::new();
        let mut builder = cranelift_frontend::FunctionBuilder::new(&mut func, &mut fb_ctx);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        let params = builder.block_params(entry).to_vec();
        let target_addr = params[0];
        let args_ptr = params[1];

        // Target callee's real signature: n x I64 -> I64 (mamba_to_cl_type
        // is uniformly I64 for every MbValue slot regardless of static
        // mamba type — jit.rs:313-319 — so this always matches the real
        // compiled entry point, whatever n is).
        let mut callee_sig = Signature::new(CallConv::SystemV);
        for _ in 0..n {
            callee_sig.params.push(AbiParam::new(cl_types::I64));
        }
        callee_sig.returns.push(AbiParam::new(cl_types::I64));
        let sig_ref = builder.import_signature(callee_sig);

        let mem_flags = MemFlags::trusted();
        let mut call_args = Vec::with_capacity(n);
        for i in 0..n {
            let offset = (i * 8) as i32;
            call_args.push(
                builder
                    .ins()
                    .load(cl_types::I64, mem_flags, args_ptr, offset),
            );
        }

        let call = builder
            .ins()
            .call_indirect(sig_ref, target_addr, &call_args);
        let results = builder.inst_results(call).to_vec();
        builder.ins().return_(&results);
        builder.finalize();

        let mut ctx = cranelift_codegen::Context::for_function(func);
        self.module
            .define_function(func_id, &mut ctx)
            .expect("define wide-call shim");

        // See module doc: serialize against the main backend's own
        // finalize_definitions() calls process-wide, not just within this
        // module's own thread_local instance.
        let _jit_lock = crate::codegen::cranelift::jit::JIT_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        self.module
            .finalize_definitions()
            .expect("finalize wide-call shim");
        drop(_jit_lock);

        let addr = self.module.get_finalized_function(func_id) as usize;
        self.compiled.insert(n, addr);
        addr
    }
}

thread_local! {
    static SHIM_JIT: RefCell<ShimJit> = RefCell::new(ShimJit::new());
}

/// Dispatch a JIT-compiled function of arbitrary arity `n` (no ceiling) by
/// lazily compiling (or reusing) a Cranelift-authored loader shim for that
/// exact arity, then calling through it. `args_ptr` must point at `n`
/// contiguous 8-byte words (the caller's `&[MbValue]` frame). See module
/// doc for the design rationale (#1950).
pub(crate) fn dispatch_wide(raw_addr: usize, args_ptr: usize, n: usize) -> u64 {
    let shim_addr = SHIM_JIT.with(|cell| cell.borrow_mut().shim_for_arity(n));
    let shim: WideShimFn = unsafe { std::mem::transmute(shim_addr) };
    shim(raw_addr, args_ptr)
}
