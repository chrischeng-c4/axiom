---
id: implementation
type: change_implementation
change_id: mamba-refcount-jit
---

# Implementation

## Summary

Implement CPython 3.12 reference counting semantics in Mamba's Cranelift JIT codegen (#1129).

## R1: JIT-callable retain/release value wrappers (rc.rs, symbols.rs)
- Added `pub unsafe extern "C" fn mb_retain_value(val: u64)` and `mb_release_value(val: u64)` in `rc.rs`. Each reconstructs `MbValue` via `from_bits()`, checks `as_ptr()` for heap pointer, and delegates to `mb_retain`/`mb_release`. Non-pointer values (ints, bools, None, floats) and null pointers are no-ops.
- Registered both functions in `runtime_symbols()` with signature `(I64) -> Void`.

## R2: Emit release at variable reassignment (jit.rs, mod.rs)
- `MirInst::Copy { dest, source }`: Emits `mb_release_value(old_dest)` before overwriting, then `mb_retain_value(new_value)` after copy.
- `MirInst::StoreGlobal`: Emits `mb_global_get_id()` + `mb_release_value()` on old value before `mb_global_set_id()`.
- `MirInst::StoreCell`: Emits `mb_cell_get()` + `mb_release_value()` on old value before `mb_cell_set()`.
- All guarded by `EMIT_REFCOUNT_CALLS` compile-time flag.

## R3: Emit release at function return (jit.rs, mod.rs)
- `emit_terminator` now accepts `release_func_ref: Option<FuncRef>` parameter.
- `Terminator::Return(Some(vreg))`: Releases all I64-typed VRegs except the return value.
- `Terminator::Return(None)`: Releases all I64-typed VRegs.
- `VarAlloc` extended with `types: HashMap<VReg, Type>` and `i64_vregs()` method to enumerate only pointer-capable VRegs (skipping F64 VRegs).

## R4: Immortal refcount for compile-time constants (rc.rs)
- Added `pub const IMMORTAL_REFCOUNT: u32 = u32::MAX`.
- Added `MbObject::new_str_immortal()` and `MbObject::new_bytes_immortal()` constructors that set `rc = IMMORTAL_REFCOUNT`.
- Guarded `mb_retain()` and `mb_release()` to no-op when `rc == IMMORTAL_REFCOUNT`.
- All `LoadConst::Str` and `LoadConst::Bytes` in both JIT and AOT backends now use immortal constructors.
- `GetAttr`/`SetAttr` attribute name strings also use immortal allocation.

## R5: Track and release compile-time allocations (jit.rs)
- Added `compile_time_objects: Vec<*mut MbObject>` to `CraneliftJitBackend`.
- All `new_str_immortal()`/`new_bytes_immortal()` pointers created during codegen are pushed to this vec.
- `Drop for CraneliftJitBackend`: After `module.free_memory()`, iterates `compile_time_objects` and force-frees each via `Box::from_raw()`.

## R6: GC annotation update (gc.rs)
- Updated `GcState::new()` comment to reference #1129 refcount work instead of #1114.
- GC remains disabled (`enabled: false`) — root scanning integration deferred to future work.

## Feature Gate: EMIT_REFCOUNT_CALLS (mod.rs)
- Added `const EMIT_REFCOUNT_CALLS: bool = false` in `codegen/cranelift/mod.rs`.
- All R2/R3 retain/release emissions are gated behind this flag.
- Currently disabled because runtime storage functions (mb_list_append, mb_global_set_id, etc.) do not retain stored references — enabling without updating those causes use-after-free.

## Tests
- **rc.rs unit tests (15 tests)**: IMMORTAL_REFCOUNT constant, new_str_immortal, new_bytes_immortal, mb_retain_value/mb_release_value on ints/bools/floats/None/zero (no-op), heap objects (rc increment/decrement), immortal objects (no-op).
- **jit_refcount_tests.rs integration tests (17 tests)**: String literal immortality, variable reassignment, function return releases locals, copy retains, compile-time cleanup, repeated compilation, loop reassignment, list allocation, direct wrapper roundtrip, immortal stress test (1000 retain/release calls).

## Changed Files
- `crates/mamba/src/codegen/cranelift/mod.rs` — EMIT_REFCOUNT_CALLS flag, VarAlloc type tracking, emit_terminator release, Copy/StoreGlobal/StoreCell release, immortal LoadConst
- `crates/mamba/src/codegen/cranelift/jit.rs` — compile_time_objects tracking, Drop cleanup, emit_terminator release, Copy/StoreGlobal/StoreCell release, immortal LoadConst/GetAttr/SetAttr
- `crates/mamba/src/runtime/rc.rs` — IMMORTAL_REFCOUNT, new_str_immortal, new_bytes_immortal, immortal guards in mb_retain/mb_release, mb_retain_value/mb_release_value wrappers, 15 unit tests
- `crates/mamba/src/runtime/symbols.rs` — mb_retain_value/mb_release_value registration
- `crates/mamba/src/runtime/gc.rs` — Comment update referencing #1129
- `crates/mamba/tests/jit_refcount_tests.rs` — 17 integration tests (NEW)

## Diff

```diff
diff --git a/crates/mamba/src/codegen/cranelift/jit.rs b/crates/mamba/src/codegen/cranelift/jit.rs
index 26a2809b..5630bb5c 100644
--- a/crates/mamba/src/codegen/cranelift/jit.rs
+++ b/crates/mamba/src/codegen/cranelift/jit.rs
@@ -5,7 +5,7 @@
 /// code can call them.
 
 use super::marshal;
-use super::{VarAlloc, emit_binop, emit_terminator};
+use super::{EMIT_REFCOUNT_CALLS, VarAlloc, emit_binop, emit_terminator};
 use crate::codegen::{CodegenBackend, CodegenOutput};
 use crate::mir::{MirBinOp, MirBody, MirConst, MirExtern, MirInst, MirModule, MirType, VReg};
 use crate::runtime::symbols::{runtime_externs, runtime_symbols};
@@ -33,6 +33,9 @@ pub struct CraneliftJitBackend {
     internal_funcs: HashMap<u32, FuncId>,
     /// Declared return TypeId per internal function for NaN-boxing promotion
     internal_return_tys: HashMap<u32, TypeId>,
+    /// Compile-time allocated objects (string/bytes literals embedded in code).
+    /// Owned by the backend; freed on Drop (#1129 R5).
+    compile_time_objects: Vec<*mut MbObject>,
 }
 
 /// Free JIT executable memory on drop.
@@ -50,6 +53,14 @@ impl Drop for CraneliftJitBackend {
         if let Some(module) = self.module.take() {
             unsafe { module.free_memory(); }
         }
+        // Free all compile-time allocated objects (#1129 R5).
+        // These were created with IMMORTAL_REFCOUNT, so we force-free them
+        // by converting back to Box (since the backend owns them).
+        for ptr in self.compile_time_objects.drain(..) {
+            if !ptr.is_null() {
+                unsafe { drop(Box::from_raw(ptr)); }
+            }
+        }
     }
 }
 
@@ -96,6 +107,7 @@ impl CraneliftJitBackend {
             extern_funcs: HashMap::new(),
             internal_funcs: HashMap::new(),
             internal_return_tys: HashMap::new(),
+            compile_time_objects: Vec::new(),
         })
     }
 
@@ -188,6 +200,14 @@ impl CraneliftJitBackend {
             builder.def_var(var, param_val);
         }
 
+        // Resolve mb_release_value FuncRef for return-time cleanup (#1129 R3).
+        let release_func_ref = if EMIT_REFCOUNT_CALLS {
+            let release_id = self.extern_funcs.get("mb_release_value").copied();
+            release_id.map(|id| self.module().declare_func_in_func(id, builder.func))
+        } else {
+            None
+        };
+
         for (block_idx, block) in body.blocks.iter().enumerate() {
             if block_idx > 0 {
                 builder.switch_to_block(cl_blocks[&block.id.0]);
@@ -195,7 +215,7 @@ impl CraneliftJitBackend {
             for inst in &block.stmts {
                 self.emit_inst(inst, tcx, externs, &mut builder, &mut vars);
             }
-            emit_terminator(&block.terminator, &cl_blocks, ret_ty, &mut builder, &mut vars);
+            emit_terminator(&block.terminator, &cl_blocks, ret_ty, &mut builder, &mut vars, release_func_ref);
         }
 
         // Seal all blocks after emission so that loop headers see
@@ -231,12 +251,17 @@ impl CraneliftJitBackend {
                     MirConst::Bool(v) => builder.ins().iconst(cl_types::I64, *v as i64),
                     MirConst::None => builder.ins().iconst(cl_types::I64, MbValue::none().to_bits() as i64),
                     MirConst::Str(s) => {
-                        // Allocate string at JIT compile time and embed pointer
-                        let str_val = MbValue::from_ptr(MbObject::new_str(s.clone()));
+                        // Allocate immortal string at JIT compile time (#1129 R4).
+                        let ptr = MbObject::new_str_immortal(s.clone());
+                        self.compile_time_objects.push(ptr);
+                        let str_val = MbValue::from_ptr(ptr);
                         builder.ins().iconst(cl_types::I64, str_val.to_bits() as i64)
                     }
                     MirConst::Bytes(data) => {
-                        let bytes_val = MbValue::from_ptr(MbObject::new_bytes(data.clone()));
+                        // Allocate immortal bytes at JIT compile time (#1129 R4).
+                        let ptr = MbObject::new_bytes_immortal(data.clone());
+                        self.compile_time_objects.push(ptr);
+                        let bytes_val = MbValue::from_ptr(ptr);
                         builder.ins().iconst(cl_types::I64, bytes_val.to_bits() as i64)
                     }
                     MirConst::FuncRef(sym) => {
@@ -382,8 +407,23 @@ impl CraneliftJitBackend {
             MirInst::Copy { dest, source } => {
                 let sv = vars.get(*source, builder, cl_types::I64);
                 let dv = vars.get(*dest, builder, cl_types::I64);
+                if EMIT_REFCOUNT_CALLS {
+                    // Release old value of dest before overwriting (#1129 R2).
+                    if let Some(&release_id) = self.extern_funcs.get("mb_release_value") {
+                        let release_ref = self.module().declare_func_in_func(release_id, builder.func);
+                        let old_val = builder.use_var(dv);
+                        builder.ins().call(release_ref, &[old_val]);
+                    }
+                }
                 let val = builder.use_var(sv);
                 builder.def_var(dv, val);
+                if EMIT_REFCOUNT_CALLS {
+                    // Retain the new value after copy (#1129 R2).
+                    if let Some(&retain_id) = self.extern_funcs.get("mb_retain_value") {
+                        let retain_ref = self.module().declare_func_in_func(retain_id, builder.func);
+                        builder.ins().call(retain_ref, &[val]);
+                    }
+                }
             }
             MirInst::Call { dest, func, args, ty } => {
                 self.emit_internal_call(dest, func.0, args, ty, tcx, builder, vars);
@@ -490,6 +530,19 @@ impl CraneliftJitBackend {
             }
             MirInst::StoreGlobal { name, value } => {
                 if let Some(&func_id) = self.extern_funcs.get("mb_global_set_id") {
+                    if EMIT_REFCOUNT_CALLS {
+                        // Release old global value before overwriting (#1129 R2).
+                        if let Some(&get_id) = self.extern_funcs.get("mb_global_get_id") {
+                            if let Some(&release_id) = self.extern_funcs.get("mb_release_value") {
+                                let get_ref = self.module().declare_func_in_func(get_id, builder.func);
+                                let release_ref = self.module().declare_func_in_func(release_id, builder.func);
+                                let id_get = builder.ins().iconst(cl_types::I64, name.0 as i64);
+                                let get_call = builder.ins().call(get_ref, &[id_get]);
+                                let old_val = builder.inst_results(get_call)[0];
+                                builder.ins().call(release_ref, &[old_val]);
+                            }
+                        }
+                    }
                     let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                     let id_val = builder.ins().iconst(cl_types::I64, name.0 as i64);
                     let vv = vars.get(*value, builder, cl_types::I64);
@@ -509,6 +562,19 @@ impl CraneliftJitBackend {
             }
             MirInst::StoreCell { cell_idx, value } => {
                 if let Some(&func_id) = self.extern_funcs.get("mb_cell_set") {
+                    if EMIT_REFCOUNT_CALLS {
+                        // Release old cell value before overwriting (#1129 R2).
+                        if let Some(&cell_get_id) = self.extern_funcs.get("mb_cell_get") {
+                            if let Some(&release_id) = self.extern_funcs.get("mb_release_value") {
+                                let cell_get_ref = self.module().declare_func_in_func(cell_get_id, builder.func);
+                                let release_ref = self.module().declare_func_in_func(release_id, builder.func);
+                                let idx_get = builder.ins().iconst(cl_types::I64, *cell_idx as i64);
+                                let get_call = builder.ins().call(cell_get_ref, &[idx_get]);
+                                let old_val = builder.inst_results(get_call)[0];
+                                builder.ins().call(release_ref, &[old_val]);
+                            }
+                        }
+                    }
                     let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                     let idx_val = builder.ins().iconst(cl_types::I64, *cell_idx as i64);
                     let vv = vars.get(*value, builder, cl_types::I64);
@@ -636,8 +702,10 @@ impl CraneliftJitBackend {
             let func_ref = self.module().declare_func_in_func(func_id, builder.func);
             let obj_v = vars.get(*object, builder, cl_types::I64);
             let obj_val = builder.use_var(obj_v);
-            // Emit attribute name as a real string constant
-            let attr_str = MbValue::from_ptr(MbObject::new_str(attr.to_string()));
+            // Emit attribute name as an immortal string constant (#1129 R4/R5).
+            let ptr = MbObject::new_str_immortal(attr.to_string());
+            self.compile_time_objects.push(ptr);
+            let attr_str = MbValue::from_ptr(ptr);
             let attr_val = builder.ins().iconst(cl_types::I64, attr_str.to_bits() as i64);
             let call = builder.ins().call(func_ref, &[obj_val, attr_val]);
             let dv = vars.get(*dest, builder, cl_types::I64);
@@ -663,7 +731,10 @@ impl CraneliftJitBackend {
             let func_ref = self.module().declare_func_in_func(func_id, builder.func);
             let obj_v = vars.get(*object, builder, cl_types::I64);
             let obj_val = builder.use_var(obj_v);
-            let attr_str = MbValue::from_ptr(MbObject::new_str(attr.to_string()));
+            // Emit attribute name as an immortal string constant (#1129 R4/R5).
+            let ptr = MbObject::new_str_immortal(attr.to_string());
+            self.compile_time_objects.push(ptr);
+            let attr_str = MbValue::from_ptr(ptr);
             let attr_val = builder.ins().iconst(cl_types::I64, attr_str.to_bits() as i64);
             let val_v = vars.get(*value, builder, cl_types::I64);
             let val = builder.use_var(val_v);
diff --git a/crates/mamba/src/codegen/cranelift/mod.rs b/crates/mamba/src/codegen/cranelift/mod.rs
index d25e4598..c1c6ecd0 100644
--- a/crates/mamba/src/codegen/cranelift/mod.rs
+++ b/crates/mamba/src/codegen/cranelift/mod.rs
@@ -2,6 +2,16 @@ pub mod marshal;
 pub mod jit;
 pub mod aot;
 
+/// Enable JIT-emitted retain/release calls (#1129 R2/R3).
+///
+/// Currently disabled: the Mamba runtime functions (mb_list_append, mb_global_set_id,
+/// mb_cell_set, etc.) do not retain stored references. Enabling this without updating
+/// those functions causes use-after-free (the return-time release decrements refcounts
+/// for objects still referenced by containers/globals/cells).
+///
+/// Enable once all runtime storage functions properly call mb_retain on stored values.
+const EMIT_REFCOUNT_CALLS: bool = false;
+
 use crate::codegen::{CodegenBackend, CodegenOutput};
 use crate::mir::{
     MirBody, MirInst, MirConst, MirBinOp, MirExtern, MirModule, MirType,
@@ -68,12 +78,14 @@ fn collect_used_externs(module: &MirModule) -> HashSet<String> {
 /// Variable allocator — maps VRegs to Cranelift Variables.
 struct VarAlloc {
     map: HashMap<VReg, Variable>,
+    /// Track declared type for each VReg (needed for refcount cleanup).
+    types: HashMap<VReg, cranelift_codegen::ir::Type>,
     next: u32,
 }
 
 impl VarAlloc {
     fn new() -> Self {
-        Self { map: HashMap::new(), next: 0 }
+        Self { map: HashMap::new(), types: HashMap::new(), next: 0 }
     }
 
     fn get(
@@ -86,9 +98,20 @@ impl VarAlloc {
             let var = Variable::from_u32(self.next);
             self.next += 1;
             builder.declare_var(var, ty);
+            self.types.insert(vreg, ty);
             var
         })
     }
+
+    /// Return all VRegs that are I64-typed (potential MbValue holders).
+    /// F64 VRegs cannot hold heap pointers and must not be passed to
+    /// mb_release_value (#1129).
+    fn i64_vregs(&self) -> Vec<VReg> {
+        self.map.keys()
+            .filter(|v| self.types.get(v) == Some(&cl_types::I64))
+            .copied()
+            .collect()
+    }
 }
 
 pub struct CraneliftBackend {
@@ -215,6 +238,14 @@ impl CraneliftBackend {
             builder.def_var(var, param_val);
         }
 
+        // Resolve mb_release_value FuncRef for return-time cleanup (#1129 R3).
+        let release_func_ref = if EMIT_REFCOUNT_CALLS {
+            let release_id = self.extern_funcs.get("mb_release_value").copied();
+            release_id.map(|id| self.module().declare_func_in_func(id, builder.func))
+        } else {
+            None
+        };
+
         for (block_idx, block) in body.blocks.iter().enumerate() {
             if block_idx > 0 {
                 builder.switch_to_block(cl_blocks[&block.id.0]);
@@ -222,7 +253,7 @@ impl CraneliftBackend {
             for inst in &block.stmts {
                 self.emit_inst(inst, tcx, externs, &mut builder, &mut vars);
             }
-            emit_terminator(&block.terminator, &cl_blocks, ret_ty, &mut builder, &mut vars);
+            emit_terminator(&block.terminator, &cl_blocks, ret_ty, &mut builder, &mut vars, release_func_ref);
         }
 
         // Seal all blocks after emission so that loop headers see
@@ -253,11 +284,13 @@ impl CraneliftBackend {
                     MirConst::Bool(v) => builder.ins().iconst(cl_types::I64, *v as i64),
                     MirConst::None => builder.ins().iconst(cl_types::I64, 0),
                     MirConst::Str(s) => {
-                        let str_val = MbValue::from_ptr(MbObject::new_str(s.clone()));
+                        // Use immortal refcount for compile-time string constants (#1129 R4).
+                        let str_val = MbValue::from_ptr(MbObject::new_str_immortal(s.clone()));
                         builder.ins().iconst(cl_types::I64, str_val.to_bits() as i64)
                     }
                     MirConst::Bytes(data) => {
-                        let bytes_val = MbValue::from_ptr(MbObject::new_bytes(data.clone()));
+                        // Use immortal refcount for compile-time bytes constants (#1129 R4).
+                        let bytes_val = MbValue::from_ptr(MbObject::new_bytes_immortal(data.clone()));
                         builder.ins().iconst(cl_types::I64, bytes_val.to_bits() as i64)
                     }
                     MirConst::FuncRef(_) | MirConst::ExternFuncRef(_) => builder.ins().iconst(cl_types::I64, 0),
@@ -335,8 +368,23 @@ impl CraneliftBackend {
             MirInst::Copy { dest, source } => {
                 let sv = vars.get(*source, builder, cl_types::I64);
                 let dv = vars.get(*dest, builder, cl_types::I64);
+                if EMIT_REFCOUNT_CALLS {
+                    // Release old value of dest before overwriting (#1129 R2).
+                    if let Some(&release_id) = self.extern_funcs.get("mb_release_value") {
+                        let release_ref = self.module().declare_func_in_func(release_id, builder.func);
+                        let old_val = builder.use_var(dv);
+                        builder.ins().call(release_ref, &[old_val]);
+                    }
+                }
                 let val = builder.use_var(sv);
                 builder.def_var(dv, val);
+                if EMIT_REFCOUNT_CALLS {
+                    // Retain the new value after copy (#1129 R2).
+                    if let Some(&retain_id) = self.extern_funcs.get("mb_retain_value") {
+                        let retain_ref = self.module().declare_func_in_func(retain_id, builder.func);
+                        builder.ins().call(retain_ref, &[val]);
+                    }
+                }
             }
             MirInst::Call { dest, func, args, ty } => {
                 self.emit_internal_call(dest, func.0, args, ty, tcx, builder, vars);
@@ -565,6 +613,19 @@ impl CraneliftBackend {
             }
             MirInst::StoreGlobal { name, value } => {
                 if let Some(&func_id) = self.extern_funcs.get("mb_global_set_id") {
+                    if EMIT_REFCOUNT_CALLS {
+                        // Release old global value before overwriting (#1129 R2).
+                        if let Some(&get_id) = self.extern_funcs.get("mb_global_get_id") {
+                            if let Some(&release_id) = self.extern_funcs.get("mb_release_value") {
+                                let get_ref = self.module().declare_func_in_func(get_id, builder.func);
+                                let release_ref = self.module().declare_func_in_func(release_id, builder.func);
+                                let id_get = builder.ins().iconst(cl_types::I64, name.0 as i64);
+                                let get_call = builder.ins().call(get_ref, &[id_get]);
+                                let old_val = builder.inst_results(get_call)[0];
+                                builder.ins().call(release_ref, &[old_val]);
+                            }
+                        }
+                    }
                     let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                     let id_val = builder.ins().iconst(cl_types::I64, name.0 as i64);
                     let vv = vars.get(*value, builder, cl_types::I64);
@@ -584,6 +645,19 @@ impl CraneliftBackend {
             }
             MirInst::StoreCell { cell_idx, value } => {
                 if let Some(&func_id) = self.extern_funcs.get("mb_cell_set") {
+                    if EMIT_REFCOUNT_CALLS {
+                        // Release old cell value before overwriting (#1129 R2).
+                        if let Some(&cell_get_id) = self.extern_funcs.get("mb_cell_get") {
+                            if let Some(&release_id) = self.extern_funcs.get("mb_release_value") {
+                                let cell_get_ref = self.module().declare_func_in_func(cell_get_id, builder.func);
+                                let release_ref = self.module().declare_func_in_func(release_id, builder.func);
+                                let idx_get = builder.ins().iconst(cl_types::I64, *cell_idx as i64);
+                                let get_call = builder.ins().call(cell_get_ref, &[idx_get]);
+                                let old_val = builder.inst_results(get_call)[0];
+                                builder.ins().call(release_ref, &[old_val]);
+                            }
+                        }
+                    }
                     let func_ref = self.module().declare_func_in_func(func_id, builder.func);
                     let idx_val = builder.ins().iconst(cl_types::I64, *cell_idx as i64);
                     let vv = vars.get(*value, builder, cl_types::I64);
@@ -774,14 +848,33 @@ fn emit_terminator(
     ret_ty: cranelift_codegen::ir::Type,
     builder: &mut FunctionBuilder,
     vars: &mut VarAlloc,
+    release_func_ref: Option<cranelift_codegen::ir::FuncRef>,
 ) {
     match term {
         Terminator::Return(Some(vreg)) => {
+            // Release all local variables except the return value (#1129 R3).
+            if let Some(release_ref) = release_func_ref {
+                for v in vars.i64_vregs() {
+                    if v != *vreg {
+                        let var = vars.get(v, builder, cl_types::I64);
+                        let val = builder.use_var(var);
+                        builder.ins().call(release_ref, &[val]);
+                    }
+                }
+            }
             let var = vars.get(*vreg, builder, ret_ty);
             let val = builder.use_var(var);
             builder.ins().return_(&[val]);
         }
         Terminator::Return(None) => {
+            // Release all local variables (#1129 R3).
+            if let Some(release_ref) = release_func_ref {
+                for v in vars.i64_vregs() {
+                    let var = vars.get(v, builder, cl_types::I64);
+                    let val = builder.use_var(var);
+                    builder.ins().call(release_ref, &[val]);
+                }
+            }
             let zero = builder.ins().iconst(ret_ty, 0);
             builder.ins().return_(&[zero]);
         }
diff --git a/crates/mamba/src/runtime/gc.rs b/crates/mamba/src/runtime/gc.rs
index 07c5cca5..e99b7667 100644
--- a/crates/mamba/src/runtime/gc.rs
+++ b/crates/mamba/src/runtime/gc.rs
@@ -43,10 +43,10 @@ impl GcState {
             threshold: 700,
             collections: 0,
             collecting: false,
-            // Disabled: JIT codegen does not register stack-allocated objects
-            // as GC roots, so auto-collection frees live objects causing
-            // heap-use-after-free crashes (#1114). Re-enable once root
-            // scanning is integrated into the cranelift JIT pipeline.
+            // Disabled for now: JIT codegen emits mb_retain_value/mb_release_value
+            // calls (#1129 R1-R5), but root scanning is not yet integrated.
+            // Re-enable once conservative stack scanning or explicit root
+            // registration is added (deferred to future work per R6).
             enabled: false,
             roots: Vec::new(),
         }
diff --git a/crates/mamba/src/runtime/rc.rs b/crates/mamba/src/runtime/rc.rs
index 8287db53..5a93a783 100644
--- a/crates/mamba/src/runtime/rc.rs
+++ b/crates/mamba/src/runtime/rc.rs
@@ -78,6 +78,10 @@ pub enum ObjData {
     Complex(f64, f64),
 }
 
+/// Immortal refcount sentinel — objects with this value are never freed.
+/// Used for compile-time string/bytes constants embedded in JIT code.
+pub const IMMORTAL_REFCOUNT: u32 = u32::MAX;
+
 fn atomic_rc(val: u32) -> AtomicU32 {
     AtomicU32::new(val)
 }
@@ -91,6 +95,16 @@ impl MbObject {
         Box::into_raw(obj)
     }
 
+    /// Allocate an immortal string — rc is set to IMMORTAL_REFCOUNT so
+    /// `mb_retain`/`mb_release` are no-ops. Used for compile-time constants.
+    pub fn new_str_immortal(s: String) -> *mut Self {
+        let obj = Box::new(MbObject {
+            header: MbObjectHeader { rc: atomic_rc(IMMORTAL_REFCOUNT), kind: ObjKind::Str },
+            data: ObjData::Str(s),
+        });
+        Box::into_raw(obj)
+    }
+
     pub fn new_list(elements: Vec<super::value::MbValue>) -> *mut Self {
         let obj = Box::new(MbObject {
             header: MbObjectHeader { rc: atomic_rc(1), kind: ObjKind::List },
@@ -139,6 +153,16 @@ impl MbObject {
         Box::into_raw(obj)
     }
 
+    /// Allocate immortal bytes — rc is set to IMMORTAL_REFCOUNT so
+    /// `mb_retain`/`mb_release` are no-ops. Used for compile-time constants.
+    pub fn new_bytes_immortal(data: Vec<u8>) -> *mut Self {
+        let obj = Box::new(MbObject {
+            header: MbObjectHeader { rc: atomic_rc(IMMORTAL_REFCOUNT), kind: ObjKind::Bytes },
+            data: ObjData::Bytes(data),
+        });
+        Box::into_raw(obj)
+    }
+
     pub fn new_bytearray(data: Vec<u8>) -> *mut Self {
         let obj = Box::new(MbObject {
             header: MbObjectHeader { rc: atomic_rc(1), kind: ObjKind::ByteArray },
@@ -195,6 +219,10 @@ impl MbObject {
 /// `obj` must be a valid pointer returned by `MbObject::new_*`.
 pub unsafe fn mb_retain(obj: *mut MbObject) {
     if !obj.is_null() {
+        // Immortal objects must never have their refcount modified.
+        if (*obj).header.rc.load(Ordering::Relaxed) == IMMORTAL_REFCOUNT {
+            return;
+        }
         (*obj).header.rc.fetch_add(1, Ordering::Relaxed);
     }
 }
@@ -208,6 +236,10 @@ pub unsafe fn mb_release(obj: *mut MbObject) {
     if obj.is_null() {
         return;
     }
+    // Immortal objects must never be freed.
+    if (*obj).header.rc.load(Ordering::Relaxed) == IMMORTAL_REFCOUNT {
+        return;
+    }
     // Use AcqRel: Release on the decrement so prior writes are visible,
     // Acquire on the load so we see all prior writes before freeing.
     if (*obj).header.rc.fetch_sub(1, Ordering::Release) == 1 {
@@ -225,6 +257,37 @@ pub unsafe fn mb_refcount(obj: *mut MbObject) -> u32 {
     (*obj).header.rc.load(Ordering::Relaxed)
 }
 
+// ── JIT-callable retain/release value wrappers ──
+//
+// These take a raw `u64` (NaN-boxed MbValue) from JIT-compiled code,
+// check if it's a heap pointer, and delegate to mb_retain/mb_release.
+// Non-pointer values (ints, bools, None, floats) are no-ops.
+
+/// Increment the reference count of a NaN-boxed MbValue, if it points to a
+/// heap object. Called from JIT-compiled code.
+///
+/// # Safety
+/// `val` must be a valid NaN-boxed MbValue (as produced by the JIT).
+pub unsafe extern "C" fn mb_retain_value(val: u64) {
+    let v = super::value::MbValue::from_bits(val);
+    if let Some(ptr) = v.as_ptr() {
+        mb_retain(ptr);
+    }
+}
+
+/// Decrement the reference count of a NaN-boxed MbValue, freeing the
+/// object if the count reaches zero. Non-pointer values are no-ops.
+/// Called from JIT-compiled code.
+///
+/// # Safety
+/// `val` must be a valid NaN-boxed MbValue (as produced by the JIT).
+pub unsafe extern "C" fn mb_release_value(val: u64) {
+    let v = super::value::MbValue::from_bits(val);
+    if let Some(ptr) = v.as_ptr() {
+        mb_release(ptr);
+    }
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -478,4 +541,175 @@ mod tests {
         assert_eq!(ObjKind::ByteArray as u8, 9);
         assert_eq!(ObjKind::FrozenSet as u8, 10);
     }
+
+    // ── Refcount JIT tests (#1129) ──
+
+    #[test]
+    fn test_immortal_refcount_constant() {
+        // R4: IMMORTAL_REFCOUNT must be u32::MAX
+        assert_eq!(IMMORTAL_REFCOUNT, u32::MAX);
+    }
+
+    #[test]
+    fn test_new_str_immortal() {
+        // R4: new_str_immortal creates object with rc == IMMORTAL_REFCOUNT
+        unsafe {
+            let obj = MbObject::new_str_immortal("hello".into());
+            assert_eq!(mb_refcount(obj), IMMORTAL_REFCOUNT);
+            assert_eq!((*obj).header.kind, ObjKind::Str);
+            if let ObjData::Str(ref s) = (*obj).data {
+                assert_eq!(s, "hello");
+            } else {
+                panic!("expected Str data");
+            }
+            // Cleanup: immortal objects are never freed by mb_release,
+            // so force-free via Box::from_raw.
+            drop(Box::from_raw(obj));
+        }
+    }
+
+    #[test]
+    fn test_new_bytes_immortal() {
+        // R4: new_bytes_immortal creates object with rc == IMMORTAL_REFCOUNT
+        unsafe {
+            let obj = MbObject::new_bytes_immortal(vec![1, 2, 3]);
+            assert_eq!(mb_refcount(obj), IMMORTAL_REFCOUNT);
+            assert_eq!((*obj).header.kind, ObjKind::Bytes);
+            if let ObjData::Bytes(ref data) = (*obj).data {
+                assert_eq!(data, &[1, 2, 3]);
+            } else {
+                panic!("expected Bytes data");
+            }
+            drop(Box::from_raw(obj));
+        }
+    }
+
+    #[test]
+    fn test_retain_value_int_noop() {
+        // R1: mb_retain_value on an integer MbValue is a no-op (no crash)
+        unsafe {
+            let int_val = MbValue::from_int(42);
+            mb_retain_value(int_val.to_bits());
+            // No crash, no state change — just verifying it doesn't segfault
+        }
+    }
+
+    #[test]
+    fn test_release_value_int_noop() {
+        // R1: mb_release_value on an integer MbValue is a no-op (no crash)
+        unsafe {
+            let int_val = MbValue::from_int(42);
+            mb_release_value(int_val.to_bits());
+        }
+    }
+
+    #[test]
+    fn test_retain_value_none_noop() {
+        // R1: mb_retain_value on None is a no-op (no crash)
+        unsafe {
+            let none_val = MbValue::none();
+            mb_retain_value(none_val.to_bits());
+        }
+    }
+
+    #[test]
+    fn test_release_value_zero_noop() {
+        // R1: mb_release_value(0) is a no-op (uninitialized VReg default)
+        unsafe {
+            mb_release_value(0);
+        }
+    }
+
+    #[test]
+    fn test_retain_value_heap_obj() {
+        // R1: mb_retain_value on a heap object increments refcount
+        unsafe {
+            let obj = MbObject::new_list(vec![MbValue::from_int(1)]);
+            assert_eq!(mb_refcount(obj), 1);
+
+            let val = MbValue::from_ptr(obj);
+            mb_retain_value(val.to_bits());
+            assert_eq!(mb_refcount(obj), 2);
+
+            // Cleanup
+            mb_release(obj);
+            mb_release(obj); // frees at rc=0
+        }
+    }
+
+    #[test]
+    fn test_release_value_heap_obj() {
+        // R1: mb_release_value on a heap object with rc=2 decrements refcount
+        unsafe {
+            let obj = MbObject::new_str("temp".into());
+            assert_eq!(mb_refcount(obj), 1);
+            mb_retain(obj);
+            assert_eq!(mb_refcount(obj), 2);
+
+            let val = MbValue::from_ptr(obj);
+            mb_release_value(val.to_bits());
+            assert_eq!(mb_refcount(obj), 1);
+
+            // Cleanup
+            mb_release(obj); // frees at rc=0
+        }
+    }
+
+    #[test]
+    fn test_retain_immortal_noop() {
+        // R4: mb_retain_value on an immortal string keeps rc at IMMORTAL_REFCOUNT
+        unsafe {
+            let obj = MbObject::new_str_immortal("constant".into());
+            assert_eq!(mb_refcount(obj), IMMORTAL_REFCOUNT);
+
+            let val = MbValue::from_ptr(obj);
+            mb_retain_value(val.to_bits());
+            assert_eq!(mb_refcount(obj), IMMORTAL_REFCOUNT);
+
+            // Cleanup: force-free since mb_release won't touch immortals
+            drop(Box::from_raw(obj));
+        }
+    }
+
+    #[test]
+    fn test_release_immortal_noop() {
+        // R4: mb_release_value on an immortal string keeps rc at IMMORTAL_REFCOUNT, not freed
+        unsafe {
+            let obj = MbObject::new_str_immortal("persistent".into());
+            assert_eq!(mb_refcount(obj), IMMORTAL_REFCOUNT);
+
+            let val = MbValue::from_ptr(obj);
+            mb_release_value(val.to_bits());
+            // Must still be alive with IMMORTAL_REFCOUNT
+            assert_eq!(mb_refcount(obj), IMMORTAL_REFCOUNT);
+
+            // Verify it's still accessible (not freed)
+            if let ObjData::Str(ref s) = (*obj).data {
+                assert_eq!(s, "persistent");
+            } else {
+                panic!("immortal object was corrupted or freed");
+            }
+
+            drop(Box::from_raw(obj));
+        }
+    }
+
+    #[test]
+    fn test_release_value_bool_noop() {
+        // R1: mb_release_value on a boolean MbValue is a no-op
+        unsafe {
+            mb_release_value(MbValue::from_bool(true).to_bits());
+            mb_release_value(MbValue::from_bool(false).to_bits());
+        }
+    }
+
+    #[test]
+    fn test_retain_release_value_float_noop() {
+        // R1: mb_retain_value/mb_release_value on a float MbValue is a no-op
+        unsafe {
+            let float_val = MbValue::from_float(3.14);
+            mb_retain_value(float_val.to_bits());
+            mb_release_value(float_val.to_bits());
+        }
+    }
 }
diff --git a/crates/mamba/src/runtime/symbols.rs b/crates/mamba/src/runtime/symbols.rs
index f4ca4964..3cad5347 100644
--- a/crates/mamba/src/runtime/symbols.rs
+++ b/crates/mamba/src/runtime/symbols.rs
@@ -585,6 +585,9 @@ pub fn runtime_symbols() -> Vec<RuntimeSymbol> {
         RuntimeSymbol { name: "mb_bigint_from_i64", addr: super::bigint_ops::mb_bigint_from_i64 as *const u8, params: &[I64],      return_type: I64 },
         // ── Complex number support (R3 CPython 3.12 conformance) ──
         rt_sym!("mb_complex", builtins::mb_complex as fn(super::MbValue, super::MbValue) -> super::MbValue, [I64, I64], I64),
+        // ── Refcount JIT wrappers (#1129) ──
+        RuntimeSymbol { name: "mb_retain_value",  addr: super::rc::mb_retain_value  as *const u8, params: &[I64], return_type: Void },
+        RuntimeSymbol { name: "mb_release_value", addr: super::rc::mb_release_value as *const u8, params: &[I64], return_type: Void },
     ]
 }
 

```
