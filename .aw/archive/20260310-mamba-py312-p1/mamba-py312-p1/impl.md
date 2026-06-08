# Implementation Diff

## Summary

```
crates/mamba/Cargo.toml                      |   4 +
 crates/mamba/src/codegen/cranelift/jit.rs    |  37 ++++-
 crates/mamba/src/codegen/cranelift/mod.rs    |  13 +-
 crates/mamba/src/driver/mod.rs               |   6 +-
 crates/mamba/src/lower/ast_to_hir.rs         |   7 +-
 crates/mamba/src/lower/hir_to_mir.rs         | 140 ++++++++++++++++-
 crates/mamba/src/lower/mod.rs                |   1 +
 crates/mamba/src/runtime/builtins.rs         | 111 ++++++++++----
 crates/mamba/src/runtime/mod.rs              |   1 +
 crates/mamba/src/runtime/output.rs           |  99 ++++++++++++
 crates/mamba/src/runtime/symbols.rs          |  10 ++
 crates/mamba/tests/conformance_tests.rs      | 169 +++++++++++++++++++++
 .../conformance/arithmetic/float_basic.expected    |   6 +
 .../fixtures/conformance/arithmetic/float_basic.py |   7 +
 .../conformance/arithmetic/int_basic.expected      |   7 +
 .../fixtures/conformance/arithmetic/int_basic.py   |   8 +
 .../conformance/arithmetic/mixed_types.expected    |   4 +
 .../fixtures/conformance/arithmetic/mixed_types.py |   6 +
 .../conformance/arithmetic/unary_ops.expected      |   6 +
 .../fixtures/conformance/arithmetic/unary_ops.py   |   7 +
 .../conformance/builtins/type_conversions.expected |   6 +
 .../conformance/builtins/type_conversions.py       |   7 +
 .../conformance/comparison/int_compare.expected    |  12 ++
 .../fixtures/conformance/comparison/int_compare.py |  13 ++
 .../conformance/truthiness/bool_values.expected    |   8 +
 .../fixtures/conformance/truthiness/bool_values.py |   9 ++
 crates/mamba/tests/regen_golden.py           |  63 ++++++++
 27 files changed, 715 insertions(+), 52 deletions(-)
```

## Diff

```diff
diff --git a/crates/mamba/Cargo.toml b/crates/mamba/Cargo.toml
index e60fe0a..14348c6 100644
--- a/crates/mamba/Cargo.toml
+++ b/crates/mamba/Cargo.toml
@@ -53,3 +53,7 @@ harness = false
 [[test]]
 name = "cpython_compat"
 harness = false
+
+[[test]]
+name = "conformance_tests"
+harness = false
diff --git a/crates/mamba/src/codegen/cranelift/jit.rs b/crates/mamba/src/codegen/cranelift/jit.rs
index 5820524..61def1a 100644
--- a/crates/mamba/src/codegen/cranelift/jit.rs
+++ b/crates/mamba/src/codegen/cranelift/jit.rs
@@ -9,6 +9,7 @@ use super::{VarAlloc, emit_binop, emit_terminator};
 use crate::codegen::{CodegenBackend, CodegenOutput};
 use crate::mir::{MirBinOp, MirBody, MirConst, MirExtern, MirInst, MirModule, MirType};
 use crate::runtime::symbols::{runtime_externs, runtime_symbols};
+use crate::runtime::value::MbValue;
 use crate::types::{Ty, TypeContext};
 
 use cranelift_codegen::ir::{types as cl_types, AbiParam, Function, InstBuilder, Signature};
@@ -175,7 +176,7 @@ impl CraneliftJitBackend {
                     MirConst::Int(v) => builder.ins().iconst(cl_types::I64, *v),
                     MirConst::Float(v) => builder.ins().f64const(*v),
                     MirConst::Bool(v) => builder.ins().iconst(cl_types::I64, *v as i64),
-                    MirConst::None => builder.ins().iconst(cl_types::I64, 0x6_i64),
+                    MirConst::None => builder.ins().iconst(cl_types::I64, MbValue::none().to_bits() as i64),
                     MirConst::Str(_) => builder.ins().iconst(cl_types::I64, 0),
                     MirConst::FuncRef(sym) => {
                         // Load function address for async body fn pointer (#313 R1)
@@ -196,7 +197,24 @@ impl CraneliftJitBackend {
                     MirBinOp::In | MirBinOp::NotIn => false,
                     _ => matches!(resolved_ty, Ty::Int | Ty::Float | Ty::Bool),
                 };
-                if matches!(op, MirBinOp::In | MirBinOp::NotIn) {
+                if matches!(op, MirBinOp::Pow) && matches!(resolved_ty, Ty::Int) {
+                    // Integer power → call mb_pow_int runtime function
+                    if let Some(&func_id) = self.extern_funcs.get("mb_pow_int") {
+                        let func_ref = self.module().declare_func_in_func(func_id, builder.func);
+                        let lv = vars.get(*lhs, builder, cl_types::I64);
+                        let rv = vars.get(*rhs, builder, cl_types::I64);
+                        let l = builder.use_var(lv);
+                        let r = builder.use_var(rv);
+                        let call = builder.ins().call(func_ref, &[l, r]);
+                        let result = builder.inst_results(call)[0];
+                        let dv = vars.get(*dest, builder, cl_types::I64);
+                        builder.def_var(dv, result);
+                    } else {
+                        let zero = builder.ins().iconst(cl_types::I64, 0);
+                        let dv = vars.get(*dest, builder, cl_types::I64);
+                        builder.def_var(dv, zero);
+                    }
+                } else if matches!(op, MirBinOp::In | MirBinOp::NotIn) {
                     if let Some(&func_id) = self.extern_funcs.get("mb_obj_contains") {
                         let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                         let rv = vars.get(*rhs, builder, cl_types::I64);
@@ -592,11 +610,22 @@ impl CraneliftJitBackend {
         if let Some(&func_id) = self.extern_funcs.get(name) {
             let func_ref = self.module().declare_func_in_func(func_id, builder.func);
             let arg_vals: Vec<_> = args.iter().enumerate().map(|(i, a)| {
-                let v = vars.get(*a, builder, cl_types::I64);
+                // Use the extern's param type to determine the correct CL type
+                // so float VRegs (F64) aren't misread as I64 (#752).
+                let arg_cl_type = if let Some(ext) = ext {
+                    if i < ext.params.len() {
+                        marshal::mamba_repr_type(&ext.params[i])
+                    } else {
+                        cl_types::I64
+                    }
+                } else {
+                    cl_types::I64
+                };
+                let v = vars.get(*a, builder, arg_cl_type);
                 let val = builder.use_var(v);
                 if let Some(ext) = ext {
                     if i < ext.params.len() {
-                        return marshal::marshal_arg(builder, val, cl_types::I64, &ext.params[i]);
+                        return marshal::marshal_arg(builder, val, arg_cl_type, &ext.params[i]);
                     }
                 }
                 val
diff --git a/crates/mamba/src/codegen/cranelift/mod.rs b/crates/mamba/src/codegen/cranelift/mod.rs
index 096040a..ca729e6 100644
--- a/crates/mamba/src/codegen/cranelift/mod.rs
+++ b/crates/mamba/src/codegen/cranelift/mod.rs
@@ -676,27 +676,28 @@ fn emit_binop(
         (MirBinOp::Sub, Ty::Float) => builder.ins().fsub(l, r),
         (MirBinOp::Mul, Ty::Float) => builder.ins().fmul(l, r),
         (MirBinOp::Div, Ty::Float) => builder.ins().fdiv(l, r),
-        (MirBinOp::Eq, Ty::Int) | (MirBinOp::Eq, Ty::Bool) => {
+        // Comparisons — result type is Bool but operands may be Int/Float/Bool
+        (MirBinOp::Eq, _) => {
             let cmp = builder.ins().icmp(IntCC::Equal, l, r);
             builder.ins().uextend(cl_types::I64, cmp)
         }
-        (MirBinOp::NotEq, Ty::Int) => {
+        (MirBinOp::NotEq, _) => {
             let cmp = builder.ins().icmp(IntCC::NotEqual, l, r);
             builder.ins().uextend(cl_types::I64, cmp)
         }
-        (MirBinOp::Lt, Ty::Int) => {
+        (MirBinOp::Lt, _) => {
             let cmp = builder.ins().icmp(IntCC::SignedLessThan, l, r);
             builder.ins().uextend(cl_types::I64, cmp)
         }
-        (MirBinOp::Gt, Ty::Int) => {
+        (MirBinOp::Gt, _) => {
             let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, l, r);
             builder.ins().uextend(cl_types::I64, cmp)
         }
-        (MirBinOp::LtEq, Ty::Int) => {
+        (MirBinOp::LtEq, _) => {
             let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, l, r);
             builder.ins().uextend(cl_types::I64, cmp)
         }
-        (MirBinOp::GtEq, Ty::Int) => {
+        (MirBinOp::GtEq, _) => {
             let cmp = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, l, r);
             builder.ins().uextend(cl_types::I64, cmp)
         }
diff --git a/crates/mamba/src/driver/mod.rs b/crates/mamba/src/driver/mod.rs
index 5da7b72..e97bda7 100644
--- a/crates/mamba/src/driver/mod.rs
+++ b/crates/mamba/src/driver/mod.rs
@@ -92,8 +92,8 @@ impl CompilerSession {
             return Ok(Vec::new());
         }
 
-        // Lower HIR → MIR
-        let mir_module = lower::lower_hir_to_mir(&hir, &checker.tcx);
+        // Lower HIR → MIR (with builtin resolution)
+        let mir_module = lower::lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);
 
         if let Some(EmitMode::Mir) = self.config.emit {
             println!("{mir_module:#?}");
@@ -147,7 +147,7 @@ impl CompilerSession {
         // Lower
         let hir = lower::lower_module(&module, &checker)
             .map_err(|errs| errs.into_iter().next().unwrap())?;
-        let mir_module = lower::lower_hir_to_mir(&hir, &checker.tcx);
+        let mir_module = lower::lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);
 
         // JIT compile
         let mut backend = CraneliftJitBackend::new()
diff --git a/crates/mamba/src/lower/ast_to_hir.rs b/crates/mamba/src/lower/ast_to_hir.rs
index fc774eb..1529851 100644
--- a/crates/mamba/src/lower/ast_to_hir.rs
+++ b/crates/mamba/src/lower/ast_to_hir.rs
@@ -442,7 +442,12 @@ impl<'a> AstLowerer<'a> {
                 let l = self.lower_expr(lhs)?;
                 let r = self.lower_expr(rhs)?;
                 let hir_op = lower_bin_op(*op)?;
-                let ty = l.ty(); // simplified: result type = lhs type
+                let ty = match hir_op {
+                    HirBinOp::Eq | HirBinOp::NotEq | HirBinOp::Lt | HirBinOp::Gt
+                    | HirBinOp::LtEq | HirBinOp::GtEq | HirBinOp::Is | HirBinOp::IsNot
+                    | HirBinOp::In | HirBinOp::NotIn => self.checker.tcx.bool(),
+                    _ => l.ty(), // arithmetic/logical: result type = lhs type
+                };
                 Some(HirExpr::BinOp {
                     op: hir_op, lhs: Box::new(l), rhs: Box::new(r), ty,
                 })
diff --git a/crates/mamba/src/lower/hir_to_mir.rs b/crates/mamba/src/lower/hir_to_mir.rs
index d29ecf4..0ac3951 100644
--- a/crates/mamba/src/lower/hir_to_mir.rs
+++ b/crates/mamba/src/lower/hir_to_mir.rs
@@ -6,9 +6,33 @@
 
 use crate::hir::*;
 use crate::mir::*;
-use crate::resolve::SymbolId;
+use crate::resolve::{SymbolId, SymbolTable};
 use crate::types::{TypeContext, TypeId};
-use std::collections::HashMap;
+use std::collections::{HashMap, HashSet};
+
+/// Mapping from Python builtin name to mb_* runtime extern name.
+fn builtin_extern_map() -> HashMap<&'static str, &'static str> {
+    [
+        ("print", "mb_print"), ("len", "mb_len"), ("int", "mb_int"),
+        ("float", "mb_float"), ("bool", "mb_bool"), ("str", "mb_str"),
+        ("abs", "mb_abs"), ("type", "mb_type"), ("range", "mb_range"),
+        ("min", "mb_min"), ("max", "mb_max"), ("sum", "mb_sum"),
+        ("sorted", "mb_sorted"), ("repr", "mb_repr"), ("hash", "mb_hash"),
+        ("id", "mb_id"), ("input", "mb_input"), ("chr", "mb_chr"),
+        ("ord", "mb_ord"), ("isinstance", "mb_isinstance"),
+        ("issubclass", "mb_issubclass"), ("callable", "mb_callable"),
+        ("hasattr", "mb_hasattr"), ("getattr", "mb_getattr"),
+        ("setattr", "mb_setattr"), ("delattr", "mb_delattr"),
+        ("iter", "mb_iter"), ("next", "mb_next"),
+        ("reversed", "mb_reversed"), ("enumerate", "mb_enumerate"),
+        ("zip", "mb_zip"), ("map", "mb_map"), ("filter", "mb_filter"),
+        ("any", "mb_any"), ("all", "mb_all"),
+        ("hex", "mb_hex"), ("oct", "mb_oct"), ("bin", "mb_bin"),
+        ("format", "mb_format"), ("vars", "mb_vars"), ("dir", "mb_dir"),
+        ("round", "mb_round"), ("pow", "mb_pow"), ("divmod", "mb_divmod"),
+        ("super", "mb_super"),
+    ].into_iter().collect()
+}
 
 /// Lower a complete HIR module to a MIR module.
 pub fn lower_hir_to_mir(hir: &HirModule, tcx: &TypeContext) -> MirModule {
@@ -28,6 +52,37 @@ pub fn lower_hir_to_mir(hir: &HirModule, tcx: &TypeContext) -> MirModule {
     }
 }
 
+/// Lower with symbol table for builtin resolution.
+pub fn lower_hir_to_mir_with_symbols(
+    hir: &HirModule,
+    tcx: &TypeContext,
+    symbols: &SymbolTable,
+) -> MirModule {
+    let user_funcs: HashSet<u32> = hir.functions.iter().map(|f| f.name.0).collect();
+    let extern_map = builtin_extern_map();
+    let mut builtin_syms: HashMap<u32, String> = HashMap::new();
+    for (&py_name, &mb_name) in &extern_map {
+        if let Some(sym_id) = symbols.lookup(py_name) {
+            if !user_funcs.contains(&sym_id.0) {
+                builtin_syms.insert(sym_id.0, mb_name.to_string());
+            }
+        }
+    }
+    let mut lowerer = HirToMir::new_with_builtins(tcx, user_funcs, builtin_syms);
+    for func in &hir.functions {
+        let body = lowerer.lower_function(func);
+        lowerer.bodies.push(body);
+    }
+    if !hir.top_level.is_empty() {
+        let main_body = lowerer.lower_top_level(&hir.top_level);
+        lowerer.bodies.push(main_body);
+    }
+    MirModule {
+        bodies: lowerer.bodies,
+        externs: Vec::new(),
+    }
+}
+
 /// REPL-aware lowering: includes accumulated functions from previous
 /// iterations, restores globals, saves all top-level variables,
 /// and returns the last expression value for echo.
@@ -73,6 +128,10 @@ struct HirToMir<'a> {
     current_block_id: Option<BlockId>,
     /// VReg holding coroutine handle for async functions.
     async_coro_vreg: Option<VReg>,
+    /// User-defined function SymbolIds (to distinguish from builtins).
+    user_funcs: HashSet<u32>,
+    /// SymbolId.0 → mb_* extern name for builtin calls.
+    builtin_syms: HashMap<u32, String>,
 }
 
 impl<'a> HirToMir<'a> {
@@ -89,6 +148,30 @@ impl<'a> HirToMir<'a> {
             loop_header: None,
             current_block_id: None,
             async_coro_vreg: None,
+            user_funcs: HashSet::new(),
+            builtin_syms: HashMap::new(),
+        }
+    }
+
+    fn new_with_builtins(
+        tcx: &'a TypeContext,
+        user_funcs: HashSet<u32>,
+        builtin_syms: HashMap<u32, String>,
+    ) -> Self {
+        Self {
+            tcx,
+            bodies: Vec::new(),
+            next_vreg: 0,
+            next_block: 0,
+            blocks: Vec::new(),
+            current_stmts: Vec::new(),
+            sym_to_vreg: HashMap::new(),
+            loop_exit: None,
+            loop_header: None,
+            current_block_id: None,
+            async_coro_vreg: None,
+            user_funcs,
+            builtin_syms,
         }
     }
 
@@ -1322,9 +1405,56 @@ impl<'a> HirToMir<'a> {
                     HirExpr::Var(sym, _) => *sym,
                     _ => SymbolId(u32::MAX), // indirect call placeholder
                 };
-                self.current_stmts.push(MirInst::Call {
-                    dest: Some(dest), func: func_sym, args: arg_vregs, ty: *ty,
-                });
+                // Check if this is a builtin call that maps to an extern
+                if let Some(extern_name) = self.builtin_syms.get(&func_sym.0).cloned() {
+                    // Box primitive arguments for runtime functions
+                    let boxed_args: Vec<VReg> = args.iter().zip(arg_vregs.iter()).map(|(arg_expr, &vreg)| {
+                        let arg_ty = self.tcx.get(arg_expr.ty());
+                        match arg_ty {
+                            crate::types::Ty::Bool => {
+                                let boxed = self.fresh_vreg();
+                                self.current_stmts.push(MirInst::CallExtern {
+                                    dest: Some(boxed),
+                                    name: "mb_box_bool".to_string(),
+                                    args: vec![vreg],
+                                    ty: self.tcx.any(),
+                                });
+                                boxed
+                            }
+                            crate::types::Ty::Int => {
+                                let boxed = self.fresh_vreg();
+                                self.current_stmts.push(MirInst::CallExtern {
+                                    dest: Some(boxed),
+                                    name: "mb_box_int".to_string(),
+                                    args: vec![vreg],
+                                    ty: self.tcx.any(),
+                                });
+                                boxed
+                            }
+                            crate::types::Ty::Float => {
+                                let boxed = self.fresh_vreg();
+                                self.current_stmts.push(MirInst::CallExtern {
+                                    dest: Some(boxed),
+                                    name: "mb_box_float".to_string(),
+                                    args: vec![vreg],
+                                    ty: self.tcx.any(),
+                                });
+                                boxed
+                            }
+                            _ => vreg, // already NaN-boxed (objects, etc.)
+                        }
+                    }).collect();
+                    self.current_stmts.push(MirInst::CallExtern {
+                        dest: Some(dest),
+                        name: extern_name,
+                        args: boxed_args,
+                        ty: *ty,
+                    });
+                } else {
+                    self.current_stmts.push(MirInst::Call {
+                        dest: Some(dest), func: func_sym, args: arg_vregs, ty: *ty,
+                    });
+                }
                 dest
             }
             HirExpr::Attr { object, attr, ty } => {
diff --git a/crates/mamba/src/lower/mod.rs b/crates/mamba/src/lower/mod.rs
index 376d8a2..c3125b4 100644
--- a/crates/mamba/src/lower/mod.rs
+++ b/crates/mamba/src/lower/mod.rs
@@ -6,3 +6,4 @@ pub use ast_to_hir::lower_module_repl;
 pub use ast_to_hir::ReplSymInfo;
 pub use hir_to_mir::lower_hir_to_mir;
 pub use hir_to_mir::lower_hir_to_mir_repl;
+pub use hir_to_mir::lower_hir_to_mir_with_symbols;
diff --git a/crates/mamba/src/runtime/builtins.rs b/crates/mamba/src/runtime/builtins.rs
index da14cb6..f78c053 100644
--- a/crates/mamba/src/runtime/builtins.rs
+++ b/crates/mamba/src/runtime/builtins.rs
@@ -5,6 +5,51 @@
 
 use super::value::MbValue;
 use super::rc::{MbObject, ObjData};
+use super::output::{write_captured, writeln_captured};
+
+/// Write to capture buffer if active, else to stdout.
+macro_rules! mb_out {
+    ($($arg:tt)*) => {{
+        let s = format!($($arg)*);
+        if !write_captured(&s) {
+            print!("{}", s);
+        }
+    }};
+}
+
+/// Writeln to capture buffer if active, else to stdout.
+macro_rules! mb_outln {
+    ($($arg:tt)*) => {{
+        let s = format!($($arg)*);
+        if !writeln_captured(&s) {
+            println!("{}", s);
+        }
+    }};
+}
+
+/// Box a raw i64 into a NaN-boxed MbValue integer.
+/// Used by JIT to convert primitive int results before passing to runtime fns.
+pub fn mb_box_int(raw: i64) -> MbValue {
+    MbValue::from_int(raw)
+}
+
+/// Integer power: base ** exp (for JIT use).
+pub fn mb_pow_int(base: i64, exp: i64) -> i64 {
+    if exp < 0 {
+        return 0; // Python returns float for negative exponents; int approx = 0
+    }
+    (base as i128).pow(exp as u32) as i64
+}
+
+/// Box a raw i64 (0/1) into a NaN-boxed MbValue bool.
+pub fn mb_box_bool(raw: i64) -> MbValue {
+    MbValue::from_bool(raw != 0)
+}
+
+/// Box a raw f64 into a NaN-boxed MbValue float.
+pub fn mb_box_float(f: f64) -> MbValue {
+    MbValue::from_float(f)
+}
 
 /// Check if a value is None. Returns bool MbValue.
 /// Used by for-loop lowering to detect iterator exhaustion.
@@ -12,77 +57,77 @@ pub fn mb_is_none(val: MbValue) -> MbValue {
     MbValue::from_bool(val.is_none())
 }
 
-/// print(value) — print a value to stdout.
+/// print(value) — print a value to stdout (or capture buffer).
 pub fn mb_print(val: MbValue) {
     if let Some(i) = val.as_int() {
-        println!("{i}");
+        mb_outln!("{i}");
     } else if let Some(f) = val.as_float() {
         // Match Python: print 1.0 not 1
         if f == f.floor() && f.is_finite() {
-            println!("{f:.1}");
+            mb_outln!("{f:.1}");
         } else {
-            println!("{f}");
+            mb_outln!("{f}");
         }
     } else if let Some(b) = val.as_bool() {
-        println!("{}", if b { "True" } else { "False" });
+        mb_outln!("{}", if b { "True" } else { "False" });
     } else if val.is_none() {
-        println!("None");
+        mb_outln!("None");
     } else if let Some(ptr) = val.as_ptr() {
         unsafe {
             match &(*ptr).data {
-                ObjData::Str(s) => println!("{s}"),
+                ObjData::Str(s) => mb_outln!("{s}"),
                 ObjData::List(ref lock) => {
                     let items = lock.read().unwrap();
-                    print!("[");
+                    mb_out!("[");
                     for (i, item) in items.iter().enumerate() {
-                        if i > 0 { print!(", "); }
+                        if i > 0 { mb_out!(", "); }
                         print_repr(*item);
                     }
-                    println!("]");
+                    mb_outln!("]");
                 }
                 ObjData::Dict(ref lock) => {
                     let map = lock.read().unwrap();
-                    print!("{{");
+                    mb_out!("{{");
                     for (i, (k, v)) in map.iter().enumerate() {
-                        if i > 0 { print!(", "); }
-                        print!("'{k}': ");
+                        if i > 0 { mb_out!(", "); }
+                        mb_out!("'{k}': ");
                         print_repr(*v);
                     }
-                    println!("}}");
+                    mb_outln!("}}");
                 }
                 ObjData::Tuple(items) => {
-                    print!("(");
+                    mb_out!("(");
                     for (i, item) in items.iter().enumerate() {
-                        if i > 0 { print!(", "); }
+                        if i > 0 { mb_out!(", "); }
                         print_repr(*item);
                     }
-                    if items.len() == 1 { print!(","); }
-                    println!(")");
+                    if items.len() == 1 { mb_out!(","); }
+                    mb_outln!(")");
                 }
                 ObjData::Instance { class_name, .. } => {
-                    println!("<{class_name} instance>");
+                    mb_outln!("<{class_name} instance>");
                 }
                 ObjData::Set(ref lock) => {
                     let items = lock.read().unwrap();
-                    print!("{{");
+                    mb_out!("{{");
                     for (i, item) in items.iter().enumerate() {
-                        if i > 0 { print!(", "); }
+                        if i > 0 { mb_out!(", "); }
                         print_repr(*item);
                     }
-                    println!("}}");
+                    mb_outln!("}}");
                 }
                 ObjData::FrozenSet(items) => {
-                    print!("frozenset({{");
+                    mb_out!("frozenset({{");
                     for (i, item) in items.iter().enumerate() {
-                        if i > 0 { print!(", "); }
+                        if i > 0 { mb_out!(", "); }
                         print_repr(*item);
                     }
-                    println!("}})");
+                    mb_outln!("}})");
                 }
-                ObjData::Bytes(data) => println!("b\"{}\"", String::from_utf8_lossy(data)),
+                ObjData::Bytes(data) => mb_outln!("b\"{}\"", String::from_utf8_lossy(data)),
                 ObjData::ByteArray(ref lock) => {
                     let data = lock.read().unwrap();
-                    println!("bytearray(b\"{}\")", String::from_utf8_lossy(&data));
+                    mb_outln!("bytearray(b\"{}\")", String::from_utf8_lossy(&data));
                 }
             }
         }
@@ -91,19 +136,19 @@ pub fn mb_print(val: MbValue) {
 
 fn print_repr(val: MbValue) {
     if let Some(i) = val.as_int() {
-        print!("{i}");
+        mb_out!("{i}");
     } else if let Some(f) = val.as_float() {
-        print!("{f}");
+        mb_out!("{f}");
     } else if let Some(b) = val.as_bool() {
-        print!("{}", if b { "True" } else { "False" });
+        mb_out!("{}", if b { "True" } else { "False" });
     } else if val.is_none() {
-        print!("None");
+        mb_out!("None");
     } else if let Some(ptr) = val.as_ptr() {
         unsafe {
             if let ObjData::Str(s) = &(*ptr).data {
-                print!("'{s}'");
+                mb_out!("'{s}'");
             } else {
-                print!("...");
+                mb_out!("...");
             }
         }
     }
diff --git a/crates/mamba/src/runtime/mod.rs b/crates/mamba/src/runtime/mod.rs
index 2c681d3..88eb32d 100644
--- a/crates/mamba/src/runtime/mod.rs
+++ b/crates/mamba/src/runtime/mod.rs
@@ -1,5 +1,6 @@
 pub mod value;
 pub mod rc;
+pub mod output;
 pub mod builtins;
 pub mod string_ops;
 pub mod list_ops;
diff --git a/crates/mamba/src/runtime/output.rs b/crates/mamba/src/runtime/output.rs
new file mode 100644
index 0000000..7cc17f6
--- /dev/null
+++ b/crates/mamba/src/runtime/output.rs
@@ -0,0 +1,99 @@
+/// Thread-local output capture for conformance testing.
+///
+/// When capture is active, `mb_print` and other output functions write to a
+/// thread-local buffer instead of stdout. This allows `cargo test` to compare
+/// mamba output against golden files without subprocess overhead.
+
+use std::cell::RefCell;
+use std::io::Write;
+
+thread_local! {
+    static CAPTURE_BUF: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
+}
+
+/// Begin capturing stdout output to an internal buffer.
+/// Returns any previously captured content (useful for nested captures).
+pub fn begin_capture() -> Option<Vec<u8>> {
+    CAPTURE_BUF.with(|buf| buf.borrow_mut().replace(Vec::new()))
+}
+
+/// End capturing and return the captured bytes as a UTF-8 string.
+/// Restores the previous capture state if `prev` is provided.
+pub fn end_capture(prev: Option<Vec<u8>>) -> String {
+    let captured = CAPTURE_BUF.with(|buf| {
+        let mut b = buf.borrow_mut();
+        let result = b.take().unwrap_or_default();
+        *b = prev;
+        result
+    });
+    String::from_utf8(captured).unwrap_or_else(|e| String::from_utf8_lossy(&e.into_bytes()).into_owned())
+}
+
+/// Write a string to the capture buffer if active, otherwise to stdout.
+/// Returns `true` if written to capture buffer.
+pub fn write_captured(s: &str) -> bool {
+    CAPTURE_BUF.with(|buf| {
+        let mut b = buf.borrow_mut();
+        if let Some(ref mut vec) = *b {
+            let _ = vec.write_all(s.as_bytes());
+            true
+        } else {
+            false
+        }
+    })
+}
+
+/// Write a line (with newline) to the capture buffer if active, otherwise to stdout.
+/// Returns `true` if written to capture buffer.
+pub fn writeln_captured(s: &str) -> bool {
+    CAPTURE_BUF.with(|buf| {
+        let mut b = buf.borrow_mut();
+        if let Some(ref mut vec) = *b {
+            let _ = writeln!(vec, "{s}");
+            true
+        } else {
+            false
+        }
+    })
+}
+
+/// Check if capture is currently active.
+pub fn is_capturing() -> bool {
+    CAPTURE_BUF.with(|buf| buf.borrow().is_some())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_basic_capture() {
+        let prev = begin_capture();
+        assert!(is_capturing());
+        write_captured("hello ");
+        writeln_captured("world");
+        let output = end_capture(prev);
+        assert_eq!(output, "hello world\n");
+        assert!(!is_capturing());
+    }
+
+    #[test]
+    fn test_no_capture() {
+        assert!(!is_capturing());
+        assert!(!write_captured("ignored"));
+        assert!(!writeln_captured("ignored"));
+    }
+
+    #[test]
+    fn test_nested_capture() {
+        let prev1 = begin_capture();
+        write_captured("outer ");
+        let prev2 = begin_capture();
+        write_captured("inner");
+        let inner = end_capture(prev2);
+        assert_eq!(inner, "inner");
+        write_captured("more");
+        let outer = end_capture(prev1);
+        assert_eq!(outer, "outer more");
+    }
+}
diff --git a/crates/mamba/src/runtime/symbols.rs b/crates/mamba/src/runtime/symbols.rs
index 07ac730..8d682ac 100644
--- a/crates/mamba/src/runtime/symbols.rs
+++ b/crates/mamba/src/runtime/symbols.rs
@@ -49,6 +49,16 @@ pub fn runtime_symbols() -> Vec<RuntimeSymbol> {
     use super::tokio_exec;
 
     vec![
+        // ── Boxing (raw → NaN-boxed MbValue) ──
+        rt_sym!("mb_box_int", builtins::mb_box_int as fn(i64) -> super::MbValue, [I64], I64),
+        rt_sym!("mb_box_bool", builtins::mb_box_bool as fn(i64) -> super::MbValue, [I64], I64),
+        rt_sym!("mb_pow_int", builtins::mb_pow_int as fn(i64, i64) -> i64, [I64, I64], I64),
+        RuntimeSymbol {
+            name: "mb_box_float",
+            addr: builtins::mb_box_float as *const u8,
+            params: &[MirType::F64],
+            return_type: MirType::I64,
+        },
         // ── Builtins ──
         rt_sym!("mb_print", builtins::mb_print as fn(super::MbValue), [I64], Void),
         rt_sym!("mb_is_none", builtins::mb_is_none as fn(super::MbValue) -> super::MbValue, [I64], I64),
diff --git a/crates/mamba/tests/conformance_tests.rs b/crates/mamba/tests/conformance_tests.rs
new file mode 100644
index 0000000..a6d0dc9
--- /dev/null
+++ b/crates/mamba/tests/conformance_tests.rs
@@ -0,0 +1,169 @@
+//! Py3.12 runtime conformance test harness (#752).
+//!
+//! Discovers `.py` fixtures under `tests/fixtures/conformance/` and runs each
+//! through the Mamba JIT pipeline, comparing captured stdout against `.expected`
+//! golden files pre-generated from CPython 3.12.
+//!
+//! Directives (in `.py` file comments):
+//!   `# mamba-xfail: <reason>` — mark as expected failure
+//!
+//! Golden file regeneration:
+//!   `python3 tests/regen_golden.py`
+
+use cclab_mamba::codegen::cranelift::jit::CraneliftJitBackend;
+use cclab_mamba::codegen::{CodegenBackend, CodegenOutput};
+use cclab_mamba::lower::{lower_hir_to_mir_with_symbols, lower_module};
+use cclab_mamba::parser;
+use cclab_mamba::runtime::output::{begin_capture, end_capture};
+use cclab_mamba::source::span::FileId;
+use cclab_mamba::types::TypeChecker;
+use datatest_stable::harness;
+use std::path::Path;
+
+// ── Directive parsing ─────────────────────────────────────────────
+
+struct Directives {
+    xfail: Option<String>,
+}
+
+fn parse_directives(src: &str) -> Directives {
+    let mut xfail = None;
+    for line in src.lines() {
+        let t = line.trim();
+        if let Some(reason) = t.strip_prefix("# mamba-xfail:") {
+            xfail = Some(reason.trim().to_string());
+        }
+    }
+    Directives { xfail }
+}
+
+// ── JIT execution with output capture ─────────────────────────────
+
+/// Run a Python source through the full JIT pipeline and return captured stdout.
+fn run_and_capture(src: &str, path: &Path) -> Result<String, String> {
+    // Parse
+    let module = parser::parse(src, FileId(0))
+        .map_err(|e| format!("{}: parse error: {e}", path.display()))?;
+
+    // Type check
+    let mut checker = TypeChecker::new();
+    let errors = checker.check_module(&module);
+    if !errors.is_empty() {
+        return Err(format!(
+            "{}: type errors: {:?}",
+            path.display(),
+            errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
+        ));
+    }
+
+    // Lower (with builtin resolution for print/len/int/etc.)
+    let hir = lower_module(&module, &checker)
+        .map_err(|errs| format!("{}: HIR error: {:?}", path.display(), errs))?;
+    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);
+
+    // JIT
+    let mut backend = CraneliftJitBackend::new()
+        .map_err(|e| format!("{}: JIT init: {e}", path.display()))?;
+    let output = backend
+        .codegen(&mir, &checker.tcx)
+        .map_err(|e| format!("{}: codegen: {e}", path.display()))?;
+
+    match output {
+        CodegenOutput::Jit { entry } => {
+            let prev = begin_capture();
+            let main_fn: fn() -> i64 = unsafe { std::mem::transmute(entry) };
+            let _result = main_fn();
+            let captured = end_capture(prev);
+            Ok(captured)
+        }
+        _ => Err(format!("{}: expected JIT output", path.display())),
+    }
+}
+
+// ── Harness runner ────────────────────────────────────────────────
+
+fn run_conformance(path: &Path) -> datatest_stable::Result<()> {
+    let src = std::fs::read_to_string(path)?;
+    let directives = parse_directives(&src);
+
+    // Load golden file
+    let expected_path = path.with_extension("expected");
+    let expected = if expected_path.exists() {
+        std::fs::read_to_string(&expected_path)?
+    } else {
+        return Err(format!(
+            "{}: missing golden file {}",
+            path.display(),
+            expected_path.display()
+        )
+        .into());
+    };
+
+    // Run mamba
+    let actual = match run_and_capture(&src, path) {
+        Ok(output) => output,
+        Err(err) => {
+            if let Some(reason) = &directives.xfail {
+                eprintln!("  [xfail] {}: {reason}", path.display());
+                return Ok(());
+            }
+            return Err(err.into());
+        }
+    };
+
+    // Check xfail
+    if let Some(reason) = &directives.xfail {
+        if actual == expected {
+            eprintln!(
+                "  [xpass] {} passed unexpectedly (xfail: {reason}). \
+                 Consider removing # mamba-xfail.",
+                path.display()
+            );
+        } else {
+            eprintln!("  [xfail] {}: {reason}", path.display());
+        }
+        return Ok(());
+    }
+
+    // Compare output
+    if actual != expected {
+        let diff = format_diff(&expected, &actual);
+        return Err(format!(
+            "{}: output mismatch\n\n--- expected ({})\n+++ actual (mamba)\n{}",
+            path.display(),
+            expected_path.display(),
+            diff
+        )
+        .into());
+    }
+
+    Ok(())
+}
+
+/// Simple line-by-line diff for readable test output.
+fn format_diff(expected: &str, actual: &str) -> String {
+    let mut out = String::new();
+    let exp_lines: Vec<&str> = expected.lines().collect();
+    let act_lines: Vec<&str> = actual.lines().collect();
+    let max = exp_lines.len().max(act_lines.len());
+
+    for i in 0..max {
+        let e = exp_lines.get(i).copied().unwrap_or("");
+        let a = act_lines.get(i).copied().unwrap_or("");
+        if e != a {
+            out.push_str(&format!("  line {}: expected {:?}, got {:?}\n", i + 1, e, a));
+        }
+    }
+
+    if exp_lines.len() != act_lines.len() {
+        out.push_str(&format!(
+            "  (expected {} lines, got {} lines)\n",
+            exp_lines.len(),
+            act_lines.len()
+        ));
+    }
+
+    out
+}
+
+harness!(run_conformance, "tests/fixtures/conformance", r".*\.py$");
diff --git a/crates/mamba/tests/fixtures/conformance/arithmetic/float_basic.expected b/crates/mamba/tests/fixtures/conformance/arithmetic/float_basic.expected
new file mode 100644
index 0000000..fc02fa4
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/arithmetic/float_basic.expected
@@ -0,0 +1,6 @@
+4.0
+6.5
+6.0
+3.5
+1.0
+0.30000000000000004
diff --git a/crates/mamba/tests/fixtures/conformance/arithmetic/float_basic.py b/crates/mamba/tests/fixtures/conformance/arithmetic/float_basic.py
new file mode 100644
index 0000000..c3b3eaf
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/arithmetic/float_basic.py
@@ -0,0 +1,7 @@
+# Basic float arithmetic
+print(1.5 + 2.5)
+print(10.0 - 3.5)
+print(2.0 * 3.0)
+print(7.0 / 2.0)
+print(1.0)
+print(0.1 + 0.2)
diff --git a/crates/mamba/tests/fixtures/conformance/arithmetic/int_basic.expected b/crates/mamba/tests/fixtures/conformance/arithmetic/int_basic.expected
new file mode 100644
index 0000000..f648739
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/arithmetic/int_basic.expected
@@ -0,0 +1,7 @@
+3
+7
+20
+3
+1
+1024
+-42
diff --git a/crates/mamba/tests/fixtures/conformance/arithmetic/int_basic.py b/crates/mamba/tests/fixtures/conformance/arithmetic/int_basic.py
new file mode 100644
index 0000000..cc57aad
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/arithmetic/int_basic.py
@@ -0,0 +1,8 @@
+# Basic integer arithmetic
+print(1 + 2)
+print(10 - 3)
+print(4 * 5)
+print(7 // 2)
+print(7 % 2)
+print(2 ** 10)
+print(-42)
diff --git a/crates/mamba/tests/fixtures/conformance/arithmetic/mixed_types.expected b/crates/mamba/tests/fixtures/conformance/arithmetic/mixed_types.expected
new file mode 100644
index 0000000..5dae20f
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/arithmetic/mixed_types.expected
@@ -0,0 +1,4 @@
+3.0
+6.5
+10.0
+3.5
diff --git a/crates/mamba/tests/fixtures/conformance/arithmetic/mixed_types.py b/crates/mamba/tests/fixtures/conformance/arithmetic/mixed_types.py
new file mode 100644
index 0000000..11e5824
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/arithmetic/mixed_types.py
@@ -0,0 +1,6 @@
+# mamba-xfail: type checker rejects int+float mixed ops (no implicit coercion yet)
+# Mixed int/float arithmetic
+print(1 + 2.0)
+print(10 - 3.5)
+print(4 * 2.5)
+print(7 / 2)
diff --git a/crates/mamba/tests/fixtures/conformance/arithmetic/unary_ops.expected b/crates/mamba/tests/fixtures/conformance/arithmetic/unary_ops.expected
new file mode 100644
index 0000000..c814542
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/arithmetic/unary_ops.expected
@@ -0,0 +1,6 @@
+-42
+10
+5
+-1
+-2
+0
diff --git a/crates/mamba/tests/fixtures/conformance/arithmetic/unary_ops.py b/crates/mamba/tests/fixtures/conformance/arithmetic/unary_ops.py
new file mode 100644
index 0000000..29564d7
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/arithmetic/unary_ops.py
@@ -0,0 +1,7 @@
+# Unary operators
+print(-42)
+print(-(-10))
+print(+5)
+print(~0)
+print(~1)
+print(~(-1))
diff --git a/crates/mamba/tests/fixtures/conformance/builtins/type_conversions.expected b/crates/mamba/tests/fixtures/conformance/builtins/type_conversions.expected
new file mode 100644
index 0000000..16005d2
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/builtins/type_conversions.expected
@@ -0,0 +1,6 @@
+3
+1
+0
+42.0
+1.0
+0.0
diff --git a/crates/mamba/tests/fixtures/conformance/builtins/type_conversions.py b/crates/mamba/tests/fixtures/conformance/builtins/type_conversions.py
new file mode 100644
index 0000000..92c80a6
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/builtins/type_conversions.py
@@ -0,0 +1,7 @@
+# Type conversion builtins
+print(int(3.7))
+print(int(True))
+print(int(False))
+print(float(42))
+print(float(True))
+print(float(False))
diff --git a/crates/mamba/tests/fixtures/conformance/comparison/int_compare.expected b/crates/mamba/tests/fixtures/conformance/comparison/int_compare.expected
new file mode 100644
index 0000000..1cad7eb
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/comparison/int_compare.expected
@@ -0,0 +1,12 @@
+True
+False
+True
+False
+True
+False
+True
+False
+True
+True
+True
+False
diff --git a/crates/mamba/tests/fixtures/conformance/comparison/int_compare.py b/crates/mamba/tests/fixtures/conformance/comparison/int_compare.py
new file mode 100644
index 0000000..a6ed866
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/comparison/int_compare.py
@@ -0,0 +1,13 @@
+# Integer comparison operators
+print(1 == 1)
+print(1 == 2)
+print(1 != 2)
+print(1 != 1)
+print(3 < 5)
+print(5 < 3)
+print(5 > 3)
+print(3 > 5)
+print(3 <= 3)
+print(3 <= 5)
+print(5 >= 5)
+print(3 >= 5)
diff --git a/crates/mamba/tests/fixtures/conformance/truthiness/bool_values.expected b/crates/mamba/tests/fixtures/conformance/truthiness/bool_values.expected
new file mode 100644
index 0000000..dbf73d3
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/truthiness/bool_values.expected
@@ -0,0 +1,8 @@
+False
+True
+True
+False
+True
+False
+True
+False
diff --git a/crates/mamba/tests/fixtures/conformance/truthiness/bool_values.py b/crates/mamba/tests/fixtures/conformance/truthiness/bool_values.py
new file mode 100644
index 0000000..5d3dc78
--- /dev/null
+++ b/crates/mamba/tests/fixtures/conformance/truthiness/bool_values.py
@@ -0,0 +1,9 @@
+# Truthiness / bool() tests
+print(bool(0))
+print(bool(1))
+print(bool(-1))
+print(bool(0.0))
+print(bool(1.5))
+print(bool(None))
+print(bool(True))
+print(bool(False))
diff --git a/crates/mamba/tests/regen_golden.py b/crates/mamba/tests/regen_golden.py
new file mode 100644
index 0000000..f9c664a
--- /dev/null
+++ b/crates/mamba/tests/regen_golden.py
@@ -0,0 +1,63 @@
+#!/usr/bin/env python3
+"""Regenerate conformance golden files from CPython 3.12.
+
+Usage:
+    python3 tests/regen_golden.py [tests/fixtures/conformance]
+
+Walks all .py files under the conformance directory, runs each with
+CPython, and writes the stdout to a .expected file alongside.
+"""
+
+import os
+import subprocess
+import sys
+from pathlib import Path
+
+
+def main():
+    base = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("tests/fixtures/conformance")
+    if not base.exists():
+        print(f"Error: {base} does not exist", file=sys.stderr)
+        sys.exit(1)
+
+    py_files = sorted(base.rglob("*.py"))
+    # Exclude this script itself
+    py_files = [f for f in py_files if f.name != "regen_golden.py"]
+
+    updated = 0
+    for py_file in py_files:
+        expected_file = py_file.with_suffix(".expected")
+
+        try:
+            result = subprocess.run(
+                [sys.executable, str(py_file)],
+                capture_output=True,
+                text=True,
+                timeout=10,
+            )
+            output = result.stdout
+            if result.returncode != 0 and result.stderr:
+                # Capture exception type from last line of stderr
+                lines = result.stderr.strip().split("\n")
+                exc_line = lines[-1] if lines else ""
+                output += f"EXCEPTION: {exc_line}\n"
+        except subprocess.TimeoutExpired:
+            output = "TIMEOUT\n"
+        except Exception as e:
+            print(f"  SKIP {py_file}: {e}", file=sys.stderr)
+            continue
+
+        # Only write if changed
+        old = expected_file.read_text() if expected_file.exists() else None
+        if old != output:
+            expected_file.write_text(output)
+            print(f"  UPDATED {expected_file}")
+            updated += 1
+        else:
+            print(f"  OK      {expected_file}")
+
+    print(f"\nDone: {len(py_files)} files, {updated} updated.")
+
+
+if __name__ == "__main__":
+    main()
```
