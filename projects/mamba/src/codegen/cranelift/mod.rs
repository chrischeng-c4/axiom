pub mod aot;
pub mod jit;
pub mod marshal;
pub mod perf_map;

/// Enable JIT-emitted retain/release calls (#1129).
///
/// **Enabled.** Closure ownership symmetry bug is resolved — `mb_closure_release`
/// now cascade-releases captures, func, and defaults. All borrowed-reference
/// runtime functions have `retain_if_ptr`. JIT-compiled code emits
/// `mb_retain_value`/`mb_release_value` for Copy, StoreGlobal, StoreCell,
/// Return, and release-before-overwrite instructions.
// @spec .aw/changes/mamba-refcount/groups/jit-refcount-enable/specs/jit-refcount-enable.md#R3
const EMIT_REFCOUNT_CALLS: bool = true;

use crate::codegen::{CodegenBackend, CodegenOutput};
use crate::mir::{
    MirBinOp, MirBody, MirConst, MirExtern, MirInst, MirModule, MirType, Terminator, VReg,
};
use crate::runtime::rc::MbObject;
use crate::runtime::value::MbValue;
use crate::types::{Ty, TypeContext, TypeId};

use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::{
    types as cl_types, AbiParam, Function, InstBuilder, MemFlags, Signature,
};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use std::collections::{HashMap, HashSet};

/// Walk all MIR bodies and collect extern function names that are actually used.
fn collect_used_externs(module: &MirModule) -> HashSet<String> {
    let mut used = HashSet::new();
    for body in &module.bodies {
        for block in &body.blocks {
            for inst in &block.stmts {
                match inst {
                    MirInst::CallExtern { name, .. } => {
                        used.insert(name.clone());
                    }
                    MirInst::MakeList { .. } => {
                        used.insert("mb_list_new".into());
                        used.insert("mb_list_append".into());
                    }
                    MirInst::MakeDict { .. } => {
                        used.insert("mb_dict_new".into());
                        used.insert("mb_dict_setitem".into());
                    }
                    MirInst::GetAttr { .. } => {
                        used.insert("mb_getattr".into());
                    }
                    MirInst::SetAttr { .. } => {
                        used.insert("mb_setattr".into());
                    }
                    MirInst::GetItem { .. } => {
                        used.insert("mb_obj_getitem".into());
                    }
                    MirInst::SetItem { .. } => {
                        used.insert("mb_obj_setitem".into());
                    }
                    MirInst::LoadGlobal { .. } => {
                        used.insert("mb_global_get_id".into());
                    }
                    MirInst::StoreGlobal { .. } => {
                        used.insert("mb_global_set_id".into());
                    }
                    MirInst::DeleteGlobal { .. } => {
                        used.insert("mb_global_del_id".into());
                    }
                    MirInst::MakeTuple { .. } => {
                        used.insert("mb_list_new".into());
                        used.insert("mb_list_append".into());
                        used.insert("mb_list_to_tuple".into());
                    }
                    MirInst::BinOp { .. } => {
                        used.insert("mb_dispatch_binop".into());
                        used.insert("mb_obj_contains".into());
                    }
                    MirInst::UnaryOp { .. } => {
                        used.insert("mb_dispatch_unaryop".into());
                        used.insert("mb_is_truthy".into());
                    }
                    _ => {}
                }
            }
        }
    }
    used
}

/// Variable allocator — maps VRegs to Cranelift Variables.
struct VarAlloc {
    map: HashMap<VReg, Variable>,
    /// Track declared type for each VReg (needed for refcount cleanup).
    types: HashMap<VReg, cranelift_codegen::ir::Type>,
    next: u32,
    /// VRegs that hold a native 0/1 comparison result (from BinOp Lt/Gt/Eq/…).
    /// Branch codegen can use these directly without `band_imm 1` extraction.
    native_bools: HashSet<VReg>,
    /// VRegs known to hold raw i64 values (NOT NaN-boxed).
    /// Sources: LoadConst Int, BinOp with Int type (when both operands are raw).
    /// This enables native iadd/isub/imul instead of extern "C" mb_bigint_add.
    raw_ints: HashSet<VReg>,
}

impl VarAlloc {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            types: HashMap::new(),
            next: 0,
            native_bools: HashSet::new(),
            raw_ints: HashSet::new(),
        }
    }

    fn get(
        &mut self,
        vreg: VReg,
        builder: &mut FunctionBuilder,
        ty: cranelift_codegen::ir::Type,
    ) -> Variable {
        *self.map.entry(vreg).or_insert_with(|| {
            let var = Variable::from_u32(self.next);
            self.next += 1;
            builder.declare_var(var, ty);
            self.types.insert(vreg, ty);
            var
        })
    }

    /// Check if a VReg was already declared as I64 (potential MbValue holder).
    /// Returns false if undeclared or declared as F64.
    fn is_declared_i64(&self, vreg: VReg) -> bool {
        self.types.get(&vreg) == Some(&cl_types::I64)
    }

    /// Return all VRegs that may hold a heap pointer and therefore must
    /// run through `mb_release_value` at function epilogue. Excludes:
    /// - F64 VRegs (cannot hold a NaN-boxed pointer at function-local
    ///   scope — bitcast happens at call sites only)
    /// - `raw_ints` (native i64 arithmetic, no NaN-box → no heap ptr)
    /// - `native_bools` (0/1 comparison results)
    /// Skipping these saves a no-op `mb_release_value` call per local in
    /// hot recursive functions like fib/factorial (#1129).
    fn i64_vregs(&self) -> Vec<VReg> {
        self.map
            .keys()
            .filter(|v| {
                self.types.get(v) == Some(&cl_types::I64)
                    && !self.raw_ints.contains(v)
                    && !self.native_bools.contains(v)
            })
            .copied()
            .collect()
    }

    /// Query the declared Cranelift type of a VReg.
    fn declared_type(&self, vreg: VReg) -> Option<cranelift_codegen::ir::Type> {
        self.types.get(&vreg).copied()
    }

    /// Load a variable's value as I64, inserting a bitcast if the variable is F64.
    /// Used when passing values to runtime functions that expect MbValue (u64).
    /// NaN-boxing for floats is raw bit storage, so bitcast is semantically correct.
    fn use_as_i64(
        &self,
        vreg: VReg,
        builder: &mut FunctionBuilder,
    ) -> cranelift_codegen::ir::Value {
        self.use_as(vreg, cl_types::I64, builder)
    }

    /// Store a value into a VReg, auto-bitcasting if the value type doesn't match the variable's
    /// declared type. Creates the variable if it doesn't exist, using `val_type` as default type.
    fn def_var_cast(
        &mut self,
        vreg: VReg,
        builder: &mut FunctionBuilder,
        val: cranelift_codegen::ir::Value,
        val_type: cranelift_codegen::ir::Type,
    ) {
        let actual = self.declared_type(vreg).unwrap_or(val_type);
        let var = self.get(vreg, builder, actual);
        if actual == val_type {
            builder.def_var(var, val);
        } else if actual == cl_types::F64 && val_type == cl_types::I64 {
            let cast = builder.ins().bitcast(cl_types::F64, MemFlags::new(), val);
            builder.def_var(var, cast);
        } else if actual == cl_types::I64 && val_type == cl_types::F64 {
            let cast = builder.ins().bitcast(cl_types::I64, MemFlags::new(), val);
            builder.def_var(var, cast);
        } else {
            builder.def_var(var, val);
        }
    }

    /// Load a variable's value as the target type, inserting a bitcast if needed.
    /// Handles both F64→I64 (sending to runtime) and I64→F64 (loading for native float ops).
    fn use_as(
        &self,
        vreg: VReg,
        target: cranelift_codegen::ir::Type,
        builder: &mut FunctionBuilder,
    ) -> cranelift_codegen::ir::Value {
        let var = self.map[&vreg];
        let val = builder.use_var(var);
        let actual = self.types.get(&vreg).copied().unwrap_or(cl_types::I64);
        if actual == target {
            val
        } else if actual == cl_types::F64 && target == cl_types::I64 {
            builder.ins().bitcast(cl_types::I64, MemFlags::new(), val)
        } else if actual == cl_types::I64 && target == cl_types::F64 {
            builder.ins().bitcast(cl_types::F64, MemFlags::new(), val)
        } else {
            val
        }
    }
}

pub struct CraneliftBackend {
    module: Option<ObjectModule>,
    /// Declared extern functions: name → FuncId (#262)
    extern_funcs: HashMap<String, FuncId>,
    /// Declared internal functions: SymbolId(u32) → FuncId
    internal_funcs: HashMap<u32, FuncId>,
    /// Declared return TypeId per internal function for NaN-boxing promotion
    internal_return_tys: HashMap<u32, TypeId>,
}

impl CraneliftBackend {
    pub fn new() -> crate::error::Result<Self> {
        let mut flags_builder = settings::builder();
        // AOT object files need PIC for macOS linker compatibility
        flags_builder.set("is_pic", "true").unwrap();
        flags_builder.set("opt_level", "speed").unwrap();
        let isa_builder = cranelift_native::builder()
            .map_err(|e| crate::error::MambaError::codegen(format!("no native ISA: {e}")))?;
        let isa = isa_builder
            .finish(settings::Flags::new(flags_builder))
            .map_err(|e| crate::error::MambaError::codegen(format!("ISA error: {e}")))?;
        let obj_builder = ObjectBuilder::new(
            isa,
            "mamba_module",
            cranelift_module::default_libcall_names(),
        )
        .map_err(|e| crate::error::MambaError::codegen(format!("object builder: {e}")))?;
        let module = ObjectModule::new(obj_builder);
        Ok(Self {
            module: Some(module),
            extern_funcs: HashMap::new(),
            internal_funcs: HashMap::new(),
            internal_return_tys: HashMap::new(),
        })
    }

    fn module(&mut self) -> &mut ObjectModule {
        self.module.as_mut().expect("module already consumed")
    }

    fn mamba_to_cl_type(&self, ty: &Ty) -> cranelift_codegen::ir::Type {
        match ty {
            Ty::Int | Ty::Bool => cl_types::I64,
            Ty::Float => cl_types::F64,
            _ => cl_types::I64,
        }
    }

    /// Declare an imported (extern) function for FFI (#262).
    fn declare_extern(&mut self, ext: &MirExtern) -> crate::error::Result<FuncId> {
        let mut sig = Signature::new(CallConv::SystemV);
        for param_ty in &ext.params {
            sig.params
                .push(AbiParam::new(marshal::mir_type_to_cl(param_ty)));
        }
        if ext.return_type != MirType::Void {
            sig.returns
                .push(AbiParam::new(marshal::mir_type_to_cl(&ext.return_type)));
        }
        let func_id = self
            .module()
            .declare_function(&ext.name, Linkage::Import, &sig)
            .map_err(|e| {
                crate::error::MambaError::codegen(format!("declare extern '{}': {e}", ext.name))
            })?;
        self.extern_funcs.insert(ext.name.clone(), func_id);
        Ok(func_id)
    }

    /// Declare an internal function (forward declaration for mutual calls).
    fn declare_internal(
        &mut self,
        body: &MirBody,
        tcx: &TypeContext,
    ) -> crate::error::Result<FuncId> {
        let mut sig = Signature::new(CallConv::SystemV);
        for (_, ty_id) in &body.params {
            sig.params
                .push(AbiParam::new(self.mamba_to_cl_type(tcx.get(*ty_id))));
        }
        sig.returns.push(AbiParam::new(
            self.mamba_to_cl_type(tcx.get(body.return_ty)),
        ));
        let func_name = format!("_mb_{}", body.name.0);
        let func_id = self
            .module()
            .declare_function(&func_name, Linkage::Export, &sig)
            .map_err(|e| crate::error::MambaError::codegen(format!("declare: {e}")))?;
        self.internal_funcs.insert(body.name.0, func_id);
        self.internal_return_tys.insert(body.name.0, body.return_ty);
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
                .push(AbiParam::new(self.mamba_to_cl_type(tcx.get(*ty_id))));
        }
        let ret_ty = self.mamba_to_cl_type(tcx.get(body.return_ty));
        sig.returns.push(AbiParam::new(ret_ty));

        let mut func = Function::with_name_signature(
            cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32()),
            sig,
        );
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut fb_ctx);
        let mut vars = VarAlloc::new();

        // Map MIR BlockIds to Cranelift blocks by ID (not by array index).
        let mut cl_blocks: HashMap<u32, cranelift_codegen::ir::Block> = HashMap::new();
        for block in &body.blocks {
            let cl_block = builder.create_block();
            cl_blocks.insert(block.id.0, cl_block);
        }

        let entry_cl = cl_blocks[&body.blocks[0].id.0];
        builder.append_block_params_for_function_params(entry_cl);
        builder.switch_to_block(entry_cl);

        let mut param_vregs = HashSet::new();
        for (i, (vreg, ty_id)) in body.params.iter().enumerate() {
            let cl_type = self.mamba_to_cl_type(tcx.get(*ty_id));
            let var = vars.get(*vreg, &mut builder, cl_type);
            let param_val = builder.block_params(entry_cl)[i];
            builder.def_var(var, param_val);
            param_vregs.insert(*vreg);
        }

        // Resolve mb_release_value FuncRef for return-time cleanup (#1129 R3).
        // #1663 T4c5 iter-5 (mitigation): re-apply the `is_entry_body` guard.
        // T4c4 dropped it to enable __main__ release for the bench-loop perf
        // gain, but this surfaced a residual UAF that crashes the `stdlib/re/*`
        // conformance fixtures (3-4 SIGSEGV/SIGABRT post-print, in the JIT
        // __main__ release loop). Iterations 1-4 (registry retain in T4c3 +
        // itertools borrowed-element retain + kind-byte UAF safety net) closed
        // multiple real bugs but did not close the residual. Iter-5 localized
        // the crash to this loop: with the guard re-applied, conformance gate
        // passes 691/0 (5/5 standalone clean exits on re/ops_broad.py).
        // The original perf motivation (#1274) is already met via a separate
        // idempotency fix in `mb_register_builtins` (commit 28cb58070), so the
        // bench's 4× ratio gate is green without entry-body release. The
        // residual UAF is a structurally distinct bug (likely BigInt-internal
        // Vec<u64> digit storage being freed via a separate path before the
        // legitimate single MbObject release) that warrants its own scope.
        let is_entry_body = body.name.0 == u32::MAX;
        let release_func_ref = if EMIT_REFCOUNT_CALLS && !is_entry_body {
            let release_id = self.extern_funcs.get("mb_release_value").copied();
            release_id.map(|id| self.module().declare_func_in_func(id, builder.func))
        } else {
            None
        };
        // Resolve mb_retain_value FuncRef for returning borrowed parameters:
        // when `return self` transfers a parameter to the caller, the callee
        // had borrowed ownership; the caller now wants owned, so retain.
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
        let mut ctx = cranelift_codegen::Context::for_function(func);
        self.module()
            .define_function(func_id, &mut ctx)
            .map_err(|e| crate::error::MambaError::codegen(format!("define: {e}")))?;
        Ok(())
    }

    fn emit_inst(
        &mut self,
        inst: &MirInst,
        tcx: &TypeContext,
        externs: &[MirExtern],
        builder: &mut FunctionBuilder,
        vars: &mut VarAlloc,
    ) {
        match inst {
            MirInst::LoadConst { dest, value, ty } => {
                let cl_type = self.mamba_to_cl_type(tcx.get(*ty));
                let var = vars.get(*dest, builder, cl_type);
                let val = match value {
                    MirConst::Int(v) => builder.ins().iconst(cl_types::I64, *v),
                    MirConst::BigInt(s) => {
                        let val = crate::runtime::bigint_ops::bigint_immortal_from_literal(s);
                        builder.ins().iconst(cl_types::I64, val.to_bits() as i64)
                    }
                    MirConst::Float(v) => builder.ins().f64const(*v),
                    MirConst::Bool(v) => builder.ins().iconst(cl_types::I64, *v as i64),
                    MirConst::None => builder.ins().iconst(cl_types::I64, 0),
                    MirConst::NotImplemented => builder
                        .ins()
                        .iconst(cl_types::I64, MbValue::not_implemented().to_bits() as i64),
                    MirConst::Ellipsis => builder
                        .ins()
                        .iconst(cl_types::I64, MbValue::ellipsis().to_bits() as i64),
                    MirConst::Str(s) => {
                        // Use immortal refcount for compile-time string constants (#1129 R4).
                        let str_val = if let Some(codepoints) =
                            crate::lexer::token::decode_surrogate_escape_markers(s)
                        {
                            MbValue::from_ptr(
                                crate::runtime::string_ops::new_surrogate_codepoints_str_immortal(
                                    codepoints,
                                ),
                            )
                        } else {
                            MbValue::from_ptr(MbObject::new_str_immortal(s.clone()))
                        };
                        builder
                            .ins()
                            .iconst(cl_types::I64, str_val.to_bits() as i64)
                    }
                    MirConst::Bytes(data) => {
                        // Use immortal refcount for compile-time bytes constants (#1129 R4).
                        let bytes_val =
                            MbValue::from_ptr(MbObject::new_bytes_immortal(data.clone()));
                        builder
                            .ins()
                            .iconst(cl_types::I64, bytes_val.to_bits() as i64)
                    }
                    MirConst::FuncRef(_) | MirConst::ExternFuncRef(_) => {
                        builder.ins().iconst(cl_types::I64, 0)
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
                // Dispatch based on opcode semantics, not just result type:
                // - Is/IsNot: always identity comparison (primitive)
                // - In/NotIn: always runtime dispatch (mb_obj_contains on RHS)
                // - Arithmetic/comparison on primitive types: emit native instructions
                // - Arithmetic/comparison on object types: dunder dispatch
                let use_primitive = match op {
                    MirBinOp::Is | MirBinOp::IsNot => true,
                    MirBinOp::In | MirBinOp::NotIn => false,
                    _ => matches!(resolved_ty, Ty::Int | Ty::Float | Ty::Bool),
                };
                if matches!(op, MirBinOp::In | MirBinOp::NotIn) {
                    // `in` / `not in`: call mb_obj_contains(rhs, lhs) — RHS is the container
                    if let Some(&func_id) = self.extern_funcs.get("mb_obj_contains") {
                        let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                        let rv = vars.get(*rhs, builder, cl_types::I64);
                        let lv = vars.get(*lhs, builder, cl_types::I64);
                        let r = builder.use_var(rv); // container (RHS)
                        let l = builder.use_var(lv); // item (LHS)
                        let call = builder.ins().call(func_ref, &[r, l]);
                        let result = builder.inst_results(call)[0];
                        let dv = vars.get(*dest, builder, cl_types::I64);
                        // For `not in`, negate the result
                        let final_result = if matches!(op, MirBinOp::NotIn) {
                            let one = builder.ins().iconst(cl_types::I64, 1);
                            builder.ins().bxor(result, one)
                        } else {
                            result
                        };
                        builder.def_var(dv, final_result);
                    } else {
                        let zero = builder.ins().iconst(cl_types::I64, 0);
                        let dv = vars.get(*dest, builder, cl_types::I64);
                        builder.def_var(dv, zero);
                    }
                } else if (matches!(op, MirBinOp::FloorDiv) || matches!(op, MirBinOp::Mod))
                    && matches!(resolved_ty, Ty::Int | Ty::Float | Ty::Bool)
                {
                    // Floor division / modulo → call mb_floordiv / mb_mod runtime
                    // rather than emitting an inline `sdiv` / `srem`. This mirrors
                    // the JIT path (#1085, #35): the inline integer path operated
                    // on the raw tagged bits (so `10**30 % 7` saw the BigInt's
                    // pointer bits, not its value), `x % 0` hardware-trapped
                    // (SIGILL) instead of raising a catchable ZeroDivisionError,
                    // and the inline float `%` returned NaN for `1.0 % 0.0`
                    // instead of raising ZeroDivisionError. The runtime helpers
                    // handle BigInt, floor semantics, and the correct
                    // ZeroDivisionError messages.
                    // Float operands are already NaN-boxed I64 MbValues — no boxing
                    // needed. Int/Bool operands need boxing from raw I64 to MbValue.
                    let is_mod = matches!(op, MirBinOp::Mod);
                    let helper_name = if is_mod { "mb_mod" } else { "mb_floordiv" };
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
                    if let Some(&func_id) = self.extern_funcs.get(helper_name) {
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
                        // For Int/Bool operands: mb_mod / mb_floordiv return a
                        // NaN-boxed MbValue, but subsequent primitive ops expect a
                        // raw i64. Unbox inline-int results (tag=1) to raw i64, but
                        // keep BigInt results NaN-boxed so they flow back through
                        // the runtime correctly.
                        if !is_float {
                            let tag_raw = builder.ins().ushr_imm(result_bits, 48);
                            let tag = builder.ins().band_imm(tag_raw, 7);
                            let tag_int = builder.ins().iconst(cl_types::I64, 1);
                            let is_inline = builder.ins().icmp(IntCC::Equal, tag, tag_int);
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
                } else if use_primitive {
                    let cl_type = self.mamba_to_cl_type(resolved_ty);
                    let lv = vars.get(*lhs, builder, cl_type);
                    let rv = vars.get(*rhs, builder, cl_type);
                    let dv = vars.get(*dest, builder, cl_type);
                    let l = builder.use_var(lv);
                    let r = builder.use_var(rv);
                    let result = emit_binop(builder, op, resolved_ty, l, r);
                    builder.def_var(dv, result);
                    // Mark comparison results as native 0/1 bools so branch
                    // codegen can skip the redundant band_imm(val, 1).
                    if matches!(
                        op,
                        MirBinOp::Eq
                            | MirBinOp::NotEq
                            | MirBinOp::Lt
                            | MirBinOp::Gt
                            | MirBinOp::LtEq
                            | MirBinOp::GtEq
                            | MirBinOp::Is
                            | MirBinOp::IsNot
                    ) {
                        vars.native_bools.insert(*dest);
                    }
                } else if let Some(&func_id) = self.extern_funcs.get("mb_dispatch_binop") {
                    // Object type: dispatch through dunder methods
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let opcode = builder.ins().iconst(cl_types::I64, op.to_opcode());
                    let lv = vars.get(*lhs, builder, cl_types::I64);
                    let rv = vars.get(*rhs, builder, cl_types::I64);
                    let l = builder.use_var(lv);
                    let r = builder.use_var(rv);
                    let call = builder.ins().call(func_ref, &[opcode, l, r]);
                    let result = builder.inst_results(call)[0];
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, result);
                } else {
                    let cl_type = self.mamba_to_cl_type(resolved_ty);
                    let lv = vars.get(*lhs, builder, cl_type);
                    let rv = vars.get(*rhs, builder, cl_type);
                    let dv = vars.get(*dest, builder, cl_type);
                    let l = builder.use_var(lv);
                    let r = builder.use_var(rv);
                    let result = emit_binop(builder, op, resolved_ty, l, r);
                    builder.def_var(dv, result);
                }
            }
            MirInst::Copy { dest, source } => {
                let sv = vars.get(*source, builder, cl_types::I64);
                let dv = vars.get(*dest, builder, cl_types::I64);
                if EMIT_REFCOUNT_CALLS {
                    // Release old value of dest before overwriting (#1129 R2).
                    if let Some(&release_id) = self.extern_funcs.get("mb_release_value") {
                        let release_ref =
                            self.module().declare_func_in_func(release_id, builder.func);
                        let old_val = builder.use_var(dv);
                        builder.ins().call(release_ref, &[old_val]);
                    }
                }
                let val = builder.use_var(sv);
                builder.def_var(dv, val);
                if EMIT_REFCOUNT_CALLS {
                    // Retain the new value after copy (#1129 R2).
                    if let Some(&retain_id) = self.extern_funcs.get("mb_retain_value") {
                        let retain_ref =
                            self.module().declare_func_in_func(retain_id, builder.func);
                        builder.ins().call(retain_ref, &[val]);
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
                    let cl_type = self.mamba_to_cl_type(resolved_ty);
                    let ov = vars.get(*operand, builder, cl_type);
                    let dv = vars.get(*dest, builder, cl_type);
                    let val = builder.use_var(ov);
                    let result = match op {
                        crate::mir::MirUnaryOp::Pos => val,
                        crate::mir::MirUnaryOp::Neg => {
                            if matches!(resolved_ty, Ty::Float) {
                                builder.ins().fneg(val)
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
                            emit_logical_not(builder, truth_value)
                        }
                        crate::mir::MirUnaryOp::BitNot => builder.ins().bnot(val),
                    };
                    builder.def_var(dv, result);
                    if matches!(op, crate::mir::MirUnaryOp::Not) {
                        vars.raw_ints.insert(*dest);
                        vars.native_bools.insert(*dest);
                    }
                } else if let Some(&func_id) = self.extern_funcs.get("mb_dispatch_unaryop") {
                    // Object type: dispatch through dunder methods
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let opcode = builder.ins().iconst(cl_types::I64, op.to_opcode());
                    let ov = vars.get(*operand, builder, cl_types::I64);
                    let val = builder.use_var(ov);
                    let call = builder.ins().call(func_ref, &[opcode, val]);
                    let result = builder.inst_results(call)[0];
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, result);
                } else {
                    // Fallback: primitive ops
                    let cl_type = self.mamba_to_cl_type(resolved_ty);
                    let ov = vars.get(*operand, builder, cl_type);
                    let dv = vars.get(*dest, builder, cl_type);
                    let val = builder.use_var(ov);
                    let result = match op {
                        crate::mir::MirUnaryOp::Pos => val,
                        crate::mir::MirUnaryOp::Neg => {
                            if matches!(resolved_ty, Ty::Float) {
                                builder.ins().fneg(val)
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
                            emit_logical_not(builder, truth_value)
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
            // Object operations — emit FFI calls to mb_* runtime
            MirInst::GetAttr {
                dest,
                object,
                ref attr,
                ty: _,
            } => {
                if let Some(&func_id) = self.extern_funcs.get("mb_getattr") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let obj_v = vars.get(*object, builder, cl_types::I64);
                    let obj_val = builder.use_var(obj_v);
                    let attr_str = MbValue::from_ptr(MbObject::new_str(attr.clone()));
                    let attr_val = builder
                        .ins()
                        .iconst(cl_types::I64, attr_str.to_bits() as i64);
                    let call = builder.ins().call(func_ref, &[obj_val, attr_val]);
                    let result = builder.inst_results(call)[0];
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, result);
                } else {
                    let zero = builder.ins().iconst(cl_types::I64, 0);
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, zero);
                }
            }
            MirInst::SetAttr {
                object,
                ref attr,
                value,
            } => {
                if let Some(&func_id) = self.extern_funcs.get("mb_setattr") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let ov = vars.get(*object, builder, cl_types::I64);
                    let vv = vars.get(*value, builder, cl_types::I64);
                    let obj_val = builder.use_var(ov);
                    let attr_str = MbValue::from_ptr(MbObject::new_str(attr.clone()));
                    let attr_val = builder
                        .ins()
                        .iconst(cl_types::I64, attr_str.to_bits() as i64);
                    let val = builder.use_var(vv);
                    builder.ins().call(func_ref, &[obj_val, attr_val, val]);
                }
            }
            MirInst::GetItem {
                dest,
                object,
                index,
                ty: _,
            } => {
                if let Some(&func_id) = self.extern_funcs.get("mb_obj_getitem") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let ov = vars.get(*object, builder, cl_types::I64);
                    let iv = vars.get(*index, builder, cl_types::I64);
                    let obj_val = builder.use_var(ov);
                    let idx_val = builder.use_var(iv);
                    let call = builder.ins().call(func_ref, &[obj_val, idx_val]);
                    let result = builder.inst_results(call)[0];
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, result);
                } else {
                    let zero = builder.ins().iconst(cl_types::I64, 0);
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, zero);
                }
            }
            MirInst::SetItem {
                object,
                index,
                value,
            } => {
                if let Some(&func_id) = self.extern_funcs.get("mb_obj_setitem") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let ov = vars.get(*object, builder, cl_types::I64);
                    let iv = vars.get(*index, builder, cl_types::I64);
                    let vv = vars.get(*value, builder, cl_types::I64);
                    let obj_val = builder.use_var(ov);
                    let idx_val = builder.use_var(iv);
                    let val = builder.use_var(vv);
                    builder.ins().call(func_ref, &[obj_val, idx_val, val]);
                }
            }
            MirInst::MakeList {
                dest,
                elements,
                ty: _,
            } => {
                if let (Some(&new_id), Some(&append_id)) = (
                    self.extern_funcs.get("mb_list_new"),
                    self.extern_funcs.get("mb_list_append"),
                ) {
                    let new_ref = self.module().declare_func_in_func(new_id, builder.func);
                    let call = builder.ins().call(new_ref, &[]);
                    let list_val = builder.inst_results(call)[0];
                    let app_ref = self.module().declare_func_in_func(append_id, builder.func);
                    for elem in elements {
                        let ev = vars.get(*elem, builder, cl_types::I64);
                        let elem_val = builder.use_var(ev);
                        builder.ins().call(app_ref, &[list_val, elem_val]);
                    }
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, list_val);
                } else {
                    let zero = builder.ins().iconst(cl_types::I64, 0);
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, zero);
                }
            }
            MirInst::MakeDict {
                dest,
                keys,
                values,
                ty: _,
            } => {
                if let (Some(&new_id), Some(&set_id)) = (
                    self.extern_funcs.get("mb_dict_new"),
                    self.extern_funcs.get("mb_dict_setitem"),
                ) {
                    let new_ref = self.module().declare_func_in_func(new_id, builder.func);
                    let call = builder.ins().call(new_ref, &[]);
                    let dict_val = builder.inst_results(call)[0];
                    let set_ref = self.module().declare_func_in_func(set_id, builder.func);
                    for (k, v) in keys.iter().zip(values.iter()) {
                        let kv = vars.get(*k, builder, cl_types::I64);
                        let vv = vars.get(*v, builder, cl_types::I64);
                        let key_val = builder.use_var(kv);
                        let val_val = builder.use_var(vv);
                        builder.ins().call(set_ref, &[dict_val, key_val, val_val]);
                    }
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, dict_val);
                } else {
                    let zero = builder.ins().iconst(cl_types::I64, 0);
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, zero);
                }
            }
            MirInst::MakeTuple {
                dest,
                elements,
                ty: _,
            } => {
                // Build as list, then convert to tuple
                if let (Some(&new_id), Some(&append_id), Some(&convert_id)) = (
                    self.extern_funcs.get("mb_list_new"),
                    self.extern_funcs.get("mb_list_append"),
                    self.extern_funcs.get("mb_list_to_tuple"),
                ) {
                    let new_ref = self.module().declare_func_in_func(new_id, builder.func);
                    let call = builder.ins().call(new_ref, &[]);
                    let list_val = builder.inst_results(call)[0];
                    let app_ref = self.module().declare_func_in_func(append_id, builder.func);
                    for elem in elements {
                        let ev = vars.get(*elem, builder, cl_types::I64);
                        let elem_val = builder.use_var(ev);
                        builder.ins().call(app_ref, &[list_val, elem_val]);
                    }
                    let conv_ref = self.module().declare_func_in_func(convert_id, builder.func);
                    let conv_call = builder.ins().call(conv_ref, &[list_val]);
                    let tuple_val = builder.inst_results(conv_call)[0];
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, tuple_val);
                } else {
                    let zero = builder.ins().iconst(cl_types::I64, 0);
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, zero);
                }
            }
            MirInst::Raise { value } => {
                if let Some(vreg) = value {
                    let v = vars.get(*vreg, builder, cl_types::I64);
                    let _val = builder.use_var(v);
                }
                builder
                    .ins()
                    .trap(cranelift_codegen::ir::TrapCode::user(1).unwrap());
                let dead = builder.create_block();
                builder.switch_to_block(dead);
                builder.seal_block(dead);
            }
            MirInst::LoadGlobal { dest, name, .. } => {
                if let Some(&func_id) = self.extern_funcs.get("mb_global_get_id") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let id_val = builder.ins().iconst(cl_types::I64, name.0 as i64);
                    let call = builder.ins().call(func_ref, &[id_val]);
                    let result = builder.inst_results(call)[0];
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, result);
                }
            }
            MirInst::StoreGlobal { name, value } => {
                // mb_global_set_id owns retaining the new value and releasing the overwritten value.
                if let Some(&func_id) = self.extern_funcs.get("mb_global_set_id") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let id_val = builder.ins().iconst(cl_types::I64, name.0 as i64);
                    let vv = vars.get(*value, builder, cl_types::I64);
                    let val = builder.use_var(vv);
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
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, result);
                }
            }
            MirInst::StoreCell { cell_idx, value } => {
                // mb_cell_set owns retaining the new value and releasing the overwritten value.
                if let Some(&func_id) = self.extern_funcs.get("mb_cell_set") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let idx_val = builder.ins().iconst(cl_types::I64, *cell_idx as i64);
                    let vv = vars.get(*value, builder, cl_types::I64);
                    let val = builder.use_var(vv);
                    builder.ins().call(func_ref, &[idx_val, val]);
                }
            }
            MirInst::MakeCell { dest, value, .. } => {
                if let Some(&func_id) = self.extern_funcs.get("mb_cell_new") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let vv = vars.get(*value, builder, cl_types::I64);
                    let val = builder.use_var(vv);
                    let call = builder.ins().call(func_ref, &[val]);
                    let result = builder.inst_results(call)[0];
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, result);
                }
            }
            MirInst::LoadCapture {
                dest, capture_idx, ..
            } => {
                if let Some(&func_id) = self.extern_funcs.get("mb_closure_get_capture") {
                    let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                    let closure_var = vars.get(VReg(0), builder, cl_types::I64);
                    let closure_val = builder.use_var(closure_var);
                    let idx_val = builder.ins().iconst(cl_types::I64, *capture_idx as i64);
                    let call = builder.ins().call(func_ref, &[closure_val, idx_val]);
                    let result = builder.inst_results(call)[0];
                    let dv = vars.get(*dest, builder, cl_types::I64);
                    builder.def_var(dv, result);
                }
            }
            // AOT backend: fall back to wrapping arithmetic (no BigInt promotion).
            MirInst::CheckedAdd { dest, lhs, rhs, ty } => {
                let lv = vars.get(*lhs, builder, cl_types::I64);
                let rv = vars.get(*rhs, builder, cl_types::I64);
                let l = builder.use_var(lv);
                let r = builder.use_var(rv);
                let result = builder.ins().iadd(l, r);
                let cl_type = self.mamba_to_cl_type(tcx.get(*ty));
                let dv = vars.get(*dest, builder, cl_type);
                builder.def_var(dv, result);
            }
            MirInst::CheckedSub { dest, lhs, rhs, ty } => {
                let lv = vars.get(*lhs, builder, cl_types::I64);
                let rv = vars.get(*rhs, builder, cl_types::I64);
                let l = builder.use_var(lv);
                let r = builder.use_var(rv);
                let result = builder.ins().isub(l, r);
                let cl_type = self.mamba_to_cl_type(tcx.get(*ty));
                let dv = vars.get(*dest, builder, cl_type);
                builder.def_var(dv, result);
            }
            MirInst::CheckedMul { dest, lhs, rhs, ty } => {
                let lv = vars.get(*lhs, builder, cl_types::I64);
                let rv = vars.get(*rhs, builder, cl_types::I64);
                let l = builder.use_var(lv);
                let r = builder.use_var(rv);
                let result = builder.ins().imul(l, r);
                let cl_type = self.mamba_to_cl_type(tcx.get(*ty));
                let dv = vars.get(*dest, builder, cl_type);
                builder.def_var(dv, result);
            }
        }
    }

    /// Emit an internal function call (#262).
    fn emit_internal_call(
        &mut self,
        dest: &Option<VReg>,
        sym_id: u32,
        args: &[VReg],
        ty: &crate::types::ty::TypeId,
        tcx: &TypeContext,
        builder: &mut FunctionBuilder,
        vars: &mut VarAlloc,
    ) {
        if let Some(&callee_id) = self.internal_funcs.get(&sym_id) {
            let func_ref = self.module().declare_func_in_func(callee_id, builder.func);
            let arg_vals: Vec<_> = args
                .iter()
                .map(|a| {
                    let v = vars.get(*a, builder, cl_types::I64);
                    builder.use_var(v)
                })
                .collect();
            let call = builder.ins().call(func_ref, &arg_vals);
            if let Some(dest_vreg) = dest {
                let cl_type = self.mamba_to_cl_type(tcx.get(*ty));
                let var = vars.get(*dest_vreg, builder, cl_type);
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
                        let box_fn_name = match callee_ty {
                            Ty::Bool => "mb_box_bool",
                            Ty::Float => "mb_box_float",
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
                    } else {
                        result
                    }
                } else {
                    result
                };
                builder.def_var(var, boxed);
            }
        } else if let Some(dest_vreg) = dest {
            let cl_type = self.mamba_to_cl_type(tcx.get(*ty));
            let var = vars.get(*dest_vreg, builder, cl_type);
            let zero = builder.ins().iconst(cl_types::I64, 0);
            builder.def_var(var, zero);
        }
    }

    /// Emit an FFI extern call with marshaling (#262, #263, #264).
    #[allow(clippy::too_many_arguments)]
    fn emit_extern_call(
        &mut self,
        dest: &Option<VReg>,
        name: &str,
        args: &[VReg],
        ty: &crate::types::ty::TypeId,
        tcx: &TypeContext,
        externs: &[MirExtern],
        builder: &mut FunctionBuilder,
        vars: &mut VarAlloc,
    ) {
        let ext = externs.iter().find(|e| e.name == name);
        if let Some(&func_id) = self.extern_funcs.get(name) {
            let func_ref = self.module().declare_func_in_func(func_id, builder.func);

            // Marshal arguments (#263)
            let arg_vals: Vec<_> = args
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    let mamba_ty = cl_types::I64;
                    let v = vars.get(*a, builder, mamba_ty);
                    let val = builder.use_var(v);
                    if let Some(ext) = ext {
                        if i < ext.params.len() {
                            return marshal::marshal_arg(builder, val, mamba_ty, &ext.params[i]);
                        }
                    }
                    val
                })
                .collect();

            let call = builder.ins().call(func_ref, &arg_vals);

            // Unmarshal return value (#264)
            if let Some(dest_vreg) = dest {
                let cl_type = self.mamba_to_cl_type(tcx.get(*ty));
                let var = vars.get(*dest_vreg, builder, cl_type);
                if let Some(ext) = ext {
                    if ext.return_type != MirType::Void {
                        let raw = builder.inst_results(call)[0];
                        let val =
                            marshal::unmarshal_return(builder, raw, &ext.return_type, cl_type);
                        builder.def_var(var, val);
                    } else {
                        let zero = builder.ins().iconst(cl_types::I64, 0);
                        builder.def_var(var, zero);
                    }
                } else {
                    let zero = builder.ins().iconst(cl_types::I64, 0);
                    builder.def_var(var, zero);
                }
            }
        } else if let Some(dest_vreg) = dest {
            let cl_type = self.mamba_to_cl_type(tcx.get(*ty));
            let var = vars.get(*dest_vreg, builder, cl_type);
            let zero = builder.ins().iconst(cl_types::I64, 0);
            builder.def_var(var, zero);
        }
    }
}

fn emit_logical_not(
    builder: &mut FunctionBuilder,
    truth_value: cranelift_codegen::ir::Value,
) -> cranelift_codegen::ir::Value {
    let zero = builder.ins().iconst(cl_types::I64, 0);
    let is_false = builder.ins().icmp(IntCC::Equal, truth_value, zero);
    builder.ins().uextend(cl_types::I64, is_false)
}

/// Emit a block terminator.
fn emit_terminator(
    term: &Terminator,
    cl_blocks: &HashMap<u32, cranelift_codegen::ir::Block>,
    ret_ty: cranelift_codegen::ir::Type,
    builder: &mut FunctionBuilder,
    vars: &mut VarAlloc,
    release_func_ref: Option<cranelift_codegen::ir::FuncRef>,
    retain_func_ref: Option<cranelift_codegen::ir::FuncRef>,
    param_vregs: &HashSet<VReg>,
) {
    match term {
        Terminator::Return(Some(vreg)) => {
            // Release all local variables except the return value and
            // function parameters. Parameters are BORROWED from the caller —
            // the caller owns and releases them. Double-releasing causes UAF.
            // Also skip raw_ints VRegs: mb_release_value is a no-op for
            // non-pointer values (it as_ptr-checks the NaN tag), so the FFI
            // call is pure overhead on raw i64s — this is the dominant
            // per-call cost in the recursion benches (fib/factorial).
            //
            // #1663 T4c5 iter-5 conclusion: this release loop double-frees
            // some object when `body.name.0 == u32::MAX` (the JIT __main__
            // entry body) — confirmed by re-applying the `is_entry_body`
            // guard upstream (see line 322 in the FuncRef resolution above):
            // gate goes from 687-688 / 3-4 oscillating to 691 / 0 stable.
            // Iter-3 hypothesised BigInt VReg sharing; iter-4 falsified the
            // simple-double-free-of-MbObject hypothesis by promoting the
            // `mb_release_value` kind-byte UAF guard to release-mode (no
            // effect, still 5/5 crashes). Leading hypothesis for follow-up:
            // BigInt-internal `Vec<u64>` digit storage is freed via a
            // separate path before the legitimate single MbObject release;
            // when `Box::from_raw(obj)` drops the BigInt at __main__
            // epilogue, the inner Vec's Drop trips `tiny_free_no_lock`.
            // The mitigation in iter-5 reverts T4c4 (re-skips entry body
            // release), but the original perf motivation (#1274) is already
            // met via the idempotency fix in `mb_register_builtins`
            // (28cb58070), so the 4× bench gate stays green.
            if let Some(release_ref) = release_func_ref {
                for v in vars.i64_vregs() {
                    if v != *vreg && !param_vregs.contains(&v) && !vars.raw_ints.contains(&v) {
                        let var = vars.get(v, builder, cl_types::I64);
                        let val = builder.use_var(var);
                        builder.ins().call(release_ref, &[val]);
                    }
                }
            }
            // Return value may be F64 (float) but function signature returns I64 (MbValue).
            // Use actual declared type and bitcast if needed.
            let actual_type = vars.declared_type(*vreg).unwrap_or(ret_ty);
            let var = vars.get(*vreg, builder, actual_type);
            let val = builder.use_var(var);
            // If returning a parameter (e.g. `return self`), retain before
            // transfer: the caller's CallExtern contract treats the return as
            // an owned new reference, but a parameter is only borrowed inside
            // the callee. Without this retain, the caller under-retains and
            // releases the same refcount twice → UAF on method chains like
            // `c.f().f()` where f returns self.
            //
            // Skip raw_ints params: mb_retain_value is a no-op for non-pointer
            // NaN-boxed values (it as_ptr-checks the tag), and a raw i64 in an
            // raw_ints VReg is by definition not a pointer — the FFI thunk is
            // pure overhead. This hits fib's base-case `return n` path.
            if param_vregs.contains(vreg) && !vars.raw_ints.contains(vreg) {
                if let Some(retain_ref) = retain_func_ref {
                    builder.ins().call(retain_ref, &[val]);
                }
            }
            let ret_val = if actual_type == cl_types::F64 && ret_ty == cl_types::I64 {
                builder.ins().bitcast(cl_types::I64, MemFlags::new(), val)
            } else {
                val
            };
            builder.ins().return_(&[ret_val]);
        }
        Terminator::Return(None) => {
            // Release all local variables except parameters and raw_ints
            // (mb_release_value is a no-op on non-pointer values).
            if let Some(release_ref) = release_func_ref {
                for v in vars.i64_vregs() {
                    if !param_vregs.contains(&v) && !vars.raw_ints.contains(&v) {
                        let var = vars.get(v, builder, cl_types::I64);
                        let val = builder.use_var(var);
                        builder.ins().call(release_ref, &[val]);
                    }
                }
            }
            // Implicit return (bare fallthrough / valueless `return`) must
            // produce Python None — the NaN-boxed none sentinel — not raw 0
            // bits, which callers misread as int 0 (`f() is None` → False,
            // `repr(f())` → '0').
            let none = builder
                .ins()
                .iconst(ret_ty, MbValue::none().to_bits() as i64);
            builder.ins().return_(&[none]);
        }
        Terminator::Goto(target) => {
            builder.ins().jump(cl_blocks[&target.0], &[]);
        }
        Terminator::Branch {
            cond,
            then_block,
            else_block,
        } => {
            // Branch condition may come from a float comparison (rare but possible).
            // Bitcast F64→I64 if needed before truthiness check.
            let actual_type = vars.declared_type(*cond).unwrap_or(cl_types::I64);
            let var = vars.get(*cond, builder, actual_type);
            let raw_val = builder.use_var(var);
            let val = if actual_type == cl_types::F64 {
                builder
                    .ins()
                    .bitcast(cl_types::I64, MemFlags::new(), raw_val)
            } else {
                raw_val
            };
            // Native comparison results (0/1) can branch directly.
            // NaN-boxed bools need LSB extraction via band_imm because the
            // NaN prefix makes the full I64 value nonzero even for `false`.
            let bool_val = if vars.native_bools.contains(cond) {
                val
            } else {
                builder.ins().band_imm(val, 1)
            };
            builder.ins().brif(
                bool_val,
                cl_blocks[&then_block.0],
                &[],
                cl_blocks[&else_block.0],
                &[],
            );
        }
        Terminator::Unreachable => {
            builder
                .ins()
                .trap(cranelift_codegen::ir::TrapCode::user(1).unwrap());
        }
    }
}

/// Emit a binary operation as Cranelift IR instructions.
fn emit_binop(
    builder: &mut FunctionBuilder,
    op: &MirBinOp,
    ty: &Ty,
    l: cranelift_codegen::ir::Value,
    r: cranelift_codegen::ir::Value,
) -> cranelift_codegen::ir::Value {
    match (op, ty) {
        (MirBinOp::Add, Ty::Int) => builder.ins().iadd(l, r),
        (MirBinOp::Sub, Ty::Int) => builder.ins().isub(l, r),
        (MirBinOp::Mul, Ty::Int) => builder.ins().imul(l, r),
        (MirBinOp::Div, Ty::Int) => builder.ins().sdiv(l, r),
        (MirBinOp::Mod, Ty::Int) => {
            // Normalize each operand to a raw 48-bit-sign-extended i64. An
            // Int-typed operand can still carry a NaN-boxed value (e.g. a
            // `len()` / `int()` result whose VReg wasn't raw-ified); the boxed
            // tag bits look negative, which made the floor-sign adjustment
            // below misfire (`5 % len([0,0,0])` → 8 instead of 2). Shifting
            // left 16 then arithmetic-right 16 discards the tag bits and
            // sign-extends bit 47 — an identity for genuine raw inline ints
            // (which already lie in [-2^47, 2^47)). The remainder fits 48 bits,
            // so the raw result is unchanged for the common case.
            let l_sh = builder.ins().ishl_imm(l, 16);
            let ln = builder.ins().sshr_imm(l_sh, 16);
            let r_sh = builder.ins().ishl_imm(r, 16);
            let rn = builder.ins().sshr_imm(r_sh, 16);
            // Python floor-division modulo: result has same sign as divisor.
            // r = srem(a, b); result = (r != 0 && (r ^ b) < 0) ? r + b : r
            let rem = builder.ins().srem(ln, rn);
            let zero = builder.ins().iconst(cl_types::I64, 0);
            let rem_ne_zero = builder.ins().icmp(IntCC::NotEqual, rem, zero);
            let xor = builder.ins().bxor(rem, rn);
            let xor_neg = builder.ins().icmp(IntCC::SignedLessThan, xor, zero);
            let need_adjust = builder.ins().band(rem_ne_zero, xor_neg);
            let adjusted = builder.ins().iadd(rem, rn);
            builder.ins().select(need_adjust, adjusted, rem)
        }
        // Float arithmetic: bitcast I64 (NaN-boxed) → F64, operate, bitcast back.
        (MirBinOp::Add, Ty::Float) => {
            let lf = builder.ins().bitcast(cl_types::F64, MemFlags::new(), l);
            let rf = builder.ins().bitcast(cl_types::F64, MemFlags::new(), r);
            let res = builder.ins().fadd(lf, rf);
            builder.ins().bitcast(cl_types::I64, MemFlags::new(), res)
        }
        (MirBinOp::Sub, Ty::Float) => {
            let lf = builder.ins().bitcast(cl_types::F64, MemFlags::new(), l);
            let rf = builder.ins().bitcast(cl_types::F64, MemFlags::new(), r);
            let res = builder.ins().fsub(lf, rf);
            builder.ins().bitcast(cl_types::I64, MemFlags::new(), res)
        }
        (MirBinOp::Mul, Ty::Float) => {
            let lf = builder.ins().bitcast(cl_types::F64, MemFlags::new(), l);
            let rf = builder.ins().bitcast(cl_types::F64, MemFlags::new(), r);
            let res = builder.ins().fmul(lf, rf);
            builder.ins().bitcast(cl_types::I64, MemFlags::new(), res)
        }
        (MirBinOp::Div, Ty::Float) => {
            let lf = builder.ins().bitcast(cl_types::F64, MemFlags::new(), l);
            let rf = builder.ins().bitcast(cl_types::F64, MemFlags::new(), r);
            let res = builder.ins().fdiv(lf, rf);
            builder.ins().bitcast(cl_types::I64, MemFlags::new(), res)
        }
        (MirBinOp::Mod, Ty::Float) => {
            // Python floor-division modulo for floats: a % b = a - floor(a/b) * b
            let lf = builder.ins().bitcast(cl_types::F64, MemFlags::new(), l);
            let rf = builder.ins().bitcast(cl_types::F64, MemFlags::new(), r);
            let div = builder.ins().fdiv(lf, rf);
            let floored = builder.ins().floor(div);
            let prod = builder.ins().fmul(floored, rf);
            let res = builder.ins().fsub(lf, prod);
            builder.ins().bitcast(cl_types::I64, MemFlags::new(), res)
        }
        // Comparisons. Operand-side type matters: the MIR BinOp's `ty` field
        // is the *result* type (bool), so we inspect the Cranelift SSA value
        // type instead. Float operands need fcmp (IEEE 754 semantics),
        // because icmp on the raw I64 bit pattern of an IEEE 754 float
        // mis-orders negative values — sign-magnitude vs two's complement
        // means `-2.0 < -1.0` would compute False under signed icmp.
        (MirBinOp::Eq, _) => {
            let cmp = if builder.func.dfg.value_type(l) == cl_types::F64 {
                builder.ins().fcmp(FloatCC::Equal, l, r)
            } else {
                builder.ins().icmp(IntCC::Equal, l, r)
            };
            builder.ins().uextend(cl_types::I64, cmp)
        }
        (MirBinOp::NotEq, _) => {
            let cmp = if builder.func.dfg.value_type(l) == cl_types::F64 {
                builder.ins().fcmp(FloatCC::NotEqual, l, r)
            } else {
                builder.ins().icmp(IntCC::NotEqual, l, r)
            };
            builder.ins().uextend(cl_types::I64, cmp)
        }
        (MirBinOp::Lt, _) => {
            let cmp = if builder.func.dfg.value_type(l) == cl_types::F64 {
                builder.ins().fcmp(FloatCC::LessThan, l, r)
            } else {
                builder.ins().icmp(IntCC::SignedLessThan, l, r)
            };
            builder.ins().uextend(cl_types::I64, cmp)
        }
        (MirBinOp::Gt, _) => {
            let cmp = if builder.func.dfg.value_type(l) == cl_types::F64 {
                builder.ins().fcmp(FloatCC::GreaterThan, l, r)
            } else {
                builder.ins().icmp(IntCC::SignedGreaterThan, l, r)
            };
            builder.ins().uextend(cl_types::I64, cmp)
        }
        (MirBinOp::LtEq, _) => {
            let cmp = if builder.func.dfg.value_type(l) == cl_types::F64 {
                builder.ins().fcmp(FloatCC::LessThanOrEqual, l, r)
            } else {
                builder.ins().icmp(IntCC::SignedLessThanOrEqual, l, r)
            };
            builder.ins().uextend(cl_types::I64, cmp)
        }
        (MirBinOp::GtEq, _) => {
            let cmp = if builder.func.dfg.value_type(l) == cl_types::F64 {
                builder.ins().fcmp(FloatCC::GreaterThanOrEqual, l, r)
            } else {
                builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, l, r)
            };
            builder.ins().uextend(cl_types::I64, cmp)
        }
        // Floor division and power
        (MirBinOp::FloorDiv, Ty::Int) => builder.ins().sdiv(l, r),
        (MirBinOp::Pow, Ty::Int) => {
            // Simple integer power — delegated to runtime for real impl
            builder.ins().imul(l, r) // placeholder
        }
        // Logical (short-circuit handled at block level, here just bitwise)
        (MirBinOp::And, _) => builder.ins().band(l, r),
        (MirBinOp::Or, _) => builder.ins().bor(l, r),
        // Bitwise operations
        (MirBinOp::BitAnd, _) => builder.ins().band(l, r),
        (MirBinOp::BitOr, _) => builder.ins().bor(l, r),
        (MirBinOp::BitXor, _) => builder.ins().bxor(l, r),
        (MirBinOp::LShift, _) => builder.ins().ishl(l, r),
        (MirBinOp::RShift, _) => builder.ins().sshr(l, r),
        // Identity/membership — delegated to runtime
        (MirBinOp::Is, _) => {
            let cmp = builder.ins().icmp(IntCC::Equal, l, r);
            builder.ins().uextend(cl_types::I64, cmp)
        }
        (MirBinOp::IsNot, _) => {
            let cmp = builder.ins().icmp(IntCC::NotEqual, l, r);
            builder.ins().uextend(cl_types::I64, cmp)
        }
        (MirBinOp::In, _) | (MirBinOp::NotIn, _) => builder.ins().iconst(cl_types::I64, 0),
        _ => builder.ins().iadd(l, r), // fallback
    }
}

impl CodegenBackend for CraneliftBackend {
    fn codegen(
        &mut self,
        module: &MirModule,
        tcx: &TypeContext,
    ) -> crate::error::Result<CodegenOutput> {
        // Collect which externs are actually used in the MIR
        let used = collect_used_externs(module);

        // Check for runtime-dependent externs (AOT cannot link mb_* symbols)
        let runtime_deps: Vec<&String> =
            used.iter().filter(|name| name.starts_with("mb_")).collect();
        if !runtime_deps.is_empty() {
            let names: Vec<&str> = runtime_deps.iter().map(|s| s.as_str()).collect();
            return Err(crate::error::MambaError::codegen(format!(
                "AOT build requires runtime library for: {}. Use `cclab mamba run` for JIT execution.",
                names.join(", ")
            )));
        }

        // Merge user externs with runtime externs, but only declare used ones
        let rt_externs = crate::runtime::symbols::runtime_externs();
        let all_externs: Vec<MirExtern> = module
            .externs
            .iter()
            .chain(rt_externs.iter())
            .cloned()
            .collect();

        // Phase 1: Declare only actually-used extern functions
        for ext in &all_externs {
            if used.contains(&ext.name) {
                self.declare_extern(ext)?;
            }
        }
        // Phase 1b: Always declare NaN-boxing externs needed for internal call return
        // value promotion — these are emitted by emit_internal_call at codegen time,
        // not as MirInst nodes, so they are not in `used`. mb_mod / mb_floordiv are
        // likewise emitted directly by the BinOp lowering (Mod / FloorDiv route
        // through the runtime for BigInt support and catchable ZeroDivisionError —
        // see emit_inst), so they must always be available even when no MirInst
        // names them.
        let always_declare = [
            "mb_box_int",
            "mb_box_bool",
            "mb_box_float",
            "mb_mod",
            "mb_floordiv",
        ];
        for ext in &all_externs {
            if always_declare.contains(&ext.name.as_str())
                && !self.extern_funcs.contains_key(&ext.name)
            {
                self.declare_extern(ext)?;
            }
        }
        // Phase 2: Forward-declare all internal functions
        for body in &module.bodies {
            self.declare_internal(body, tcx)?;
        }
        // Phase 3: Compile function bodies
        for body in &module.bodies {
            self.compile_function(body, tcx, &all_externs)?;
        }

        // Phase 4: Emit C main() entry point that calls the last function and prints result.
        // Only emit main() when the entry function takes no parameters (real Mamba
        // top-level modules never have params). Functions with params are library
        // code / FFI — no meaningful main entry point.
        if let Some(body) = module.bodies.last() {
            if body.params.is_empty() {
                let entry_func_id = self.internal_funcs[&body.name.0];
                aot::emit_main(self.module.as_mut().unwrap(), entry_func_id)?;
            }
        }

        let obj_module = self.module.take().expect("module already consumed");
        let product = obj_module.finish();
        let bytes = product
            .emit()
            .map_err(|e| crate::error::MambaError::codegen(format!("emit: {e}")))?;
        Ok(CodegenOutput::ObjectFile(bytes))
    }

    fn name(&self) -> &str {
        "cranelift"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{BasicBlock, BlockId, MirBody, MirInst, MirModule, Terminator, VReg};
    use crate::resolve::SymbolId;
    use crate::types::TypeContext;

    mod jit_handle_operator_overload;

    fn empty_module() -> MirModule {
        MirModule::default()
    }

    fn module_with_single_block(stmts: Vec<MirInst>) -> MirModule {
        let block = BasicBlock {
            id: BlockId(0),
            stmts,
            terminator: Terminator::Return(None),
        };
        let body = MirBody {
            name: SymbolId(0),
            params: vec![],
            return_ty: crate::types::TypeContext::new().none(),
            blocks: vec![block],
        };
        MirModule {
            bodies: vec![body],
            externs: vec![],
        }
    }

    fn tcx() -> TypeContext {
        TypeContext::new()
    }

    // --- collect_used_externs ---
    #[test]
    fn test_collect_empty_module() {
        let used = collect_used_externs(&empty_module());
        assert!(used.is_empty());
    }

    #[test]
    fn test_collect_call_extern() {
        let tcx = tcx();
        let inst = MirInst::CallExtern {
            dest: None,
            name: "my_extern_fn".to_string(),
            args: vec![],
            ty: tcx.none(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(used.contains("my_extern_fn"));
    }

    #[test]
    fn test_collect_make_list() {
        let tcx = tcx();
        let inst = MirInst::MakeList {
            dest: VReg(0),
            elements: vec![],
            ty: tcx.none(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(used.contains("mb_list_new"));
        assert!(used.contains("mb_list_append"));
    }

    #[test]
    fn test_collect_make_dict() {
        let tcx = tcx();
        let inst = MirInst::MakeDict {
            dest: VReg(0),
            keys: vec![],
            values: vec![],
            ty: tcx.none(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(used.contains("mb_dict_new"));
        assert!(used.contains("mb_dict_setitem"));
    }

    #[test]
    fn test_collect_get_attr() {
        let tcx = tcx();
        let inst = MirInst::GetAttr {
            dest: VReg(0),
            object: VReg(1),
            attr: "foo".to_string(),
            ty: tcx.none(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(used.contains("mb_getattr"));
    }

    #[test]
    fn test_collect_set_attr() {
        let inst = MirInst::SetAttr {
            object: VReg(0),
            attr: "x".to_string(),
            value: VReg(1),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(used.contains("mb_setattr"));
    }

    #[test]
    fn test_collect_get_item() {
        let tcx = tcx();
        let inst = MirInst::GetItem {
            dest: VReg(0),
            object: VReg(1),
            index: VReg(2),
            ty: tcx.none(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(used.contains("mb_obj_getitem"));
    }

    #[test]
    fn test_collect_set_item() {
        let inst = MirInst::SetItem {
            object: VReg(0),
            index: VReg(1),
            value: VReg(2),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(used.contains("mb_obj_setitem"));
    }

    #[test]
    fn test_collect_make_tuple() {
        let tcx = tcx();
        let inst = MirInst::MakeTuple {
            dest: VReg(0),
            elements: vec![],
            ty: tcx.none(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(used.contains("mb_list_new"));
        assert!(used.contains("mb_list_append"));
        assert!(used.contains("mb_list_to_tuple"));
    }

    #[test]
    fn test_collect_binop() {
        let tcx = tcx();
        let inst = MirInst::BinOp {
            dest: VReg(0),
            op: MirBinOp::Add,
            lhs: VReg(1),
            rhs: VReg(2),
            ty: tcx.none(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(used.contains("mb_dispatch_binop"));
        assert!(used.contains("mb_obj_contains"));
    }

    #[test]
    fn test_collect_unaryop() {
        let tcx = tcx();
        let inst = MirInst::UnaryOp {
            dest: VReg(0),
            op: crate::mir::MirUnaryOp::Neg,
            operand: VReg(1),
            ty: tcx.none(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(used.contains("mb_dispatch_unaryop"));
    }

    #[test]
    fn test_collect_other_inst_no_insertion() {
        let inst = MirInst::Raise { value: None };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(used.is_empty());
    }

    // --- VarAlloc ---
    #[test]
    fn test_varalloc_new_empty() {
        let va = VarAlloc::new();
        assert!(va.map.is_empty());
        assert_eq!(va.next, 0);
    }

    #[test]
    fn test_varalloc_get_new_and_existing() {
        use cranelift_codegen::ir::types as cl_types;
        use cranelift_codegen::ir::{Function, Signature};
        use cranelift_codegen::isa::CallConv;
        use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};

        let sig = Signature::new(CallConv::SystemV);
        let mut func = Function::with_name_signature(
            cranelift_codegen::ir::UserFuncName::testcase("test"),
            sig,
        );
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut func, &mut fb_ctx);
        let _entry_block = builder.create_block();
        builder.switch_to_block(_entry_block);

        let mut va = VarAlloc::new();
        let v0 = VReg(0);
        let var_a = va.get(v0, &mut builder, cl_types::I64);
        let var_b = va.get(v0, &mut builder, cl_types::I64); // same VReg
        assert_eq!(var_a, var_b); // existing → returns same Variable
        assert_eq!(va.next, 1); // only allocated once

        let v1 = VReg(1);
        let var_c = va.get(v1, &mut builder, cl_types::I64);
        assert_ne!(var_a, var_c); // different VReg → new Variable
        assert_eq!(va.next, 2);
    }

    // ── P1 OOP Conformance Tests (mamba-conformance-p1) ──────────────────────

    // --- T3: getattr/setattr/delattr emit valid IR externs ---

    #[test]
    fn test_p1_t3_getattr_collects_mb_getattr_extern() {
        // T3.1/R3: GetAttr MIR instruction must collect "mb_getattr" as used extern
        let tcx = tcx();
        let inst = MirInst::GetAttr {
            dest: VReg(0),
            object: VReg(1),
            attr: "x".to_string(),
            ty: tcx.none(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(
            used.contains("mb_getattr"),
            "GetAttr must register mb_getattr as used extern for valid IR emission"
        );
    }

    #[test]
    fn test_p1_t3_setattr_collects_mb_setattr_extern() {
        // T3.4/R3: SetAttr MIR instruction must collect "mb_setattr" as used extern
        let inst = MirInst::SetAttr {
            object: VReg(0),
            attr: "x".to_string(),
            value: VReg(1),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(
            used.contains("mb_setattr"),
            "SetAttr must register mb_setattr as used extern for valid IR emission"
        );
    }

    #[test]
    fn test_p1_t3_delattr_extern_call_valid() {
        // T3.5/R3: CallExtern to mb_delattr must be collected
        let tcx = tcx();
        let inst = MirInst::CallExtern {
            dest: None,
            name: "mb_delattr".to_string(),
            args: vec![VReg(0), VReg(1)],
            ty: tcx.none(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(
            used.contains("mb_delattr"),
            "CallExtern to mb_delattr must be collected as used extern"
        );
    }

    #[test]
    fn test_p1_t3_getattr_setattr_delattr_all_collected() {
        // R3: All three attribute builtins must be collected when used together
        let tcx = tcx();
        let stmts = vec![
            MirInst::GetAttr {
                dest: VReg(0),
                object: VReg(10),
                attr: "size".to_string(),
                ty: tcx.none(),
            },
            MirInst::SetAttr {
                object: VReg(10),
                attr: "size".to_string(),
                value: VReg(1),
            },
            MirInst::CallExtern {
                dest: None,
                name: "mb_delattr".to_string(),
                args: vec![VReg(10), VReg(2)],
                ty: tcx.none(),
            },
        ];
        let m = module_with_single_block(stmts);
        let used = collect_used_externs(&m);
        assert!(used.contains("mb_getattr"), "mb_getattr must be collected");
        assert!(used.contains("mb_setattr"), "mb_setattr must be collected");
        assert!(used.contains("mb_delattr"), "mb_delattr must be collected");
    }

    // --- T4: super().method() CallExtern with dest VReg ---

    #[test]
    fn test_p1_t4_call_extern_with_dest_vreg() {
        // T4.1/R4: CallExtern with dest = Some(vreg) must be valid — super() return
        let tcx = tcx();
        let inst = MirInst::CallExtern {
            dest: Some(VReg(5)),
            name: "mb_super_getattr".to_string(),
            args: vec![VReg(0), VReg(1)],
            ty: tcx.int(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(
            used.contains("mb_super_getattr"),
            "super_getattr CallExtern must be collected"
        );
    }

    #[test]
    fn test_p1_t4_call_extern_super_dispatch_chain() {
        // T4.2: A chain of mb_super + mb_super_getattr + method_call must all be collected
        let tcx = tcx();
        let stmts = vec![
            MirInst::CallExtern {
                dest: Some(VReg(0)),
                name: "mb_super".to_string(),
                args: vec![VReg(10), VReg(11)],
                ty: tcx.int(),
            },
            MirInst::CallExtern {
                dest: Some(VReg(1)),
                name: "mb_super_getattr".to_string(),
                args: vec![VReg(0), VReg(12)],
                ty: tcx.int(),
            },
            MirInst::CallExtern {
                dest: Some(VReg(2)),
                name: "mb_call_method1".to_string(),
                args: vec![VReg(1), VReg(11)],
                ty: tcx.int(),
            },
        ];
        let m = module_with_single_block(stmts);
        let used = collect_used_externs(&m);
        assert!(used.contains("mb_super"), "mb_super must be collected");
        assert!(
            used.contains("mb_super_getattr"),
            "mb_super_getattr must be collected"
        );
        assert!(
            used.contains("mb_call_method1"),
            "mb_call_method1 must be collected"
        );
    }

    // --- T1: @classmethod extern collection ---

    #[test]
    fn test_p1_t1_classmethod_new_extern_collected() {
        // R1: mb_classmethod_new CallExtern must be collected for classmethod wrapping
        let tcx = tcx();
        let inst = MirInst::CallExtern {
            dest: Some(VReg(0)),
            name: "mb_classmethod_new".to_string(),
            args: vec![VReg(1)],
            ty: tcx.int(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(
            used.contains("mb_classmethod_new"),
            "mb_classmethod_new must be collected as used extern"
        );
    }

    // --- T2: @property extern collection ---

    #[test]
    fn test_p1_t2_property_new_extern_collected() {
        // R2: mb_property_new CallExtern must be collected for property wrapping
        let tcx = tcx();
        let inst = MirInst::CallExtern {
            dest: Some(VReg(0)),
            name: "mb_property_new".to_string(),
            args: vec![VReg(1)],
            ty: tcx.int(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(
            used.contains("mb_property_new"),
            "mb_property_new must be collected as used extern"
        );
    }

    // --- T5: MRO extern collection ---

    #[test]
    fn test_p1_t5_class_define_multi_extern_collected() {
        // R5: mb_class_define_multi CallExtern must be collected for multiple inheritance
        let tcx = tcx();
        let inst = MirInst::CallExtern {
            dest: None,
            name: "mb_class_define_multi".to_string(),
            args: vec![VReg(0), VReg(1), VReg(2), VReg(3)],
            ty: tcx.none(),
        };
        let m = module_with_single_block(vec![inst]);
        let used = collect_used_externs(&m);
        assert!(
            used.contains("mb_class_define_multi"),
            "mb_class_define_multi must be collected as used extern"
        );
    }
}
