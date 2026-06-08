/// AOT-specific code generation helpers.
///
/// Generates a C-compatible `main` function that calls the Mamba entry point
/// and prints its i64 return value using `putchar` (avoids variadic printf
/// issues on ARM64 macOS).
use cranelift_codegen::ir::{
    condcodes::IntCC, types as cl_types, AbiParam, Function, InstBuilder, Signature,
};
use cranelift_codegen::isa::CallConv;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::ObjectModule;

/// Emit a `_mb_print_i64` helper and a `main` function into the object module.
///
/// `main` calls the Mamba entry function, passes the i64 result to
/// `_mb_print_i64` (which prints the decimal representation via `putchar`),
/// then returns 0.
pub fn emit_main(module: &mut ObjectModule, entry_func_id: FuncId) -> crate::error::Result<()> {
    // Declare putchar: (i32) -> i32
    let mut putchar_sig = Signature::new(CallConv::SystemV);
    putchar_sig.params.push(AbiParam::new(cl_types::I32));
    putchar_sig.returns.push(AbiParam::new(cl_types::I32));
    let putchar_id = module
        .declare_function("putchar", Linkage::Import, &putchar_sig)
        .map_err(|e| crate::error::MambaError::codegen(format!("declare putchar: {e}")))?;

    // Build and define _mb_print_i64(n: i64) -> void
    let print_func_id = emit_print_i64(module, putchar_id)?;

    // Declare main: () -> i32
    let mut main_sig = Signature::new(CallConv::SystemV);
    main_sig.returns.push(AbiParam::new(cl_types::I32));
    let main_id = module
        .declare_function("main", Linkage::Export, &main_sig)
        .map_err(|e| crate::error::MambaError::codegen(format!("declare main: {e}")))?;

    // Build main function body
    let mut func = Function::with_name_signature(
        cranelift_codegen::ir::UserFuncName::user(0, main_id.as_u32()),
        main_sig,
    );
    let mut fb_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut fb_ctx);

    let entry_block = builder.create_block();
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);

    // Call Mamba entry function -> i64 result
    let entry_ref = module.declare_func_in_func(entry_func_id, builder.func);
    let call = builder.ins().call(entry_ref, &[]);
    let result = builder.inst_results(call)[0];

    // Call _mb_print_i64(result)
    let print_ref = module.declare_func_in_func(print_func_id, builder.func);
    builder.ins().call(print_ref, &[result]);

    // putchar('\n')
    let putchar_ref = module.declare_func_in_func(putchar_id, builder.func);
    let newline = builder.ins().iconst(cl_types::I32, 10);
    builder.ins().call(putchar_ref, &[newline]);

    // Return 0
    let zero = builder.ins().iconst(cl_types::I32, 0);
    builder.ins().return_(&[zero]);

    builder.finalize();

    let mut ctx = cranelift_codegen::Context::for_function(func);
    module
        .define_function(main_id, &mut ctx)
        .map_err(|e| crate::error::MambaError::codegen(format!("define main: {e}")))?;

    Ok(())
}

/// Emit `_mb_print_i64(n: i64)` — prints a signed 64-bit integer using putchar.
///
/// Algorithm:
///   if n < 0: putchar('-'); n = -n
///   if n == 0: putchar('0'); return
///   divisor = 1; temp = n
///   while temp >= 10: temp /= 10; divisor *= 10
///   while divisor > 0: digit = n/divisor; putchar(digit+'0'); n %= divisor; divisor /= 10
fn emit_print_i64(module: &mut ObjectModule, putchar_id: FuncId) -> crate::error::Result<FuncId> {
    let mut sig = Signature::new(CallConv::SystemV);
    sig.params.push(AbiParam::new(cl_types::I64));
    let func_id = module
        .declare_function("_mb_print_i64", Linkage::Local, &sig)
        .map_err(|e| crate::error::MambaError::codegen(format!("declare _mb_print_i64: {e}")))?;

    let mut func = Function::with_name_signature(
        cranelift_codegen::ir::UserFuncName::user(0, func_id.as_u32()),
        sig,
    );
    let mut fb_ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut fb_ctx);

    let v_n = Variable::from_u32(0);
    let v_div = Variable::from_u32(1);
    let v_temp = Variable::from_u32(2);
    builder.declare_var(v_n, cl_types::I64);
    builder.declare_var(v_div, cl_types::I64);
    builder.declare_var(v_temp, cl_types::I64);

    let b_entry = builder.create_block();
    let b_neg = builder.create_block();
    let b_after_neg = builder.create_block();
    let b_zero_case = builder.create_block();
    let b_find_div_init = builder.create_block();
    let b_find_div_check = builder.create_block();
    let b_find_div_update = builder.create_block();
    let b_print_check = builder.create_block();
    let b_print_emit = builder.create_block();
    let b_done = builder.create_block();

    builder.append_block_params_for_function_params(b_entry);

    // ── entry: check if negative ──
    builder.switch_to_block(b_entry);
    let param_n = builder.block_params(b_entry)[0];
    builder.def_var(v_n, param_n);
    let zero = builder.ins().iconst(cl_types::I64, 0);
    let is_neg = builder.ins().icmp(IntCC::SignedLessThan, param_n, zero);
    builder.ins().brif(is_neg, b_neg, &[], b_after_neg, &[]);

    // ── neg: print '-', negate n ──
    builder.switch_to_block(b_neg);
    let putchar_ref = module.declare_func_in_func(putchar_id, builder.func);
    let minus = builder.ins().iconst(cl_types::I32, 45); // '-'
    builder.ins().call(putchar_ref, &[minus]);
    let n1 = builder.use_var(v_n);
    let negated = builder.ins().ineg(n1);
    builder.def_var(v_n, negated);
    builder.ins().jump(b_after_neg, &[]);

    // ── after_neg: check if zero ──
    builder.switch_to_block(b_after_neg);
    let n2 = builder.use_var(v_n);
    let zero2 = builder.ins().iconst(cl_types::I64, 0);
    let is_zero = builder.ins().icmp(IntCC::Equal, n2, zero2);
    builder
        .ins()
        .brif(is_zero, b_zero_case, &[], b_find_div_init, &[]);

    // ── zero_case: print '0', done ──
    builder.switch_to_block(b_zero_case);
    let putchar_ref2 = module.declare_func_in_func(putchar_id, builder.func);
    let ch_zero = builder.ins().iconst(cl_types::I32, 48); // '0'
    builder.ins().call(putchar_ref2, &[ch_zero]);
    builder.ins().jump(b_done, &[]);

    // ── find_div_init: divisor = 1, temp = n ──
    builder.switch_to_block(b_find_div_init);
    let one = builder.ins().iconst(cl_types::I64, 1);
    builder.def_var(v_div, one);
    let n3 = builder.use_var(v_n);
    builder.def_var(v_temp, n3);
    builder.ins().jump(b_find_div_check, &[]);

    // ── find_div_check: if temp >= 10 goto update else goto print ──
    builder.switch_to_block(b_find_div_check);
    let temp = builder.use_var(v_temp);
    let ten = builder.ins().iconst(cl_types::I64, 10);
    let ge10 = builder
        .ins()
        .icmp(IntCC::SignedGreaterThanOrEqual, temp, ten);
    builder
        .ins()
        .brif(ge10, b_find_div_update, &[], b_print_check, &[]);

    // ── find_div_update: temp /= 10, divisor *= 10, loop back ──
    builder.switch_to_block(b_find_div_update);
    let t2 = builder.use_var(v_temp);
    let ten2 = builder.ins().iconst(cl_types::I64, 10);
    let new_temp = builder.ins().sdiv(t2, ten2);
    builder.def_var(v_temp, new_temp);
    let d2 = builder.use_var(v_div);
    let ten3 = builder.ins().iconst(cl_types::I64, 10);
    let new_div = builder.ins().imul(d2, ten3);
    builder.def_var(v_div, new_div);
    builder.ins().jump(b_find_div_check, &[]);

    // ── print_check: if divisor > 0 goto emit else done ──
    builder.switch_to_block(b_print_check);
    let div = builder.use_var(v_div);
    let zero3 = builder.ins().iconst(cl_types::I64, 0);
    let div_gt0 = builder.ins().icmp(IntCC::SignedGreaterThan, div, zero3);
    builder.ins().brif(div_gt0, b_print_emit, &[], b_done, &[]);

    // ── print_emit: putchar(n/div + '0'), n %= div, div /= 10, loop ──
    builder.switch_to_block(b_print_emit);
    let n4 = builder.use_var(v_n);
    let d3 = builder.use_var(v_div);
    let digit = builder.ins().sdiv(n4, d3);
    let ascii_0 = builder.ins().iconst(cl_types::I64, 48);
    let ch = builder.ins().iadd(digit, ascii_0);
    let ch32 = builder.ins().ireduce(cl_types::I32, ch);
    let putchar_ref3 = module.declare_func_in_func(putchar_id, builder.func);
    builder.ins().call(putchar_ref3, &[ch32]);
    let n5 = builder.use_var(v_n);
    let d4 = builder.use_var(v_div);
    let new_n = builder.ins().srem(n5, d4);
    builder.def_var(v_n, new_n);
    let d5 = builder.use_var(v_div);
    let ten4 = builder.ins().iconst(cl_types::I64, 10);
    let new_div2 = builder.ins().sdiv(d5, ten4);
    builder.def_var(v_div, new_div2);
    builder.ins().jump(b_print_check, &[]);

    // ── done: return ──
    builder.switch_to_block(b_done);
    builder.ins().return_(&[]);

    builder.seal_all_blocks();
    builder.finalize();

    let mut ctx = cranelift_codegen::Context::for_function(func);
    module
        .define_function(func_id, &mut ctx)
        .map_err(|e| crate::error::MambaError::codegen(format!("define _mb_print_i64: {e}")))?;

    Ok(func_id)
}

#[cfg(test)]
mod tests {
    use super::emit_main;
    use cranelift_codegen::ir::{types as cl_types, AbiParam, Signature};
    use cranelift_codegen::isa::CallConv;
    use cranelift_codegen::settings::{self, Configurable};
    use cranelift_module::{Linkage, Module};
    use cranelift_object::{ObjectBuilder, ObjectModule};

    fn make_object_module() -> ObjectModule {
        let mut flags_builder = settings::builder();
        flags_builder.set("is_pic", "true").unwrap();
        let isa_builder = cranelift_native::builder().expect("no native ISA");
        let isa = isa_builder
            .finish(settings::Flags::new(flags_builder))
            .expect("ISA error");
        let obj_builder = ObjectBuilder::new(
            isa,
            "test_module",
            cranelift_module::default_libcall_names(),
        )
        .expect("object builder error");
        ObjectModule::new(obj_builder)
    }

    #[test]
    fn test_emit_main_succeeds() {
        let mut module = make_object_module();
        // Declare a dummy entry function: () -> i64
        let mut sig = Signature::new(CallConv::SystemV);
        sig.returns.push(AbiParam::new(cl_types::I64));
        let entry_id = module
            .declare_function("_mb_0", Linkage::Local, &sig)
            .unwrap();
        let result = emit_main(&mut module, entry_id);
        assert!(
            result.is_ok(),
            "emit_main should succeed: {:?}",
            result.err()
        );
    }
}
